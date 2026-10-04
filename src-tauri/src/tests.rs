use age::secrecy::SecretString;
use chrono::{Days, NaiveDate};
use uuid::Uuid;

use crate::database::testing::TestDatabase;
use crate::errors::AppError;
use crate::managers::forecasts::ForecastRequest;
use crate::models::dates::{add_months, today};
use crate::models::{
    Account, AccountInput, AccountKind, CreditInput, EntryKind, ForecastMethod, Frequency,
    FrequencyUnit, RecurrenceInput, TransactionFilter, TransactionInput, TransactionKind,
    TransferInput,
};
use crate::module::{AppContext, Module};

const PASSWORD: &str = "test-password";

fn module(db: &TestDatabase) -> Module {
    let context = AppContext {
        data_dir: db.dir.clone(),
        password: SecretString::from(PASSWORD.to_string()),
    };
    Module::new(db.pool.clone(), context)
}

async fn account(module: &Module, name: &str, initial_balance: i64) -> Account {
    let input = AccountInput {
        name: name.into(),
        kind: AccountKind::Bank,
        currency: "EUR".into(),
        initial_balance,
        color: None,
        icon: None,
    };
    module.accounts.create(today(), input).await.unwrap()
}

fn transaction(
    account_id: Uuid,
    kind: EntryKind,
    amount: i64,
    date: NaiveDate,
) -> TransactionInput {
    TransactionInput {
        account_id,
        category_id: None,
        kind,
        amount,
        date,
        description: "Supermercado".into(),
        notes: None,
        confirmed: false,
    }
}

fn monthly() -> Frequency {
    Frequency::new(FrequencyUnit::Month, 1).unwrap()
}

#[tokio::test]
async fn balances_and_filters_follow_transactions() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 100_000).await;
    let tomorrow = today().checked_add_days(Days::new(1)).unwrap();

    module
        .transactions
        .create(transaction(main.id, EntryKind::Income, 50_000, today()))
        .await
        .unwrap();
    module
        .transactions
        .create(transaction(main.id, EntryKind::Outcome, 20_000, today()))
        .await
        .unwrap();
    module
        .transactions
        .create(transaction(main.id, EntryKind::Outcome, 5_000, tomorrow))
        .await
        .unwrap();

    let main = module.accounts.get(today(), main.id).await.unwrap();
    assert_eq!(main.balance, 130_000);
    assert_eq!(main.available_balance, 125_000);
    assert_eq!(main.projected_balance, 125_000);

    let filter = TransactionFilter {
        kind: Some(TransactionKind::Outcome),
        ..Default::default()
    };
    let page = module.transactions.page(&filter, 50, 0).await.unwrap();
    assert_eq!(
        (page.total_count, page.income, page.outcome),
        (2, 0, 25_000)
    );

    let search = TransactionFilter {
        search: Some("super".into()),
        min_amount: Some(10_000),
        ..Default::default()
    };
    assert_eq!(
        module
            .transactions
            .page(&search, 50, 0)
            .await
            .unwrap()
            .total_count,
        2
    );
}

#[tokio::test]
async fn transfers_move_money_without_counting_as_income() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 100_000).await;
    let savings = account(&module, "Poupança", 0).await;

    let input = TransferInput {
        from_account_id: main.id,
        to_account_id: savings.id,
        amount: 30_000,
        date: today(),
        description: "Poupar".into(),
    };
    let transfer = module.transfers.create(today(), input).await.unwrap();

    assert_eq!(
        module.accounts.get(today(), main.id).await.unwrap().balance,
        70_000
    );
    assert_eq!(
        module
            .accounts
            .get(today(), savings.id)
            .await
            .unwrap()
            .balance,
        30_000
    );

    let page = module
        .transactions
        .page(&TransactionFilter::default(), 50, 0)
        .await
        .unwrap();
    assert_eq!((page.total_count, page.income, page.outcome), (2, 0, 0));

    let leg = page.items[0].id;
    assert!(matches!(
        module
            .transactions
            .update(leg, transaction(main.id, EntryKind::Income, 1, today()))
            .await,
        Err(AppError::Conflict(_))
    ));

    module.transactions.delete(leg).await.unwrap();
    assert!(module.transfers.get(transfer.id).await.is_err());
    assert_eq!(
        module
            .transactions
            .page(&TransactionFilter::default(), 50, 0)
            .await
            .unwrap()
            .total_count,
        0
    );
}

#[tokio::test]
async fn categories_in_use_need_reassignment() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 0).await;
    let categories = module.categories.list(false).await.unwrap();
    let supermarket = categories
        .iter()
        .find(|c| c.name == "Supermercado")
        .unwrap();
    let restaurants = categories
        .iter()
        .find(|c| c.name == "Restaurantes")
        .unwrap();
    let food = categories.iter().find(|c| c.name == "Alimentação").unwrap();

    let input = TransactionInput {
        category_id: Some(supermarket.id),
        ..transaction(main.id, EntryKind::Outcome, 1_000, today())
    };
    module.transactions.create(input.clone()).await.unwrap();

    let income = TransactionInput {
        kind: EntryKind::Income,
        ..input
    };
    assert!(matches!(
        module.transactions.create(income).await,
        Err(AppError::Validation(_))
    ));
    assert!(matches!(
        module.categories.delete(food.id, None).await,
        Err(AppError::Conflict(_))
    ));
    assert!(matches!(
        module.categories.delete(supermarket.id, None).await,
        Err(AppError::Conflict(_))
    ));

    module
        .categories
        .delete(supermarket.id, Some(restaurants.id))
        .await
        .unwrap();
    let filter = TransactionFilter {
        category_id: Some(food.id),
        ..Default::default()
    };
    let page = module.transactions.page(&filter, 50, 0).await.unwrap();
    assert_eq!(
        page.items[0].category_name.as_deref(),
        Some("Alimentação / Restaurantes")
    );
}

#[tokio::test]
async fn recurrences_materialize_once_and_respect_skips() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 0).await;

    let input = RecurrenceInput {
        account_id: main.id,
        category_id: None,
        kind: EntryKind::Outcome,
        amount: 80_000,
        description: "Renda".into(),
        start_date: today(),
        end_date: None,
        frequency: monthly(),
    };
    let rent = module.recurrences.create(input).await.unwrap();

    let second = add_months(today(), 1);
    module.recurrences.skip(rent.id, second).await.unwrap();
    module
        .recurrences
        .modify(rent.id, add_months(today(), 2), Some(90_000), None)
        .await
        .unwrap();

    let later = add_months(today(), 2);
    assert_eq!(module.recurrences.materialize_due(later).await.unwrap(), 2);
    assert_eq!(module.recurrences.materialize_due(later).await.unwrap(), 0);

    let page = module
        .transactions
        .page(&TransactionFilter::default(), 50, 0)
        .await
        .unwrap();
    let amounts: Vec<i64> = page.items.iter().map(|t| t.amount).collect();
    assert_eq!(amounts, vec![-90_000, -80_000]);
    assert!(page.items.iter().all(|t| t.recurrence_id == Some(rent.id)));

    let upcoming = module
        .recurrences
        .occurrences(later, add_months(today(), 4), None)
        .await
        .unwrap();
    assert_eq!(upcoming.len(), 2);
}

#[tokio::test]
async fn credit_installments_record_payments() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 1_000_000).await;

    let input = CreditInput {
        name: "Carro".into(),
        institution: Some("Banco".into()),
        principal: 1_200_000,
        opening_balance: 1_200_000,
        annual_rate: 6.0,
        installment: 50_000,
        frequency: monthly(),
        start_date: today(),
        end_date: None,
        installments: Some(26),
        account_id: Some(main.id),
    };
    let credit = module.credits.create(today(), input, true).await.unwrap();
    let recurrences = module.recurrences.list().await.unwrap();
    assert_eq!(recurrences[0].credit_id, Some(credit.id));

    module
        .recurrences
        .materialize_due(add_months(today(), 1))
        .await
        .unwrap();
    let credit = module.credits.get(credit.id).await.unwrap();
    let payments = module.credits.payments(Some(credit.id)).await.unwrap();

    assert_eq!(payments.len(), 2);
    assert_eq!(payments[1].interest, 6_000);
    assert_eq!(payments[1].principal, 44_000);
    assert_eq!(credit.remaining(), 1_200_000 - credit.principal_paid);
    assert!(credit.summary(today()).remaining_installments > 0);
}

#[tokio::test]
async fn forecast_includes_recurrences_and_goals() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 100_000).await;

    let salary = RecurrenceInput {
        account_id: main.id,
        category_id: None,
        kind: EntryKind::Income,
        amount: 200_000,
        description: "Salário".into(),
        start_date: add_months(today(), 1),
        end_date: None,
        frequency: monthly(),
    };
    module.recurrences.create(salary).await.unwrap();
    let goal = crate::models::GoalInput {
        name: "Fundo".into(),
        account_id: main.id,
        target_amount: 500_000,
        target_date: None,
    };
    module.goals.create(today(), goal).await.unwrap();

    let request = ForecastRequest {
        months: 4,
        method: ForecastMethod::Recurring,
        history_months: 6,
        adjustments: Vec::new(),
        recurrence_changes: Vec::new(),
    };
    let forecast = module.forecasts.forecast(today(), &request).await.unwrap();
    assert_eq!(forecast.months.len(), 4);
    assert_eq!(forecast.months[3].total, 100_000 + 3 * 200_000);
    assert_eq!(forecast.goals_reached.len(), 1);
}

#[tokio::test]
async fn export_and_import_roundtrip() {
    let source = TestDatabase::new().await;
    let source_module = module(&source);
    let main = account(&source_module, "Principal", 100_000).await;
    source_module
        .transactions
        .create(transaction(main.id, EntryKind::Outcome, 1_000, today()))
        .await
        .unwrap();

    let file = source.dir.join("export.json");
    source_module
        .backups
        .export(&file, Some("export-secret".into()))
        .await
        .unwrap();
    assert!(
        std::fs::read(&file)
            .unwrap()
            .starts_with(b"age-encryption.org/")
    );

    let target = TestDatabase::new().await;
    let target_module = module(&target);
    assert!(target_module.backups.inspect(&file, None).await.is_err());
    assert!(
        target_module
            .backups
            .inspect(&file, Some("wrong".into()))
            .await
            .is_err()
    );

    let preview = target_module
        .backups
        .inspect(&file, Some("export-secret".into()))
        .await
        .unwrap();
    assert!(preview.conflicts > 0);

    target_module
        .backups
        .import(&file, Some("export-secret".into()), true)
        .await
        .unwrap();
    let accounts = target_module.accounts.list(today(), false).await.unwrap();
    assert_eq!(accounts[0].balance, 99_000);

    let merge = target_module
        .backups
        .import(&file, Some("export-secret".into()), false)
        .await
        .unwrap();
    assert_eq!(merge.inserted, 0);

    let garbage = target.dir.join("garbage.json");
    std::fs::write(&garbage, b"{\"format\":\"other\"}").unwrap();
    assert!(matches!(
        target_module.backups.inspect(&garbage, None).await,
        Err(AppError::InvalidBackup(_))
    ));
}

#[tokio::test]
async fn snapshots_are_encrypted_and_verifiable() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    account(&module, "Principal", 100_000).await;

    let backup = module.backups.create_backup("").await.unwrap();
    let header = std::fs::read(&backup.path).unwrap();
    assert!(!header.starts_with(b"SQLite format 3"));

    module.backups.verify(&backup.path, None).await.unwrap();
    assert!(matches!(
        module
            .backups
            .verify(&backup.path, Some("wrong".into()))
            .await,
        Err(AppError::WrongPassword)
    ));
    assert_eq!(module.backups.list().await.unwrap().len(), 1);
}

#[tokio::test]
async fn rekey_changes_password() {
    let db = TestDatabase::new().await;
    let path = db.dir.join("test.db");
    db.pool.close().await;

    crate::database::rekey(&path, PASSWORD, "new-password-123")
        .await
        .unwrap();
    assert!(matches!(
        crate::database::verify(&path, PASSWORD).await,
        Err(AppError::WrongPassword)
    ));
    crate::database::verify(&path, "new-password-123")
        .await
        .unwrap();
}

#[tokio::test]
async fn reports_and_reconciliation_work_together() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 10_000).await;
    module
        .transactions
        .create(transaction(main.id, EntryKind::Outcome, 2_000, today()))
        .await
        .unwrap();

    let summary = module
        .reports
        .month_summary(today(), today())
        .await
        .unwrap();
    assert_eq!(summary.outcome, 2_000);
    assert_eq!(summary.categories[0].current, 2_000);

    let history = module
        .reports
        .balance_history(today(), 3, None)
        .await
        .unwrap();
    assert_eq!(history.last().unwrap().balance, 8_000);

    let reconciliation = module
        .reconciliation
        .reconcile(main.id, today(), 8_000, Some(add_months(today(), -1)))
        .await
        .unwrap();
    assert_eq!(reconciliation.calculated_balance, 8_000);
    let status = module
        .reconciliation
        .status(main.id, today())
        .await
        .unwrap();
    assert_eq!(status.unconfirmed_count, 0);
}
