use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::fs;

#[derive(Deserialize, Debug, Clone)]
pub struct DeviceConfig {
    pub mac: String,
    pub text: Option<String>,
    pub device_type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BudsConfig {
    pub mac: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub devices: HashMap<String, DeviceConfig>,
    pub buds: Option<BudsConfig>,
}

pub async fn load_config() -> Result<Config> {
    let home = std::env::var("HOME").context("Failed to get HOME env var")?;
    let path = format!("{home}/.config/plants/devices.toml");

    let content = fs::read_to_string(&path)
        .await
        .context(format!("Failed to read config file: {path}"))?;

    let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

    Ok(config)
}
