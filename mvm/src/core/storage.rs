// use crate::models::account::Account;
// use consensus::types::AccountKey;
use alloy_rlp::{Decodable, Encodable};
use utils::error::Result;

pub trait DbStorageTransaction {
    fn get_data<K: Encodable, V: Decodable>(&mut self, key: K) -> Result<Option<V>>;
    fn set_data<K: Encodable, V: Encodable>(&mut self, key: K, value: V) -> Result<()>;
    fn delete_data<K: Encodable>(&mut self, key: K) -> Result<()>;
    fn commit(&mut self) -> Result<()>;
}

pub trait DbStorage {
    fn begin_transaction<T: DbStorageTransaction>(&self) -> Result<T>;
}

// pub struct VmStorage<S: DbStorage> {
//     account_storage: S,
//     mt_storage: S,
// }

// impl<S: DbStorage> VmStorage<S> {
//     pub fn new(account_storage: S, mt_storage: S) -> Self {
//         Self {
//             account_storage,
//             mt_storage,
//         }
//     }

//     pub fn get_account(&self, key: AccountKey) -> Result<Option<Account>> {
//         let account = self.account_storage.get_data(key)?;
//         Ok(account)
//     }

//     pub fn set_account(&self, key: AccountKey, account: Account) -> Result<()> {
//         self.account_storage.set_data(key, account)?;
//         Ok(())
//     }
// }
