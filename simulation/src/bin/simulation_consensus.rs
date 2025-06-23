#![allow(clippy::unwrap_used)]
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
    for i in 1..18 {
        run_sim("test.db", 1000, 100000, f64::from(i));
    }
}
