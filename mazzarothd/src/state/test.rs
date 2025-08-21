use std::sync::{Arc, Mutex};

use crate::state::{
    block_storage::{BlockStorage, gen_consensus_header_with_global_storage, set_block},
    tips::u32_to_block_key,
};
use mvm::models::block::{Block, BlockInner};
use utils::time::get_current_time_ms;

pub fn gen_test_block_and_save(
    key: u32,
    parent_keys: &[u32],
    block_storage: &Arc<Mutex<BlockStorage>>,
) -> anyhow::Result<()> {
    let parent_keys = parent_keys
        .iter()
        .map(|k| u32_to_block_key(*k))
        .collect::<Vec<_>>();
    let now_timestamp_ms = get_current_time_ms();
    let consensus_header =
        gen_consensus_header_with_global_storage(block_storage, &parent_keys, now_timestamp_ms)?;
    let block = Block {
        key: u32_to_block_key(key),
        nonce: 0,
        inner: BlockInner {
            header: consensus_header,
            ..Default::default()
        },
    };
    set_block(block_storage, &block.key, &block)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::state::{
        mvm::get_mvm_move_path,
        mz_state::get_mz_state,
        test::gen_test_block_and_save,
        tips::{block_key_to_u32, u32_to_block_key},
    };

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_mvm_move_path() {
        let mz_state = get_mz_state("test_mvm_move_path").unwrap();
        gen_test_block_and_save(1, &[0], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(3, &[1], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(5, &[1], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(7, &[5], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(9, &[3, 5], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(11, &[9], &mz_state.block_storage).unwrap();

        gen_test_block_and_save(2, &[3, 5], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(4, &[2], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(6, &[2], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(8, &[0], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(10, &[6, 4], &mz_state.block_storage).unwrap();
        gen_test_block_and_save(12, &[10, 8], &mz_state.block_storage).unwrap();

        /*
        u32_now_path: [11, 9, 3, 5, 1, 0]
        u32_next_path: [12, 8, 10, 4, 6, 2, 3, 5, 1, 0]
        */

        let mvm_move_path = get_mvm_move_path(
            u32_to_block_key(11),
            u32_to_block_key(12),
            &mz_state.block_storage,
        )
        .unwrap();
        let u32_now_path = mvm_move_path
            .now_to_head_path
            .iter()
            .map(|k| block_key_to_u32(*k))
            .collect::<Vec<_>>();
        let u32_next_path = mvm_move_path
            .next_to_head_path
            .iter()
            .map(|k| block_key_to_u32(*k))
            .collect::<Vec<_>>();
        eprintln!("u32_now_path: {:?}", u32_now_path);
        eprintln!("u32_next_path: {:?}", u32_next_path);
        let u32_now_path_end = *u32_now_path.last().unwrap();
        let u32_next_path_end = *u32_next_path.last().unwrap();
        assert_eq!(u32_now_path_end, u32_next_path_end);
        assert!(u32_now_path_end != 0);
    }
}
