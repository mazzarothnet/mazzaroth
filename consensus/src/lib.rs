pub mod block_header;
pub mod part_sort_header;
pub mod traits;
pub mod types;

pub const MAX_PART_SORT_SIZE: usize = 200;
pub const MAX_ANCESTOR_SIZE: usize = 15;
pub const BLOCK_PER_SECOND: u64 = 1;
pub const SECOND_PER_DAY: u64 = 24 * 60 * 60;
pub const DAY_PER_UPDATE: u64 = 2;
pub const POW_TARGET_SIZE: u64 = BLOCK_PER_SECOND * SECOND_PER_DAY * DAY_PER_UPDATE;
pub const POW_TARGET_INTERVAL_MS: u64 = 1000 * SECOND_PER_DAY * DAY_PER_UPDATE;

pub const WEI_PER_MTH: u128 = 1_000_000_000_000_000_000;
pub const BEGIN_BLOCK_REWARD: u128 = 2 * WEI_PER_MTH;
pub const HALF_BLOCK_REWARD_SIZE: u64 = BLOCK_PER_SECOND * 60 * 60 * 24 * 365 * 3;

pub const STO_BYTES_PER_ACCOUNT: u128 = 33u128 + 32u128 + 16u128 + 32u128 * 32u128;
pub const STO_WEI_PER_BYTE: u128 = 200_000_000_000_000;
pub const STO_ACCOUNT_MIN_BALANCE: u128 = STO_BYTES_PER_ACCOUNT * STO_WEI_PER_BYTE;

pub const BLOCK_GAS_LIMIT: u128 = 30_000_000;
pub const TRANSFER_GAS: u128 = 30_000;
pub const MERGE_GAS: u128 = 50_000;

pub fn get_now_block_reward(block_num: u64) -> u128 {
    let now_stage = block_num / HALF_BLOCK_REWARD_SIZE;
    BEGIN_BLOCK_REWARD / (1 << now_stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_now_block_reward() {
        let reward1 = get_now_block_reward(0);
        assert_eq!(reward1, BEGIN_BLOCK_REWARD);
        let reward2 = get_now_block_reward(HALF_BLOCK_REWARD_SIZE);
        assert_eq!(reward2, BEGIN_BLOCK_REWARD / 2);
        let reward3 = get_now_block_reward(HALF_BLOCK_REWARD_SIZE * 2);
        assert_eq!(reward3, BEGIN_BLOCK_REWARD / 4);
        let reward4 = get_now_block_reward(HALF_BLOCK_REWARD_SIZE * 3);
        assert_eq!(reward4, BEGIN_BLOCK_REWARD / 8);
        let reward5 = get_now_block_reward(HALF_BLOCK_REWARD_SIZE * 4);
        assert_eq!(reward5, BEGIN_BLOCK_REWARD / 16);
    }
}
