use super::sim_block::{SimBlock, SimKey};
use anyhow::Context;
use consensus::traits::{ConsensusBlock, DagStorage, Key};
use serde::{Deserialize, Serialize};
use utils::error::{Error, Result};

pub const BLOCK_DATA_TYPE: u8 = 0;
pub const PART_SORT_DATA_TYPE: u8 = 1;

#[derive(Serialize, Deserialize)]
pub struct KeyWrapper {
    pub key: SimKey,
    pub data_type: u8,
}

pub struct SimDagStorage {
    db: rocksdb::DB,
}

impl SimDagStorage {
    pub fn new(db: rocksdb::DB) -> Self {
        Self { db }
    }

    pub fn set_block(&mut self, key: SimKey, block: &SimBlock) -> Result<()> {
        let key_wrapper = KeyWrapper {
            key,
            data_type: BLOCK_DATA_TYPE,
        };
        let key_vec = bincode::serialize(&key_wrapper).context("set_block error")?;
        let value = bincode::serialize(&block).context("set_block error")?;
        self.db.put(&key_vec, &value).context("set block failed")?;
        Ok(())
    }

    pub fn get_block(&self, key: &SimKey) -> Result<Option<SimBlock>> {
        let key_wrapper = KeyWrapper {
            key: *key,
            data_type: BLOCK_DATA_TYPE,
        };
        let key_vec = bincode::serialize(&key_wrapper).context("get_block error")?;
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

pub struct DagStorageKey {
    pub key: SimKey,
}

impl DagStorage for SimDagStorage {
    type KeyType = SimKey;

    fn get_parent_keys(&self, key: &Self::KeyType) -> Result<Vec<Self::KeyType>> {
        let key_wrapper = KeyWrapper {
            key: *key,
            data_type: BLOCK_DATA_TYPE,
        };
        let key_vec = bincode::serialize(&key_wrapper).context("get_parent_keys error")?;
        let value = self
            .db
            .get(&key_vec)
            .context("get parent keys failed")?
            .ok_or_else(|| Error::UnknownBlock {
                key: key.serde_to_string(),
            })?;
        let block: SimBlock =
            bincode::deserialize(&value).context("deserialize sim block failed")?;
        Ok(block.parent_keys)
    }

    fn get_consensus_block_of_key(
        &self,
        key: &Self::KeyType,
    ) -> Result<Option<ConsensusBlock<Self::KeyType>>> {
        let key_wrapper = KeyWrapper {
            key: *key,
            data_type: PART_SORT_DATA_TYPE,
        };
        let key_vec = bincode::serialize(&key_wrapper).context("get_part_sort_of_key error")?;
        let vv = self
            .db
            .get(&key_vec)
            .context("get part sort of key failed")?;
        let value = if let Some(value) = vv {
            value
        } else {
            return Ok(None);
        };
        let consensus_block: ConsensusBlock<SimKey> =
            bincode::deserialize(&value).context("deserialize consensus block failed")?;
        Ok(Some(consensus_block))
    }

    fn set_consensus_block_of_key(
        &mut self,
        key: Self::KeyType,
        package: &ConsensusBlock<Self::KeyType>,
    ) -> Result<()> {
        let key_wrapper = KeyWrapper {
            key,
            data_type: PART_SORT_DATA_TYPE,
        };
        let key_vec = bincode::serialize(&key_wrapper).context("set_part_sort_of_key error")?;
        let value = bincode::serialize(&package).context("set_part_sort_of_key error")?;
        self.db
            .put(&key_vec, &value)
            .context("set part sort of key failed")?;
        Ok(())
    }
}
