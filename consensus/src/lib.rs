pub mod traits;
pub mod part_sort_header;
pub mod pow_header;

pub const EXPECTED_BLOCK_SIZE: usize = 1024 * 256; // 256KB
pub const MAX_PART_SORT_SIZE: usize = 10 * 60; // 600
pub const RECALCULATE_POW_LONG_TARGET_INTERVAL: u64 = 1209600; // 1000 blocks, 2 * 60 * 60 * 24 * 7
