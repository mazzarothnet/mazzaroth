use std::collections::BTreeSet;

use super::{
    sim_block::SimKey,
    sim_miner::{gen_sim_minner_list, select_miner},
    sim_storage::SimDagStorage,
};

#[derive(Debug, Clone)]
pub struct Tips {
    pub tips: BTreeSet<SimKey>,
}

impl Tips {
    pub fn new() -> Self {
        Self {
            tips: BTreeSet::new(),
        }
    }

    pub fn add_tip(&mut self, tip: SimKey, parents: &[SimKey]) {
        self.tips.insert(tip);
        for parent in parents {
            self.tips.remove(parent);
        }
    }

    pub fn remove_tip(&mut self, tip: SimKey) {
        
    }
}

#[allow(clippy::unwrap_used)]
pub fn run_sim(db_path: &str, miner_num: u64, block_num: u64) {
    let db = rocksdb::DB::open_default(db_path).unwrap();
    let mut storage = SimDagStorage::new(db);
    let miners = gen_sim_minner_list(miner_num);

    for i in 0..block_num {
        let selected_miner = select_miner(&miners);
        
    }
}
