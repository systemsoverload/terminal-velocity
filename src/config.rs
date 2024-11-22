use crate::errors::Error;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(skip)]
    pub site_dir: PathBuf,
    #[serde(deserialize_with = "normalize_url")]
    pub base_url: String,
    pub title: String,
    pub description: String,
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
    #[serde(default)]
    pub verbose: bool,
    pub output_dir: String,
    pub posts_dir: String,
    pub templates_dir: String,
    pub static_dir: String,
    #[serde(default = "default_post_assets_dir")]
    pub post_assets_dir: String,
    #[serde(deserialize_with = "validate_port")]
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_hot_reload")]
    pub hot_reload: bool,
}
fn default_hot_reload() -> bool {
    true
}

fn default_port() -> u16 {
    8080
}

fn default_post_assets_dir() -> String {
    "assets".to_string()
}

impl Config {
    pub fn load(site_dir: &Path) -> Result<Self, Error> {
        let config_path = site_dir.join("config.toml");

        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            let mut config: Config =
                toml::from_str(&content).map_err(|e| Error::ConfigParse(e.to_string()))?;
            config.site_dir = site_dir.to_path_buf();
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn default() -> Self {
        // TODO - Probably be stricter here and just fail if the config doesnt exist on disk.
        // This was useful for early dev, but will probably cause confusion for real use
        let default_config = include_str!(concat!(env!("OUT_DIR"), "/templates/config.toml"));
        toml::from_str(default_config).expect("Failed to parse default config")
    }

    pub fn posts_dir(&self) -> PathBuf {
        self.site_dir.join(&self.build.posts_dir)
    }

    pub fn output_dir(&self) -> PathBuf {
        if Path::new(&self.build.output_dir).is_absolute() {
            PathBuf::from(&self.build.output_dir)
        } else {
            self.site_dir.join(&self.build.output_dir)
        }
    }

    pub fn set_output_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.build.output_dir = path
            .as_ref()
            .to_str()
            .expect("Invalid output path")
            .to_string();
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.site_dir.join(&self.build.templates_dir)
    }

    pub fn static_dir(&self) -> PathBuf {
        self.site_dir.join(&self.build.static_dir)
    }
}

fn normalize_url<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let url = String::deserialize(deserializer)?;
    Ok(url.trim_end_matches('/').to_string())
}

fn validate_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let port = u16::deserialize(deserializer)?;
    if !(1024..=65535).contains(&port) {
        return Err(serde::de::Error::custom(
            "Port must be between 1024 and 65535",
        ));
    }
    Ok(port)
}
