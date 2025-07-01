use crate::{POW_TARGET_INTERVAL, POW_TARGET_INTERVAL_MS, traits::PartSortHeader, types::BlockKey};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use crypto_bigint::U256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, RlpDecodable, RlpEncodable)]
pub struct ConsensusHeader {
    pub part_sort_header: PartSortHeader,
    pub pow_header: PowHeader,
}

#[derive(Clone, Serialize, Deserialize, RlpDecodable, RlpEncodable)]
pub struct PowHeader {
    pub target: BlockKey,
    pub target_timestamp_ms: u64,
    pub now_timestamp_ms: u64,
}

impl Default for PowHeader {
    fn default() -> Self {
        Self {
            target: BlockKey::from(U256::MAX),
            target_timestamp_ms: 0,
            now_timestamp_ms: 0,
        }
    }
}

pub fn gen_pow_header(
    head_block_pow_header: &PowHeader,
    head_size: u64,
    now_size: u64,
) -> PowHeader {
    let now_timestamp_ms = utils::time::get_current_time_ms();
    if now_size / POW_TARGET_INTERVAL == head_size / POW_TARGET_INTERVAL {
        return PowHeader {
            target: head_block_pow_header.target,
            target_timestamp_ms: head_block_pow_header.target_timestamp_ms,
            now_timestamp_ms,
        };
    }
    let cast_time_ms = now_timestamp_ms - head_block_pow_header.target_timestamp_ms;
    let new_target = get_pow_target(
        head_block_pow_header.target,
        cast_time_ms,
        POW_TARGET_INTERVAL_MS,
    );
    PowHeader {
        target: new_target,
        target_timestamp_ms: now_timestamp_ms,
        now_timestamp_ms,
    }
}

fn get_pow_target(old_target: BlockKey, cast_time_ms: u64, target_interval_ms: u64) -> BlockKey {
    old_target / BlockKey::from(U256::from_u64(target_interval_ms))
        * BlockKey::from(U256::from_u64(cast_time_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pow_target() {
        let target = get_pow_target(BlockKey::from(U256::from_u64(100)), 10 * 2, 10);
        assert_eq!(target, BlockKey::from(U256::from_u64(200)));
        let target = get_pow_target(BlockKey::from(U256::from_u64(100)), 5, 10);
        assert_eq!(target, BlockKey::from(U256::from_u64(50)));
    }
}
