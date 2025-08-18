use alloy_rlp::{RlpDecodable, RlpEncodable};
use rand::Rng;
use serde::{Deserialize, Serialize};
const HEIGHT: i64 = 100;
const WIDTH: i64 = 100;
const ADD_DELAY: f64 = 4.0;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, RlpDecodable, RlpEncodable)]
pub struct Position {
    pub x: u64,
    pub y: u64,
}

#[derive(Copy, Clone)]
pub struct SimMiner {
    pub id: u64,
    pub position: Position,
    pub power: f64,
}

pub fn gen_sim_minner_list(miner_num: u64, rng: &mut rand::rngs::StdRng) -> Vec<SimMiner> {
    let mut miners = Vec::new();
    for i in 0..miner_num {
        let x = rng.random_range(0..HEIGHT) as u64;
        let y = rng.random_range(0..WIDTH) as u64;
        let power = f64::from(rng.random_range(0..100));
        miners.push(SimMiner {
            id: i,
            position: Position { x, y },
            power,
        });
    }
    miners
}

pub fn select_miner(miners: &[SimMiner], rng: &mut rand::rngs::StdRng) -> SimMiner {
    let mut vv = Vec::new();
    vv.push(miners[0].power as u64);
    for i in 1..miners.len() {
        vv.push(vv[i - 1] + miners[i].power as u64);
    }
    let end = vv[vv.len() - 1];
    let r = rng.random_range(0..end);
    for i in 0..vv.len() {
        if r <= vv[i] {
            return miners[i];
        }
    }

    miners[0]
}

pub fn calc_distance_delay(miner1: &Position, miner2: &Position, block_per_step: f64) -> u64 {
    let dx = (miner1.x as f64 - miner2.x as f64).abs();
    let dy = (miner1.y as f64 - miner2.y as f64).abs();
    let distance = (dx * dx + dy * dy).sqrt();
    let max_distance = (HEIGHT as f64 * HEIGHT as f64 + WIDTH as f64 * WIDTH as f64).sqrt();
    let delay = (distance / max_distance + ADD_DELAY) * block_per_step;
    delay as u64
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn test_gen_sim_minner_list() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1112331);
        let miners = gen_sim_minner_list(10, &mut rng);
        assert_eq!(miners.len(), 10);
    }

    #[test]
    fn test_select_miner() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1112331);
        let miners = gen_sim_minner_list(10, &mut rng);
        let miner = select_miner(&miners, &mut rng);
        assert!(miner.id < 10);
    }
}
