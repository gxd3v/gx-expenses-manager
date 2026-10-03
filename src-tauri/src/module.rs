use sqlx::SqlitePool;

use crate::managers::accounts::AccountsManager;
use crate::repositories::accounts::AccountsRepository;

pub struct Module {
    pub accounts: AccountsManager,
}

impl Module {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            accounts: AccountsManager::new(AccountsRepository::new(pool)),
        }
    }
}
