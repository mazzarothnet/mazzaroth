use alloy_rlp::{Decodable, Encodable};
use consensus::types::AccountKey;
use utils::error::Result;
use crate::models::account::Account;

pub trait DbStorage {
    fn get_data<K: Encodable, V: Decodable>(&self, key: K) -> Result<Option<V>>;
    fn set_data<K: Encodable, V: Encodable>(&self, key: K, value: V) -> Result<()>;
    fn delete_data<K: Encodable>(&self, key: K) -> Result<()>;
}

pub struct VmStorage<S: DbStorage> {
    account_storage: S,
    mt_storage: S,
}

impl<S: DbStorage> VmStorage<S> {
    pub fn new(account_storage: S, mt_storage: S) -> Self {
        Self {
            account_storage,
            mt_storage,
        }
    }

    pub fn get_account(&self, key: AccountKey) -> Result<Option<Account>> {
        let account = self.account_storage.get_data(key)?;
        Ok(account)
    }

    pub fn set_account(&self, key: AccountKey, account: Account) -> Result<()> {
        self.account_storage.set_data(key, account)?;
        Ok(())
    }
}
