use crate::errors::Error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    #[serde(skip)]
    pub site_dir: PathBuf,
    pub base_url: String,
    pub title: String,
    pub description: String,
    pub author: Author,
    pub build: BuildConfig,
    pub server: ServerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            site_dir: PathBuf::new(),
            base_url: "http://localhost".into(),
            title: "My Terminal Velocity Blog".into(),
            description: "A blazingly fast tech blog".into(),
            author: Author::default(),
            build: BuildConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Author {
    pub name: String,
    pub email: String,
}

impl Default for Author {
    fn default() -> Self {
        Self {
            name: "Anonymous".into(),
            email: "author@example.com".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct BuildConfig {
    pub verbose: bool,
    pub output_dir: String,
    pub posts_dir: String,
    pub templates_dir: String,
    pub static_dir: String,
    pub post_assets_dir: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            output_dir: "dist".into(),
            posts_dir: "posts".into(),
            templates_dir: "templates".into(),
            static_dir: "static".into(),
            post_assets_dir: "assets".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ServerConfig {
    pub auto_build: bool,
    pub port: u16,
    pub hot_reload: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            auto_build: true,
            port: 8080,
            hot_reload: true,
        }
    }
}

#[derive(Default)]
pub struct ConfigOverrides {
    pub port: Option<u16>,
    pub verbose: Option<bool>,
    pub hot_reload: Option<bool>,
    pub output_dir: Option<PathBuf>,
    pub author: Option<String>,
    pub auto_build: Option<bool>,
}

impl Config {
    pub fn load(site_dir: &Path) -> Result<Self, Error> {
        let config_path = site_dir.join("config.toml");

        let mut config = if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            toml::from_str(&content).map_err(|e| Error::ConfigParse(e.to_string()))?
        } else {
            Config::default()
        };

        // Always store absolute path for site_dir
        config.site_dir = site_dir
            .canonicalize()
            .unwrap_or_else(|_| site_dir.to_path_buf());
        Ok(config)
    }

    pub fn with_overrides(mut self, overrides: ConfigOverrides) -> Self {
        if let Some(port) = overrides.port {
            self.server.port = port;
        }
        if let Some(verbose) = overrides.verbose {
            self.build.verbose = verbose;
        }
        if let Some(hot_reload) = overrides.hot_reload {
            self.server.hot_reload = hot_reload;
        }
        if let Some(auto_build) = overrides.auto_build {
            self.server.auto_build = auto_build;
        }
        if let Some(author) = overrides.author {
            self.author.name = author;
        }
        if let Some(output_dir) = overrides.output_dir {
            self.build.output_dir = output_dir.to_string_lossy().into();
        }
        self
    }

    // Gets the absolute path to a directory, resolving it relative to site_dir if necessary
    fn resolve_path(&self, path: &str) -> PathBuf {
        if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.site_dir.join(path)
        }
    }

    // Path helper methods now all return absolute paths
    pub fn posts_dir(&self) -> PathBuf {
        self.resolve_path(&self.build.posts_dir)
    }

    pub fn output_dir(&self) -> PathBuf {
        self.resolve_path(&self.build.output_dir)
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.resolve_path(&self.build.templates_dir)
    }

    pub fn static_dir(&self) -> PathBuf {
        self.resolve_path(&self.build.static_dir)
    }

    pub fn get_absolute_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.site_dir.join(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ... (keep existing tests)

    #[test]
    fn test_path_resolution() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        let absolute_path = temp_dir.path().canonicalize()?;

        let config = Config::load(&absolute_path)?;

        // Test relative paths
        assert_eq!(config.posts_dir(), absolute_path.join("posts"));
        assert_eq!(config.output_dir(), absolute_path.join("dist"));

        // Test absolute paths
        let abs_output = if cfg!(windows) {
            PathBuf::from(r"C:\custom\output")
        } else {
            PathBuf::from("/custom/output")
        };

        let config = Config::load(&absolute_path)?.with_overrides(ConfigOverrides {
            output_dir: Some(abs_output.clone()),
            ..Default::default()
        });

        assert_eq!(config.output_dir(), abs_output);

        // Test path resolution method
        let relative_path = PathBuf::from("relative/path");
        assert_eq!(
            config.get_absolute_path(&relative_path),
            absolute_path.join("relative/path")
        );

        let abs_path = if cfg!(windows) {
            PathBuf::from(r"C:\absolute\path")
        } else {
            PathBuf::from("/absolute/path")
        };
        assert_eq!(config.get_absolute_path(&abs_path), abs_path);

        Ok(())
    }
}
