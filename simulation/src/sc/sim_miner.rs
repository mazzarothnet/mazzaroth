use rand::Rng;
use serde::{Deserialize, Serialize};
const HEIGHT: i64 = 100;
const WIDTH: i64 = 100;
const MAX_DELAY: f64 = 10.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone)]
pub struct SimMiner {
    pub id: u64,
    pub position: Position,
    pub power: f64,
}

pub fn gen_sim_minner_list(miner_num: u64) -> Vec<SimMiner> {
    let mut rng = rand::rng();
    let mut miners = Vec::new();
    for i in 0..miner_num {
        let x = rng.random_range(0..HEIGHT) as f64;
        let y = rng.random_range(0..WIDTH) as f64;
        let power = f64::from(rng.random_range(0..100));
        miners.push(SimMiner {
            id: i,
            position: Position { x, y },
            power,
        });
    }
    miners
}

pub fn select_miner(miners: &[SimMiner]) -> SimMiner {
    let mut vv = Vec::new();
    vv.push(miners[0].power as u64);
    for i in 1..miners.len() {
        vv.push(vv[i - 1] + miners[i].power as u64);
    }
    let end = vv[vv.len() - 1];
    let mut rng = rand::rng();
    let r = rng.random_range(0..end);
    for i in 0..vv.len() {
        if r <= vv[i] {
            return miners[i].clone();
        }
    }

    miners[0].clone()
}

pub fn calc_distance_delay(miner1: &Position, miner2: &Position) -> u64 {
    let dx = miner1.x - miner2.x;
    let dy = miner1.y - miner2.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let max_distance = (HEIGHT as f64 * HEIGHT as f64 + WIDTH as f64 * WIDTH as f64).sqrt();
    let delay = distance / max_distance * MAX_DELAY;
    delay as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_sim_minner_list() {
        let miners = gen_sim_minner_list(10);
        assert_eq!(miners.len(), 10);
    }

    #[test]
    fn test_select_miner() {
        let miners = gen_sim_minner_list(10);
        let miner = select_miner(&miners);
        assert!(miner.id < 10);
    }
}
