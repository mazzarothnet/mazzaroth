use utils::error::Result;

pub trait Key: Sized {
    fn is_genesis(&self) -> bool;
    fn to_string(&self) -> String;
    fn from_string(s: &str) -> Result<Self>;
}

#[derive(Clone)]
pub struct SortStruct<K: Clone + Key> {
    pub key: K,
    pub head_key: K,
    pub part_sort: Vec<K>,
    pub size: u64,
}

/// Dag must is full connected, no isolated node
pub trait DagStorage {
    type Key: Eq + Clone + Send + 'static + Ord + Key;

    fn get_parent_keys(&self, key: &Self::Key) -> Result<Vec<Self::Key>>;

    fn get_part_sort_of_key(&self, key: &Self::Key) -> Result<Option<SortStruct<Self::Key>>>;

    fn set_part_sort_of_key(
        &mut self,
        key: Self::Key,
        package: SortStruct<Self::Key>,
    ) -> Result<()>;
}
