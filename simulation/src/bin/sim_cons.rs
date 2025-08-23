#![allow(clippy::unwrap_used)]
use log::info;
use rand::SeedableRng;
use simulation::sc::sim_run::run_sim;
use std::path::Path;
use utils::log::init_log;

fn main() {
    init_log();
    // create dir distance
    let dir = "simulation/distance";
    if !Path::new(dir).exists() {
        std::fs::create_dir(dir).unwrap();
    }
    let mut rng = rand::rngs::StdRng::seed_from_u64(1112331);
    for i in 1..18 {
        let block_num = 20000;
        let now = std::time::Instant::now();
        run_sim("test.db", 1000, block_num, f64::from(i), &mut rng);
        let time = now.elapsed().as_millis() as f64;
        info!(
            "i {} time: {:?}, cast per block: {}",
            i,
            time,
            time / block_num as f64
        );
    }
}
