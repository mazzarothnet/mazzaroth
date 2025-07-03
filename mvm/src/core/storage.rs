use crate::models::account::Account;
use alloy_rlp::{Decodable, Encodable};
use consensus::types::{AccountKey, StateHash};
use utils::{error::Result, sha256::sha256_hash_rlp};

use crate::core::merkle_tries::Tire;

pub trait DbStorageTransaction {
    fn get_data<K: Encodable, V: Decodable>(&mut self, key: K) -> Result<Option<V>>;
    fn set_data<K: Encodable, V: Encodable>(&mut self, key: K, value: V) -> Result<()>;
    fn delete_data<K: Encodable>(&mut self, key: K) -> Result<()>;
    fn commit(self) -> Result<()>;
}

pub trait DbStorage {
    type Transaction<'a>: DbStorageTransaction
    where
        Self: 'a;
    fn begin_transaction(&self) -> Result<Self::Transaction<'_>>;
}

pub struct VmStorage<S: DbStorage> {
    account_storage: S,
    mt_storage: Tire<S>,
}

impl<S: DbStorage> VmStorage<S> {
    pub fn new(account_storage: S, mt_storage: S) -> Result<Self> {
        Ok(Self {
            account_storage,
            mt_storage: Tire::new(mt_storage)?,
        })
    }

    pub fn get_account(&self, key: AccountKey) -> Result<Option<Account>> {
        let mut transaction: S::Transaction<'_> = self.account_storage.begin_transaction()?;
        let account = transaction.get_data(key)?;
        Ok(account)
    }

    pub fn set_account(&mut self, key: AccountKey, account: Account) -> Result<()> {
        let account_hash = sha256_hash_rlp(&account);
        let mut transaction: S::Transaction<'_> = self.account_storage.begin_transaction()?;
        transaction.set_data(key, account)?;
        let mt_transaction = self
            .mt_storage
            .set_state_hash(key, StateHash(account_hash))?;
        mt_transaction.commit()?;
        transaction.commit()?;
        Ok(())
    }

    pub fn delete_account(&mut self, key: AccountKey) -> Result<()> {
        let mut transaction: S::Transaction<'_> = self.account_storage.begin_transaction()?;
        transaction.delete_data(key)?;
        let mt_transaction = self.mt_storage.delete_state_hash(key)?;
        mt_transaction.commit()?;
        transaction.commit()?;
        Ok(())
    }

    pub fn get_state_root(&self) -> StateHash {
        self.mt_storage.get_state_hash()
    }
}
