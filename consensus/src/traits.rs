use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utils::error::Result;

pub trait Key:
    Clone + Copy + Ord + Eq + Serialize + DeserializeOwned + Sized + Send + 'static
{
    fn is_genesis(&self) -> bool;
    fn serde_to_string(&self) -> String;
    fn from_string(s: &str) -> Result<Self>;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "K: DeserializeOwned", serialize = "K: Serialize"))]
pub struct ConsensusHeader<K: Key> {
    pub head_key: Option<K>,
    pub size: u64,
    pub distance: Option<u64>,
    pub parent_keys: Vec<K>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "K: DeserializeOwned", serialize = "K: Serialize"))]
pub struct ConsensusBlock<K: Key> {
    pub key: K,
    pub part_sort: Vec<K>,
    pub header: ConsensusHeader<K>,
}

/// Dag must is full connected, no isolated node
pub trait DagStorage {
    type KeyType: Key;

    fn get_parent_keys(&self, key: &Self::KeyType) -> Result<Vec<Self::KeyType>>;

    fn get_consensus_block_of_key(
        &self,
        key: &Self::KeyType,
    ) -> Result<Option<ConsensusBlock<Self::KeyType>>>;

    fn set_consensus_block_of_key(
        &mut self,
        key: Self::KeyType,
        package: &ConsensusBlock<Self::KeyType>,
    ) -> Result<()>;
}