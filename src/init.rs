use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::Error;

pub fn create_directory_structure(path: &PathBuf) -> Result<(), Error> {
    let dirs = [
        path,
        &path.join("posts"),
        &path.join("templates"),
        &path.join("static"),
        &path.join("components"),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir)?;
    }

    // Create default templates
    let templates = [
        ("templates/post.html", include_str!("templates/post.html")),
        ("templates/index.html", include_str!("templates/index.html")),
        ("templates/base.html", include_str!("templates/base.html")),
    ];

    for (file, content) in &templates {
        fs::write(path.join(file), content)?;
    }

    // Create example post
    let example_post = path.join("posts/example.md");
    fs::write(example_post, include_str!("templates/example.md"))?;

    // Create config file
    let config = path.join("config.toml");
    fs::write(config, include_str!("templates/config.toml"))?;

    Ok(())
}

pub fn validate_site_directory(path: &Path) -> Result<(), Error> {
    let posts_dir = path.join("posts");
    let templates_dir = path.join("templates");

    if !path.exists() {
        return Err(Error::DirectoryNotFound(path.to_path_buf()));
    }
    if !posts_dir.exists() {
        return Err(Error::MissingDirectory("posts".to_string()));
    }
    if !templates_dir.exists() {
        return Err(Error::MissingDirectory("templates".to_string()));
    }

    Ok(())
}
