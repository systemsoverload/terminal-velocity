pub mod anthropic;
pub mod config;
pub mod constants;
pub mod errors;
pub mod generator;
pub mod git;
pub mod init;
pub mod markdown;
pub mod post;
pub mod serve;

#[cfg(test)]
pub mod tests {
    use crate::config::{Author, BuildConfig, Config, ServerConfig};
    use tempfile::TempDir;

    pub fn create_test_config(temp_dir: &TempDir) -> Config {
        let site_dir = temp_dir.path().to_path_buf();
        Config {
            site_dir,
            base_url: "http://localhost:8000".to_string(),
            title: "Test Blog".to_string(),
            description: "Test Description".to_string(),
            author: Author {
                name: "Test Author".to_string(),
                email: "test@example.com".to_string(),
            },
            server: ServerConfig {
                auto_build: true,
                port: 8000,
                hot_reload: true,
            },
            build: BuildConfig {
                verbose: false,
                output_dir: "dist".to_string(),
                posts_dir: "posts".to_string(),
                templates_dir: "templates".to_string(),
                static_dir: "static".to_string(),
                post_assets_dir: "assets".to_string(),
            },
        }
    }
}
