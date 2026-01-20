use std::fs;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[allow(dead_code)]
    config_version: i32,
    pub listen: String,
    pub default_server: String,
    pub servers: Vec<Server>,
}

impl Config {
    pub fn read_config() -> Result<Config> {
        let conf = fs::read_to_string("./config.json")?;
        Ok(serde_json::from_str::<Config>(&conf).unwrap_or_default())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_version: 0,
            listen: "localhost:25565".to_string(),
            default_server: "localhost:25564".to_string(),
            servers: Vec::new(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub hostname: String,
    pub dest: String,
}
