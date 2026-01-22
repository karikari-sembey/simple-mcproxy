use std::fs;

use anyhow::Result;
use serde::Deserialize;

const CONFIG_PATH: &str = "./config/config.json";

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
        let conf_file = match fs::read_to_string(CONFIG_PATH) {
            Ok(file) => file,
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    log::error!("An error occurred while reading {CONFIG_PATH}: {error}");
                    Err(error)?
                }
                log::warn!("Configuration file not found");
                String::new()
            }
        };

        Ok(match serde_json::from_str::<Config>(&conf_file) {
            Ok(conf) => conf,
            Err(error) => {
                log::error!("An error occurred while parsing config: {error}");
                log::warn!("Default config has been loaded");
                Config::default()
            }
        })
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
