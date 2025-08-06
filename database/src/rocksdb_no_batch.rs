use std::collections::{HashMap, HashSet};

use alloy_rlp::{Decodable, Encodable};
use anyhow::Context;
// use log::info;
use mvm::core::storage::{DbStorage, DbStorageTransaction};
use rocksdb::{DB, WriteBatch};
use utils::error::Result;

pub struct RocksDbStorage(pub DB);

impl RocksDbStorage {
    pub fn new(path: &str) -> Result<Self> {
        let db = DB::open_default(path).with_context(|| "Failed to open database")?;
        Ok(Self(db))
    }
}

impl DbStorage for RocksDbStorage {
    type Transaction<'a> = RocksDbTransaction<'a>;
    fn begin_transaction(&self) -> Result<Self::Transaction<'_>> {
        Ok(RocksDbTransaction::new(&self.0))
    }

    fn get_data<K: Encodable, V: Decodable>(&self, key: &K) -> Result<Option<V>> {
        let mut key_bytes = Vec::new();
        key.encode(&mut key_bytes);
        let value = self
            .0
            .get(key_bytes.clone())
            .with_context(|| "Failed to get data")?;
        if let Some(value) = value {
            let v: V = Decodable::decode(&mut value.as_slice())
                .with_context(|| "Failed to decode data")?;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }

    fn set_data<K: Encodable, V: Encodable>(&self, key: &K, value: &V) -> Result<()> {
        let mut key_bytes = Vec::new();
        key.encode(&mut key_bytes);
        let mut value_bytes = Vec::new();
        value.encode(&mut value_bytes);
        self.0
            .put(key_bytes, value_bytes)
            .with_context(|| "Failed to set data")?;
        Ok(())
    }
}

pub struct RocksDbTransaction<'db> {
    db_ref: &'db DB,
    cache: HashMap<Vec<u8>, Vec<u8>>,
    deleted: HashSet<Vec<u8>>,
}

impl<'db> RocksDbTransaction<'db> {
    pub fn new(db_ref: &'db DB) -> Self {
        Self {
            db_ref,
            cache: HashMap::new(),
            deleted: HashSet::new(),
        }
    }
}

impl<'db> RocksDbTransaction<'db> {
    fn get_data<K: Encodable, V: Decodable>(&mut self, key: K) -> Result<Option<V>> {
        let mut key_bytes = Vec::new();
        key.encode(&mut key_bytes);

        let value = self
            .db_ref
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
}

impl<'db> DbStorageTransaction for RocksDbTransaction<'db> {
    fn batch_read<K: Encodable, V: Decodable>(&mut self, keys: Vec<K>) -> Result<Vec<V>> {
        let mut result = Vec::new();
        for key in keys {
            let value = self.get_data(key)?;
            if let Some(value) = value {
                result.push(value);
            }
        }
        Ok(result)
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
        let mut batch = WriteBatch::default();
        for (key, value) in self.cache {
            batch.put(key, value);
        }
        for key in self.deleted {
            batch.delete(key);
        }
        //info!("batch_write len: {:?}", batch.len());
        self.db_ref
            .write(batch)
            .with_context(|| "Failed to write batch")?;
        Ok(())
    }
}
