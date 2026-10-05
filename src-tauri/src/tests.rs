use age::secrecy::SecretString;
use chrono::{Days, NaiveDate};
use uuid::Uuid;

use crate::database::testing::TestDatabase;
use crate::errors::AppError;
use crate::managers::forecasts::ForecastRequest;
use crate::managers::purchases::PurchaseRequest;
use crate::models::dates::{add_months, today};
use crate::models::{
    Account, AccountInput, AccountKind, CreditInput, EntryKind, ForecastMethod, Frequency,
    FrequencyUnit, Interest, InterestTier, RecurrenceInput, TransactionFilter, TransactionInput,
    TransactionKind, TransferInput, WishlistInput,
};
use crate::module::{AppContext, Module};

const PASSWORD: &str = "test-password";

fn module(db: &TestDatabase) -> Module {
    let context = AppContext {
        data_dir: db.dir.clone(),
        password: SecretString::from(PASSWORD.to_string()),
        simulation: false,
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
        interest: None,
        overdraft_limit: 0,
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
        one_off: false,
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
        to_account_id: None,
        variable_amount: false,
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
        to_account_id: None,
        variable_amount: false,
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
async fn history_does_not_double_count_recurring_flows() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 0).await;

    for months_ago in 1..=3 {
        let date = add_months(today(), -months_ago);
        let salary = TransactionInput {
            description: "Salário".into(),
            ..transaction(main.id, EntryKind::Income, 200_000, date)
        };
        module.transactions.create(salary).await.unwrap();
        module
            .transactions
            .create(transaction(main.id, EntryKind::Outcome, 30_000, date))
            .await
            .unwrap();
    }

    let salary = RecurrenceInput {
        account_id: main.id,
        category_id: None,
        kind: EntryKind::Income,
        amount: 200_000,
        description: "salário".into(),
        start_date: add_months(today(), 1),
        end_date: None,
        frequency: monthly(),
        to_account_id: None,
        variable_amount: false,
    };
    module.recurrences.create(salary).await.unwrap();

    let request = ForecastRequest {
        months: 3,
        method: ForecastMethod::History,
        history_months: 3,
        adjustments: Vec::new(),
        recurrence_changes: Vec::new(),
    };
    let forecast = module.forecasts.forecast(today(), &request).await.unwrap();
    assert_eq!(forecast.months[1].income, 200_000);
    assert_eq!(forecast.months[1].variable_income, 0);
    assert_eq!(forecast.months[1].variable_outcome, 30_000);
}

#[tokio::test]
async fn corrupted_files_are_rejected() {
    let db = TestDatabase::new().await;
    let corrupted = db.dir.join("corrupted.db");
    std::fs::write(&corrupted, vec![7u8; 8192]).unwrap();

    let opened = crate::database::open(&corrupted, PASSWORD, &db.dir).await;
    assert!(matches!(opened, Err(AppError::WrongPassword)));
    assert!(crate::database::verify(&corrupted, PASSWORD).await.is_err());
    assert!(
        crate::database::verify(&db.dir.join("missing.db"), PASSWORD)
            .await
            .is_err()
    );
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
    crate::database::close(&db.pool).await;

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

#[tokio::test]
async fn balance_records_ignore_future_and_other_accounts() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 10_000).await;
    let other = account(&module, "Outra", 50_000).await;
    let earlier = add_months(today(), -14);
    let tomorrow = today().checked_add_days(Days::new(1)).unwrap();
    for (kind, amount, date) in [
        (EntryKind::Outcome, 9_000, earlier),
        (EntryKind::Income, 30_000, today()),
        (EntryKind::Income, 99_000, tomorrow),
    ] {
        module
            .transactions
            .create(transaction(main.id, kind, amount, date))
            .await
            .unwrap();
    }

    let records = module
        .reports
        .balance_records(today(), 1, Some(main.id))
        .await
        .unwrap();
    let all = &records[0];
    assert_eq!((all.low.balance, all.low.date), (1_000, earlier));
    assert_eq!((all.high.balance, all.high.date), (31_000, today()));

    let total = module
        .reports
        .balance_records(today(), 1, None)
        .await
        .unwrap();
    assert_eq!(total[0].high.balance, 31_000 + other.initial_balance);
    assert_eq!(total[3].low.balance, 1_000 + other.initial_balance);
}

#[tokio::test]
async fn recurring_transfers_move_money_between_accounts() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 100_000).await;
    let savings = account(&module, "Poupança", 0).await;

    let input = RecurrenceInput {
        account_id: main.id,
        category_id: None,
        kind: EntryKind::Income,
        amount: 20_000,
        description: "Poupar".into(),
        start_date: today(),
        end_date: None,
        frequency: monthly(),
        to_account_id: Some(savings.id),
        variable_amount: false,
    };
    let recurrence = module.recurrences.create(input.clone()).await.unwrap();
    assert_eq!(recurrence.kind, EntryKind::Outcome);
    assert_eq!(recurrence.to_account_name.as_deref(), Some("Poupança"));

    module.recurrences.materialize_due(today()).await.unwrap();
    let page = module
        .transactions
        .page(&TransactionFilter::default(), 50, 0)
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
    assert_eq!((page.income, page.outcome), (0, 0));
    assert!(
        page.items
            .iter()
            .all(|t| t.kind == TransactionKind::Transfer && t.recurrence_id == Some(recurrence.id))
    );
    let leg = page.items.iter().find(|t| t.account_id == main.id).unwrap();
    assert_eq!(leg.counterpart_account_id, Some(savings.id));

    let request = ForecastRequest {
        months: 3,
        method: ForecastMethod::Recurring,
        history_months: 6,
        adjustments: Vec::new(),
        recurrence_changes: Vec::new(),
    };
    let forecast = module.forecasts.forecast(today(), &request).await.unwrap();
    let last = forecast.months.last().unwrap();
    assert_eq!(last.total, 100_000);
    assert_eq!((last.income, last.outcome), (0, 0));

    let same = RecurrenceInput {
        to_account_id: Some(main.id),
        ..input
    };
    assert!(module.recurrences.create(same).await.is_err());
}

#[tokio::test]
async fn variable_recurrences_wait_for_confirmation_and_endings_warn() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 0).await;

    let salary = RecurrenceInput {
        account_id: main.id,
        category_id: None,
        kind: EntryKind::Income,
        amount: 150_000,
        description: "Salário".into(),
        start_date: today(),
        end_date: Some(today().checked_add_days(Days::new(2)).unwrap()),
        frequency: monthly(),
        to_account_id: None,
        variable_amount: true,
    };
    let recurrence = module.recurrences.create(salary).await.unwrap();
    assert!(recurrence.variable_amount);
    module.recurrences.materialize_due(today()).await.unwrap();

    let pending = module
        .transactions
        .pending_confirmations(today())
        .await
        .unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].amount, 150_000);

    let alerts = module.alerts.alerts(today()).await.unwrap();
    assert!(alerts.iter().any(|a| a.key.starts_with("ending:")));

    module
        .transactions
        .set_confirmed(&[pending[0].id], true)
        .await
        .unwrap();
    assert!(
        module
            .transactions
            .pending_confirmations(today())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn one_off_movements_stay_out_of_averages() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 0).await;
    let last_month = add_months(today(), -1);
    module
        .transactions
        .create(transaction(main.id, EntryKind::Outcome, 10_000, last_month))
        .await
        .unwrap();
    let air_conditioning = TransactionInput {
        one_off: true,
        ..transaction(main.id, EntryKind::Outcome, 200_000, last_month)
    };
    let created = module.transactions.create(air_conditioning).await.unwrap();
    assert!(created.one_off);

    let summary = module
        .reports
        .month_summary(today(), today())
        .await
        .unwrap();
    assert_eq!(summary.previous_outcome, 10_000);
    assert_eq!(summary.categories[0].previous, 10_000);

    let request = ForecastRequest {
        months: 2,
        method: ForecastMethod::History,
        history_months: 1,
        adjustments: Vec::new(),
        recurrence_changes: Vec::new(),
    };
    let forecast = module.forecasts.forecast(today(), &request).await.unwrap();
    assert_eq!(forecast.months[1].variable_outcome, 10_000);
}

#[tokio::test]
async fn interest_accounts_round_trip_and_feed_forecasts() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let input = AccountInput {
        name: "Fundo de emergência".into(),
        kind: AccountKind::Savings,
        currency: "EUR".into(),
        initial_balance: 1_200_000,
        color: None,
        icon: None,
        interest: Some(Interest {
            period_months: 1,
            tiers: vec![
                InterestTier {
                    min_balance: 0,
                    rate: 1.0,
                },
                InterestTier {
                    min_balance: 500_000,
                    rate: 1.25,
                },
            ],
        }),
        overdraft_limit: 0,
    };
    let fund = module.accounts.create(today(), input).await.unwrap();
    let interest = fund.interest.clone().unwrap();
    assert_eq!(interest.tiers.len(), 2);
    assert_eq!(interest.rate_for(fund.balance), 1.25);

    let meal = AccountInput {
        name: "Cartão refeição".into(),
        kind: AccountKind::Meal,
        currency: "EUR".into(),
        initial_balance: 0,
        color: None,
        icon: None,
        interest: None,
        overdraft_limit: 0,
    };
    let meal = module.accounts.create(today(), meal).await.unwrap();
    assert_eq!(meal.kind, AccountKind::Meal);
    assert!(meal.interest.is_none());

    let request = ForecastRequest {
        months: 1,
        method: ForecastMethod::Recurring,
        history_months: 6,
        adjustments: Vec::new(),
        recurrence_changes: Vec::new(),
    };
    let forecast = module.forecasts.forecast(today(), &request).await.unwrap();
    assert_eq!(forecast.months[0].interest, 900);
}

#[tokio::test]
async fn dismissed_alerts_stay_hidden() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 0).await;
    let rent = RecurrenceInput {
        account_id: main.id,
        category_id: None,
        kind: EntryKind::Outcome,
        amount: 50_000,
        description: "Renda".into(),
        start_date: today().checked_add_days(Days::new(1)).unwrap(),
        end_date: None,
        frequency: monthly(),
        to_account_id: None,
        variable_amount: false,
    };
    module.recurrences.create(rent).await.unwrap();

    let alerts = module.alerts.alerts(today()).await.unwrap();
    let key = alerts
        .iter()
        .find(|a| a.key.starts_with("occurrence:"))
        .unwrap()
        .key
        .clone();
    module.alerts.dismiss(&key).await.unwrap();
    module.alerts.dismiss(&key).await.unwrap();
    let alerts = module.alerts.alerts(today()).await.unwrap();
    assert!(alerts.iter().all(|a| a.key != key));
}

#[tokio::test]
async fn saved_forecast_method_migrates_to_recurring() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let settings = crate::models::Settings {
        forecast_method: ForecastMethod::History,
        ..crate::models::Settings::default()
    };
    module.settings.update(settings).await.unwrap();

    sqlx::raw_sql(include_str!("../migrations/0007_recurring_forecasts.sql"))
        .execute(&db.pool)
        .await
        .unwrap();
    let settings = module.settings.get().await.unwrap();
    assert_eq!(settings.forecast_method, ForecastMethod::Recurring);
    assert_eq!(settings.currency, "EUR");
}

#[tokio::test]
async fn credit_cards_stay_out_of_the_total_balance() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    account(&module, "Principal", 100_000).await;
    let card = AccountInput {
        name: "Cartão de crédito".into(),
        kind: AccountKind::CreditCard,
        currency: "EUR".into(),
        initial_balance: -30_000,
        color: None,
        icon: None,
        interest: None,
        overdraft_limit: 150_000,
    };
    let card = module.accounts.create(today(), card).await.unwrap();
    assert_eq!(card.overdraft_limit, 150_000);

    let summary = module.reports.balance_summary(today()).await.unwrap();
    assert_eq!(summary.total, 100_000);
    assert_eq!(summary.debt, 30_000);

    let records = module
        .reports
        .balance_records(today(), 1, None)
        .await
        .unwrap();
    assert_eq!(records[0].high.balance, 100_000);

    let request = ForecastRequest {
        months: 1,
        method: ForecastMethod::Recurring,
        history_months: 6,
        adjustments: Vec::new(),
        recurrence_changes: Vec::new(),
    };
    let forecast = module.forecasts.forecast(today(), &request).await.unwrap();
    assert_eq!(forecast.months[0].total, 100_000);
}

#[tokio::test]
async fn purchases_wait_until_future_payments_stay_covered() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = AccountInput {
        name: "Principal".into(),
        kind: AccountKind::Bank,
        currency: "EUR".into(),
        initial_balance: 50_000,
        color: None,
        icon: None,
        interest: None,
        overdraft_limit: 60_000,
    };
    let main = module.accounts.create(today(), main).await.unwrap();
    let tomorrow = today().checked_add_days(Days::new(1)).unwrap();
    let in_ten_days = today().checked_add_days(Days::new(10)).unwrap();
    for (kind, amount, description, start_date) in [
        (EntryKind::Outcome, 40_000, "Renda", tomorrow),
        (EntryKind::Income, 100_000, "Salário", in_ten_days),
    ] {
        let recurrence = RecurrenceInput {
            account_id: main.id,
            category_id: None,
            kind,
            amount,
            description: description.into(),
            start_date,
            end_date: None,
            frequency: monthly(),
            to_account_id: None,
            variable_amount: false,
        };
        module.recurrences.create(recurrence).await.unwrap();
    }

    let request = PurchaseRequest {
        account_id: main.id,
        amount: 30_000,
        margin: 0,
        allow_overdraft: false,
        months: 3,
    };
    let plan = module.purchases.plan(today(), &request).await.unwrap();
    assert_eq!(plan.balance, 50_000);
    assert_eq!(plan.lowest_if_today, 10_000 - 30_000);
    assert_eq!(plan.earliest, Some(in_ten_days));

    let overdraft = PurchaseRequest {
        allow_overdraft: true,
        ..request
    };
    let plan = module.purchases.plan(today(), &overdraft).await.unwrap();
    assert_eq!(plan.floor, -60_000);
    assert_eq!(plan.earliest, Some(today()));

    for (name, amount) in [("Telemóvel", 30_000), ("Portátil", 80_000)] {
        let item = WishlistInput {
            name: name.into(),
            amount,
            account_id: main.id,
            priority: 2,
            notes: None,
        };
        module.purchases.create(item).await.unwrap();
    }
    let planned = module.purchases.wishlist(today(), 0, false).await.unwrap();
    assert_eq!(planned[0].1, Some(in_ten_days));
    let second = planned[1].1.unwrap();
    assert!(second > in_ten_days);

    let bought = module
        .purchases
        .set_purchased(planned[0].0.id, true)
        .await
        .unwrap();
    assert!(bought.purchased_at.is_some());
}

#[tokio::test]
async fn meal_cards_stay_out_of_totals_and_estimates() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    account(&module, "Principal", 100_000).await;
    let meal = AccountInput {
        name: "Refeição".into(),
        kind: AccountKind::Meal,
        currency: "EUR".into(),
        initial_balance: 19_000,
        color: None,
        icon: None,
        interest: None,
        overdraft_limit: 0,
    };
    let meal = module.accounts.create(today(), meal).await.unwrap();
    let allowance = module
        .categories
        .list(true)
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.name == "Subsídio de refeição")
        .unwrap();
    let subsidy = RecurrenceInput {
        account_id: meal.id,
        category_id: Some(allowance.id),
        kind: EntryKind::Income,
        amount: 19_800,
        description: "Subsídio de refeição".into(),
        start_date: today().checked_add_days(Days::new(1)).unwrap(),
        end_date: None,
        frequency: monthly(),
        to_account_id: None,
        variable_amount: false,
    };
    module.recurrences.create(subsidy).await.unwrap();

    let summary = module.reports.balance_summary(today()).await.unwrap();
    assert_eq!(summary.total, 100_000);
    assert_eq!(summary.debt, 0);

    let request = ForecastRequest {
        months: 3,
        method: ForecastMethod::Recurring,
        history_months: 6,
        adjustments: Vec::new(),
        recurrence_changes: Vec::new(),
    };
    let forecast = module.forecasts.forecast(today(), &request).await.unwrap();
    assert!(forecast.months.iter().all(|m| m.income == 0));
    assert_eq!(forecast.months[2].total, 100_000);

    let month = module
        .reports
        .month_summary(today(), add_months(today(), 1))
        .await
        .unwrap();
    assert_eq!(month.pending_income, 0);
}

#[tokio::test]
async fn simulations_work_on_a_disposable_copy() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = populated(&module).await;

    let copy = db.dir.join("simulation.db");
    crate::database::snapshot(&db.pool, &copy).await.unwrap();
    let pool = crate::database::open(&copy, PASSWORD, &db.dir.join("backups"))
        .await
        .unwrap();
    let simulated = Module::new(
        pool.clone(),
        AppContext {
            data_dir: db.dir.clone(),
            password: SecretString::from(PASSWORD.to_string()),
            simulation: true,
        },
    );
    simulated
        .transactions
        .create(transaction(main.id, EntryKind::Outcome, 50_000, today()))
        .await
        .unwrap();
    let real = module.accounts.get(today(), main.id).await.unwrap();
    let virtual_account = simulated.accounts.get(today(), main.id).await.unwrap();
    assert_eq!(virtual_account.balance, real.balance - 50_000);

    assert!(simulated.backups.create_backup("").await.is_err());
    assert!(simulated.backups.auto_backup().await.unwrap().is_none());
    assert!(
        simulated
            .backups
            .export(&db.dir.join("out.gxbackup"), None)
            .await
            .is_err()
    );
    assert!(
        crate::database::verify(&copy, "wrong-password")
            .await
            .is_err()
    );

    crate::database::close(&pool).await;
    crate::database::discard(&copy).unwrap();
    assert!(!copy.exists());
    assert!(!db.dir.join("simulation.db-wal").exists());
}

async fn populated(module: &Module) -> Account {
    let main = account(module, "Principal", 100_000).await;
    let savings = account(module, "Poupança", 0).await;
    module
        .transactions
        .create(transaction(main.id, EntryKind::Outcome, 1_000, today()))
        .await
        .unwrap();
    let transfer = TransferInput {
        from_account_id: main.id,
        to_account_id: savings.id,
        amount: 5_000,
        date: today(),
        description: "Poupar".into(),
    };
    module.transfers.create(today(), transfer).await.unwrap();
    let rent = RecurrenceInput {
        account_id: main.id,
        category_id: None,
        kind: EntryKind::Outcome,
        amount: 50_000,
        description: "Renda".into(),
        start_date: add_months(today(), 1),
        end_date: None,
        frequency: monthly(),
        to_account_id: None,
        variable_amount: false,
    };
    module.recurrences.create(rent).await.unwrap();
    let goal = crate::models::GoalInput {
        name: "Fundo".into(),
        account_id: savings.id,
        target_amount: 100_000,
        target_date: None,
    };
    module.goals.create(today(), goal).await.unwrap();
    main
}

fn horizon() -> ForecastRequest {
    ForecastRequest {
        months: 6,
        method: ForecastMethod::History,
        history_months: 3,
        adjustments: Vec::new(),
        recurrence_changes: Vec::new(),
    }
}

#[tokio::test]
async fn migration_to_another_machine_keeps_everything() {
    let source = TestDatabase::new().await;
    let source_module = module(&source);
    populated(&source_module).await;
    let file = source.dir.join("export.gxbackup");
    source_module.backups.export(&file, None).await.unwrap();

    let target = TestDatabase::new().await;
    let target_module = module(&target);
    target_module
        .backups
        .import(&file, None, true)
        .await
        .unwrap();

    let dump = |pool| async move {
        crate::repositories::backups::BackupsRepository::new(pool)
            .dump()
            .await
            .unwrap()
    };
    assert_eq!(
        dump(source.pool.clone()).await,
        dump(target.pool.clone()).await
    );

    let before = source_module
        .forecasts
        .forecast(today(), &horizon())
        .await
        .unwrap();
    let after = target_module
        .forecasts
        .forecast(today(), &horizon())
        .await
        .unwrap();
    let totals = |f: &crate::managers::forecasts::Forecast| {
        f.months.iter().map(|m| m.total).collect::<Vec<_>>()
    };
    assert_eq!(totals(&before), totals(&after));
}

#[tokio::test]
async fn restore_replaces_database_and_keeps_safety_copy() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = populated(&module).await;
    let backup = module.backups.create_backup("").await.unwrap();
    module
        .transactions
        .create(transaction(main.id, EntryKind::Outcome, 7_000, today()))
        .await
        .unwrap();
    crate::database::close(&db.pool).await;

    let database = db.dir.join("test.db");
    let safety = db.dir.join("safety");
    crate::database::replace(&database, &backup.path, &safety).unwrap();
    assert_eq!(std::fs::read_dir(&safety).unwrap().count(), 1);

    let pool = crate::database::open(&database, PASSWORD, &safety)
        .await
        .unwrap();
    let restored = Module::new(
        pool,
        AppContext {
            data_dir: db.dir.clone(),
            password: SecretString::from(PASSWORD.to_string()),
            simulation: false,
        },
    );
    let balance = restored
        .accounts
        .get(today(), main.id)
        .await
        .unwrap()
        .balance;
    assert_eq!(balance, 100_000 - 1_000 - 5_000);
}

#[tokio::test]
async fn reset_starts_clean_with_the_same_password() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    populated(&module).await;
    let categories = module.categories.list(true).await.unwrap().len();
    crate::database::close(&db.pool).await;

    let database = db.dir.join("test.db");
    let safety = db.dir.join("safety");
    crate::database::reset(&database, &safety).unwrap();
    assert_eq!(std::fs::read_dir(&safety).unwrap().count(), 1);

    let pool = crate::database::open(&database, PASSWORD, &safety)
        .await
        .unwrap();
    let fresh = Module::new(
        pool,
        AppContext {
            data_dir: db.dir.clone(),
            password: SecretString::from(PASSWORD.to_string()),
            simulation: false,
        },
    );
    assert!(fresh.accounts.list(today(), true).await.unwrap().is_empty());
    assert!(fresh.recurrences.list().await.unwrap().is_empty());
    assert_eq!(fresh.categories.list(true).await.unwrap().len(), categories);
}

#[tokio::test]
async fn interrupted_writes_leave_database_consistent() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = populated(&module).await;

    let mut tx = db.pool.begin().await.unwrap();
    sqlx::query("UPDATE accounts SET initial_balance = 0 WHERE id = ?1")
        .bind(main.id.hyphenated())
        .execute(&mut *tx)
        .await
        .unwrap();
    drop(tx);
    crate::database::close(&db.pool).await;

    let path = db.dir.join("test.db");
    crate::database::verify(&path, PASSWORD).await.unwrap();
    let pool = crate::database::open(&path, PASSWORD, &db.dir)
        .await
        .unwrap();
    let initial: i64 = sqlx::query_scalar("SELECT initial_balance FROM accounts WHERE id = ?1")
        .bind(main.id.hyphenated())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(initial, 100_000);
}

#[tokio::test]
async fn introspection_fits_query_limits() {
    let db = TestDatabase::new().await;
    let context = AppContext {
        data_dir: db.dir.clone(),
        password: SecretString::from(PASSWORD.to_string()),
        simulation: false,
    };
    let schema = crate::graphql::schema(db.pool.clone(), context);
    let query = "query { __schema { types { name fields { name args { name type { ...Ref } } type { ...Ref } } } } } \
        fragment Ref on __Type { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name \
        ofType { kind name ofType { kind name ofType { kind name } } } } } } } }";
    let response = schema.execute(query).await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
}

#[tokio::test]
async fn credits_expose_history_and_link_one_recurrence() {
    let db = TestDatabase::new().await;
    let module = module(&db);
    let main = account(&module, "Principal", 1_000_000).await;
    let input = CreditInput {
        name: "Carro".into(),
        institution: None,
        principal: 600_000,
        opening_balance: 600_000,
        annual_rate: 6.0,
        installment: 50_000,
        frequency: monthly(),
        start_date: today(),
        end_date: None,
        installments: None,
        account_id: Some(main.id),
    };
    let credit = module.credits.create(today(), input, false).await.unwrap();
    let payment = crate::models::PaymentInput {
        credit_id: credit.id,
        date: today(),
        amount: 50_000,
        account_id: None,
        transaction_id: None,
        principal: None,
        interest: None,
    };
    module.credits.register_payment(payment).await.unwrap();

    let history = module
        .credits
        .balance_history(credit.id, today())
        .await
        .unwrap();
    assert_eq!(history[0].balance, 600_000 - 47_000);
    assert!(!history[0].projected);
    assert_eq!(history.last().unwrap().balance, 0);

    module
        .credits
        .add_recurrence(credit.id, today())
        .await
        .unwrap();
    assert!(matches!(
        module.credits.add_recurrence(credit.id, today()).await,
        Err(AppError::Conflict(_))
    ));
}
