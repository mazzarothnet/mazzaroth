use alloy_rlp::{Decodable, Encodable};
use anyhow::Context;
use mvm::core::storage::{DbStorage, DbStorageTransaction};
use rocksdb::{SingleThreaded, Transaction, TransactionDB};
use utils::error::Result;

pub struct RocksDbStorage(pub TransactionDB<SingleThreaded>);

impl RocksDbStorage {
    pub fn new(path: &str) -> Result<Self> {
        let db = TransactionDB::open_default(path).with_context(|| "Failed to open database")?;
        Ok(Self(db))
    }
}

impl DbStorage for RocksDbStorage {
    type Transaction<'a> = RocksDbTransaction<'a>;
    fn begin_transaction(&self) -> Result<Self::Transaction<'_>> {
        let tx = self.0.transaction();
        Ok(RocksDbTransaction::new(tx))
    }
}

pub struct RocksDbTransaction<'db>(pub Transaction<'db, TransactionDB<SingleThreaded>>);

impl<'db> RocksDbTransaction<'db> {
    pub fn new(trans: Transaction<'db, TransactionDB<SingleThreaded>>) -> Self {
        Self(trans)
    }
}

impl<'db> DbStorageTransaction for RocksDbTransaction<'db> {
    fn get_data<K: Encodable, V: Decodable>(&mut self, key: K) -> Result<Option<V>> {
        let mut key_bytes = Vec::new();
        key.encode(&mut key_bytes);
        let value = self
            .0
            .get(key_bytes)
            .with_context(|| "Failed to get data")?;
        if let Some(value) = value {
            let v: V = Decodable::decode(&mut value.as_slice())
                .with_context(|| "Failed to decode data")?;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }
    fn set_data<K: Encodable, V: Encodable>(&mut self, key: K, value: V) -> Result<()> {
        let mut key_bytes = Vec::new();
        key.encode(&mut key_bytes);
        let mut value_bytes = Vec::new();
        value.encode(&mut value_bytes);
        self.0
            .put(key_bytes, value_bytes)
            .with_context(|| "Failed to set data")?;
        Ok(())
    }
    fn delete_data<K: Encodable>(&mut self, key: K) -> Result<()> {
        let mut key_bytes = Vec::new();
        key.encode(&mut key_bytes);
        self.0
            .delete(key_bytes)
            .with_context(|| "Failed to delete data")?;
        Ok(())
    }

    fn commit(self) -> Result<()> {
        self.0
            .commit()
            .with_context(|| "Failed to commit transaction")?;
        Ok(())
    }
}
