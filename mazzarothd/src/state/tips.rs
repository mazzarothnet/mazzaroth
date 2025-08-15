use crate::state::block::{has_block, set_block};
use consensus::types::BlockKey;
use mvm::models::block::Block;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    sync::Mutex,
};
use utils::time::get_current_time_ms;

lazy_static::lazy_static! {
    static ref TIPS: Mutex<BTreeMap<BlockKey, u64>> = Mutex::new(BTreeMap::new());
    static ref TEMP_BLOCKS: Mutex<TempBlock> = Mutex::new(TempBlock::new(Box::new(|block| {
        has_block(block)
    })));
}

const TIPS_EXPIRE_MS: u64 = 1000 * 30; // tips expire time 30s

// about check
pub fn push_block(block: Block) -> anyhow::Result<()> {
    if !check_block_format(&block)? {
        return Err(anyhow::anyhow!("block format is not valid"));
    }

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

fn check_block_format(block: &Block) -> anyhow::Result<bool> {
    fn check_vec_unique(vec: &Vec<BlockKey>) -> bool {
        let mut set = HashSet::new();
        for key in vec {
            if set.contains(key) {
                return false;
            }
            set.insert(key);
        }
        true
    }

    if !check_vec_unique(&block.inner.header.part_sort_header.parent_keys) {
        return Err(anyhow::anyhow!("parent_keys is not unique"));
    }

    if !check_vec_unique(&block.inner.header.part_sort_header.part_sort) {
        return Err(anyhow::anyhow!("part_sort is not unique"));
    }

    Ok(true)
}

struct TempBlock {
    unknown_keys: BTreeMap<BlockKey, Vec<BlockKey>>,
    temp_blocks: BTreeMap<BlockKey, (Block, BTreeSet<BlockKey>)>,
    ready_pop: BTreeMap<BlockKey, Block>,
    check_block_fn: Box<dyn Fn(&BlockKey) -> anyhow::Result<bool> + Send + Sync>,
}

impl TempBlock {
    fn new(check_block_fn: Box<dyn Fn(&BlockKey) -> anyhow::Result<bool> + Send + Sync>) -> Self {
        Self {
            unknown_keys: BTreeMap::new(),
            temp_blocks: BTreeMap::new(),
            ready_pop: BTreeMap::new(),
            check_block_fn,
        }
    }
}

impl TempBlock {
    fn push_block(&mut self, block: Block) -> anyhow::Result<()> {
        let mut now_block_unknown_keys = BTreeSet::new();
        for parent in block.inner.header.part_sort_header.parent_keys.iter() {
            if self.unknown_keys.contains_key(parent) || !(self.check_block_fn)(parent)? {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use consensus::{
        block_header::ConsensusHeader,
        types::{AccountKey, Hash},
    };
    use crypto_bigint::U256;
    use mvm::models::block::BlockInner;
    use rand::{Rng, SeedableRng, seq::SliceRandom};

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_push_block() {
        let block_db: Arc<Mutex<BTreeMap<BlockKey, Block>>> = Arc::new(Mutex::new(BTreeMap::new()));
        let block_db_clone = block_db.clone();
        let mut temp_blocks = TempBlock::new(Box::new(move |block_key| {
            let block_db = block_db_clone.lock().unwrap();
            Ok(block_db.contains_key(block_key))
        }));

        let block_size: usize = 1210;
        let mut real_blocks: Vec<Block> = Vec::new();
        let mut rng = rand::rngs::StdRng::seed_from_u64(121234);
        real_blocks.push(gen_test_block(
            block_size as u32,
            vec![(block_size + 1) as u32].into_iter().collect(),
        ));
        for i in 0..block_size {
            let parent_keys: HashSet<u32> = if i > 0 {
                (0..10).map(|_j| rng.random_range(0..i as u32)).collect()
            } else {
                HashSet::new()
            };
            let block = gen_test_block(i as u32, parent_keys);
            real_blocks.push(block);
        }
        real_blocks.shuffle(&mut rng);
        for block in real_blocks {
            temp_blocks.push_block(block).unwrap();
            while let Some(block) = temp_blocks.pop_block() {
                let mut block_db = block_db.lock().unwrap();
                block_db.insert(block.key, block);
            }
        }
        let block_db = block_db.lock().unwrap();
        assert_eq!(block_db.len(), block_size);
    }

    fn gen_test_block(key: u32, parent_keys: HashSet<u32>) -> Block {
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

    fn u32_to_block_key(key: u32) -> BlockKey {
        BlockKey(U256::from_u32(key))
    }
}
