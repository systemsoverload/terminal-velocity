use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::Error;
use crate::git::init_git_repository;

pub fn create_directory_structure(path: &PathBuf) -> Result<(), Error> {
    let dirs = [
        path,
        &path.join("posts"),
        &path.join("posts/example/assets"),
        &path.join("templates"),
        &path.join("static"),
        &path.join("static/css"),
        &path.join("components"),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir)?;
    }

    // Create default templates
    let templates = [
        (
            "templates/post.html",
            include_str!(concat!(env!("OUT_DIR"), "/templates/post.html")),
        ),
        (
            "templates/index.html",
            include_str!(concat!(env!("OUT_DIR"), "/templates/index.html")),
        ),
        (
            "templates/base.html",
            include_str!(concat!(env!("OUT_DIR"), "/templates/base.html")),
        ),
    ];

    for (file, content) in &templates {
        fs::write(path.join(file), content)?;
    }

    // Create default stylesheet
    fs::write(
        path.join("static/css/style.css"),
        include_str!(concat!(env!("OUT_DIR"), "/templates/style.css")),
    )?;

    // Create example post
    fs::write(
        path.join("posts/example/example.md"),
        include_str!(concat!(env!("OUT_DIR"), "/templates/example.md")),
    )?;

    // Create config file
    fs::write(
        path.join("config.toml"),
        include_str!(concat!(env!("OUT_DIR"), "/templates/config.toml")),
    )?;

    // Create sample post assset
    fs::write(
        path.join("posts/example/assets/terminal_velocity.png"),
        include_bytes!(concat!(env!("OUT_DIR"), "/templates/terminal_velocity.png")),
    )?;

    // Create .gitignore
    let gitignore_content = r#"dist/
    target/
    **/.DS_Store
    .env"#;
    fs::write(path.join(".gitignore"), gitignore_content)?;

    // Initialize git repository
    init_git_repository(path)?;

    Ok(())
}

pub fn validate_site_directory(path: &Path) -> Result<(), Error> {
    let required_dirs = [
        ("posts", "posts"),
        ("templates", "templates"),
        ("static/css", "static/css"),
    ];

    if !path.exists() {
        return Err(Error::DirectoryNotFound(path.to_path_buf()));
    }

    for (dir, name) in required_dirs {
        if !path.join(dir).exists() {
            return Err(Error::MissingDirectory(name.to_string()));
        }
    }

    Ok(())
}
