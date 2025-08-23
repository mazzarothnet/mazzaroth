use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::Write;

pub fn write_to_json<T: Serialize>(path: &str, data: &T) -> anyhow::Result<()> {
    let string = serde_json::to_string(data)?;
    let mut file = File::create(path)?;
    file.write_all(string.as_bytes())?;
    Ok(())
}

pub fn read_from_json<T: DeserializeOwned>(path: &str) -> anyhow::Result<T> {
    let file = File::open(path)?;
    let data: T = serde_json::from_reader(file)?;
    Ok(data)
}
