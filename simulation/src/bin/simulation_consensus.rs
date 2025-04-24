#![allow(clippy::unwrap_used)]
use simulation::sc::sim_run::run_sim;
use utils::log::init_log;

fn main() {
    init_log();
    run_sim("test.db", 1000, 10000);
}
