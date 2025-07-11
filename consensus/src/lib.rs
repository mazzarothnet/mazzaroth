pub mod block_header;
pub mod part_sort_header;
pub mod traits;
pub mod types;

pub const MAX_PART_SORT_SIZE: usize = 200;
pub const MAX_ANCESTOR_SIZE: usize = 20;
pub const BLOCK_PER_SECOND: u64 = 1;
pub const SECOND_PER_DAY: u64 = 24 * 60 * 60;
pub const DAY_PER_UPDATE: u64 = 2;
pub const POW_TARGET_SIZE: u64 = BLOCK_PER_SECOND * SECOND_PER_DAY * DAY_PER_UPDATE;
pub const POW_TARGET_INTERVAL_MS: u64 = 1000 * SECOND_PER_DAY * DAY_PER_UPDATE;

pub const WEI_PER_MTH: u128 = 1_000_000_000_000_000_000;
pub const BEGIN_BLOCK_REWARD: u128 = 2 * WEI_PER_MTH;
pub const HALF_BLOCK_REWARD: u64 = BLOCK_PER_SECOND * 60 * 60 * 24 * 365 * 3;

pub const STO_BYTES_PER_ACCOUNT: u128 = 33u128 + 32u128 + 16u128 + 32u128 * 32u128;
pub const STO_WEI_PER_BYTE: u128 = 420_000_000_000_000;

pub const BLOCK_GAS_LIMIT: u128 = 30_000_000;
pub const TRANSFER_GAS: u128 = 30_000;
pub const MERGE_GAS: u128 = 50_000;
