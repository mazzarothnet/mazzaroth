use super::traits::{PartSortHeader, PartSortPackage};
use crate::{
    MAX_ANCESTOR_SIZE, MAX_PART_SORT_SIZE,
    traits::{BlockKeyTrait, BlockStorage},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::error::{Error, Result};

pub fn gen_part_sort_block<K: BlockKeyTrait, S: BlockStorage<KeyType = K>>(
    storage: &S,
    parent_keys: &[K],
) -> Result<PartSortPackage<K>> {
    let (selected_head_key, selected_head_size) = get_max_size_key(storage, parent_keys)?;
    let link_set = get_link_set(storage, selected_head_key)?;
    let well_connected_keys = get_well_connected_keys(storage, &link_set, parent_keys)?;
    let in_degree = cal_in_degree_without_check(storage, &well_connected_keys, &link_set.link_set)?;
    let real_tips_without_head = cal_real_tips_without_head(well_connected_keys, &in_degree)?;
    let top_sort = cal_top_sort(storage, &real_tips_without_head, in_degree)?;
    check_top_sort(&top_sort)?;
    let mut parent_keys = real_tips_without_head;
    parent_keys.push(selected_head_key);
    let now_size = selected_head_size + top_sort.len() as u64 + 1;
    let part_sort_block = PartSortPackage {
        part_sort: top_sort,
        header: PartSortHeader {
            head_key: Some(selected_head_key),
            size: now_size,
            parent_keys,
        },
    };
    Ok(part_sort_block)
}

fn get_part_sort_block<K: BlockKeyTrait, S: BlockStorage<KeyType = K>>(
    storage: &S,
    key: &K,
) -> Result<PartSortPackage<K>> {
    if let Some(part_sort_block) = storage.get_part_sort_block_of_key(key)? {
        Ok(part_sort_block)
    } else {
        Err(Error::ParentNotSorted {
            key: key.serde_to_string()?,
        })
    }
}

fn get_max_size_key<K: BlockKeyTrait, S: BlockStorage<KeyType = K>>(
    storage: &S,
    parent_keys: &[K],
) -> Result<(K, u64)> {
    let mut selected_key = *parent_keys.first().ok_or_else(|| Error::EmptyParentKeys)?;
    let mut selected_size = 0;
    for parent_key in parent_keys {
        let part_sort_block = get_part_sort_block(storage, parent_key)?;
        if part_sort_block.header.size > selected_size
            || (part_sort_block.header.size == selected_size && parent_key > &selected_key)
        {
            selected_key = *parent_key;
            selected_size = part_sort_block.header.size;
        }
    }
    Ok((selected_key, selected_size))
}

struct LinkSet<K> {
    link_set: BTreeSet<K>,
    head_link_set: BTreeSet<K>,
}

fn get_link_set<K: BlockKeyTrait, S: BlockStorage<KeyType = K>>(
    storage: &S,
    head_key: K,
) -> Result<LinkSet<K>> {
    let mut link_set: BTreeSet<K> = BTreeSet::new();
    let mut head_link_set: BTreeSet<K> = BTreeSet::new();
    let mut now_key = head_key;
    while head_link_set.len() < MAX_ANCESTOR_SIZE {
        let part_sort = get_part_sort_block(storage, &now_key)?;
        link_set.extend(part_sort.part_sort.into_iter());
        link_set.insert(now_key);
        head_link_set.insert(now_key);
        if let Some(head_key) = part_sort.header.head_key {
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

fn get_well_connected_keys<K: BlockKeyTrait, S: BlockStorage<KeyType = K>>(
    storage: &S,
    link_set: &LinkSet<K>,
    parent_keys: &[K],
) -> Result<Vec<K>> {
    let mut well_connected_keys: Vec<K> = Vec::new();
    for key in parent_keys {
        if check_well_connected_block(storage, *key, &link_set.link_set)?
            && check_head_well_connected_block(storage, *key, &link_set.head_link_set)?
        {
            well_connected_keys.push(*key);
        }
    }
    Ok(well_connected_keys)
}

fn check_well_connected_block<K: BlockKeyTrait, S: BlockStorage<KeyType = K>>(
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

fn check_head_well_connected_block<K: BlockKeyTrait, S: BlockStorage<KeyType = K>>(
    storage: &S,
    key: K,
    head_link_set: &BTreeSet<K>,
) -> Result<bool> {
    if head_link_set.contains(&key) {
        return Ok(false);
    }
    let mut count = 0;
    let mut now_key = key;
    while count < MAX_ANCESTOR_SIZE {
        let part_sort = get_part_sort_block(storage, &now_key)?;
        if let Some(head_key) = part_sort.header.head_key {
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

fn cal_in_degree_without_check<K: BlockKeyTrait, S: BlockStorage<KeyType = K>>(
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

fn cal_real_tips_without_head<K: BlockKeyTrait>(
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

fn push_top_sort<K: BlockKeyTrait>(top_sort: &mut TopSort<K>, key: K, size: u64) {
    top_sort.entry(size).or_default().insert(key);
}

fn pop_top_sort<K: BlockKeyTrait>(top_sort: &mut TopSort<K>) -> Option<K> {
    let (size, mut keys) = top_sort.pop_first()?;
    let key = keys.pop_first()?;
    if keys.is_empty() {
        top_sort.insert(size, keys);
    }
    Some(key)
}

#[allow(clippy::comparison_chain)]
fn cal_top_sort<K: BlockKeyTrait, S: BlockStorage<KeyType = K>>(
    storage: &S,
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
                        key: parent_key.serde_to_string()?,
                    });
                }
            }
        }
        temp_sort.push_front(key);
    }
    Ok(temp_sort.into_iter().collect())
}

fn check_top_sort<K: BlockKeyTrait>(top_sort: &[K]) -> Result<()> {
    let sort_set = top_sort.iter().cloned().collect::<BTreeSet<_>>();
    if sort_set.len() != top_sort.len() {
        return Err(Error::TopSortError);
    }
    Ok(())
}
