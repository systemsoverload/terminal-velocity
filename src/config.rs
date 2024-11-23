use crate::errors::Error;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

// Default config.toml values
const DEFAULT_BASE_URL: &str = "http://localhost:8080";
const DEFAULT_TITLE: &str = "My Terminal Velocity Blog";
const DEFAULT_DESCRIPTION: &str = "A blazingly fast tech blog";
const DEFAULT_AUTHOR_NAME: &str = "Anonymous";
const DEFAULT_AUTHOR_EMAIL: &str = "author@example.com";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_OUTPUT_DIR: &str = "dist";
const DEFAULT_POSTS_DIR: &str = "posts";
const DEFAULT_TEMPLATES_DIR: &str = "templates";
const DEFAULT_STATIC_DIR: &str = "static";
const DEFAULT_POST_ASSETS_DIR: &str = "assets";

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
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
#[serde(default)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct BuildConfig {
    pub verbose: bool,
    pub output_dir: String,
    pub posts_dir: String,
    pub templates_dir: String,
    pub static_dir: String,
    pub post_assets_dir: String,
    #[serde(deserialize_with = "validate_port")]
    pub port: u16,
    pub hot_reload: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            site_dir: PathBuf::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            title: DEFAULT_TITLE.to_string(),
            description: DEFAULT_DESCRIPTION.to_string(),
            author: Author::default(),
            build: BuildConfig::default(),
        }
    }
}

impl Default for Author {
    fn default() -> Self {
        Self {
            name: DEFAULT_AUTHOR_NAME.to_string(),
            email: DEFAULT_AUTHOR_EMAIL.to_string(),
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            output_dir: DEFAULT_OUTPUT_DIR.to_string(),
            posts_dir: DEFAULT_POSTS_DIR.to_string(),
            templates_dir: DEFAULT_TEMPLATES_DIR.to_string(),
            static_dir: DEFAULT_STATIC_DIR.to_string(),
            post_assets_dir: DEFAULT_POST_ASSETS_DIR.to_string(),
            port: DEFAULT_PORT,
            hot_reload: true,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
        assert_eq!(config.title, DEFAULT_TITLE);
        assert_eq!(config.build.port, DEFAULT_PORT);
        assert_eq!(config.build.output_dir, DEFAULT_OUTPUT_DIR);
    }

    #[test]
    fn test_load_empty_config() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        let config = Config::load(temp_dir.path())?;
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
        assert_eq!(config.build.port, DEFAULT_PORT);
        Ok(())
    }

    #[test]
    fn test_load_partial_config() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        let config_content = r#"
            title = "Custom Blog"
            [author]
            name = "Test Author"
        "#;
        fs::write(temp_dir.path().join("config.toml"), config_content)?;

        let config = Config::load(temp_dir.path())?;
        assert_eq!(config.title, "Custom Blog");
        assert_eq!(config.author.name, "Test Author");
        assert_eq!(config.author.email, DEFAULT_AUTHOR_EMAIL);
        assert_eq!(config.build.port, DEFAULT_PORT);
        Ok(())
    }

    #[test]
    fn test_invalid_port() {
        let config_str = r#"
            [build]
            port = 80
        "#;
        let result: Result<Config, _> = toml::from_str(config_str);
        assert!(result.is_err());
    }
}
