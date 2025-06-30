pub mod block_header;
pub mod part_sort_header;
pub mod traits;

pub const EXPECTED_BLOCK_SIZE: usize = 1024 * 256; // 256KB
pub const MAX_PART_SORT_SIZE: usize = 200;
pub const MAX_ANCESTOR_SIZE: usize = 20;
pub const BLOCK_PER_SECOND: u64 = 1;
pub const SECOND_PER_DAY: u64 = 24 * 60 * 60;
pub const DAY_PER_UPDATE: u64 = 2;
pub const POW_TARGET_INTERVAL: u64 = BLOCK_PER_SECOND * SECOND_PER_DAY * DAY_PER_UPDATE;
pub const POW_TARGET_INTERVAL_MS: u64 = 1000 * SECOND_PER_DAY * DAY_PER_UPDATE;

pub const MAZ_PER_TOKEN: u128 = 100_000_000;
pub const BEGIN_BLOCK_REWARD: u128 = 2 * MAZ_PER_TOKEN;
pub const HALF_BLOCK_REWARD: u64 = BLOCK_PER_SECOND * 60 * 24 * 365 * 10;