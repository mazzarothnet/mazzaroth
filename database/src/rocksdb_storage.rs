use std::collections::{HashMap, HashSet};

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

pub struct RocksDbTransaction<'db> {
    transaction: Transaction<'db, TransactionDB<SingleThreaded>>,
    cache: HashMap<Vec<u8>, Vec<u8>>,
    deleted: HashSet<Vec<u8>>,
}

impl<'db> RocksDbTransaction<'db> {
    pub fn new(trans: Transaction<'db, TransactionDB<SingleThreaded>>) -> Self {
        Self {
            transaction: trans,
            cache: HashMap::new(),
            deleted: HashSet::new(),
        }
    }
}

impl<'db> DbStorageTransaction for RocksDbTransaction<'db> {
    fn get_data<K: Encodable, V: Decodable>(&mut self, key: K) -> Result<Option<V>> {
        let mut key_bytes = Vec::new();
        key.encode(&mut key_bytes);
        if self.deleted.contains(&key_bytes) {
            return Ok(None);
        }
        if let Some(value) = self.cache.get(&key_bytes) {
            let v: V = Decodable::decode(&mut value.as_slice())
                .with_context(|| "Failed to decode data")?;
            return Ok(Some(v));
        }

        let value = self
            .transaction
            .get(key_bytes.clone())
            .with_context(|| "Failed to get data")?;
        if let Some(value) = value {
            let v: V = Decodable::decode(&mut value.as_slice())
                .with_context(|| "Failed to decode data")?;
            self.cache.insert(key_bytes, value);
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
        self.deleted.remove(&key_bytes);
        self.cache.insert(key_bytes, value_bytes);
        Ok(())
    }
    fn delete_data<K: Encodable>(&mut self, key: K) -> Result<()> {
        let mut key_bytes = Vec::new();
        key.encode(&mut key_bytes);
        self.deleted.insert(key_bytes);
        Ok(())
    }

    fn commit(self) -> Result<()> {
        for (key, value) in self.cache {
            self.transaction
                .put(key, value)
                .with_context(|| "Failed to put data")?;
        }
        for key in self.deleted {
            self.transaction
                .delete(key)
                .with_context(|| "Failed to delete data")?;
        }
        self.transaction
            .commit()
            .with_context(|| "Failed to commit transaction")?;
        Ok(())
    }
}
