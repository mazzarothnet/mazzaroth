use anyhow::Context;
use consensus::traits::{DagStorage, Key, SortStruct};
use serde::{Deserialize, Serialize};
use utils::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimKey(i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimBlock {
    pub key: SimKey,
    pub ts: i64,
    pub parent_keys: Vec<SimKey>,
}

impl Key for SimKey {
    fn is_genesis(&self) -> bool {
        self.0 == 0
    }
}

pub struct SimDagStorage {
    db: rocksdb::DB,
}

// impl DagStorage for SimDagStorage {
//     type Key = SimKey;

//     fn get_parent_keys(&self, key: &Self::Key) -> Result<Vec<Self::Key>> {
//         let key = key.to_string();
//         let value = self
//             .db
//             .get(key.as_bytes())
//             .context("get parent keys failed")?
//             .ok_or_else(|| Error::UnknownBlock {
//                 message: format!("unknown block: {}", key),
//             })?;
//         let block: SimBlock = bincode::deserialize(&value).context("deserialize sim block failed")?;
//         Ok(block.parent_keys)
//     }

//     fn get_part_sort_of_key(&self, key: &Self::Key) -> Result<Option<SortStruct<Self::Key>>> {
//         let key = key.to_string();
//         let value = self.db.get(key.as_bytes()).context("get part sort of key failed")?;
//         Ok(value)
//     }
// }
