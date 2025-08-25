use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;

// todo: merge old version config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub http_port: u16,
    pub gossip_tcp_port: u16,
    pub gossip_udp_port: u16,
    pub bootstrap_addr: Option<String>,
    pub bootstrap_peer_id: Option<String>,
    pub block_sync_host: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http_port: 8080,
            gossip_tcp_port: 43221,
            gossip_udp_port: 43222,
            bootstrap_addr: Some(
                "/ip6/2409:8a00:31d4:48b0:e1bb:3842:e9cf:836f/udp/43222/quic-v1".to_string(),
            ),
            bootstrap_peer_id: Some(
                "12D3KooWGfiJK6duQayz8cnWNJ8aBKNgUhsnfmq1jJTBsJvWzLEq".to_string(),
            ),
            block_sync_host: Some("[2409:8a00:31d4:48b0:e1bb:3842:e9cf:836f]:8080".to_string()),
        }
    }
}

impl Config {
    pub fn init(config_path: &str) -> anyhow::Result<Self> {
        if !Path::new(&config_path).exists() {
            Self::save_init_config(config_path).with_context(|| "Failed to save init config")?;
        }

        let config = toml::from_str(
            &std::fs::read_to_string(config_path).with_context(|| "Failed to read config file")?,
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
