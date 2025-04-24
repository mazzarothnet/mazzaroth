use utils::error::{Error, Result};

use crate::{
    real_tips::{
        cal_in_degree_without_check, cal_real_tips_without_head, cal_top_sort, check_top_sort,
        get_link_set, get_max_size_key, get_well_connected_keys,
    },
    traits::{DagStorage, Key, SortStruct},
};

pub fn part_sort_with_cache<K: Key, S: DagStorage<KeyType = K>>(
    storage: &mut S,
    now_key: K,
) -> Result<SortStruct<K>> {
    if let Some(part_sort) = storage.get_part_sort_of_key(&now_key)? {
        return Ok(part_sort);
    }
    let ans = part_sort(storage, now_key)?;
    storage.set_part_sort_of_key(now_key, ans.clone())?;
    Ok(ans)
}

/// can't recursion because of the size limit of the stack
pub fn part_sort<K: Key, S: DagStorage<KeyType = K>>(
    storage: &mut S,
    now_key: K,
) -> Result<SortStruct<K>> {
    if now_key.is_genesis() {
        return gen_genesis_part_sort(now_key);
    }
    let parent_keys = storage.get_parent_keys(&now_key)?;
    let (selected_head_key, selected_head_size) =
        get_max_size_key(storage, &parent_keys).map_err(|e| Error::IsolateBlock {
            message: format!("{:?} {:?}", serde_json::to_string(&now_key), e),
        })?;
    let link_set = get_link_set(storage, selected_head_key)?;
    let well_connected_keys = get_well_connected_keys(storage, &link_set, &parent_keys)?;
    let in_degree = cal_in_degree_without_check(storage, &well_connected_keys, &link_set)?;
    let real_tips_without_head = cal_real_tips_without_head(well_connected_keys, &in_degree)?;
    let top_sort = cal_top_sort(storage, now_key, real_tips_without_head, in_degree)?;
    check_top_sort(&top_sort, now_key)?;

    let size = selected_head_size + top_sort.len() as u64;
    let self_part_sort = SortStruct {
        key: now_key,
        head_key: Some(selected_head_key),
        part_sort: top_sort,
        size,
    };
    Ok(self_part_sort)
}

pub fn gen_genesis_part_sort<K: Key>(now_key: K) -> Result<SortStruct<K>> {
    let ans = SortStruct {
        key: now_key,
        head_key: None,
        part_sort: vec![now_key],
        size: 1,
    };
    Ok(ans)
}
