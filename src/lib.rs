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
    use crate::post::{Post, PostMetadata};
    use std::fs;
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

    pub fn setup_test_site(temp_dir: &TempDir) -> std::io::Result<()> {
        // Create required directories
        fs::create_dir(temp_dir.path().join("posts"))?;
        fs::create_dir(temp_dir.path().join("templates"))?;
        fs::create_dir_all(temp_dir.path().join("static/css"))?;

        // Create test post with assets
        let test_post = Post {
            metadata: PostMetadata {
                title: "Test Post".to_string(),
                date: "2024-01-01".to_string(),
                author: "Test Author".to_string(),
                tags: vec!["test".to_string()],
                preview: "Test preview".to_string(),
                slug: "test-post".to_string(),
                read_time: 0,
            },
            content: String::new(),
            html_content: String::new(),
        };

        let config = create_test_config(temp_dir);
        let post_dir = config.posts_dir().join(&test_post.metadata.slug);
        let assets_dir = test_post.assets_dir(&config);

        fs::create_dir_all(&post_dir)?;
        fs::create_dir_all(&assets_dir)?;

        // Create a test asset
        fs::write(assets_dir.join("test-image.txt"), "test image content")?;

        // Create test post
        fs::write(
            post_dir.join("post.md"),
            r#"---
title: "Test Post"
date: 2024-01-01
author: "Test Author"
tags: ["test"]
preview: "Test preview"
slug: "test-post"
---
# Test Content
Test body

![Test Image](./assets/test-image.txt)"#,
        )?;

        // Create test templates
        fs::write(
            temp_dir.path().join("templates/base.html"),
            "{% block content %}{% endblock %}",
        )?;
        fs::write(
            temp_dir.path().join("templates/post.html"),
            "{% extends \"base.html\" %}{% block content %}{{ post.html_content | safe }}{% endblock %}",
        )?;
        fs::write(
            temp_dir.path().join("templates/index.html"),
            "{% extends \"base.html\" %}{% block content %}{% for post in posts %}{{ post.metadata.title }}{% endfor %}{% endblock %}",
        )?;

        // Create test static file
        fs::write(
            temp_dir.path().join("static/css/style.css"),
            "body { color: black; }",
        )?;

        Ok(())
    }
}
