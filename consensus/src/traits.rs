use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utils::error::Result;

pub trait BlockKeyTrait:
    Clone + Copy + Ord + Eq + Serialize + DeserializeOwned + Sized + Send + 'static
{
    fn is_genesis(&self) -> bool;
    fn serde_to_string(&self) -> Result<String>;
    fn from_string(s: &str) -> Result<Self>;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "K: DeserializeOwned", serialize = "K: Serialize"))]
pub struct PartSortHeader<K: BlockKeyTrait> {
    pub head_key: Option<K>,
    pub size: u64,
    pub parent_keys: Vec<K>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "K: DeserializeOwned", serialize = "K: Serialize"))]
pub struct PartSortPackage<K: BlockKeyTrait> {
    pub part_sort: Vec<K>,
    pub header: PartSortHeader<K>,
}

/// Dag must is full connected, no isolated node
pub trait BlockStorage {
    type KeyType: BlockKeyTrait;

    fn get_parent_keys(&self, key: &Self::KeyType) -> Result<Vec<Self::KeyType>>;

    fn get_part_sort_block_of_key(
        &self,
        key: &Self::KeyType,
    ) -> Result<Option<PartSortPackage<Self::KeyType>>>;

    fn set_part_sort_block_of_key(
        &mut self,
        key: Self::KeyType,
        package: &PartSortPackage<Self::KeyType>,
    ) -> Result<()>;
}