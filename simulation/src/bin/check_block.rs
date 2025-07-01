use consensus::types::BlockKey;
use crypto_bigint::U256;
use simulation::sc::sim_storage::SimConsensusHeaderStorage;
use utils::{file::write_to_json, log::init_log};


fn main() {
    init_log();
    let storage = SimConsensusHeaderStorage::new("test.db");
    for i in 2..100 { 
        let block = storage.get_block(&BlockKey::from(U256::from_u64(i))).unwrap().unwrap();
        write_to_json(&format!("debug/block_{}.json", i), &block).unwrap();
    }
}