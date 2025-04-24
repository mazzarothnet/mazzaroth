pub mod part_sort;
pub mod real_tips;
pub mod traits;

pub const EXPECTED_BLOCK_SIZE: usize = 1024 * 256; // 256KB
pub const EXPECTED_BLOCK_NUMBER_PER_SECOND: usize = 2;
pub const MAX_PART_SORT_SIZE: usize = EXPECTED_BLOCK_NUMBER_PER_SECOND * 60 * 3; // 360
