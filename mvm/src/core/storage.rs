use crate::models::account::Account;
use alloy_rlp::{Decodable, Encodable};
use consensus::types::{AccountKey, StateHash};
use utils::{error::Result, sha256::sha256_hash_rlp};

use super::merkle_tree::MerkleTree;

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
    mt_storage: MerkleTree<S>,
}

impl<S: DbStorage> VmStorage<S> {
    pub fn new(account_storage: S, mt_storage: S) -> Result<Self> {
        Ok(Self {
            account_storage,
            mt_storage: MerkleTree::new(mt_storage)?,
        })
    }

    pub fn get_account(&self, key: AccountKey) -> Result<Option<Account>> {
        let mut transaction: S::Transaction<'_> = self.account_storage.begin_transaction()?;
        let account = transaction.get_data(key)?;
        Ok(account)
    }

    pub fn update_account(
        &mut self,
        set_account: Vec<(AccountKey, Account)>,
        delete_account: Vec<AccountKey>,
    ) -> Result<()> {
        let mut transaction: S::Transaction<'_> = self.account_storage.begin_transaction()?;
        for (key, account) in set_account.iter() {
            transaction.set_data(key, account)?;
        }
        for key in delete_account.iter() {
            transaction.delete_data(key)?;
        }
        let set_account = set_account
            .into_iter()
            .map(|(key, account)| {
                let account_hash = sha256_hash_rlp(&account);
                (key, StateHash(account_hash))
            })
            .collect::<Vec<_>>();
        let mt_transaction = self.mt_storage.update_tree(set_account, delete_account)?;
        mt_transaction.commit()?;
        transaction.commit()?;
        Ok(())
    }

    pub fn get_state_root(&self) -> Result<StateHash> {
        self.mt_storage.get_state_root()
    }
}
