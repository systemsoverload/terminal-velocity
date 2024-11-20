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

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}