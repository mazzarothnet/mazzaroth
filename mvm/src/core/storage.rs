use alloy_rlp::{Decodable, Encodable};
use utils::error::Result;

pub trait DbStorageTransaction {
    fn set_data<K: Encodable, V: Encodable>(&mut self, key: K, value: V) -> Result<()>;
    fn batch_read<K: Encodable, V: Decodable>(&mut self, keys: Vec<K>) -> Result<Vec<V>>;
    fn delete_data<K: Encodable>(&mut self, key: K) -> Result<()>;
    fn commit(self) -> Result<()>;
}

pub trait DbStorage {
    type Transaction<'a>: DbStorageTransaction
    where
        Self: 'a;
    fn begin_transaction(&self) -> Result<Self::Transaction<'_>>;
}
