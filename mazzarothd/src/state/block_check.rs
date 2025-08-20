use crate::state::block_storage::gen_consensus_header_with_global_storage;
use consensus::{block_header::MAX_TARGET, types::BlockKey};
use crypto_bigint::U256;
use mining::sha256_mining::gen_sha256_by_block_hash_and_nonce;
use mvm::models::block::Block;
use std::collections::HashSet;
use utils::sha256::sha256_hash_rlp;

// not check timestamp because fn will be used in sync history block
// please check timestamp in gossip
pub fn normal_check_block_format(block: &Block) -> anyhow::Result<()> {
    if block.inner.header.pow_header.target > MAX_TARGET {
        return Err(anyhow::anyhow!("target is greater than MAX_TARGET"));
    }
    if block.key > block.inner.header.pow_header.target {
        return Err(anyhow::anyhow!("block key is greater than target"));
    }
    let block_hash = sha256_hash_rlp(&block.inner);
    let mined_hash = gen_sha256_by_block_hash_and_nonce(block_hash, block.nonce);
    let mined_block_key = BlockKey(U256::from_be_slice(&mined_hash));
    if mined_block_key != block.key {
        return Err(anyhow::anyhow!("mined block key is not equal to block key"));
    }

    if !check_vec_unique(&block.inner.header.part_sort_header.parent_keys) {
        return Err(anyhow::anyhow!("parent_keys is not unique"));
    }

    if !check_vec_unique(&block.inner.header.part_sort_header.part_sort) {
        return Err(anyhow::anyhow!("part_sort is not unique"));
    }

    Ok(())
}

pub fn save_block_check(block: &Block) -> anyhow::Result<()> {
    let consensus_header = gen_consensus_header_with_global_storage(
        &block.inner.header.part_sort_header.parent_keys,
        block.inner.header.pow_header.now_timestamp_ms,
    )?;
    if consensus_header != block.inner.header {
        return Err(anyhow::anyhow!(
            "consensus_header is not equal to block.inner.header"
        ));
    }

    Ok(())
}

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
