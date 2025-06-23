#![allow(clippy::unwrap_used)]
use std::collections::BTreeMap;
use utils::file::{read_from_json, write_to_json};

fn main() {
    let mut x_data: Vec<f64> = Vec::new();
    let mut y_data: Vec<f64> = Vec::new();
    x_data.push(0.001);
    y_data.push(0.001);
    let mut tm: BTreeMap<i64, f64> = BTreeMap::new();
    for i in 1..18 {
        let file_path = format!("simulation/distance/distance_{i}.json");
        println!("path: {file_path}");
        let mut nm: BTreeMap<i64, i64> = read_from_json(&file_path).unwrap();
        nm.remove(&-1);
        let mut ans: f64 = 0.0;
        let mut cnt: f64 = 0.0;
        for (k, v) in nm.iter() {
            ans += (k * v) as f64;
            cnt += *v as f64;
        }
        ans /= cnt;
        tm.insert(i, ans);
        x_data.push(ans);
        y_data.push(i as f64);
    }
    let output_path = "simulation/distance/tm.json";
    write_to_json(output_path, &tm).unwrap();
    println!("x_data = np.array({x_data:?})");
    println!("y_data = np.array({y_data:?})");
}
