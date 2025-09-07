use crate::state::{
    block_storage::{BlockStorage, get_block, get_block_hard},
    mz_state::MzState,
    tips::get_tips,
};
use anyhow::Context;
use consensus::{traits::GENESIS_BLOCK_KEY, types::BlockKey};
use database::rocksdb_no_batch::RocksDbStorage;
use log::info;
use mvm::core::{merkle_tree::MerkleTree, vm::Mvm};
use std::{sync::Arc, time::Duration};
use utils::mutex_log::Mutex;

const MVM_MOVE_INTERVAL_MS: u64 = 500;

/// it will block thread
fn mvm_process_block(mz_state: &MzState) -> anyhow::Result<()> {
    let mut now_key = get_mvm_now_key(mz_state)?;
    let mut now_time = std::time::Instant::now();
    loop {
        let cast_ms = now_time.elapsed().as_millis() as u64;
        if cast_ms < MVM_MOVE_INTERVAL_MS {
            std::thread::sleep(Duration::from_millis(MVM_MOVE_INTERVAL_MS - cast_ms));
        }
        now_time = std::time::Instant::now();

        let tips = get_tips(mz_state)?;
        if tips.contains(&now_key) {
            continue;
        }
        let next_key = if let Some(next_key) = tips.first() {
            *next_key
        } else {
            continue;
        };

        move_mvm_to_next_key(now_key, next_key, mz_state)?;
        info!("move mvm to next key success: {} -> {}", now_key, next_key);
        now_key = next_key;
    }
}

#[derive(Debug, Default, Clone)]
pub struct MvmMovePath {
    pub now_to_head_path: Vec<BlockKey>,
    pub next_to_head_path: Vec<BlockKey>,
}

pub fn get_mvm_move_path(
    now_key: BlockKey,
    next_key: BlockKey,
    block_storage_arc: &Arc<Mutex<BlockStorage>>,
) -> anyhow::Result<MvmMovePath> {
    let mut path = MvmMovePath::default();
    let mut now_node = get_block_hard(block_storage_arc, &now_key)
        .with_context(|| "get_mvm_move_path now_node")?;
    let mut next_node = get_block_hard(block_storage_arc, &next_key)
        .with_context(|| "get_mvm_move_path next_node")?;
    while now_node.inner.header.part_sort_header.head_key
        != next_node.inner.header.part_sort_header.head_key
    {
        if now_node.inner.header.part_sort_header.size
            > next_node.inner.header.part_sort_header.size
        {
            let head_key = now_node.inner.header.part_sort_header.head_key;
            path.now_to_head_path.push(head_key);
            now_node = get_block_hard(block_storage_arc, &head_key)
                .with_context(|| "get_mvm_move_path now_node head_key")?;
        } else {
            let head_key = next_node.inner.header.part_sort_header.head_key;
            path.next_to_head_path.push(head_key);
            next_node = get_block_hard(block_storage_arc, &head_key)
                .with_context(|| "get_mvm_move_path now_node next_node")?;
        }
    }

    info!("now_node: {:?}", path.now_to_head_path);
    info!("next_node: {:?}", path.next_to_head_path);
    Ok(path)
}

fn move_mvm_to_next_key(
    now_key: BlockKey,
    next_key: BlockKey,
    mz_state: &MzState,
) -> anyhow::Result<()> {
    let MvmMovePath {
        now_to_head_path,
        next_to_head_path,
    } = get_mvm_move_path(now_key, next_key, &mz_state.block_storage)?;

    // info!(
    //     "move mvm to next key now_node path: {:?} next_node path: {:?}",
    //     now_to_head_path, next_to_head_path
    // );
    // for psk in now_to_head_path {
    //     let block = get_block(&mz_state.block_storage, &psk)?
    //         .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
    //     let mut mvm_storage = mz_state
    //         .mvm
    //         .lock()
    //         .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
    //     mvm_storage.do_block_rollback(&block)?;
    // }
    // for psk in next_to_head_path.into_iter().rev() {
    //     let block = get_block(&mz_state.block_storage, &psk)?
    //         .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
    //     let mut mvm_storage = mz_state
    //         .mvm
    //         .lock()
    //         .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
    //     mvm_storage.do_block(&block)?;
    // }

    Ok(())
}

fn get_mvm_now_key(mz_state: &MzState) -> anyhow::Result<BlockKey> {
    let mut mvm_storage = mz_state
        .mvm
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
    let mut transaction = mvm_storage.begin_transaction()?;
    let now_key = Mvm::get_block_key(&mut transaction)?;
    Ok(now_key)
}

pub fn get_mvm_storage(path: &str) -> anyhow::Result<Mvm<RocksDbStorage>> {
    let os_path = std::path::Path::new(path);
    if !os_path.exists() {
        std::fs::create_dir_all(os_path)?;
    }
    let account_path = format!("{}/account", path);
    let merkle_tree = MerkleTree::default();
    let account_db = RocksDbStorage::new(&account_path)?;
    Ok(Mvm::new(account_db, merkle_tree))
}
