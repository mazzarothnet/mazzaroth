use crate::state::app_data::get_block_db_path;
use consensus::{
    block_header::ConsensusHeader,
    part_sort_header::gen_part_sort_block,
    traits::{ConsensusHeaderStorage, PartSortHeader},
    types::BlockKey,
};
use database::rocksdb_no_batch::RocksDbStorage;
use mvm::{core::storage::DbStorage, models::block::Block};
use std::sync::Mutex;
use utils::error::{Error, Result};

pub fn get_block(block_key: &BlockKey) -> anyhow::Result<Option<Block>> {
    let block_storage = BLOCK_STORAGE
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock block storage: {}", e))?;
    block_storage.get_block(block_key)
}

pub fn set_block(block_key: &BlockKey, block: &Block) -> anyhow::Result<()> {
    let block_storage = BLOCK_STORAGE
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock block storage: {}", e))?;
    block_storage.set_block(block_key, block)
}

pub fn has_block(block_key: &BlockKey) -> anyhow::Result<bool> {
    let block_storage = BLOCK_STORAGE
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock block storage: {}", e))?;
    block_storage.has_block(block_key)
}

pub fn get_part_sort_header(parent_keys: &[BlockKey]) -> utils::error::Result<PartSortHeader> {
    let block_storage = BLOCK_STORAGE
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock block storage: {}", e))?;
    gen_part_sort_block(&*block_storage, parent_keys)
}

lazy_static::lazy_static! {
    static ref BLOCK_STORAGE: Mutex<BlockStorage> = Mutex::new(get_block_storage());
}

struct BlockStorage {
    db: RocksDbStorage,
}

impl BlockStorage {
    fn new(db: &str) -> anyhow::Result<Self> {
        let db = RocksDbStorage::new(db)?;
        Ok(Self { db })
    }

    fn get_block(&self, key: &BlockKey) -> anyhow::Result<Option<Block>> {
        let block = self.db.get_data(key)?;
        Ok(block)
    }

    fn set_block(&self, key: &BlockKey, block: &Block) -> anyhow::Result<()> {
        self.db.set_data(key, block)?;
        Ok(())
    }

    fn has_block(&self, key: &BlockKey) -> anyhow::Result<bool> {
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

#[allow(clippy::unwrap_used)]
fn get_block_storage() -> BlockStorage {
    BlockStorage::new(&get_block_db_path().unwrap()).unwrap()
}
