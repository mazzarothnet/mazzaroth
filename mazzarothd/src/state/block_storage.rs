use consensus::{
    block_header::{ConsensusHeader, PowHeader, gen_consensus_header},
    traits::{ConsensusHeaderStorage, GENESIS_BLOCK_KEY, PartSortHeader},
    types::{BlockKey, DagWork},
};
use crypto_bigint::U256;
use database::rocksdb_no_batch::RocksDbStorage;
use log::info;
use mvm::models::block::{Block, BlockInner};
use std::sync::Arc;
use utils::error::{Error, Result};
use utils::mutex_log::Mutex;

pub fn get_block(
    block_storage_arc: &Arc<Mutex<BlockStorage>>,
    block_key: &BlockKey,
) -> anyhow::Result<Option<Block>> {
    let block_storage = block_storage_arc
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock block storage: {}", e))?;
    block_storage.get_block(block_key)
}

pub fn set_block(
    block_storage_arc: &Arc<Mutex<BlockStorage>>,
    block_key: &BlockKey,
    block: &Block,
) -> anyhow::Result<()> {
    info!("set_block, key: {:?}", block_key);
    let block_storage = block_storage_arc
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock block storage: {}", e))?;
    block_storage.set_block(block_key, block)
}

pub fn has_block(
    block_storage_arc: &Arc<Mutex<BlockStorage>>,
    block_key: &BlockKey,
) -> anyhow::Result<bool> {
    let block_storage = block_storage_arc
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock block storage: {}", e))?;
    block_storage.has_block(block_key)
}

pub fn gen_consensus_header_with_global_storage(
    block_storage_arc: &Arc<Mutex<BlockStorage>>,
    parent_keys: &[BlockKey],
    now_timestamp_ms: u64,
) -> utils::error::Result<ConsensusHeader> {
    let block_storage = block_storage_arc
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock block storage: {}", e))?;
    gen_consensus_header(&*block_storage, parent_keys, now_timestamp_ms)
}

pub struct BlockStorage {
    db: RocksDbStorage,
}

impl BlockStorage {
    fn new(db: &str) -> anyhow::Result<Self> {
        let db = RocksDbStorage::new(db)?;
        Ok(Self { db })
    }

    fn get_block(&self, key: &BlockKey) -> anyhow::Result<Option<Block>> {
        if *key == GENESIS_BLOCK_KEY {
            return Ok(Some(get_genesis_block()));
        }
        let block = self.db.get_data(key)?;
        Ok(block)
    }

    fn set_block(&self, key: &BlockKey, block: &Block) -> anyhow::Result<()> {
        self.db.set_data(key, block)?;
        Ok(())
    }

    fn has_block(&self, key: &BlockKey) -> anyhow::Result<bool> {
        if *key == GENESIS_BLOCK_KEY {
            return Ok(true);
        }
        let exists = self.db.has_data(key)?;
        Ok(exists)
    }
}

impl ConsensusHeaderStorage for BlockStorage {
    fn get_consensus_header(&self, key: &BlockKey) -> Result<ConsensusHeader> {
        let block = self.get_block(key)?;
        let block = block.ok_or_else(|| Error::BlockNotFound {
            key: key.to_string(),
        })?;
        Ok(block.inner.header)
    }
}

fn get_genesis_block() -> Block {
    Block {
        key: GENESIS_BLOCK_KEY,
        nonce: 0,
        inner: BlockInner {
            version: 0,
            header: ConsensusHeader {
                part_sort_header: PartSortHeader {
                    head_key: GENESIS_BLOCK_KEY,
                    dag_work: DagWork::from(U256::ZERO),
                    ..Default::default()
                },
                pow_header: PowHeader::default(),
            },
            ..Default::default()
        },
    }
}

pub fn get_block_storage(path: &str) -> anyhow::Result<BlockStorage> {
    BlockStorage::new(path)
}
