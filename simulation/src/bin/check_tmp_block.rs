use consensus::part_sort_header::gen_part_sort_block;
use log::debug;
use simulation::sc::{sim_block::SimBlock, sim_storage::SimBlockStorage};
use utils::{file::read_from_json, log::init_log};

fn main() {
    init_log();
    let mut storage = SimBlockStorage::new("check_tmp_block.db");
    let tmp_block: Vec<SimBlock> = read_from_json("simulation/distance/tmp_block.json").unwrap();
    for block in tmp_block {
        storage.set_block(block.header.key, &block).unwrap();
        debug!("check block: {:?}", block.header.key);
        let part_sort_header =
            gen_part_sort_block(&storage, &block.header.part_sort_header.parent_keys).unwrap();
        debug!("end check block: {:?}\n", block.header.key);
        if part_sort_header != block.header.part_sort_header {
            debug!("block: {:?}", block.header.key);
            debug!("part_sort_header: {:?}", part_sort_header);
            debug!(
                "block.header.part_sort_header: {:?}",
                block.header.part_sort_header
            );
            panic!("part_sort_header != block.header.part_sort_header");
        }
    }
}
