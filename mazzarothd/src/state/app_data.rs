use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref DATA_PATH: Mutex<String> = Mutex::new("mazzaroth_data".to_string());
}

pub fn set_test_data_path_and_clean(path: &str) -> anyhow::Result<()> {
    let path_os = std::path::Path::new(path);
    if path_os.exists() {
        std::fs::remove_dir_all(path)?;
    }
    let mut data_path = DATA_PATH
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock data path: {}", e))?;
    *data_path = path.to_string();
    Ok(())
}

pub fn get_data_path() -> anyhow::Result<String> {
    let dir = DATA_PATH
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock data path: {}", e))?
        .clone();
    let path = std::path::Path::new(&dir);
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

pub fn get_config_path() -> anyhow::Result<String> {
    let path = get_data_path()?;
    let path = std::path::Path::new(&path);
    let path = path.join("config.toml");
    Ok(path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config path"))?
        .to_string())
}
