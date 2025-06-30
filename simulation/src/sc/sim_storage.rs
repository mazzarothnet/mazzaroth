use crate::sc::sim_miner::Position;

use super::sim_block::SimBlock;
use anyhow::Context;
use consensus::{
    block_header::{ConsensusHeader, PowHeader},
    traits::{BlockKey, ConsensusHeaderStorage, GENESIS_BLOCK_KEY, PartSortHeader},
};
use utils::error::{Error, Result};

pub struct SimConsensusHeaderStorage {
    db: rocksdb::DB,
}

impl SimConsensusHeaderStorage {
    pub fn new(path: &str) -> Self {
        let cache = rocksdb::Cache::new_lru_cache(1024 * 1024 * 1024);
        let mut opts = rocksdb::Options::default();
        opts.set_blob_cache(&cache);
        opts.create_if_missing(true);
        let db = rocksdb::DB::open(&opts, path).unwrap();
        Self { db }
    }

    pub fn set_block(&mut self, key: BlockKey, block: &SimBlock) -> Result<()> {
        let key_vec = bincode::serialize(&key).context("set_block error")?;
        let value = bincode::serialize(&block).context("set_block error")?;
        self.db.put(&key_vec, &value).context("set block failed")?;
        Ok(())
    }

    pub fn get_block(&self, key: &BlockKey) -> Result<Option<SimBlock>> {
        if key == &GENESIS_BLOCK_KEY {
            return Ok(Some(SimBlock {
                key: GENESIS_BLOCK_KEY,
                creator_position: Position::default(),
                header: ConsensusHeader {
                    part_sort_header: PartSortHeader::default(),
                    pow_header: PowHeader {
                        target: BlockKey::MAX,
                        target_timestamp_ms: 0,
                        now_timestamp_ms: 0,
                    },
                },
            }));
        }
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

impl ConsensusHeaderStorage for SimConsensusHeaderStorage {
    fn get_consensus_header(&self, key: &BlockKey) -> Result<ConsensusHeader> {
        let block = self.get_block(key)?.ok_or_else(|| Error::BlockNotFound {
            key: key.to_string(),
        })?;
        Ok(block.header)
    }
}
