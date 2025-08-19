use anyhow::Context;
use consensus::{traits::GENESIS_BLOCK_KEY, types::BlockKey};
use database::rocksdb_no_batch::RocksDbStorage;
use mvm::core::{
    merkle_tree::MerkleTree,
    vm::{Mvm, NOW_BLOCK_ACTION_DO, NOW_BLOCK_ACTION_ROLLBACK},
};
use std::{path::Path, sync::Mutex, time::Duration};

use crate::state::{app_data::get_mvm_db_path, block_storage::get_block, tips::get_tips};

lazy_static::lazy_static! {
    static ref MVM_STORAGE: Mutex<Mvm<RocksDbStorage>> = Mutex::new(get_mvm_storage());
}

/// it will block thread
pub fn mvm_process_block() -> anyhow::Result<()> {
    let mut now_key = get_mvm_now_key()?;
    let mut now_time = std::time::Instant::now();
    loop {
        let cast_ms = now_time.elapsed().as_millis() as u64;
        if cast_ms < 1000 {
            std::thread::sleep(Duration::from_millis(1000 - cast_ms));
        }
        now_time = std::time::Instant::now();

        let tips = get_tips()?;
        if tips.contains(&now_key) {
            continue;
        }
        let mut next_key = if let Some(next_key) = tips.first() {
            *next_key
        } else {
            continue;
        };

        let block = get_block(&next_key)
            .with_context(|| "Failed to get block")?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        next_key = block.inner.header.part_sort_header.head_key;
        move_mvm_to_next_key(now_key, next_key)?;
        now_key = next_key;
    }
}

fn move_mvm_to_next_key(now_key: BlockKey, next_key: BlockKey) -> anyhow::Result<()> {
    Ok(())
}

fn get_mvm_now_key() -> anyhow::Result<BlockKey> {
    let mut mvm_storage = MVM_STORAGE.lock().unwrap();
    let now_key = mvm_storage
        .get_now_block_key()
        .with_context(|| "Failed to get now block key")?;
    let now_key = if let Some(now_key) = now_key {
        now_key
    } else {
        return Ok(BlockKey(GENESIS_BLOCK_KEY));
    };
    let now_action = mvm_storage
        .get_now_block_action()
        .with_context(|| "Failed to get now block action")?;
    if let Some(now_action) = now_action {
        let block = get_block(&now_key)
            .with_context(|| "Failed to get block")?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        if now_action == NOW_BLOCK_ACTION_DO {
            mvm_storage
                .do_block(&block)
                .with_context(|| "Failed to do block")?;
        } else if now_action == NOW_BLOCK_ACTION_ROLLBACK {
            mvm_storage
                .do_block_rollback(&block)
                .with_context(|| "Failed to do block rollback")?;
            mvm_storage
                .do_block(&block)
                .with_context(|| "Failed to do block")?;
        }
    }

    Ok(now_key)
}

#[allow(clippy::unwrap_used)]
fn get_mvm_storage() -> Mvm<RocksDbStorage> {
    let path = get_mvm_db_path().unwrap();
    get_mvm_storage_inner(&path, false).unwrap()
}

fn get_mvm_storage_inner(path: &str, need_reset: bool) -> anyhow::Result<Mvm<RocksDbStorage>> {
    if need_reset {
        std::fs::remove_dir_all(&path)?;
    }
    {
        let path = Path::new(path);
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
    }

    let merkle_path = format!("{}/merkle", path);
    let account_path = format!("{}/account", path);
    let state_path = format!("{}/state", path);
    let merkle_tree_db =
        RocksDbStorage::new(&merkle_path).with_context(|| "Failed to create merkle tree db")?;
    let merkle_tree =
        MerkleTree::new(merkle_tree_db).with_context(|| "Failed to create merkle tree")?;
    let account_db =
        RocksDbStorage::new(&account_path).with_context(|| "Failed to create account db")?;
    let state_db = RocksDbStorage::new(&state_path).with_context(|| "Failed to create state db")?;
    Ok(Mvm::new(account_db, merkle_tree, state_db))
}
