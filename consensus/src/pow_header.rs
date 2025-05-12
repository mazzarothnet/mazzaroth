use crypto_bigint::Zero;
use serde::{Deserialize, Serialize};
use utils::error::Result;

use crate::{
    EXPECT_BLOCK_PER_DELAY, PARAM_A, PARAM_B, PARAM_C, RECALCULATE_POW_LONG_TARGET_INTERVAL,
    traits::{Key, PartSortHeader},
};

pub type BlockKey = crypto_bigint::U256;
pub type TargetSum = crypto_bigint::U512;

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub key: BlockKey,
    pub version: u32,
    pub nonce: u32,
    pub part_sort_header: PartSortHeader<BlockKey>,
    pub pow_header: PowHeader,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PowHeader {
    pub target_sum: TargetSum,
    pub short_target: BlockKey,
    pub long_target: BlockKey,
    pub distance_sum: u64,
}

pub fn gen_pow_header(
    head_block_pow_header: PowHeader,
    head_size: u64,
    parent_real_target: &[BlockKey],
    now_size: u64,
    now_distance: u64,
) -> PowHeader {
    let average_target = cal_total_distance_and_target(parent_real_target);
    let (block_key_average_target, _): (BlockKey, BlockKey) = average_target.split();
    let short_target = cal_target_by_distance(now_distance, block_key_average_target);
    let part_sort_size = now_size - head_size;
    let pow_header = if head_size / RECALCULATE_POW_LONG_TARGET_INTERVAL
        == now_size / RECALCULATE_POW_LONG_TARGET_INTERVAL
    {
        let long_target = head_block_pow_header.long_target;
        let distance_sum = head_block_pow_header.distance_sum + part_sort_size * now_distance;
        let target_sum = head_block_pow_header.target_sum
            + average_target * TargetSum::from(part_sort_size as u64);
        PowHeader {
            target_sum,
            long_target,
            short_target,
            distance_sum,
        }
    } else {
        let average_distance =
            head_block_pow_header.distance_sum / RECALCULATE_POW_LONG_TARGET_INTERVAL;
        let (average_long_target, _): (BlockKey, BlockKey) = (head_block_pow_header.target_sum
            / TargetSum::from(RECALCULATE_POW_LONG_TARGET_INTERVAL as u64))
        .split();
        let long_target = cal_target_by_distance(average_distance, average_long_target);
        let distance_sum = part_sort_size * now_distance;
        let target_sum = average_target * TargetSum::from(part_sort_size as u64);
        PowHeader {
            target_sum,
            long_target,
            short_target,
            distance_sum,
        }
    };

    pow_header
}

fn cal_total_distance_and_target(parent_real_target: &[BlockKey]) -> TargetSum {
    let mut target_sum = TargetSum::ZERO;

    for real_target in parent_real_target {
        target_sum += real_target.concat(&BlockKey::ZERO);
    }
    let parent_size = TargetSum::from(parent_real_target.len() as u64);

    target_sum / parent_size
}

impl Key for BlockKey {
    fn is_genesis(&self) -> bool {
        bool::from(self.is_zero())
    }
    fn serde_to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn from_string(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s).unwrap())
    }
}

// return mul and div to avoid accuracy loss
fn estimation_block_per_delay_by_distance(distance: u64) -> (u64, u64) {
    (
        PARAM_B * distance + PARAM_B * PARAM_C - PARAM_A,
        distance + PARAM_C,
    )
}

fn cal_target_by_distance(distance: u64, old_target: BlockKey) -> BlockKey {
    if distance == 0 {
        return old_target;
    }
    let (m, d) = estimation_block_per_delay_by_distance(distance);
    let tm = BlockKey::from(m);
    let td = BlockKey::from(d);
    let ebpd = BlockKey::from(EXPECT_BLOCK_PER_DELAY);
    let tn = old_target / tm;
    tn * td * ebpd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimation_block_per_delay_by_distance() {
        let (m, d) = estimation_block_per_delay_by_distance(50);
        assert_eq!(m, 2141);
        assert_eq!(d, 158);
        let (m, d) = estimation_block_per_delay_by_distance(10);
        assert_eq!(m, 461);
        assert_eq!(d, 118);
    }

    #[test]
    fn test_cal_target_by_distance() {
        let old_target = BlockKey::from_u64(1000_000_000u64);
        let distance = 27;
        let new_target = cal_target_by_distance(distance, old_target);
        assert_eq!(new_target, BlockKey::from_u64(574467525u64));
    }

    #[test]
    fn test_cal_total_distance_and_target() {
        let parent_real_target = vec![BlockKey::from_u64(1_000u64), BlockKey::from_u64(2_000u64)];
        let average_target = cal_total_distance_and_target(&parent_real_target);
        assert_eq!(average_target, TargetSum::from_u64(1500u64));
    }

    #[test]
    fn test_gen_pow_header_no_change_long() {
        let head_block_pow_header = PowHeader {
            target_sum: TargetSum::from_u64(0u64),
            long_target: BlockKey::from_u64(900_000_000u64),
            short_target: BlockKey::from_u64(1_000_000_000u64),
            distance_sum: 0,
        };
        let head_size = 100;
        let now_size = 110;
        let now_distance = 50;
        let parent_real_target = vec![
            BlockKey::from_u64(900_000_000u64),
            BlockKey::from_u64(900_000_000u64),
        ];
        let pow_header = gen_pow_header(
            head_block_pow_header,
            head_size,
            &parent_real_target,
            now_size,
            now_distance,
        );
        assert_eq!(pow_header.target_sum, TargetSum::from_u64(900_000_0000u64));
        assert_eq!(pow_header.distance_sum, 500);
        assert_eq!(pow_header.long_target, BlockKey::from_u64(900_000_000u64));
        assert_eq!(pow_header.short_target, BlockKey::from_u64(332087560u64));
    }

    #[test]
    fn test_gen_pow_header_change_long() {
        let head_block_pow_header = PowHeader {
            target_sum: TargetSum::from_u64(900_000_000u64) * TargetSum::from_u64(1209600u64),
            long_target: BlockKey::from_u64(900_000_000u64),
            short_target: BlockKey::from_u64(1_000_000_000u64),
            distance_sum: 1209600 * 15,
        };
        let head_size = 1209500;
        let now_size = 1209700;
        let now_distance = 50;
        let parent_real_target = vec![
            BlockKey::from_u64(900_000_000u64),
            BlockKey::from_u64(900_000_000u64),
        ];
        let pow_header = gen_pow_header(
            head_block_pow_header,
            head_size,
            &parent_real_target,
            now_size,
            now_distance,
        );
        assert_eq!(
            pow_header.target_sum,
            TargetSum::from_u64(180_000_000_000u64)
        );
        assert_eq!(pow_header.distance_sum, 10000);
        assert_eq!(pow_header.long_target, BlockKey::from_u64(824_887_815u64));
        assert_eq!(pow_header.short_target, BlockKey::from_u64(332_087_560u64));
    }
}
