use super::traits::{PartSortBlock, PartSortHeader};
use crate::{
    traits::{DagStorage, Key}, MAX_ANCESTOR_SIZE, MAX_PART_SORT_SIZE
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::error::{Error, Result};

pub fn gen_part_sort_block<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    now_key: K,
    parent_keys: &[K],
) -> Result<PartSortBlock<K>> {
    if now_key.is_genesis() {
        return gen_genesis_part_sort_block(now_key);
    }
    let (selected_head_key, selected_head_size) = get_max_size_key(storage, now_key, parent_keys)?;
    let link_set = get_link_set(storage, selected_head_key)?;
    let well_connected_keys = get_well_connected_keys(storage, &link_set, parent_keys)?;
    let in_degree = cal_in_degree_without_check(storage, &well_connected_keys, &link_set)?;
    let real_tips_without_head = cal_real_tips_without_head(well_connected_keys, &in_degree)?;
    let top_sort = cal_top_sort(storage, now_key, &real_tips_without_head, in_degree)?;
    check_top_sort(&top_sort, now_key)?;
    let mut parent_keys = real_tips_without_head;
    parent_keys.push(selected_head_key);
    let lca = cal_lca_of_tips(&parent_keys, storage)?;
    let now_size = selected_head_size + top_sort.len() as u64;
    let lca_size = get_part_sort_block(storage, &lca)?.header.size;
    let part_sort_block = PartSortBlock {
        key: now_key,
        part_sort: top_sort,
        header: PartSortHeader {
            head_key: Some(selected_head_key),
            size: now_size,
            distance: now_size - lca_size,
            parent_keys,
        },
    };
    Ok(part_sort_block)
}

pub fn cal_lca_of_tips<K: Key, S: DagStorage<KeyType = K>>(tips: &[K], storage: &S) -> Result<K> {
    let mut ancestors = Vec::new();
    for tip in tips {
        ancestors.push(cal_ancestors_link(*tip, storage)?);
    }
    let first_ancestors = ancestors.pop().ok_or(Error::EmptyParentKeys)?;
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
            return Ok(key);
        }
    }
    Err(Error::NoLcaFoundForTips)
}

fn cal_ancestors_link<K: Key, S: DagStorage<KeyType = K>>(key: K, storage: &S) -> Result<Vec<K>> {
    let mut ans = Vec::new();
    let mut current = Some(key);
    while let Some(key) = current {
        if ans.len() > MAX_ANCESTOR_SIZE {
            return Ok(ans);
        }
        ans.push(key);
        let block = get_part_sort_block(storage, &key)?;
        current = block.header.head_key;
    }
    Ok(ans)
}

fn get_part_sort_block<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    key: &K,
) -> Result<PartSortBlock<K>> {
    if let Some(part_sort_block) = storage.get_part_sort_block_of_key(key)? {
        Ok(part_sort_block)
    } else {
        Err(Error::ParentNotSorted {
            key: key.serde_to_string(),
        })
    }
}

fn gen_genesis_part_sort_block<K: Key>(now_key: K) -> Result<PartSortBlock<K>> {
    let ans = PartSortBlock {
        key: now_key,
        part_sort: vec![now_key],
        header: PartSortHeader {
            head_key: None,
            size: 1,
            distance: 0,
            parent_keys: vec![],
        },
    };
    Ok(ans)
}

fn get_max_size_key<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    now_key: K,
    parent_keys: &[K],
) -> Result<(K, u64)> {
    let mut size_and_key: Option<(K, u64)> = None;
    for parent_key in parent_keys {
        let part_sort_block = get_part_sort_block(storage, parent_key)?;
        if let Some(comp) = size_and_key {
            if part_sort_block.header.size > comp.1
                || (part_sort_block.header.size == comp.1 && part_sort_block.key > comp.0)
            {
                size_and_key = Some((part_sort_block.key, part_sort_block.header.size));
            }
        } else {
            size_and_key = Some((part_sort_block.key, part_sort_block.header.size));
        }
    }
    if let Some(size_and_key) = size_and_key {
        Ok(size_and_key)
    } else {
        Err(Error::IsolateBlock {
            key: now_key.serde_to_string(),
        })
    }
}

fn get_link_set<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    head_key: K,
) -> Result<BTreeSet<K>> {
    let mut link_set: BTreeSet<K> = BTreeSet::new();
    let mut now_key = head_key;
    while link_set.len() < MAX_PART_SORT_SIZE {
        let part_sort = get_part_sort_block(storage, &now_key)?;
        link_set.extend(part_sort.part_sort.into_iter());
        if let Some(head_key) = part_sort.header.head_key {
            now_key = head_key;
        } else {
            break;
        }
    }

    Ok(link_set)
}

fn get_well_connected_keys<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    link_set: &BTreeSet<K>,
    parent_keys: &[K],
) -> Result<Vec<K>> {
    let mut well_connected_keys: Vec<K> = Vec::new();
    for key in parent_keys {
        if check_well_connected_block(storage, *key, link_set)? {
            well_connected_keys.push(*key);
        }
    }
    Ok(well_connected_keys)
}

fn check_well_connected_block<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    key: K,
    link_set: &BTreeSet<K>,
) -> Result<bool> {
    if link_set.contains(&key) {
        return Ok(false);
    }
    let mut queue: VecDeque<K> = VecDeque::new();
    let mut readded_set: BTreeSet<K> = BTreeSet::new();
    queue.push_back(key);
    while let Some(now_key) = queue.pop_front() {
        if readded_set.contains(&now_key) {
            continue;
        }
        readded_set.insert(now_key);
        let parent_keys = storage.get_parent_keys(&now_key)?;
        if readded_set.len() >= MAX_PART_SORT_SIZE || now_key.is_genesis() || parent_keys.is_empty()
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

fn cal_in_degree_without_check<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    keys: &[K],
    link_set: &BTreeSet<K>,
) -> Result<BTreeMap<K, i32>> {
    let mut in_degree: BTreeMap<K, i32> = BTreeMap::new();
    let mut readded_set: BTreeSet<K> = BTreeSet::new();
    let mut queue: VecDeque<K> = VecDeque::new();
    for key in keys {
        queue.push_back(*key);
    }
    while let Some(now_key) = queue.pop_front() {
        if readded_set.contains(&now_key) {
            continue;
        }
        readded_set.insert(now_key);
        let parent_keys = storage.get_parent_keys(&now_key)?;
        for parent_key in parent_keys {
            if !link_set.contains(&parent_key) {
                *in_degree.entry(parent_key).or_insert(0) += 1;
                queue.push_back(parent_key);
            }
        }
    }
    Ok(in_degree)
}

fn cal_real_tips_without_head<K: Key>(
    well_connected_keys: Vec<K>,
    in_degree: &BTreeMap<K, i32>,
) -> Result<Vec<K>> {
    let mut real_tips: Vec<K> = Vec::new();
    for key in well_connected_keys {
        if in_degree.get(&key).unwrap_or(&0) == &0 {
            real_tips.push(key);
        }
    }
    Ok(real_tips)
}

type TopSort<K> = BTreeMap<u64, BTreeSet<K>>;

fn push_top_sort<K: Key>(top_sort: &mut TopSort<K>, key: K, size: u64) {
    top_sort.entry(size).or_default().insert(key);
}

fn pop_top_sort<K: Key>(top_sort: &mut TopSort<K>) -> Option<K> {
    let (size, mut keys) = top_sort.pop_first()?;
    let key = keys.pop_first()?;
    if keys.is_empty() {
        top_sort.insert(size, keys);
    }
    Some(key)
}

#[allow(clippy::comparison_chain)]
fn cal_top_sort<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    now_key: K,
    real_tips: &[K],
    mut in_degree: BTreeMap<K, i32>,
) -> Result<Vec<K>> {
    let mut top_sort: TopSort<K> = Default::default();
    for key in real_tips {
        let part_sort = get_part_sort_block(storage, key)?;
        push_top_sort(&mut top_sort, *key, part_sort.header.size);
    }
    let mut temp_sort: VecDeque<K> = VecDeque::new();
    while let Some(key) = pop_top_sort(&mut top_sort) {
        for parent_key in storage.get_parent_keys(&key)? {
            if let Some(degree) = in_degree.get_mut(&parent_key) {
                *degree -= 1;
                if *degree == 0 {
                    let part_sort = get_part_sort_block(storage, &parent_key)?;
                    push_top_sort(&mut top_sort, parent_key, part_sort.header.size);
                }
                // check
                else if *degree < 0 {
                    return Err(Error::CycleDependency {
                        key: parent_key.serde_to_string(),
                    });
                }
            }
        }
        temp_sort.push_front(key);
    }
    temp_sort.push_back(now_key);
    Ok(temp_sort.into_iter().collect())
}

fn check_top_sort<K: Key>(top_sort: &[K], now_key: K) -> Result<()> {
    let sort_set = top_sort.iter().cloned().collect::<BTreeSet<_>>();
    if sort_set.len() != top_sort.len() {
        return Err(Error::Custom {
            message: format!(
                "now key: {}, sort_set.len() != top_sort.len() {}",
                now_key.serde_to_string(),
                top_sort.len()
            ),
        });
    }
    Ok(())
}
