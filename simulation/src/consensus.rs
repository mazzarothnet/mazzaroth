use anyhow::Context;
use consensus::traits::{DagStorage, Key, SortStruct};
use serde::{Deserialize, Serialize};
use utils::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimKey(i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimBlock {
    pub key: SimKey,
    pub ts: i64,
    pub parent_keys: Vec<SimKey>,
}

impl Key for SimKey {
    fn is_genesis(&self) -> bool {
        self.0 == 0
    }
}

pub struct SimDagStorage {
    db: rocksdb::DB,
}

impl SimDagStorage {
    pub fn new(db: rocksdb::DB) -> Self {
        Self { db }
    }

    pub fn set_block(&mut self, key: SimKey, block: SimBlock) -> Result<()> {
        let key_vec = bincode::serialize(&key).context("set_block error")?;
        let value = bincode::serialize(&block).context("set_block error")?;
        self.db.put(&key_vec, &value).context("set block failed")?;
        Ok(())
    }

    pub fn get_block(&self, key: SimKey) -> Result<Option<SimBlock>> {
        let key_vec = bincode::serialize(&key).context("get_block error")?;
        let value = if let Some(value) = self.db.get(&key_vec).context("get block failed")? {
            value
        } else {
            return Ok(None);
        };
        let block: SimBlock =
            bincode::deserialize(&value).context("deserialize sim block failed")?;
        Ok(Some(block))
    }
}

impl DagStorage for SimDagStorage {
    type KeyType = SimKey;

    fn get_parent_keys(&self, key: &Self::KeyType) -> Result<Vec<Self::KeyType>> {
        let key_vec = bincode::serialize(key).context("get_parent_keys error")?;
        let value = self
            .db
            .get(&key_vec)
            .context("get parent keys failed")?
            .ok_or_else(|| Error::UnknownBlock {
                message: format!("unknown block: {:?}", key),
            })?;
        let block: SimBlock =
            bincode::deserialize(&value).context("deserialize sim block failed")?;
        Ok(block.parent_keys)
    }

    fn get_part_sort_of_key(
        &self,
        key: &Self::KeyType,
    ) -> Result<Option<SortStruct<Self::KeyType>>> {
        let key_vec = bincode::serialize(key).context("get_part_sort_of_key error")?;
        let value = if let Some(value) = self
            .db
            .get(&key_vec)
            .context("get part sort of key failed")?
        {
            value
        } else {
            return Ok(None);
        };
        let sort_struct: SortStruct<SimKey> =
            bincode::deserialize(&value).context("deserialize sort struct failed")?;
        Ok(Some(sort_struct))
    }

    fn set_part_sort_of_key(
        &mut self,
        key: Self::KeyType,
        package: SortStruct<Self::KeyType>,
    ) -> Result<()> {
        let key_vec = bincode::serialize(&key).context("set_part_sort_of_key error")?;
        let value = bincode::serialize(&package).context("set_part_sort_of_key error")?;
        self.db
            .put(&key_vec, &value)
            .context("set part sort of key failed")?;
        Ok(())
    }
}
