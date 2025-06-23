use super::{
    sim_block::{SimBlock, SimKey},
    sim_miner::{Position, gen_sim_minner_list, select_miner},
    sim_storage::SimBlockStorage,
};
use crate::sc::sim_miner::calc_distance_delay;
use consensus::{
    part_sort_header::gen_part_sort_block,
    traits::{BlockKeyTrait, BlockStorage, PartSortPackage},
};
use log::info;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::file::write_to_json;

fn cal_part_sort_block_and_storage(
    storage: &mut SimBlockStorage,
    now_key: SimKey,
    parent_keys: &[SimKey],
) -> anyhow::Result<PartSortPackage<SimKey>> {
    let part_sort_block = gen_part_sort_block(storage, parent_keys)?;
    storage.set_part_sort_block_of_key(now_key, &part_sort_block)?;
    Ok(part_sort_block)
}

pub fn run_sim(db_path: &str, miner_num: u64, block_num: u64, block_per_step: f64) {
    std::fs::remove_dir_all(db_path).unwrap();
    let db = rocksdb::DB::open_default(db_path).unwrap();
    let mut storage = SimBlockStorage::new(db);
    let miners = gen_sim_minner_list(miner_num);
    let mut tips = BTreeSet::new();
    let genesis_key = SimKey(0);
    let genesis_block = SimBlock {
        key: genesis_key,
        creator_position: Position { x: 0.0, y: 0.0 },
        parent_keys: vec![],
    };
    storage.set_block(genesis_key, &genesis_block).unwrap();
    cal_part_sort_block_and_storage(&mut storage, genesis_key, &[]).unwrap();
    tips.insert(genesis_key);
    let mut lca_distance: BTreeMap<u64, i64> = BTreeMap::new();
    let mut part_sort_size: BTreeMap<usize, i64> = BTreeMap::new();
    for i in 1..block_num {
        //let time = std::time::Instant::now();
        let selected_miner = select_miner(&miners);
        let local_tips = cal_tips_by_position(
            tips.clone(),
            selected_miner.position,
            i,
            &storage,
            block_per_step,
        );
        //let time_cal_tips = time.elapsed();
        //info!("time_cal_tips: {:?}", time_cal_tips);
        let part_sort_block =
            cal_part_sort_block_and_storage(&mut storage, SimKey(i), &local_tips).unwrap();
        //let time_cal_part_sort_block = time.elapsed();
        //info!("time_cal_part_sort_block: {:?}", time_cal_part_sort_block);
        info!("now: {i}");
        *part_sort_size
            .entry(part_sort_block.part_sort.len())
            .or_insert(0) += 1;
        let now_key = SimKey(i);
        let block = SimBlock {
            key: now_key,
            creator_position: selected_miner.position,
            parent_keys: part_sort_block.header.parent_keys.clone(),
        };
        info!("block parent len: {}", block.parent_keys.len());
        storage.set_block(block.key, &block).unwrap();
        //let time_set_block = time.elapsed();
        //info!("time_set_block: {:?}", time_set_block);
        tips.insert(now_key);
        for parent in block.parent_keys {
            tips.remove(&parent);
        }
        // let distance = part_sort_block.header.distance as i64;
        let distance = i - part_sort_block.header.size;
        let entry = lca_distance.entry(distance).or_insert(0);
        *entry += 1;
        if distance == 0 {
            info!("Error:distance is 0");
        } else {
            info!("distance: {distance}");
        }
    }
    let output_path = format!(
        "simulation/distance/distance_{}.json",
        (block_per_step as u64)
    );
    write_to_json(&output_path, &lca_distance).unwrap();
    let output_path = format!(
        "simulation/distance/part_sort_size_{}.json",
        (block_per_step as u64)
    );
    write_to_json(&output_path, &part_sort_size).unwrap();
}

pub fn cal_tips_by_position(
    tips: BTreeSet<SimKey>,
    position: Position,
    now: u64,
    storage: &SimBlockStorage,
    block_per_step: f64,
) -> Vec<SimKey> {
    let mut readded_tips = BTreeSet::new();
    let mut queue = tips.into_iter().collect::<VecDeque<_>>();
    let mut ans = BTreeSet::new();
    while let Some(tip) = queue.pop_front() {
        if readded_tips.contains(&tip) {
            continue;
        }
        readded_tips.insert(tip);
        let block = storage.get_block(&tip).unwrap().unwrap();
        let observed_time =
            block.key.0 + calc_distance_delay(&block.creator_position, &position, block_per_step);
        if observed_time <= now || tip.is_genesis() {
            ans.insert(tip);
        } else {
            for parent in block.parent_keys {
                queue.push_back(parent);
            }
        }
    }
    ans.into_iter().collect::<Vec<_>>()
}
