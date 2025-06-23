#![allow(clippy::unwrap_used)]
use consensus::traits::PartSortPackage;
use simulation::sc::{
    sim_block::{SimBlock, SimKey},
    sim_storage::{BLOCK_DATA_TYPE, KeyWrapper, PART_SORT_DATA_TYPE},
};

fn main() {
    let db = rocksdb::DB::open_default("test.db").unwrap();
    // 遍历
    let iter = db.iterator(rocksdb::IteratorMode::Start);
    for res in iter {
        let (key, value) = res.unwrap();
        let key_wrapper: KeyWrapper = bincode::deserialize(&key).unwrap();
        println!("{}", serde_json::to_string(&key_wrapper).unwrap());
        if key_wrapper.data_type == BLOCK_DATA_TYPE {
            let block: SimBlock = bincode::deserialize(&value).unwrap();
            println!("{}", serde_json::to_string(&block).unwrap());
        } else if key_wrapper.data_type == PART_SORT_DATA_TYPE {
            let part_sort: PartSortPackage<SimKey> = bincode::deserialize(&value).unwrap();
            println!("{}", serde_json::to_string(&part_sort).unwrap());
        }
    }
}
