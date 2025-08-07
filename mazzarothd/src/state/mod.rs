use std::sync::Mutex;
use consensus::types::BlockKey;
use mvm::models::block::Block;

use crate::state::{app_data::get_block_db_path, block::BlockStorage};

pub mod app_data;
pub mod block;

#[allow(clippy::unwrap_used)]
lazy_static::lazy_static! {
    pub static ref BLOCK_STORAGE: Mutex<BlockStorage> = Mutex::new(BlockStorage::new(&get_block_db_path().unwrap()).unwrap());
}

pub fn get_block(block_key: &BlockKey) -> anyhow::Result<Option<Block>> {
    let block_storage = BLOCK_STORAGE.lock().unwrap();
    block_storage.get_block(block_key)
}

pub fn set_block(block_key: &BlockKey, block: &Block) -> anyhow::Result<()> {
    let block_storage = BLOCK_STORAGE.lock().unwrap();
    block_storage.set_block(block_key, block)
}

pub fn has_block(block_key: &BlockKey) -> anyhow::Result<bool> {
    let block_storage = BLOCK_STORAGE.lock().unwrap();
    block_storage.has_block(block_key)
}