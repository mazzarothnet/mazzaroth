pub fn get_data_path() -> anyhow::Result<String> {
    let dir = "app_data";
    let path = std::path::Path::new(dir);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to get data path"))?
        .to_string())
}

pub fn get_block_db_path() -> anyhow::Result<String> {
    let path = get_data_path()?;
    let path = std::path::Path::new(&path);
    let path = path.join("block_db");
    Ok(path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to get block db path"))?
        .to_string())
}

pub fn get_mvm_db_path() -> anyhow::Result<String> {
    let path = get_data_path()?;
    let path = std::path::Path::new(&path);
    let path = path.join("mvm_db");
    Ok(path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to get mvm db path"))?
        .to_string())
}
