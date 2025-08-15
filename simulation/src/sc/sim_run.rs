use super::{
    sim_block::SimBlock,
    sim_miner::{Position, gen_sim_minner_list, select_miner},
    sim_storage::SimConsensusHeaderStorage,
};
use crate::sc::sim_miner::calc_distance_delay;
use consensus::{
    MAX_ANCESTOR_SIZE,
    block_header::{ConsensusHeader, PowHeader},
    part_sort_header::gen_part_sort_block,
    traits::GENESIS_BLOCK_KEY,
    types::{BlockKey, DagWork},
};
use crypto_bigint::U256;
use log::info;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::file::write_to_json;

fn dag_work_to_u64(dag_work: DagWork) -> u64 {
    dag_work.0.to_limbs()[0].0
}

#[allow(clippy::panic, clippy::manual_is_finite)]
pub fn run_sim(db_path: &str, miner_num: u64, block_num: u64, block_per_step: f64) {
    std::fs::remove_dir_all(db_path).unwrap();
    let mut storage = SimConsensusHeaderStorage::new(db_path);
    let miners = gen_sim_minner_list(miner_num);
    let mut tips = BTreeSet::new();
    tips.insert(BlockKey::from(GENESIS_BLOCK_KEY));
    let mut part_sort_size: BTreeMap<usize, i64> = BTreeMap::new();
    // let mut tmp_block = Vec::new();
    for i in 2..block_num {
        let selected_miner = select_miner(&miners);
        let local_tips = cal_tips_by_position(
            tips.clone(),
            selected_miner.position,
            i,
            &storage,
            block_per_step,
        );
        let part_sort_header = match gen_part_sort_block(&storage, &local_tips) {
            Ok(part_sort_header) => part_sort_header,
            Err(e) => {
                info!("gen_part_sort_block error: tips  {local_tips:?}");
                panic!("gen_part_sort_block error: {e:?}");
            }
        };
        let new_part_sort_header =
            gen_part_sort_block(&storage, &part_sort_header.parent_keys).unwrap();
        if new_part_sort_header != part_sort_header {
            panic!("new_part_sort_header != part_sort_header");
        }
        *part_sort_size
            .entry(part_sort_header.part_sort.len())
            .or_insert(0) += 1;
        let dw = dag_work_to_u64(part_sort_header.dag_work);
        if dw != part_sort_header.size {
            panic!("dw != part_sort_header.size");
        }
        let distance = i - dw;
        let now_key = BlockKey::from(U256::from_u64(i));
        for parent in &part_sort_header.parent_keys {
            tips.remove(parent);
        }
        let block = SimBlock {
            key: now_key,
            creator_position: selected_miner.position,
            header: ConsensusHeader {
                part_sort_header: part_sort_header.clone(),
                pow_header: PowHeader::default(),
            },
        };
        storage.set_block(now_key, &block).unwrap();
        // tmp_block.push(block);
        // if tmp_block.len() > 100 {
        //     write_to_json("simulation/distance/tmp_block.json", &tmp_block).unwrap();
        //     panic!("tmp_block.len() > 100");
        // }
        tips.insert(now_key);

        while tips.len() > MAX_ANCESTOR_SIZE * 2 {
            tips.pop_first();
        }

        if i % 1000 == 0 {
            info!(
                "i: {i}, distance: {distance}, parent_size: {}, part_sort_size: {}",
                part_sort_header.parent_keys.len(),
                part_sort_header.part_sort.len()
            );
        }
    }
    let output_path = format!(
        "simulation/distance/part_sort_size_{}.json",
        (block_per_step as u64)
    );
    write_to_json(&output_path, &part_sort_size).unwrap();
}

pub fn cal_tips_by_position(
    tips: BTreeSet<BlockKey>,
    position: Position,
    now: u64,
    storage: &SimConsensusHeaderStorage,
    block_per_step: f64,
) -> Vec<BlockKey> {
    let mut readded_tips = BTreeSet::new();
    let mut queue = tips.into_iter().collect::<VecDeque<_>>();
    let mut ans = BTreeSet::new();
    while let Some(tip) = queue.pop_front() {
        if readded_tips.contains(&tip) {
            continue;
        }
        readded_tips.insert(tip);
        let block = storage.get_block(&tip).unwrap().unwrap();
        let i64_key = block.key.0.to_limbs()[0].0;
        let observed_time =
            i64_key + calc_distance_delay(&block.creator_position, &position, block_per_step);
        if observed_time <= now || tip == BlockKey::from(GENESIS_BLOCK_KEY) {
            ans.insert(tip);
        } else {
            for parent in block.header.part_sort_header.parent_keys {
                queue.push_back(parent);
            }
        }
    }
    ans.into_iter().collect::<Vec<_>>()
}
