use super::sim_block::SimBlock;
use crate::sc::sim_miner::Position;
use alloy_rlp::{Decodable, Encodable};
use anyhow::Context;
use consensus::{
    block_header::{ConsensusHeader, PowHeader},
    traits::{ConsensusHeaderStorage, GENESIS_BLOCK_KEY, PartSortHeader},
    types::{BlockKey, DagWork},
};
use crypto_bigint::U256;
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
        let mut key_vec = Vec::new();
        key.encode(&mut key_vec);
        let mut value_vec = Vec::new();
        block.encode(&mut value_vec);
        self.db
            .put(&key_vec, &value_vec)
            .context("set block failed")?;
        Ok(())
    }

    pub fn get_block(&self, key: &BlockKey) -> Result<Option<SimBlock>> {
        if *key == GENESIS_BLOCK_KEY {
            return Ok(Some(SimBlock {
                key: GENESIS_BLOCK_KEY,
                creator_position: Position::default(),
                header: ConsensusHeader {
                    part_sort_header: PartSortHeader {
                        head_key: GENESIS_BLOCK_KEY,
                        dag_work: DagWork::from(U256::ZERO),
                        ..Default::default()
                    },
                    pow_header: PowHeader {
                        target: BlockKey::from(U256::MAX),
                        ..Default::default()
                    },
                },
            }));
        }
        let mut key_vec = Vec::new();
        key.encode(&mut key_vec);
        let value = if let Some(value) = self.db.get(&key_vec).context("get block failed")? {
            value
        } else {
            return Ok(None);
        };
        let block =
            SimBlock::decode(&mut value.as_slice()).context("deserialize sim block failed")?;
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
