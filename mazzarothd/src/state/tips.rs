use crate::state::{
    block_check::{normal_check_block_format, save_block_check},
    block_storage::{BlockStorage, has_block, set_block},
    mz_state::MzState,
};
use consensus::types::BlockKey;
use consensus::{
    block_header::ConsensusHeader,
    types::{AccountKey, Hash},
};
use crypto_bigint::U256;
use log::info;
use mvm::models::block::Block;
use mvm::models::block::BlockInner;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    sync::{Arc, Mutex},
};
use utils::time::get_current_time_ms;

const TIPS_EXPIRE_MS: u64 = 1000 * 30; // tips expire time 30s

// about check
pub fn push_block(block: Block, mz_state: &MzState) -> anyhow::Result<()> {
    if has_block(&mz_state.block_storage, &block.key)? {
        return Ok(());
    }
    normal_check_block_format(&block)?;

    let mut temp_blocks = mz_state
        .temp_blocks
        .lock()
        .map_err(|e| anyhow::anyhow!("push_block Failed to lock temp_blocks: {}", e))?;
    temp_blocks.push_block(block)?;
    while let Some(block) = temp_blocks.pop_block() {
        if let Err(e) = save_block_and_update_tips(&block, mz_state) {
            info!(
                "push_block, save_block_and_update_tips key: {:?}, error: {:?}",
                block.key, e
            );
        } else {
            info!(
                "push_block, save_block_and_update_tips key: {:?}, success",
                block.key
            );
        }
    }

    Ok(())
}

pub fn get_tips(mz_state: &MzState) -> anyhow::Result<BTreeSet<BlockKey>> {
    let tips = mz_state
        .tips
        .lock()
        .map_err(|e| anyhow::anyhow!("get_tips Failed to lock tips: {}", e))?
        .keys()
        .cloned()
        .collect();
    Ok(tips)
}

pub fn set_test_tips(tips: Vec<BlockKey>, mz_state: &MzState) -> anyhow::Result<()> {
    let mut tips_map = mz_state
        .tips
        .lock()
        .map_err(|e| anyhow::anyhow!("set_test_tips Failed to lock tips: {}", e))?;
    tips_map.clear();
    for tip in tips {
        tips_map.insert(tip, get_current_time_ms());
    }
    Ok(())
}

pub fn get_temp_blocks(
    mz_state: &MzState,
) -> anyhow::Result<BTreeMap<BlockKey, (Block, BTreeSet<BlockKey>)>> {
    let temp_blocks = mz_state
        .temp_blocks
        .lock()
        .map_err(|e| anyhow::anyhow!("get_temp_blocks Failed to lock temp_blocks: {}", e))?
        .temp_blocks
        .clone();
    Ok(temp_blocks)
}

pub struct TempBlock {
    pub unknown_keys: BTreeMap<BlockKey, Vec<BlockKey>>,
    pub temp_blocks: BTreeMap<BlockKey, (Block, BTreeSet<BlockKey>)>,
    pub ready_pop: BTreeMap<BlockKey, Block>,
    pub block_storage: Arc<Mutex<BlockStorage>>,
}

impl TempBlock {
    pub fn new(block_storage: Arc<Mutex<BlockStorage>>) -> Self {
        Self {
            unknown_keys: BTreeMap::new(),
            temp_blocks: BTreeMap::new(),
            ready_pop: BTreeMap::new(),
            block_storage,
        }
    }
}

impl TempBlock {
    fn push_block(&mut self, block: Block) -> anyhow::Result<()> {
        let mut now_block_unknown_keys = BTreeSet::new();
        for parent in block.inner.header.part_sort_header.parent_keys.iter() {
            if self.unknown_keys.contains_key(parent) || !has_block(&self.block_storage, parent)? {
                let entry = self
                    .unknown_keys
                    .entry(*parent)
                    .or_insert_with(|| Vec::new());
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
fn save_block_and_update_tips(block: &Block, mz_state: &MzState) -> anyhow::Result<()> {
    save_block_check(&mz_state.block_storage, block)?;
    {
        let now = get_current_time_ms();
        let mut tips = mz_state.tips.lock().map_err(|e| {
            anyhow::anyhow!("save_block_and_update_tips Failed to lock tips: {}", e)
        })?;
        for parent in block.inner.header.part_sort_header.parent_keys.iter() {
            tips.remove(parent);
        }
        tips.retain(|_, &mut timestamp| now - timestamp < TIPS_EXPIRE_MS);
        tips.insert(block.key, now);
    }
    set_block(&mz_state.block_storage, &block.key, block)
}

pub fn gen_test_block(key: u32, parent_keys: &HashSet<u32>) -> Block {
    let mut header = ConsensusHeader::default();
    header.part_sort_header.parent_keys =
        parent_keys.iter().map(|k| u32_to_block_key(*k)).collect();
    Block {
        key: u32_to_block_key(key),
        nonce: 0,
        inner: BlockInner {
            version: 0,
            header,
            transfers: vec![],
            merges: vec![],
            miner: AccountKey::default(),
            miner_last_action_hash: Hash::default(),
        },
    }
}

pub fn u32_to_block_key(key: u32) -> BlockKey {
    BlockKey(U256::from_u32(key))
}

pub fn block_key_to_u32(key: BlockKey) -> u32 {
    key.0.to_limbs()[0].0 as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::mz_state::get_mz_state;
    use rand::{Rng, SeedableRng, seq::SliceRandom};
    use std::collections::HashSet;

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_push_block() {
        let mz_state = get_mz_state("test_push_block").unwrap();
        let mut temp_blocks = mz_state.temp_blocks.lock().unwrap();
        let block_size: usize = 1210;
        let mut real_blocks: Vec<Block> = Vec::new();
        let mut rng = rand::rngs::StdRng::seed_from_u64(121234);
        real_blocks.push(gen_test_block(
            block_size as u32,
            &vec![(block_size + 1) as u32].into_iter().collect(),
        ));
        for i in 0..block_size {
            let parent_keys: HashSet<u32> = if i > 0 {
                (0..10).map(|_j| rng.random_range(0..i as u32)).collect()
            } else {
                HashSet::new()
            };
            let block = gen_test_block(i as u32, &parent_keys);
            real_blocks.push(block);
        }
        real_blocks.shuffle(&mut rng);
        let mut block_db_len = 0;
        for block in real_blocks {
            temp_blocks.push_block(block).unwrap();
            while let Some(block) = temp_blocks.pop_block() {
                block_db_len += 1;
                set_block(&mz_state.block_storage, &block.key, &block).unwrap();
            }
        }
        assert_eq!(block_db_len, block_size);
    }
}
