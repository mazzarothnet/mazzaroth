use serde::{Deserialize, Serialize};
use utils::error::Result;

use crate::block_header::BlockHeader;

pub type BlockKey = crypto_bigint::U256;
pub type DagWork = crypto_bigint::U256;
pub const GENESIS_BLOCK_KEY: BlockKey = BlockKey::ZERO;

// 如果旷工只连接少量的parent，那么它的dag_work就会很少，容易无效挖矿
// 如果连接过多的parent，那么就无法通过验证，无效挖矿
#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct PartSortHeader {
    pub head_key: Option<BlockKey>,
    pub dag_work: DagWork,
    pub parent_keys: Vec<BlockKey>,
    pub part_sort: Vec<BlockKey>,
}

/// Dag must is full connected, no isolated node
pub trait BlockStorage {
    fn get_block_header(&self, key: &BlockKey) -> Result<BlockHeader>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_key() {
        let key = BlockKey::from(1u64);
        let key_u64 = key.to_limbs()[0].0;
        // println!("key_u64: {}", key_u64);
        assert_eq!(key_u64, 1);
    }
}
