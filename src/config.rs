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

        config.site_dir = site_dir.to_path_buf();
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
        if let Some(ref output_dir) = overrides.output_dir {
            // If the path is absolute, use it as-is
            // If relative, make it relative to the current working directory, not the site dir
            let path = if output_dir.is_absolute() {
                output_dir.clone()
            } else {
                std::env::current_dir()
                    .expect("Failed to get current directory")
                    .join(output_dir)
            };
            self.build.output_dir = path.to_str().expect("Invalid output path").to_string();
        }

        if let Some(output_dir) = overrides.output_dir {
            self.build.output_dir = output_dir.to_string_lossy().into();
        }
        if let Some(author) = overrides.author {
            self.author.name = author;
        }
        self
    }

    // Path helper methods
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

    pub fn templates_dir(&self) -> PathBuf {
        self.site_dir.join(&self.build.templates_dir)
    }

    pub fn static_dir(&self) -> PathBuf {
        self.site_dir.join(&self.build.static_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_toml(content: &str) -> Result<(TempDir, PathBuf), Error> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, content)?;
        Ok((temp_dir, config_path))
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.base_url, "http://localhost");
        assert_eq!(config.title, "My Terminal Velocity Blog");
        assert_eq!(config.description, "A blazingly fast tech blog");
        assert_eq!(config.author.name, "Anonymous");
        assert_eq!(config.author.email, "author@example.com");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.build.output_dir, "dist");
        assert_eq!(config.build.posts_dir, "posts");
        assert_eq!(config.build.templates_dir, "templates");
        assert_eq!(config.build.static_dir, "static");
        assert_eq!(config.build.post_assets_dir, "assets");
        assert!(config.server.hot_reload);
        assert!(!config.build.verbose);
    }

    #[test]
    fn test_load_empty_directory() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        let config = Config::load(temp_dir.path())?;

        // Should use all defaults
        assert_eq!(config.base_url, "http://localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.site_dir, temp_dir.path());
        Ok(())
    }

    #[test]
    fn test_load_partial_config() -> Result<(), Error> {
        let config_content = r#"
            title = "Custom Blog"
            base_url = "https://example.com"

            [author]
            name = "Test Author"

            [server]
            port = 9000
        "#;

        let (temp_dir, _) = create_test_toml(config_content)?;
        let config = Config::load(temp_dir.path())?;

        // Check overridden values
        assert_eq!(config.title, "Custom Blog");
        assert_eq!(config.base_url, "https://example.com");
        assert_eq!(config.author.name, "Test Author");
        assert_eq!(config.server.port, 9000);

        // Check defaults are still used
        assert_eq!(config.author.email, "author@example.com");
        assert_eq!(config.build.output_dir, "dist");
        assert!(!config.build.verbose);
        Ok(())
    }

    #[test]
    fn test_cli_overrides() -> Result<(), Error> {
        let config_content = r#"
            title = "Base Blog"

            [build]
            port = 8080
            verbose = false
            hot_reload = true
        "#;

        let (temp_dir, _) = create_test_toml(config_content)?;
        let config = Config::load(temp_dir.path())?.with_overrides(ConfigOverrides {
            port: Some(9000),
            verbose: Some(true),
            hot_reload: Some(false),
            author: Some("CLI Author".into()),
            output_dir: Some(PathBuf::from("/custom/output")),
            auto_build: Some(true),
        });

        // Check CLI overrides
        assert_eq!(config.server.port, 9000);
        assert!(config.build.verbose);
        assert!(!config.server.hot_reload);
        assert_eq!(config.author.name, "CLI Author");
        assert_eq!(config.build.output_dir, "/custom/output");

        // Check non-overridden values remain
        assert_eq!(config.title, "Base Blog");
        Ok(())
    }

    #[test]
    fn test_partial_cli_overrides() -> Result<(), Error> {
        let config_content = r#"
            title = "Base Blog"

            [build]
            port = 8080
            verbose = false
        "#;

        let (temp_dir, _) = create_test_toml(config_content)?;
        let config = Config::load(temp_dir.path())?.with_overrides(ConfigOverrides {
            port: Some(9000),
            ..Default::default()
        });

        // Check only port is overridden
        assert_eq!(config.server.port, 9000);
        assert!(!config.build.verbose); // Unchanged
        assert!(config.server.hot_reload); // Default value
        assert_eq!(config.title, "Base Blog"); // Unchanged
        Ok(())
    }

    #[test]
    fn test_output_dir_handling() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;

        // Test relative path
        let config = Config::load(temp_dir.path())?.with_overrides(ConfigOverrides {
            output_dir: Some(PathBuf::from("relative/output")),
            ..Default::default()
        });
        assert_eq!(
            config.output_dir(),
            temp_dir.path().join("relative/output") // Joined to site_dir, not current_dir
        );

        // Test absolute path
        let abs_path = if cfg!(windows) {
            PathBuf::from(r"C:\absolute\output")
        } else {
            PathBuf::from("/absolute/output")
        };

        let config = Config::load(temp_dir.path())?.with_overrides(ConfigOverrides {
            output_dir: Some(abs_path.clone()),
            ..Default::default()
        });
        assert_eq!(config.output_dir(), abs_path);
        Ok(())
    }
    #[test]
    fn test_path_helper_methods() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        let config = Config::load(temp_dir.path())?;

        assert_eq!(config.posts_dir(), temp_dir.path().join("posts"));
        assert_eq!(config.templates_dir(), temp_dir.path().join("templates"));
        assert_eq!(config.static_dir(), temp_dir.path().join("static"));
        assert_eq!(config.output_dir(), temp_dir.path().join("dist"));
        Ok(())
    }

    #[test]
    fn test_invalid_toml() {
        let config_content = r#"
            title = "Unclosed String
            invalid toml content
        "#;

        let (temp_dir, _) = create_test_toml(config_content).unwrap();
        let result = Config::load(temp_dir.path());
        assert!(matches!(result, Err(Error::ConfigParse(_))));
    }

    #[test]
    fn test_deserialize_from_str() -> Result<(), Error> {
        let config_str = r#"
            title = "From String"
            base_url = "http://example.com"

            [server]
            port = 3000
        "#;

        let config: Config = toml::from_str(config_str)?;
        assert_eq!(config.title, "From String");
        assert_eq!(config.base_url, "http://example.com");
        assert_eq!(config.server.port, 3000);
        Ok(())
    }

    #[test]
    fn test_config_serialize() -> Result<(), Error> {
        let original_config = Config::default().with_overrides(ConfigOverrides {
            port: Some(9000),
            author: Some("Test Author".into()),
            ..Default::default()
        });

        // Serialize to TOML
        let serialized = toml::to_string(&original_config)?;

        // Deserialize back
        let deserialized: Config = toml::from_str(&serialized)?;

        assert_eq!(deserialized.server.port, 9000);
        assert_eq!(deserialized.author.name, "Test Author");
        assert_eq!(deserialized.build.output_dir, "dist");
        Ok(())
    }
}
