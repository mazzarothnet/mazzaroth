use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utils::error::Result;

pub trait Key: Clone + Ord + Eq + Serialize + DeserializeOwned + Sized + Send + 'static {
    fn is_genesis(&self) -> bool;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "K: DeserializeOwned", serialize = "K: Serialize"))]
pub struct SortStruct<K: Key> {
    pub key: K,
    pub head_key: K,
    pub part_sort: Vec<K>,
    pub size: u64,
}

/// Dag must is full connected, no isolated node
pub trait DagStorage {
    type KeyType: Key;

    fn get_parent_keys(&self, key: &Self::KeyType) -> Result<Vec<Self::KeyType>>;

    fn get_part_sort_of_key(
        &self,
        key: &Self::KeyType,
    ) -> Result<Option<SortStruct<Self::KeyType>>>;

    fn set_part_sort_of_key(
        &mut self,
        key: Self::KeyType,
        package: SortStruct<Self::KeyType>,
    ) -> Result<()>;
}
