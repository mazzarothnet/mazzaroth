use crate::state::{
    block_storage::{BlockStorage, get_block_hard},
    mz_state::MzState,
};
use anyhow::Context;
use consensus::types::BlockKey;
use database::rocksdb_no_batch::RocksDbStorage;
use log::info;
use mvm::core::{merkle_tree::MerkleTree, vm::Mvm};
use std::sync::Arc;
use utils::mutex_log::Mutex;

pub fn move_mvm_to_next_key(
    mut now_key: BlockKey,
    next_key: BlockKey,
    mz_state: &MzState,
) -> anyhow::Result<()> {
    if now_key == next_key {
        return Ok(());
    }
    let MvmMovePath {
        now_to_head_path,
        next_to_head_path,
    } = get_mvm_move_path(now_key, next_key, &mz_state.block_storage)?;
    for key in &now_to_head_path {
        rollback_mvm_to_head_key(now_key, *key, mz_state)?;
        now_key = *key;
    }
    for key in next_to_head_path.iter().rev() {
        do_mvm_to_next_key(now_key, *key, mz_state)?;
        now_key = *key;
    }
    info!(
        "move mvm to next key now_node path: {:?} next_node path: {:?}",
        now_to_head_path, next_to_head_path
    );

    Ok(())
}

pub fn get_mvm_now_key(mz_state: &MzState) -> anyhow::Result<BlockKey> {
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
    path.next_to_head_path.push(next_key);
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
    path.now_to_head_path
        .push(now_node.inner.header.part_sort_header.head_key);

    info!("now_node: {:?}", path.now_to_head_path);
    info!("next_node: {:?}", path.next_to_head_path);
    Ok(path)
}

fn do_mvm_to_next_key(
    now_key: BlockKey,
    next_key: BlockKey,
    mz_state: &MzState,
) -> anyhow::Result<()> {
    if now_key == next_key {
        return Ok(());
    }
    let next_block = get_block_hard(&mz_state.block_storage, &next_key)?;
    if next_block.inner.header.part_sort_header.head_key != now_key {
        return Err(anyhow::anyhow!(
            "do_mvm_to_next_key head_key err {:?} {:?}",
            now_key,
            next_key
        ));
    }
    let mut blocks = Vec::new();
    for key in &next_block.inner.header.part_sort_header.part_sort {
        if *key == now_key {
            break;
        }
        let block = get_block_hard(&mz_state.block_storage, key)?;
        blocks.push(block);
    }
    let mut mvm_lock = mz_state
        .mvm
        .lock()
        .map_err(|e| anyhow::anyhow!("rollback_mvm_to_head_key mvm_lock err {:?}", e))?;
    let mut transaction = mvm_lock.begin_transaction()?;
    for block in blocks.iter().rev() {
        Mvm::do_block(&mut transaction, block)?;
    }
    Mvm::do_block(&mut transaction, &next_block)?;
    transaction.commit(next_key)?;

    Ok(())
}

fn rollback_mvm_to_head_key(
    now_key: BlockKey,
    head_key: BlockKey,
    mz_state: &MzState,
) -> anyhow::Result<()> {
    if now_key == head_key {
        return Ok(());
    }
    let now_block = get_block_hard(&mz_state.block_storage, &now_key)?;
    if head_key != now_block.inner.header.part_sort_header.head_key {
        return Err(anyhow::anyhow!(
            "rollback_mvm_to_head_key head_key err {:?} {:?}",
            head_key,
            now_key
        ));
    }
    let mut blocks = Vec::new();
    for key in &now_block.inner.header.part_sort_header.part_sort {
        if *key == head_key {
            break;
        }
        let block = get_block_hard(&mz_state.block_storage, key)?;
        blocks.push(block);
    }
    let mut mvm_lock = mz_state
        .mvm
        .lock()
        .map_err(|e| anyhow::anyhow!("rollback_mvm_to_head_key mvm_lock err {:?}", e))?;
    let mut transaction = mvm_lock.begin_transaction()?;
    Mvm::do_block_rollback(&mut transaction, &now_block)?;
    for block in blocks {
        Mvm::do_block_rollback(&mut transaction, &block)?;
    }
    transaction.commit(head_key)?;

    Ok(())
}
