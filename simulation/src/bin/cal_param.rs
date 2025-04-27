#![allow(clippy::unwrap_used)]
use std::collections::BTreeMap;
use utils::file::{read_from_json, write_to_json};

fn main() {
    let mut tm: BTreeMap<i64, f64> = BTreeMap::new();
    for i in 1..20 {
        let file_path = format!("distance/distance_{}.json", i);
        println!("path: {}", file_path);
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
    }
    let output_path = "distance/tm.json";
    write_to_json(output_path, &tm).unwrap();
}
