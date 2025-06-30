use super::traits::PartSortHeader;
use crate::{
    MAX_ANCESTOR_SIZE, MAX_PART_SORT_SIZE,
    traits::{BlockKey, ConsensusHeaderStorage, DagWork, GENESIS_BLOCK_KEY},
};
use log::debug;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::error::{Error, Result};

fn get_work_from_target(target: &BlockKey) -> DagWork {
    DagWork::MAX / *target
}

pub fn gen_part_sort_block<S: ConsensusHeaderStorage>(
    storage: &S,
    parent_keys: &[BlockKey],
) -> Result<PartSortHeader> {
    let (selected_head_key, mut selected_head_dag_work, selected_size) =
        get_max_dag_work_key(storage, parent_keys)?;
    debug!("selected_head_key: {:?}", selected_head_key);
    let link_set = get_link_set(storage, selected_head_key)?;
    debug!("link_set: {:?}", link_set);
    let well_connected_keys = get_well_connected_keys(storage, &link_set, parent_keys)?;
    debug!("well_connected_keys: {:?}", well_connected_keys);
    let in_degree = cal_in_degree_without_check(storage, &well_connected_keys, &link_set.link_set)?;
    debug!("in_degree: {:?}", in_degree);
    let real_tips_without_head = cal_real_tips_without_head(well_connected_keys, &in_degree)?;
    debug!("real_tips_without_head: {:?}", real_tips_without_head);
    let top_sort = cal_top_sort(
        storage,
        &real_tips_without_head,
        &selected_head_key,
        in_degree,
    )?;
    debug!("top_sort: {:?}", top_sort);
    check_top_sort(&top_sort)?;
    let mut parent_keys = real_tips_without_head;
    parent_keys.push(selected_head_key);
    for key in &top_sort {
        let block_header = storage.get_consensus_header(key)?;
        selected_head_dag_work += get_work_from_target(&block_header.pow_header.target);
    }
    let part_sort_block = PartSortHeader {
        head_key: Some(selected_head_key),
        dag_work: selected_head_dag_work,
        parent_keys,
        size: selected_size + top_sort.len() as u64,
        part_sort: top_sort,
    };
    Ok(part_sort_block)
}

fn get_max_dag_work_key<S: ConsensusHeaderStorage>(
    storage: &S,
    parent_keys: &[BlockKey],
) -> Result<(BlockKey, DagWork, u64)> {
    let mut selected_key = *parent_keys.first().ok_or_else(|| Error::EmptyParentKeys)?;
    let mut selected_dag_work = DagWork::ZERO;
    let mut selected_size = 0;
    for parent_key in parent_keys {
        let block_header = storage.get_consensus_header(parent_key)?;
        if block_header.part_sort_header.dag_work > selected_dag_work
            || (block_header.part_sort_header.dag_work == selected_dag_work
                && parent_key > &selected_key)
        {
            selected_size = block_header.part_sort_header.size;
            selected_key = *parent_key;
            selected_dag_work = block_header.part_sort_header.dag_work;
        }
    }
    Ok((selected_key, selected_dag_work, selected_size))
}

#[derive(Debug)]
struct LinkSet<K> {
    link_set: BTreeSet<K>,
    head_link_set: BTreeSet<K>,
}

fn get_link_set<S: ConsensusHeaderStorage>(storage: &S, head_key: BlockKey) -> Result<LinkSet<BlockKey>> {
    let mut link_set: BTreeSet<BlockKey> = BTreeSet::new();
    let mut head_link_set: BTreeSet<BlockKey> = BTreeSet::new();
    let mut now_key = head_key;
    while head_link_set.len() < MAX_ANCESTOR_SIZE {
        let header = storage.get_consensus_header(&now_key)?;
        if head_link_set.contains(&now_key) {
            return Err(Error::CycleDependency {
                key: now_key.to_string(),
            });
        }
        link_set.extend(header.part_sort_header.part_sort.into_iter());
        link_set.insert(now_key);
        head_link_set.insert(now_key);
        if let Some(head_key) = header.part_sort_header.head_key {
            now_key = head_key;
        } else {
            break;
        }
    }

    Ok(LinkSet {
        link_set,
        head_link_set,
    })
}

fn get_well_connected_keys<S: ConsensusHeaderStorage>(
    storage: &S,
    link_set: &LinkSet<BlockKey>,
    parent_keys: &[BlockKey],
) -> Result<Vec<BlockKey>> {
    let mut well_connected_keys: Vec<BlockKey> = Vec::new();
    for key in parent_keys {
        if check_well_connected_block(storage, *key, &link_set.link_set)?
            && check_head_well_connected_block(storage, *key, &link_set.head_link_set)?
        {
            well_connected_keys.push(*key);
        }
    }
    Ok(well_connected_keys)
}

fn check_well_connected_block<S: ConsensusHeaderStorage>(
    storage: &S,
    key: BlockKey,
    link_set: &BTreeSet<BlockKey>,
) -> Result<bool> {
    if link_set.contains(&key) {
        return Ok(false);
    }
    let mut queue: VecDeque<BlockKey> = VecDeque::new();
    let mut readded_set: BTreeSet<BlockKey> = BTreeSet::new();
    queue.push_back(key);
    while let Some(now_key) = queue.pop_front() {
        if readded_set.contains(&now_key) {
            continue;
        }
        readded_set.insert(now_key);
        let parent_keys = storage
            .get_consensus_header(&now_key)?
            .part_sort_header
            .parent_keys;
        if readded_set.len() >= MAX_PART_SORT_SIZE
            || now_key == GENESIS_BLOCK_KEY
            || parent_keys.is_empty()
        {
            return Ok(false);
        }
        for parent_key in parent_keys {
            if !link_set.contains(&parent_key) {
                queue.push_back(parent_key);
            }
        }
    }

    Ok(true)
}

fn check_head_well_connected_block<S: ConsensusHeaderStorage>(
    storage: &S,
    key: BlockKey,
    head_link_set: &BTreeSet<BlockKey>,
) -> Result<bool> {
    if head_link_set.contains(&key) {
        return Ok(false);
    }
    let mut count = 0;
    let mut now_key = key;
    while count < MAX_ANCESTOR_SIZE {
        let header = storage.get_consensus_header(&now_key)?;
        if let Some(head_key) = header.part_sort_header.head_key {
            if head_link_set.contains(&head_key) {
                return Ok(true);
            }
            count += 1;
            now_key = head_key;
        } else {
            break;
        }
    }

    Ok(false)
}

fn cal_in_degree_without_check<S: ConsensusHeaderStorage>(
    storage: &S,
    keys: &[BlockKey],
    link_set: &BTreeSet<BlockKey>,
) -> Result<BTreeMap<BlockKey, i32>> {
    let mut in_degree: BTreeMap<BlockKey, i32> = BTreeMap::new();
    let mut readded_set: BTreeSet<BlockKey> = BTreeSet::new();
    let mut queue: VecDeque<BlockKey> = VecDeque::new();
    for key in keys {
        queue.push_back(*key);
    }
    while let Some(now_key) = queue.pop_front() {
        if readded_set.contains(&now_key) {
            continue;
        }
        readded_set.insert(now_key);
        let parent_keys = storage
            .get_consensus_header(&now_key)?
            .part_sort_header
            .parent_keys;
        for parent_key in parent_keys {
            if !link_set.contains(&parent_key) {
                *in_degree.entry(parent_key).or_insert(0) += 1;
                queue.push_back(parent_key);
            }
        }
    }
    Ok(in_degree)
}

fn cal_real_tips_without_head(
    well_connected_keys: Vec<BlockKey>,
    in_degree: &BTreeMap<BlockKey, i32>,
) -> Result<Vec<BlockKey>> {
    let mut real_tips: Vec<BlockKey> = Vec::new();
    for key in well_connected_keys {
        if in_degree.get(&key).unwrap_or(&0) == &0 {
            real_tips.push(key);
        }
    }
    Ok(real_tips)
}

type TopSort = BTreeMap<DagWork, BTreeSet<BlockKey>>;

fn push_top_sort(top_sort: &mut TopSort, key: BlockKey, dag_work: DagWork) {
    top_sort.entry(dag_work).or_default().insert(key);
}

fn pop_top_sort(top_sort: &mut TopSort) -> Option<BlockKey> {
    let (size, mut keys) = top_sort.pop_first()?;
    let key = keys.pop_first()?;
    if !keys.is_empty() {
        top_sort.insert(size, keys);
    }
    Some(key)
}

#[allow(clippy::comparison_chain)]
fn cal_top_sort<S: ConsensusHeaderStorage>(
    storage: &S,
    real_tips: &[BlockKey],
    selected_header: &BlockKey,
    mut in_degree: BTreeMap<BlockKey, i32>,
) -> Result<Vec<BlockKey>> {
    let mut top_sort: TopSort = Default::default();
    for key in real_tips {
        let header = storage.get_consensus_header(key)?;
        push_top_sort(&mut top_sort, *key, header.part_sort_header.dag_work);
    }
    let mut temp_sort: VecDeque<BlockKey> = VecDeque::new();
    while let Some(key) = pop_top_sort(&mut top_sort) {
        for parent_key in storage.get_consensus_header(&key)?.part_sort_header.parent_keys {
            if let Some(degree) = in_degree.get_mut(&parent_key) {
                *degree -= 1;
                if *degree == 0 {
                    let header = storage.get_consensus_header(&parent_key)?;
                    push_top_sort(&mut top_sort, parent_key, header.part_sort_header.dag_work);
                }
                // check
                else if *degree < 0 {
                    return Err(Error::CycleDependency {
                        key: parent_key.to_string(),
                    });
                }
            }
        }
        temp_sort.push_front(key);
    }
    temp_sort.push_front(*selected_header);
    Ok(temp_sort.into_iter().collect())
}

fn check_top_sort(top_sort: &[BlockKey]) -> Result<()> {
    let sort_set = top_sort.iter().cloned().collect::<BTreeSet<_>>();
    if sort_set.len() != top_sort.len() {
        return Err(Error::TopSortError);
    }
    Ok(())
}
