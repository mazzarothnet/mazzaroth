use super::{
    sim_block::{SimBlock, SimKey},
    sim_miner::{Position, gen_sim_minner_list, select_miner},
    sim_storage::SimDagStorage,
};
use crate::sc::sim_miner::calc_distance_delay;
use consensus::{
    MAX_PART_SORT_SIZE,
    part_sort::part_sort_with_cache,
    real_tips::{
        cal_in_degree_without_check, cal_real_tips_without_head, get_link_set, get_max_size_key,
        get_well_connected_keys,
    },
    traits::{DagStorage, Key},
};
use log::info;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::file::write_to_json;

pub fn run_sim(db_path: &str, miner_num: u64, block_num: u64, block_per_step: f64) {
    std::fs::remove_dir_all(db_path).unwrap();
    let db = rocksdb::DB::open_default(db_path).unwrap();
    let mut storage = SimDagStorage::new(db);
    let miners = gen_sim_minner_list(miner_num);
    let mut tips = BTreeSet::new();
    let genesis_key = SimKey(0);
    let genesis_block = SimBlock {
        key: genesis_key,
        creator_position: Position { x: 0.0, y: 0.0 },
        parent_keys: vec![],
    };
    storage.set_block(genesis_key, &genesis_block).unwrap();
    part_sort_with_cache(&mut storage, genesis_key).unwrap();
    tips.insert(genesis_key);
    let mut lca_distance: BTreeMap<i64, i64> = BTreeMap::new();
    for i in 1..block_num {
        let selected_miner = select_miner(&miners);
        let local_tips = cal_tips_by_position(
            tips.clone(),
            selected_miner.position,
            i,
            &storage,
            block_per_step,
        );
        info!("now: {}", i);
        let now_key = SimKey(i);
        let block = SimBlock {
            key: now_key,
            creator_position: selected_miner.position,
            parent_keys: local_tips,
        };
        info!("block parent len: {}", block.parent_keys.len());
        storage.set_block(block.key, &block).unwrap();
        tips.insert(now_key);
        for parent in block.parent_keys {
            tips.remove(&parent);
        }
        let now_size = part_sort_with_cache(&mut storage, now_key).unwrap().size;
        let lca = cal_lca_of_tips(tips.clone(), &storage);
        if let Some(lca) = lca {
            let lca_size = storage.get_part_sort_of_key(&lca).unwrap().unwrap().size;
            let distance = (now_size - lca_size) as i64;
            info!("lca: {} distance: {}", lca_size, distance);
            let entry = lca_distance.entry(distance).or_insert(0);
            *entry += 1;
        } else {
            let entry = lca_distance.entry(-1).or_insert(0);
            *entry += 1;
            info!("Error: lca is None");
            break;
        }
    }
    let output_path = format!("distance/distance_{}.json", (block_per_step as u64));
    write_to_json(&output_path, &lca_distance).unwrap();
}

fn cal_ancestors(key: SimKey, storage: &SimDagStorage) -> Vec<SimKey> {
    let mut ans = Vec::new();
    let mut current = Some(key);
    while let Some(key) = current {
        if ans.len() > MAX_PART_SORT_SIZE {
            return ans;
        }
        ans.push(key);
        let block = storage.get_part_sort_of_key(&key).unwrap().unwrap();
        current = block.head_key;
    }
    ans
}
fn cal_lca_of_tips(tips: BTreeSet<SimKey>, storage: &SimDagStorage) -> Option<SimKey> {
    let mut ancestors = Vec::new();
    for tip in tips {
        ancestors.push(cal_ancestors(tip, storage));
    }
    let first_ancestors = ancestors.pop().unwrap();
    let ancestors = ancestors
        .into_iter()
        .map(|v| v.into_iter().collect::<BTreeSet<_>>())
        .collect::<Vec<_>>();
    for key in first_ancestors {
        let mut flag = true;
        for ancestor in ancestors.iter() {
            if !ancestor.contains(&key) {
                flag = false;
                break;
            }
        }
        if flag {
            return Some(key);
        }
    }
    None
}

pub fn cal_tips_by_position(
    tips: BTreeSet<SimKey>,
    position: Position,
    now: u64,
    storage: &SimDagStorage,
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
    let parent_keys = ans.into_iter().collect::<Vec<_>>();
    let (selected_tips, _) = get_max_size_key(storage, &parent_keys).unwrap();
    let link_set = get_link_set(storage, selected_tips).unwrap();
    let well_connected_keys = get_well_connected_keys(storage, &link_set, &parent_keys).unwrap();
    let in_degree = cal_in_degree_without_check(storage, &well_connected_keys, &link_set).unwrap();
    let mut real_tips = cal_real_tips_without_head(well_connected_keys, &in_degree).unwrap();
    real_tips.push(selected_tips);
    real_tips
}
