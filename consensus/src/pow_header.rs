use std::{cell::OnceCell, num::NonZeroU64};

use crypto_bigint::{Constants, NonZero, Zero};
use utils::error::Result;

use crate::traits::{Key, PartSortHeader};

pub type BlockKey = crypto_bigint::U256;
pub type TargetSum = crypto_bigint::U512;

const PARAM_A: u64 = 4495;
const PARAM_B: u64 = 42;
const PARAM_C: u64 = 108;
const EXPECT_BLOCK_PER_DELAY: u64 = 5;

pub struct BlockHeader {
    pub key: BlockKey,
    pub version: u32,
    pub nonce: u32,
    pub part_sort_header: PartSortHeader<BlockKey>,
    pub pow_header: PowHeader,
}

pub struct PowHeader {
    pub target_sum: TargetSum,
    pub short_target: BlockKey,
    pub long_target: BlockKey,
    pub distance_sum: u64,
}

pub fn gen_pow_header(
    head_block_header: BlockHeader,
    part_sort_without_self_headers: &[BlockHeader],
    now_part_sort_header: PartSortHeader<BlockKey>,
) -> PowHeader {
    let (distance_sum, target_sum) = cal_total_distance_and_target(part_sort_without_self_headers);

    unimplemented!()
}

fn cal_total_distance_and_target(
    part_sort_without_self_headers: &[BlockHeader],
) -> (u64, TargetSum) {
    let mut distance_sum = 0;
    let mut target_sum = TargetSum::ZERO;

    for header in part_sort_without_self_headers {
        distance_sum += header.part_sort_header.distance;
        target_sum += header.pow_header.target_sum;
    }

    (distance_sum, target_sum)
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

fn get_mmod() -> NonZero<BlockKey> {
    crypto_bigint::NonZero::<BlockKey>::new(BlockKey::MAX).unwrap()
}

fn get_target_sum_mmod() -> NonZero<TargetSum> {
    crypto_bigint::NonZero::<TargetSum>::new(TargetSum::MAX).unwrap()
}

fn cal_target_by_distance(distance: u64, old_target: BlockKey) -> BlockKey {
    if distance == 0 {
        return old_target;
    }
    let (m, d) = estimation_block_per_delay_by_distance(distance);
    let tm = crypto_bigint::NonZero::<BlockKey>::from_u64(NonZeroU64::new(m).unwrap());
    let mmod = get_mmod();
    let td = BlockKey::from(d);
    let ebpd = BlockKey::from(EXPECT_BLOCK_PER_DELAY);
    let tn = old_target.div_rem(&tm).0;

    tn.mul_mod(&td, &mmod).mul_mod(&ebpd, &mmod)
}
