use std::path::PathBuf;

use age::secrecy::SecretString;
use sqlx::SqlitePool;

use crate::managers::accounts::AccountsManager;
use crate::managers::alerts::AlertsManager;
use crate::managers::backups::BackupsManager;
use crate::managers::categories::CategoriesManager;
use crate::managers::credits::CreditsManager;
use crate::managers::forecasts::ForecastsManager;
use crate::managers::goals::GoalsManager;
use crate::managers::purchases::PurchasesManager;
use crate::managers::reconciliation::ReconciliationManager;
use crate::managers::recurrences::RecurrencesManager;
use crate::managers::reports::ReportsManager;
use crate::managers::search::SearchManager;
use crate::managers::settings::SettingsManager;
use crate::managers::templates::TemplatesManager;
use crate::managers::transactions::TransactionsManager;
use crate::managers::transfers::TransfersManager;
use crate::repositories::accounts::AccountsRepository;
use crate::repositories::alerts::AlertsRepository;
use crate::repositories::backups::BackupsRepository;
use crate::repositories::categories::CategoriesRepository;
use crate::repositories::credits::CreditsRepository;
use crate::repositories::goals::GoalsRepository;
use crate::repositories::reconciliations::ReconciliationsRepository;
use crate::repositories::recurrences::RecurrencesRepository;
use crate::repositories::reports::ReportsRepository;
use crate::repositories::saved_filters::SavedFiltersRepository;
use crate::repositories::settings::SettingsRepository;
use crate::repositories::templates::TemplatesRepository;
use crate::repositories::transactions::TransactionsRepository;
use crate::repositories::transfers::TransfersRepository;
use crate::repositories::wishlist::WishlistRepository;

pub struct AppContext {
    pub data_dir: PathBuf,
    pub password: SecretString,
}

pub struct Module {
    pub accounts: AccountsManager,
    pub categories: CategoriesManager,
    pub transactions: TransactionsManager,
    pub transfers: TransfersManager,
    pub recurrences: RecurrencesManager,
    pub credits: CreditsManager,
    pub goals: GoalsManager,
    pub forecasts: ForecastsManager,
    pub purchases: PurchasesManager,
    pub reports: ReportsManager,
    pub reconciliation: ReconciliationManager,
    pub templates: TemplatesManager,
    pub settings: SettingsManager,
    pub search: SearchManager,
    pub alerts: AlertsManager,
    pub backups: BackupsManager,
}

impl Module {
    pub fn new(pool: SqlitePool, context: AppContext) -> Self {
        let accounts = AccountsRepository::new(pool.clone());
        let categories = CategoriesRepository::new(pool.clone());
        let transactions = TransactionsRepository::new(pool.clone());
        let transfers = TransfersRepository::new(pool.clone());
        let recurrences = RecurrencesRepository::new(pool.clone());
        let credits = CreditsRepository::new(pool.clone());
        let goals = GoalsRepository::new(pool.clone());
        let reports = ReportsRepository::new(pool.clone());
        let settings = SettingsRepository::new(pool.clone());

        let credits_manager = CreditsManager::new(
            credits.clone(),
            recurrences.clone(),
            transactions.clone(),
            categories.clone(),
        );
        let recurrences_manager = RecurrencesManager::new(recurrences, credits_manager.clone());
        let transactions_manager =
            TransactionsManager::new(transactions.clone(), categories.clone(), transfers.clone());
        let forecasts = ForecastsManager::new(
            accounts.clone(),
            transactions.clone(),
            recurrences_manager.clone(),
            reports.clone(),
            goals.clone(),
            credits.clone(),
        );

        Self {
            accounts: AccountsManager::new(accounts.clone()),
            categories: CategoriesManager::new(categories.clone()),
            transfers: TransfersManager::new(transfers, accounts.clone()),
            goals: GoalsManager::new(goals.clone(), forecasts.clone()),
            purchases: PurchasesManager::new(
                WishlistRepository::new(pool.clone()),
                forecasts.clone(),
            ),
            reports: ReportsManager::new(
                reports,
                accounts.clone(),
                credits.clone(),
                transactions.clone(),
                recurrences_manager.clone(),
            ),
            reconciliation: ReconciliationManager::new(ReconciliationsRepository::new(
                pool.clone(),
            )),
            templates: TemplatesManager::new(TemplatesRepository::new(pool.clone())),
            settings: SettingsManager::new(settings.clone()),
            search: SearchManager::new(
                accounts.clone(),
                categories,
                transactions,
                credits,
                goals.clone(),
                SavedFiltersRepository::new(pool.clone()),
            ),
            alerts: AlertsManager::new(
                AlertsRepository::new(pool.clone()),
                settings.clone(),
                recurrences_manager.clone(),
                accounts,
                goals,
                forecasts.clone(),
            ),
            backups: BackupsManager::new(
                BackupsRepository::new(pool),
                settings,
                transactions_manager.clone(),
                context.data_dir,
                context.password,
            ),
            transactions: transactions_manager,
            recurrences: recurrences_manager,
            credits: credits_manager,
            forecasts,
        }
    }
}
