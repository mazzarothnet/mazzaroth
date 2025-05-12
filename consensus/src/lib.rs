pub mod traits;
pub mod part_sort_header;
pub mod pow_header;

pub const EXPECTED_BLOCK_SIZE: usize = 1024 * 256; // 256KB
pub const MAX_PART_SORT_SIZE: usize = 100;
pub const MAX_ANCESTOR_SIZE: usize = 200; 
pub const RECALCULATE_POW_LONG_TARGET_INTERVAL: u64 = 1209600; // 1209600 blocks, 2 * 60 * 60 * 24 * 7
pub const PARAM_A: u64 = 4495;
pub const PARAM_B: u64 = 42;
pub const PARAM_C: u64 = 108;
pub const EXPECT_BLOCK_PER_DELAY: u64 = 5;