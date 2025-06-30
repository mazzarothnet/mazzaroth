use consensus::part_sort_header::gen_part_sort_block;
use log::debug;
use simulation::sc::{sim_block::SimBlock, sim_storage::SimConsensusHeaderStorage};
use utils::{file::read_from_json, log::init_log};

fn main() {
    init_log();
    let mut storage = SimConsensusHeaderStorage::new("check_tmp_block.db");
    let tmp_block: Vec<SimBlock> = read_from_json("simulation/distance/tmp_block.json").unwrap();
    for block in tmp_block {
        storage.set_block(block.key, &block).unwrap();
        debug!("check block: {:?}", block.key);
        let part_sort_header =
            gen_part_sort_block(&storage, &block.header.part_sort_header.parent_keys).unwrap();
        debug!("end check block: {:?}\n", block.key);
        if part_sort_header != block.header.part_sort_header {
            debug!("block: {:?}", block.key);
            debug!("part_sort_header: {:?}", part_sort_header);
            debug!(
                "block.header.part_sort_header: {:?}",
                block.header.part_sort_header
            );
            panic!("part_sort_header != block.header.part_sort_header");
        }
    }
}
