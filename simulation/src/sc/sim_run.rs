use super::{sim_miner::{gen_sim_minner_list, select_miner}, sim_storage::SimDagStorage};


#[allow(clippy::unwrap_used)]
pub fn run_sim(db_path: &str, miner_num: u64, block_num: u64) {
    let db = rocksdb::DB::open_default(db_path).unwrap();
    let mut storage = SimDagStorage::new(db);
    let miners = gen_sim_minner_list(miner_num);
    
    for i in 0..block_num {
        let selected_miner = select_miner(&miners);
        
    }
}
