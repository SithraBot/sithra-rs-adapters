use std::{fs::File, io::Write};

use serde::Deserialize;

pub const DEFAULT_CONFIG: &str = include_str!("../static/config.toml");

#[derive(Debug, Clone, Deserialize)]
pub struct OneBotConfig {
    #[serde(rename = "ws_url")]
    pub ws_url: String,
    #[serde(rename = "self_id")]
    pub self_id: String,
}

impl OneBotConfig {
    pub fn load() -> Result<Self, toml::de::Error> {
        let file_path = sithra_common::data_path!().join("config.toml");
        let config = if !file_path.exists() {
            let mut file = File::create(file_path).unwrap();
            file.write_all(DEFAULT_CONFIG.as_bytes()).unwrap();
            toml::from_str(DEFAULT_CONFIG)?
        } else {
            toml::from_str(&std::fs::read_to_string(file_path).unwrap())?
        };
        Ok(config)
    }
}
