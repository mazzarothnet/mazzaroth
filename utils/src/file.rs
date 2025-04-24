use std::fs::File;
use std::io::Write;
use serde::Serialize;


pub fn write_to_json<T: Serialize>(path: &str, data: &T) -> anyhow::Result<()> {
    let mut file = File::create(path)?;
    let string = serde_json::to_string(data)?;
    file.write_all(string.as_bytes())?;
    Ok(())
}
