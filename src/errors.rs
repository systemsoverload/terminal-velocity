use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Required directory missing: {0}")]
    MissingDirectory(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Template error: {0}")]
    Template(#[from] tera::Error),

    #[error("Frontmatter error in {file}: {message}")]
    Frontmatter { file: String, message: String },

    #[error("Server error: {0}")]
    Server(#[from] Box<dyn std::error::Error>),

    #[error("Configuration parse error: {0}")]
    ConfigParse(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Missing API key. Set ANTHROPIC_API_KEY environment variable or use --anthropic-key")]
    MissingApiKey,

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
