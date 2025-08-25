use crate::traits::{ConsensusHeaderStorage, GENESIS_BLOCK_KEY};
use crate::{POW_TARGET_INTERVAL_MS, POW_TARGET_SIZE, traits::PartSortHeader, types::BlockKey};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use crypto_bigint::{CheckedMul, U256};
use log::info;
use serde::{Deserialize, Serialize};
use utils::error::Error;
use utils::error::Result;

pub const MAX_TARGET: BlockKey = BlockKey(U256::from_be_hex(
    "00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
));

#[derive(
    Clone, Serialize, Deserialize, RlpDecodable, RlpEncodable, Debug, Default, PartialEq, Eq,
)]
pub struct ConsensusHeader {
    pub part_sort_header: PartSortHeader,
    pub pow_header: PowHeader,
}

#[derive(Clone, Serialize, Deserialize, RlpDecodable, RlpEncodable, Debug, PartialEq, Eq)]
pub struct PowHeader {
    pub target: BlockKey,
    pub target_timestamp_ms: u64,
    pub now_timestamp_ms: u64,
    pub head_timestamp_ms: u64,
}

impl Default for PowHeader {
    fn default() -> Self {
        Self {
            target: MAX_TARGET,
            target_timestamp_ms: 0,
            now_timestamp_ms: 0,
            head_timestamp_ms: 0,
        }
    }
}

pub fn gen_consensus_header<S: ConsensusHeaderStorage>(
    storage: &S,
    parent_keys: &[BlockKey],
    now_timestamp_ms: u64,
) -> Result<ConsensusHeader> {
    let part_sort_header = crate::part_sort_header::gen_part_sort_block(storage, parent_keys)?;
    let head_block_header = storage.get_consensus_header(&part_sort_header.head_key)?;
    let mut pow_header = gen_pow_header(
        &head_block_header.pow_header,
        head_block_header.part_sort_header.size,
        part_sort_header.size,
        now_timestamp_ms,
    )
    .ok_or_else(|| Error::Custom {
        message: "gen_pow_header failed".to_string(),
    })?;
    if part_sort_header.head_key == BlockKey(GENESIS_BLOCK_KEY) {
        pow_header.target_timestamp_ms = now_timestamp_ms;
    }
    if pow_header.now_timestamp_ms > now_timestamp_ms {
        return Err(Error::Custom {
            message: "pow_header.now_timestamp_ms is greater than now_timestamp_ms".to_string(),
        });
    }
    Ok(ConsensusHeader {
        part_sort_header,
        pow_header,
    })
}

pub fn gen_pow_header(
    head_block_pow_header: &PowHeader,
    head_size: u64,
    now_size: u64,
    now_timestamp_ms: u64,
) -> Option<PowHeader> {
    let head_timestamp_ms = head_block_pow_header.now_timestamp_ms;
    if now_size / POW_TARGET_SIZE == head_size / POW_TARGET_SIZE {
        return Some(PowHeader {
            target: head_block_pow_header.target,
            target_timestamp_ms: head_block_pow_header.target_timestamp_ms,
            now_timestamp_ms,
            head_timestamp_ms,
        });
    }
    let cast_time_ms = now_timestamp_ms - head_block_pow_header.target_timestamp_ms;
    info!(
        "target: {:?} cast_time_ms: {} pow_target_interval_ms: {}",
        head_block_pow_header.target, cast_time_ms, POW_TARGET_INTERVAL_MS
    );
    let new_target = std::cmp::min(
        get_pow_target(
            head_block_pow_header.target,
            cast_time_ms,
            POW_TARGET_INTERVAL_MS,
        )?,
        MAX_TARGET,
    );
    Some(PowHeader {
        target: new_target,
        target_timestamp_ms: now_timestamp_ms,
        now_timestamp_ms,
        head_timestamp_ms,
    })
}

fn get_pow_target(
    old_target: BlockKey,
    cast_time_ms: u64,
    target_interval_ms: u64,
) -> Option<BlockKey> {
    let target_interval = U256::from_u64(target_interval_ms);
    let cast_time = U256::from_u64(cast_time_ms);
    let old_target_u256 = U256::from(old_target);
    let new_target_u256: Option<U256> = old_target_u256.checked_div(&target_interval).into();
    let new_target_u256 = new_target_u256?;
    let new_target: Option<U256> = new_target_u256.checked_mul(&cast_time).into();
    let new_target = new_target?;
    Some(BlockKey(new_target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_get_pow_target() {
        let target = get_pow_target(BlockKey::from(U256::from_u64(100)), 10 * 2, 10).unwrap();
        assert_eq!(target, BlockKey::from(U256::from_u64(200)));
        let target = get_pow_target(BlockKey::from(U256::from_u64(100)), 5, 10).unwrap();
        assert_eq!(target, BlockKey::from(U256::from_u64(50)));
        let target = get_pow_target(BlockKey::from(U256::MAX), 20, 10);
        assert!(target.is_none());
    }
}
