use crate::{
    MAX_PART_SORT_SIZE,
    traits::{DagStorage, Key, SortStruct},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::error::{Error, Result};

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

fn get_part_sort<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    key: &K,
) -> Result<SortStruct<K>> {
    if let Some(part_sort) = storage.get_part_sort_of_key(key)? {
        Ok(part_sort)
    } else {
        Err(Error::ParentNotSorted {
            message: format!("{:?}", serde_json::to_string(key)),
        })
    }
}

pub fn get_max_size_key<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    parent_keys: &[K],
) -> anyhow::Result<(K, u64)> {
    let mut selected_head_size = 0;
    let mut selected_head_key = None;
    for parent_key in parent_keys {
        let part_sort = get_part_sort(storage, parent_key)?;
        if let Some(comp) = selected_head_key {
            if part_sort.size > selected_head_size
                || (part_sort.size == selected_head_size && part_sort.key > comp)
            {
                selected_head_key = Some(part_sort.key);
                selected_head_size = part_sort.size;
            }
        } else {
            selected_head_key = Some(part_sort.key);
            selected_head_size = part_sort.size;
        }
    }
    if let Some(selected_head_key) = selected_head_key {
        Ok((selected_head_key, selected_head_size))
    } else {
        Err(anyhow::anyhow!("IsolateBlock"))
    }
}

pub fn get_well_connected_keys<K: Key, S: DagStorage<KeyType = K>>(
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

pub fn check_top_sort<K: Key>(top_sort: &[K], now_key: K) -> Result<()> {
    let sort_set = top_sort.iter().cloned().collect::<BTreeSet<_>>();
    if sort_set.len() != top_sort.len() {
        return Err(Error::Custom {
            message: format!(
                "now key: {:?}, sort_set.len() != top_sort.len() {:?}",
                serde_json::to_string(&now_key),
                serde_json::to_string(top_sort)
            ),
        });
    }
    Ok(())
}

pub fn get_link_set<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    head_key: K,
) -> Result<BTreeSet<K>> {
    let mut link_set: BTreeSet<K> = BTreeSet::new();
    let mut now_key = head_key;
    while link_set.len() < MAX_PART_SORT_SIZE {
        let part_sort = get_part_sort(storage, &now_key)?;
        link_set.extend(part_sort.part_sort.into_iter());
        if let Some(head_key) = part_sort.head_key {
            now_key = head_key;
        } else {
            break;
        }
    }

    Ok(link_set)
}

pub fn cal_real_tips_without_head<K: Key>(
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

#[allow(clippy::comparison_chain)]
pub fn cal_top_sort<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    now_key: K,
    real_tips: Vec<K>,
    mut in_degree: BTreeMap<K, i32>,
) -> Result<Vec<K>> {
    let mut top_sort: TopSort<K> = Default::default();
    for key in real_tips {
        let part_sort = get_part_sort(storage, &key)?;
        push_top_sort(&mut top_sort, key, part_sort.size);
    }
    let mut temp_sort: VecDeque<K> = VecDeque::new();
    while let Some(key) = pop_top_sort(&mut top_sort) {
        for parent_key in storage.get_parent_keys(&key)? {
            if let Some(degree) = in_degree.get_mut(&parent_key) {
                *degree -= 1;
                if *degree == 0 {
                    let part_sort = get_part_sort(storage, &parent_key)?;
                    push_top_sort(&mut top_sort, parent_key, part_sort.size);
                }
                // check
                else if *degree < 0 {
                    return Err(Error::CycleDependency {
                        message: format!("{:?}", serde_json::to_string(&parent_key)),
                    });
                }
            }
        }
        temp_sort.push_front(key);
    }
    temp_sort.push_back(now_key);
    Ok(temp_sort.into_iter().collect())
}

pub fn cal_in_degree_without_check<K: Key, S: DagStorage<KeyType = K>>(
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

pub fn check_well_connected_block<K: Key, S: DagStorage<KeyType = K>>(
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
