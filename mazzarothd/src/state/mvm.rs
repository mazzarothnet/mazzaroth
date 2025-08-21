use crate::state::{
    block_storage::{BlockStorage, get_block},
    mz_state::MzState,
    tips::get_tips,
};
use anyhow::Context;
use consensus::{traits::GENESIS_BLOCK_KEY, types::BlockKey};
use database::rocksdb_no_batch::RocksDbStorage;
use log::info;
use mvm::core::{
    merkle_tree::MerkleTree,
    vm::{Mvm, NOW_BLOCK_ACTION_DO, NOW_BLOCK_ACTION_ROLLBACK},
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

const MVM_MOVE_INTERVAL_MS: u64 = 2000;

/// it will block thread
pub fn mvm_process_block(mz_state: &MzState) -> anyhow::Result<()> {
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
        let mut next_key = if let Some(next_key) = tips.first() {
            *next_key
        } else {
            continue;
        };

        let block = get_block(&mz_state.block_storage, &next_key)
            .with_context(|| "Failed to get block")?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        next_key = block.inner.header.part_sort_header.head_key;
        info!("try move mvm to next key: {} -> {}", now_key, next_key);
        move_mvm_to_next_key(now_key, next_key, mz_state)?;
        info!("move mvm to next key: {} -> {}", now_key, next_key);
        now_key = next_key;
    }
}

struct MvmMoveNode<'a> {
    block_storage_arc: &'a Arc<Mutex<BlockStorage>>,
    key: BlockKey,
    head_size: u64,
    head_key: BlockKey,
    to_head_path: Vec<BlockKey>,
}

impl<'a> MvmMoveNode<'a> {
    fn new(key: BlockKey, block_storage_arc: &'a Arc<Mutex<BlockStorage>>) -> anyhow::Result<Self> {
        let block = get_block(block_storage_arc, &key)?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        let head_size = block.inner.header.part_sort_header.size;
        let head_key = block.inner.header.part_sort_header.head_key;
        let mut to_head_path = Vec::new();
        to_head_path.push(key);
        for psk in block
            .inner
            .header
            .part_sort_header
            .part_sort
            .into_iter()
            .rev()
        {
            to_head_path.push(psk);
        }
        Ok(Self {
            block_storage_arc,
            key,
            head_size,
            head_key,
            to_head_path,
        })
    }

    fn move_to_head(&mut self) -> anyhow::Result<()> {
        let head_block = get_block(self.block_storage_arc, &self.head_key)?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        self.key = self.head_key;
        self.head_size = head_block.inner.header.part_sort_header.size;
        self.head_key = head_block.inner.header.part_sort_header.head_key;
        for psk in head_block
            .inner
            .header
            .part_sort_header
            .part_sort
            .into_iter()
            .rev()
        {
            self.to_head_path.push(psk);
        }
        Ok(())
    }
}

pub struct MvmMovePath {
    pub now_to_head_path: Vec<BlockKey>,
    pub next_to_head_path: Vec<BlockKey>,
}

pub fn get_mvm_move_path(
    now_key: BlockKey,
    next_key: BlockKey,
    block_storage_arc: &Arc<Mutex<BlockStorage>>,
) -> anyhow::Result<MvmMovePath> {
    let mut now_node = MvmMoveNode::new(now_key, block_storage_arc)?;
    let mut next_node = MvmMoveNode::new(next_key, block_storage_arc)?;
    while now_node.head_key != next_node.head_key {
        if now_node.head_size > next_node.head_size {
            now_node.move_to_head()?;
        } else {
            next_node.move_to_head()?;
        }
    }
    Ok(MvmMovePath {
        now_to_head_path: now_node.to_head_path,
        next_to_head_path: next_node.to_head_path,
    })
}

fn move_mvm_to_next_key(
    now_key: BlockKey,
    next_key: BlockKey,
    mz_state: &MzState,
) -> anyhow::Result<()> {
    let MvmMovePath {
        mut now_to_head_path,
        mut next_to_head_path,
    } = get_mvm_move_path(now_key, next_key, &mz_state.block_storage)?;

    info!(
        "move mvm to next key now_node path: {:?} next_node path: {:?}",
        now_to_head_path, next_to_head_path
    );
    now_to_head_path.pop();
    next_to_head_path.pop();
    for psk in now_to_head_path {
        let block = get_block(&mz_state.block_storage, &psk)?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        let mut mvm_storage = mz_state
            .mvm
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
        mvm_storage.do_block_rollback(&block)?;
    }
    for psk in next_to_head_path.into_iter().rev() {
        let block = get_block(&mz_state.block_storage, &psk)?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        let mut mvm_storage = mz_state
            .mvm
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
        mvm_storage.do_block(&block)?;
    }

    Ok(())
}

fn get_mvm_now_key(mz_state: &MzState) -> anyhow::Result<BlockKey> {
    let (now_key, now_action) = {
        let mvm_storage = mz_state
            .mvm
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
        mvm_storage
            .get_now_block_key_and_action()
            .with_context(|| "Failed to get now block key")?
    };
    let now_key = if let Some(now_key) = now_key {
        now_key
    } else {
        return Ok(BlockKey(GENESIS_BLOCK_KEY));
    };
    if let Some(now_action) = now_action {
        let block = get_block(&mz_state.block_storage, &now_key)
            .with_context(|| "Failed to get block")?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        let mut mvm_storage = mz_state
            .mvm
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
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

pub fn get_mvm_storage(path: &str) -> anyhow::Result<Mvm<RocksDbStorage>> {
    let os_path = std::path::Path::new(path);
    if !os_path.exists() {
        std::fs::create_dir_all(os_path)?;
    }
    let merkle_path = format!("{}/merkle", path);
    let account_path = format!("{}/account", path);
    let state_path = format!("{}/state", path);
    let merkle_tree_db = RocksDbStorage::new(&merkle_path)?;
    let merkle_tree = MerkleTree::new(merkle_tree_db)?;
    let account_db = RocksDbStorage::new(&account_path)?;
    let state_db = RocksDbStorage::new(&state_path)?;
    Ok(Mvm::new(account_db, merkle_tree, state_db))
}
