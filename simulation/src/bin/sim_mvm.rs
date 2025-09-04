#![allow(clippy::unwrap_used)]
use log::info;
use mvm::{core::vm::Mvm, models::block::Block};
use rand::{SeedableRng, rngs::StdRng};
use simulation::smvm::random_block::{gen_rand_blocks, new_test_mvm};
use std::{collections::BTreeMap, fs::File, io::Write};
use utils::{file::write_to_json, log::init_log};

fn main() {
    init_log();
    let mut mvm = new_test_mvm("test_mvm");
    let mut rng = StdRng::seed_from_u64(11891);
    let block_num = 33;
    let account_num = 50;
    let blocks = gen_rand_blocks(&mut rng, block_num, account_num);
    save_blocks(&blocks);
    info!("gen rand blocks done");
    let mut forward_map = BTreeMap::new();
    for block in &blocks {
        info!("do block {}", block.key);
        let mut transaction = mvm.begin_transaction().unwrap();
        Mvm::do_block(&mut transaction, block).unwrap();
        let now_state_map = Mvm::get_state_root(&mut transaction).unwrap();
        transaction.commit(block.key).unwrap();
        forward_map.insert(block.key, now_state_map);
    }
    let mut backward_map = BTreeMap::new();
    for block in blocks.iter().rev() {
        info!("do block rollback {}", block.key);
        let mut transaction = mvm.begin_transaction().unwrap();
        let now_state_map = Mvm::get_state_root(&mut transaction).unwrap();
        Mvm::do_block_rollback(&mut transaction, block).unwrap();
        transaction.commit(block.key).unwrap();
        info!("do block rollback {} done", block.key);
        backward_map.insert(block.key, now_state_map);
    }

    write_to_json("test_mvm/forward_map.json", &forward_map).unwrap();
    write_to_json("test_mvm/backward_map.json", &backward_map).unwrap();
}

pub fn save_blocks(blocks: &Vec<Block>) {
    for block in blocks {
        let mut buf = Vec::new();
        alloy_rlp::Encodable::encode(block, &mut buf);
        let mut file = File::create(format!("test_block/block_{}.rlp", block.key)).unwrap();
        file.write_all(&buf).unwrap();
    }
}
