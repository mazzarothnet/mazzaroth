use crate::state::block::{has_block, set_block};
use consensus::types::BlockKey;
use mvm::models::block::Block;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};
use utils::time::get_current_time_ms;

lazy_static::lazy_static! {
    static ref TIPS: Mutex<BTreeMap<BlockKey, u64>> = Mutex::new(BTreeMap::new());
    static ref TEMP_BLOCKS: Mutex<TempBlock> = Mutex::new(TempBlock::default());
}

const TIPS_EXPIRE_MS: u64 = 1000 * 30; // tips expire time 30s

// about check
pub fn push_block(block: Block) -> anyhow::Result<()> {
    let mut temp_blocks = TEMP_BLOCKS
        .lock()
        .map_err(|e| anyhow::anyhow!("push_block Failed to lock temp_blocks: {}", e))?;
    temp_blocks.push_block(block)?;
    while let Some(block) = temp_blocks.pop_block() {
        save_block_and_update_tips(&block)?;
    }

    Ok(())
}

pub fn get_tips() -> anyhow::Result<BTreeSet<BlockKey>> {
    let tips = TIPS
        .lock()
        .map_err(|e| anyhow::anyhow!("get_tips Failed to lock tips: {}", e))?
        .keys()
        .cloned()
        .collect();
    Ok(tips)
}

pub fn get_temp_blocks() -> anyhow::Result<BTreeMap<BlockKey, (Block, BTreeSet<BlockKey>)>> {
    let temp_blocks = TEMP_BLOCKS
        .lock()
        .map_err(|e| anyhow::anyhow!("get_temp_blocks Failed to lock temp_blocks: {}", e))?
        .temp_blocks
        .clone();
    Ok(temp_blocks)
}

#[derive(Debug, Default)]
struct TempBlock {
    unknown_keys: BTreeMap<BlockKey, Vec<BlockKey>>,
    temp_blocks: BTreeMap<BlockKey, (Block, BTreeSet<BlockKey>)>,
    ready_pop: BTreeMap<BlockKey, Block>,
}

impl TempBlock {
    fn push_block(&mut self, block: Block) -> anyhow::Result<()> {
        let mut now_block_unknown_keys = BTreeSet::new();
        for parent in block.inner.header.part_sort_header.parent_keys.iter() {
            if self.unknown_keys.contains_key(parent) || !(has_block(parent)?) {
                let entry = self.unknown_keys.entry(*parent).or_insert_with(|| Vec::new());
                entry.push(block.key);
                now_block_unknown_keys.insert(*parent);
            }
        }

        if now_block_unknown_keys.is_empty() {
            self.ready_pop.insert(block.key, block);
        } else {
            self.temp_blocks
                .insert(block.key, (block, now_block_unknown_keys));
        }

        Ok(())
    }

    fn pop_block(&mut self) -> Option<Block> {
        let (pop_key, block) = self.ready_pop.pop_first()?;
        let mut ready_pop_keys = BTreeSet::new();
        if let Some(keys) = self.unknown_keys.remove(&pop_key) {
            for key in keys {
                if let Some((_b, unknown_keys)) = self.temp_blocks.get_mut(&key) {
                    unknown_keys.remove(&pop_key);
                    if unknown_keys.is_empty() {
                        ready_pop_keys.insert(key);
                    }
                }
            }
        }
        for key in ready_pop_keys {
            if let Some((block, _)) = self.temp_blocks.remove(&key) {
                self.ready_pop.insert(key, block);
            }
        }

        Some(block)
    }
}

// about tips and save block to db
fn save_block_and_update_tips(block: &Block) -> anyhow::Result<()> {
    {
        let now = get_current_time_ms();
        let mut tips = TIPS.lock().map_err(|e| {
            anyhow::anyhow!("save_block_and_update_tips Failed to lock tips: {}", e)
        })?;
        for parent in block.inner.header.part_sort_header.parent_keys.iter() {
            tips.remove(parent);
        }
        tips.retain(|_, &mut timestamp| now - timestamp < TIPS_EXPIRE_MS);
        tips.insert(block.key, now);
    }
    set_block(&block.key, block)
}
