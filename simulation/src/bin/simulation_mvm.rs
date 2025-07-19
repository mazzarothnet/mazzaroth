use log::info;
use rand::{SeedableRng, rngs::StdRng};
use simulation::smvm::random_block::{gen_rand_blocks, new_test_mvm};
use std::collections::BTreeMap;
use utils::{file::write_to_json, log::init_log};

fn main() {
    init_log();
    let mut mvm = new_test_mvm("test_mvm");
    //let mut rng = StdRng::seed_from_u64(1112331);
    let mut rng = StdRng::seed_from_u64(11891);
    let block_num = 33;
    let account_num = 50;
    let blocks = gen_rand_blocks(&mut rng, block_num, account_num);
    info!("gen rand blocks done");
    let mut forward_map = BTreeMap::new();
    for block in &blocks {
        info!("do block {}", block.key);
        mvm.do_block(block).unwrap();
        let now_state_map = mvm.get_state_root().unwrap();
        info!("do block {} done", block.key);
        forward_map.insert(block.key, now_state_map);
    }
    let mut backward_map = BTreeMap::new();
    for block in blocks.iter().rev() {
        info!("do block rollback {}", block.key);
        let now_state_map = mvm.get_state_root().unwrap();
        mvm.do_block_rollback(block).unwrap();
        info!("do block rollback {} done", block.key);
        backward_map.insert(block.key, now_state_map);
    }

    write_to_json("test_mvm/forward_map.json", &forward_map).unwrap();
    write_to_json("test_mvm/backward_map.json", &backward_map).unwrap();
}
