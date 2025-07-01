use consensus::types::AccountKey;
use utils::error::Result;

use crate::models::account::Account;

pub trait AccountStorage {
    fn get_account(&self, key: AccountKey) -> Result<Account>;
}