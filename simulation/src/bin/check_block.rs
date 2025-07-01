use consensus::{part_sort_header::gen_part_sort_block, types::BlockKey};
use crypto_bigint::U256;
use simulation::sc::sim_storage::SimConsensusHeaderStorage;
use utils::{file::write_to_json, log::init_log};

#[allow(clippy::unwrap_used)]
fn main() {
    init_log();
    let storage = SimConsensusHeaderStorage::new("test.db");
    for i in 2..100 {
        let block = storage
            .get_block(&BlockKey::from(U256::from_u64(i)))
            .unwrap()
            .unwrap();
        write_to_json(&format!("debug/block_{i}.json"), &block).unwrap();
        let _ = gen_part_sort_block(&storage, &block.header.part_sort_header.parent_keys).unwrap();
    }
    let tips = vec![
        BlockKey::from(U256::from_u64(0)),
        BlockKey::from(U256::from_u64(4)),
    ];
    let part_sort_header = gen_part_sort_block(&storage, &tips).unwrap();
    println!("{part_sort_header:?}");
}
