use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;

use crate::state::app_data::get_config_path;

#[allow(clippy::expect_used)]
pub static CFG: LazyLock<Config> = LazyLock::new(|| Config::init().expect("Failed to init config"));

// todo: merge old version config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub http_port: u16,
    pub gossip_tcp_port: u16,
    pub gossip_udp_port: u16,
    pub bootstrap_addr: Option<String>,
    pub bootstrap_peer_id: Option<String>,
    pub new_genesis: bool,
    pub block_sync_host: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http_port: 8080,
            gossip_tcp_port: 43221,
            gossip_udp_port: 43222,
            bootstrap_addr: Some(
                "/ip6/2409:8a00:31d0:a3b0:7c10:9494:279:f924/udp/4001/quic-v1".to_string(),
            ),
            bootstrap_peer_id: Some(
                "12D3KooWD4qpZuZXNPC9iMxRJ9UVELUBB6spy8ijrgeJdhJPCN4n".to_string(),
            ),
            new_genesis: false,
            block_sync_host: "[2409:8a00:31d0:a3b0:7c10:9494:279:f924]:8080".to_string(),
        }
    }
}

impl Config {
    pub fn init() -> anyhow::Result<Self> {
        let config_path = get_config_path().with_context(|| "Failed to get config path")?;
        if !Path::new(&config_path).exists() {
            Self::save_init_config(&config_path).with_context(|| "Failed to save init config")?;
        }

        let config = toml::from_str(
            &std::fs::read_to_string(config_path.clone())
                .with_context(|| "Failed to read config file")?,
        )
        .with_context(|| format!("Failed to deserialize config: {}", config_path))?;

        Ok(config)
    }

    fn save_init_config(path: &str) -> anyhow::Result<()> {
        let config = Config::default();
        let config_str = toml::to_string(&config).with_context(|| "Failed to serialize config")?;
        std::fs::write(path, config_str).with_context(|| "Failed to write config file")?;

        Ok(())
    }
}
