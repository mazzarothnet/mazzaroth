use crate::state::app_data::{get_block_db_path, get_block_test_db_path};
use anyhow::Context;
use consensus::{
    block_header::{ConsensusHeader, MAX_TARGET, PowHeader},
    part_sort_header::gen_part_sort_block,
    traits::{ConsensusHeaderStorage, GENESIS_BLOCK_KEY, PartSortHeader},
    types::{AccountKey, BlockKey, DagWork, Hash},
};
use crypto_bigint::U256;
use database::rocksdb_no_batch::RocksDbStorage;
use mvm::{
    core::storage::DbStorage,
    models::block::{Block, BlockInner},
};
use std::{path::Path, sync::Mutex};
use utils::error::{Error, Result};

lazy_static::lazy_static! {
    static ref BLOCK_STORAGE: Mutex<BlockStorage> = Mutex::new(get_block_storage());
}

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

struct BlockStorage {
    db: RocksDbStorage,
}

impl BlockStorage {
    fn new(db: &str) -> anyhow::Result<Self> {
        let db = RocksDbStorage::new(db)?;
        Ok(Self { db })
    }

    fn get_block(&self, key: &BlockKey) -> anyhow::Result<Option<Block>> {
        if key == &BlockKey::from(GENESIS_BLOCK_KEY) {
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
        key: BlockKey::from(GENESIS_BLOCK_KEY),
        nonce: 0,
        inner: BlockInner {
            version: 0,
            header: ConsensusHeader {
                part_sort_header: PartSortHeader {
                    head_key: BlockKey::from(GENESIS_BLOCK_KEY),
                    dag_work: DagWork::from(U256::ZERO),
                    size: 0,
                    parent_keys: vec![],
                    part_sort: vec![],
                },
                pow_header: PowHeader {
                    target: MAX_TARGET,
                    target_timestamp_ms: 0,
                    now_timestamp_ms: 0,
                },
            },
            transfers: vec![],
            merges: vec![],
            miner: AccountKey::default(),
            miner_last_action_hash: Hash::default(),
        },
    }
}

#[allow(clippy::unwrap_used)]
fn get_block_storage() -> BlockStorage {
    BlockStorage::new(&get_block_db_path().unwrap()).unwrap()
}

pub fn use_test_db_and_refresh_block_storage(name: &str) -> anyhow::Result<()> {
    let block_db_path =
        get_block_test_db_path(name).with_context(|| "Failed to get block test db path")?;
    if Path::new(&block_db_path).exists() {
        println!("remove block db path: {:?}", block_db_path);
        std::fs::remove_dir_all(&block_db_path)?;
    }
    let block_storage =
        BlockStorage::new(&block_db_path).with_context(|| "Failed to create block storage")?;
    *BLOCK_STORAGE
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock block storage: {}", e))? = block_storage;
    Ok(())
}
