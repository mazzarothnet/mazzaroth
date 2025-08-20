use crate::state::{app_data::get_mvm_db_path, block_storage::get_block, tips::get_tips};
use anyhow::Context;
use consensus::{traits::GENESIS_BLOCK_KEY, types::BlockKey};
use database::rocksdb_no_batch::RocksDbStorage;
use log::info;
use mvm::core::{
    merkle_tree::MerkleTree,
    vm::{Mvm, NOW_BLOCK_ACTION_DO, NOW_BLOCK_ACTION_ROLLBACK},
};
use std::{sync::Mutex, time::Duration};

lazy_static::lazy_static! {
    static ref MVM_STORAGE: Mutex<Mvm<RocksDbStorage>> = Mutex::new(get_mvm_storage());
}

const MVM_MOVE_INTERVAL_MS: u64 = 1000;
const MVM_MAST_SLEEP_MS: u64 = 1000;

/// it will block thread
pub fn mvm_process_block() -> anyhow::Result<()> {
    let mut now_key = get_mvm_now_key()?;
    let mut now_time = std::time::Instant::now();
    loop {
        let cast_ms = now_time.elapsed().as_millis() as u64;
        if cast_ms < MVM_MOVE_INTERVAL_MS {
            std::thread::sleep(Duration::from_millis(MVM_MOVE_INTERVAL_MS - cast_ms));
        }
        std::thread::sleep(Duration::from_millis(MVM_MAST_SLEEP_MS));
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
        info!("try move mvm to next key: {} -> {}", now_key, next_key);
        move_mvm_to_next_key(now_key, next_key)?;
        info!("move mvm to next key: {} -> {}", now_key, next_key);
        now_key = next_key;
    }
}

struct MvmMoveNode {
    key: BlockKey,
    head_size: u64,
    head_key: BlockKey,
    to_head_path: Vec<BlockKey>,
}

impl MvmMoveNode {
    fn new(key: BlockKey) -> anyhow::Result<Self> {
        let block = get_block(&key)?.ok_or_else(|| anyhow::anyhow!("Block not found"))?;
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
            key,
            head_size,
            head_key,
            to_head_path,
        })
    }

    fn move_to_head(&mut self) -> anyhow::Result<()> {
        let head_block =
            get_block(&self.head_key)?.ok_or_else(|| anyhow::anyhow!("Block not found"))?;
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

fn move_mvm_to_next_key(now_key: BlockKey, next_key: BlockKey) -> anyhow::Result<()> {
    let mut now_node = MvmMoveNode::new(now_key)?;
    let mut next_node = MvmMoveNode::new(next_key)?;
    while now_node.head_key != next_node.head_key {
        if now_node.head_size > next_node.head_size {
            now_node.move_to_head()?;
        } else {
            next_node.move_to_head()?;
        }
    }

    info!(
        "move mvm to next key now_node path: {:?} \n next_node path: {:?}",
        now_node.to_head_path, next_node.to_head_path
    );
    now_node.to_head_path.pop();
    next_node.to_head_path.pop();
    for psk in now_node.to_head_path {
        let block = get_block(&psk)?.ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        let mut mvm_storage = MVM_STORAGE
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
        mvm_storage.do_block_rollback(&block)?;
    }
    for psk in next_node.to_head_path.into_iter().rev() {
        let block = get_block(&psk)?.ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        let mut mvm_storage = MVM_STORAGE
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
        mvm_storage.do_block(&block)?;
    }

    Ok(())
}

fn get_mvm_now_key() -> anyhow::Result<BlockKey> {
    let mut mvm_storage = MVM_STORAGE
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock mvm storage: {}", e))?;
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
    let merkle_path = format!("{}/merkle", path);
    let account_path = format!("{}/account", path);
    let state_path = format!("{}/state", path);
    let merkle_tree_db = RocksDbStorage::new(&merkle_path).unwrap();
    let merkle_tree = MerkleTree::new(merkle_tree_db).unwrap();
    let account_db = RocksDbStorage::new(&account_path).unwrap();
    let state_db = RocksDbStorage::new(&state_path).unwrap();
    Mvm::new(account_db, merkle_tree, state_db)
}
