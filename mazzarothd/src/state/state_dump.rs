use crate::state::{
    block_storage::{gen_consensus_header_with_global_storage, get_block},
    mz_state::MzState,
    tips::{get_temp_blocks, get_tips},
};
use consensus::{traits::GENESIS_BLOCK_KEY, types::BlockKey};
use serde::{Deserialize, Serialize};
use utils::{file::write_to_json, time::get_current_time_ms};

#[derive(Serialize, Deserialize)]
pub struct DumpBlock {
    pub key: BlockKey,
    pub head_key: BlockKey,
    pub part_sort_key: Vec<BlockKey>,
}

#[derive(Serialize, Deserialize)]
pub struct DumpTempBlock {
    pub key: BlockKey,
    pub head_key: BlockKey,
    pub unknown_keys: Vec<BlockKey>,
}
pub fn dump_blocks(mz_state: &MzState) -> anyhow::Result<()> {
    let tips = get_tips(mz_state)?;
    let tips_path = format!("{}/tips.json", mz_state.path);
    write_to_json(&tips_path, &tips)?;
    let now = get_current_time_ms();
    let mut head = gen_consensus_header_with_global_storage(
        &mz_state.block_storage,
        &tips.into_iter().collect::<Vec<_>>(),
        now,
    )?
    .part_sort_header
    .head_key;
    let mut blocks = Vec::new();
    while head != GENESIS_BLOCK_KEY {
        let block = get_block(&mz_state.block_storage, &head)?
            .ok_or_else(|| anyhow::anyhow!("block not found"))?;
        blocks.push(DumpBlock {
            key: block.key,
            head_key: block.inner.header.part_sort_header.head_key,
            part_sort_key: block.inner.header.part_sort_header.part_sort,
        });
        head = block.inner.header.part_sort_header.head_key;
    }
    let blocks_path = format!("{}/blocks.json", mz_state.path);
    write_to_json(&blocks_path, &blocks)?;

    let temp_blocks = get_temp_blocks(mz_state)?;
    let mut temp_blocks_vec = Vec::new();
    for (key, (block, unknown_keys)) in temp_blocks.into_iter() {
        temp_blocks_vec.push(DumpTempBlock {
            key,
            head_key: block.inner.header.part_sort_header.head_key,
            unknown_keys: unknown_keys.into_iter().collect::<Vec<_>>(),
        });
    }
    let temp_blocks_path = format!("{}/temp_blocks.json", mz_state.path);
    write_to_json(&temp_blocks_path, &temp_blocks_vec)?;
    Ok(())
}
