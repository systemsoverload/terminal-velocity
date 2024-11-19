use crate::errors::Error;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub author: Author,
    pub build: BuildConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BuildConfig {
    pub output_dir: String,
    pub port: u16,
}

impl Config {
    pub fn load(site_dir: &Path) -> Result<Self, Error> {
        let config_path = site_dir.join("config.toml");

        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            toml::from_str(&content).map_err(|e| Error::ConfigParse(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn default() -> Self {
        let default_config = include_str!(concat!(env!("OUT_DIR"), "/templates/config.toml"));
        toml::from_str(default_config).expect("Failed to parse default config")
    }
}
