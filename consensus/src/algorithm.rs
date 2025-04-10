use crate::{
    MAX_PART_SORT_SIZE,
    traits::{DagStorage, Key, SortStruct},
};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::error::{Error, Result};

type TopSort<K> = BTreeMap<u64, BTreeSet<K>>;

fn push_top_sort<K:  Key>(
    top_sort: &mut TopSort<K>,
    key: K,
    size: u64,
) {
    top_sort.entry(size).or_insert(BTreeSet::new()).insert(key);
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

/// can't recursion because of the size limit of the stack
pub fn part_sort<K: Key, S: DagStorage<KeyType = K>>(
    storage: &mut S,
    now_key: K,
) -> Result<SortStruct<K>> {
    if let Some(part_sort) = storage.get_part_sort_of_key(&now_key)? {
        return Ok(part_sort);
    }
    if now_key.is_genesis() {
        let ans = SortStruct {
            key: now_key.clone(),
            head_key: now_key.clone(),
            part_sort: vec![now_key.clone()],
            size: 1,
        };
        storage.set_part_sort_of_key(now_key, ans.clone())?;
        return Ok(ans);
    }
    let parent_keys = storage.get_parent_keys(&now_key)?;
    let mut selected_head_key = now_key.clone();
    let mut selected_head_size = 0;
    for parent_key in &parent_keys {
        let part_sort = get_part_sort(storage, parent_key)?;
        if part_sort.size > selected_head_size
            || (part_sort.size == selected_head_size && part_sort.key > selected_head_key)
        {
            selected_head_key = part_sort.key.clone();
            selected_head_size = part_sort.size;
        }
    }
    if selected_head_size == 0 || selected_head_key == now_key {
        return Err(Error::IsolateBlock {
            message: format!("{:?}", serde_json::to_string(&now_key)),
        });
    }

    let link_set = get_temp_link_set(storage, &selected_head_key)?;
    let mut in_degree: BTreeMap<K, i32> = BTreeMap::new();
    let mut well_connected_keys: BTreeSet<K> = BTreeSet::new();
    let mut readded_set: BTreeSet<K> = BTreeSet::new();
    for key in parent_keys {
        if check_well_connected_block(storage, &key, &link_set, &mut in_degree, &mut readded_set)? {
            well_connected_keys.insert(key);
        }
    }

    let mut top_sort: TopSort<K> = Default::default();
    for key in well_connected_keys {
        if in_degree.get(&key).unwrap_or(&0) == &0 {
            let part_sort = get_part_sort(storage, &key)?;
            push_top_sort(&mut top_sort, key, part_sort.size);
        }
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
    temp_sort.push_back(now_key.clone());

    // check
    {
        let sort_set = temp_sort.iter().cloned().collect::<BTreeSet<_>>();
        if sort_set.len() != temp_sort.len() {
            tracing::error!(
                "sort_set.len() != well_connected_keys.len() {:?}",
                serde_json::to_string(&now_key)
            );
        }
    }

    let size = selected_head_size + temp_sort.len() as u64;
    let self_part_sort = SortStruct {
        key: now_key.clone(),
        head_key: selected_head_key.clone(),
        part_sort: temp_sort.into_iter().collect(),
        size,
    };
    storage.set_part_sort_of_key(now_key, self_part_sort.clone())?;
    Ok(self_part_sort)
}

pub fn get_temp_link_set<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    head_key: &K,
) -> Result<BTreeSet<K>> {
    let mut link_set: BTreeSet<K> = BTreeSet::new();
    let mut checked_set: BTreeSet<K> = BTreeSet::new();
    let mut now_key = head_key.clone();
    while link_set.len() < MAX_PART_SORT_SIZE {
        if checked_set.contains(&now_key) {
            continue;
        }
        let part_sort = get_part_sort(storage, &now_key)?;
        link_set.extend(part_sort.part_sort.into_iter());
        checked_set.insert(now_key);
        now_key = part_sort.head_key;
    }

    Ok(link_set)
}

pub fn check_well_connected_block<K: Key, S: DagStorage<KeyType = K>>(
    storage: &S,
    key: &K,
    link_set: &BTreeSet<K>,
    in_degree: &mut BTreeMap<K, i32>,
    readded_set: &mut BTreeSet<K>,
) -> Result<bool> {
    let mut queue: VecDeque<K> = VecDeque::new();
    let mut temp_in_degree: BTreeMap<K, i32> = BTreeMap::new();
    if readded_set.contains(key) {
        return Ok(false);
    }
    queue.push_back(key.clone());
    readded_set.insert(key.clone());
    while let Some(now_key) = queue.pop_front() {
        let parent_keys = storage.get_parent_keys(&now_key)?;
        for parent_key in parent_keys {
            if !link_set.contains(&parent_key) {
                *temp_in_degree.entry(parent_key.clone()).or_insert(0) += 1;
                if !readded_set.contains(&parent_key) {
                    readded_set.insert(parent_key.clone());
                    queue.push_back(parent_key);
                    if readded_set.len() >= MAX_PART_SORT_SIZE {
                        return Ok(false);
                    }
                }
            }
        }
    }
    for (key, degree) in temp_in_degree {
        *in_degree.entry(key).or_insert(0) += degree;
    }

    Ok(true)
}
