use crate::{
    block_header::ConsensusHeader,
    types::{BlockKey, DagWork},
};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};
use utils::error::Result;

pub const GENESIS_BLOCK_KEY: crypto_bigint::U256 = crypto_bigint::U256::ZERO;

// 如果旷工只连接少量的parent，那么它的dag_work就会很少，容易无效挖矿
// 如果连接过多的parent，那么就无法通过验证，无效挖矿
#[derive(
    Clone, Serialize, Deserialize, PartialEq, Eq, Debug, RlpDecodable, RlpEncodable, Default,
)]
pub struct PartSortHeader {
    pub head_key: BlockKey,
    pub dag_work: DagWork,
    pub size: u64,
    pub parent_keys: Vec<BlockKey>,
    pub part_sort: Vec<BlockKey>,
}

/// Dag must is full connected, no isolated node
pub trait ConsensusHeaderStorage {
    fn get_consensus_header(&self, key: &BlockKey) -> Result<ConsensusHeader>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_key() {
        let key = crypto_bigint::U256::from(1u64);
        let key_u64 = key.to_limbs()[0].0;
        eprintln!("key_u64: {}", key_u64);
        assert_eq!(key_u64, 1);
    }

    #[test]
    fn test_block_key_trans_u256() {
        let kk = crypto_bigint::U256::from(123123u64);
        let kk2 = BlockKey::from(kk);
        let kk3: crypto_bigint::U256 = kk2.into();
        let kk4: BlockKey = kk3.into();
        eprintln!("kk: {:?}", kk);
        eprintln!("kk2: {:?}", kk2);
        eprintln!("kk3: {:?}", kk3);
        eprintln!("kk4: {:?}", kk4);
        assert_eq!(kk, kk3);
        assert_eq!(kk2, kk4);
    }
}
