use crate::post::Post;
use crate::post::PostMetadata;
use chrono::NaiveDate;
use pulldown_cmark::{html, Options, Parser as MarkdownParser};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use tera::{Context, Tera};
use walkdir::WalkDir;
use yaml_front_matter::{Document, YamlFrontMatter};

use crate::config::Config;
use crate::errors::Error;

pub struct SiteGenerator {
    posts_dir: PathBuf,
    output_dir: PathBuf,
    templates_dir: PathBuf,
    tera: Tera,
    config: Config,
}

impl SiteGenerator {
    pub fn new(site_dir: &Path, config: Option<Config>) -> Result<Self, Error> {
        let config = match config {
            Some(config) => config,
            None => Config::load(site_dir)?,
        };

        let posts_dir = site_dir.join("posts");
        let output_dir = PathBuf::from(&config.build.output_dir);
        let templates_dir = site_dir.join("templates");

        fs::create_dir_all(&output_dir)?;

        let tera = Tera::new(&format!("{}/**/*.html", templates_dir.display()))
            .map_err(Error::Template)?;

        Ok(Self {
            posts_dir,
            output_dir,
            templates_dir,
            tera,
            config,
        })
    }

    fn read_posts(&self) -> Result<Vec<Post>, Error> {
        let mut posts = Vec::new();

        for entry in WalkDir::new(&self.posts_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().map_or(false, |ext| ext == "md") {
                let content = fs::read_to_string(entry.path())?;
                let file_name = entry.path().display().to_string();

                let doc: Document<PostMetadata> =
                    YamlFrontMatter::parse(&content).map_err(|e| Error::Frontmatter {
                        file: file_name.clone(),
                        message: e.to_string(),
                    })?;

                let mut options = Options::empty();
                options.insert(Options::ENABLE_STRIKETHROUGH);
                options.insert(Options::ENABLE_TABLES);
                options.insert(Options::ENABLE_FOOTNOTES);
                options.insert(Options::ENABLE_TASKLISTS);

                let parser = MarkdownParser::new_ext(&doc.content, options);
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);

                posts.push(Post {
                    metadata: doc.metadata,
                    content: doc.content,
                    html_content: html_output,
                });
            }
        }

        Ok(posts)
    }

    fn generate_post_page(&self, post: &Post) -> Result<(), Error> {
        let mut context = Context::new();
        context.insert("post", post);
        context.insert("config", &self.config);
        context.insert("title", &post.metadata.title); // Required by base.html

        let html = self.tera.render("post.html", &context)?;

        let output_path = self.output_dir.join("posts").join(&post.metadata.slug);
        fs::create_dir_all(&output_path)?;

        fs::write(output_path.join("index.html"), html)?;
        Ok(())
    }

    fn generate_index_page(&self, posts: &[Post]) -> Result<(), Error> {
        let mut context = Context::new();
        context.insert("posts", posts);
        context.insert("config", &self.config);
        context.insert("title", &self.config.title); // Required by base.html

        let html = self.tera.render("index.html", &context)?;
        fs::write(self.output_dir.join("index.html"), html)?;
        Ok(())
    }
    fn copy_static_assets(&self) -> Result<(), Error> {
        let static_dir = self.templates_dir.join("static");
        if static_dir.exists() {
            let output_static = self.output_dir.join("static");
            fs::create_dir_all(&output_static)?;

            for entry in WalkDir::new(&static_dir).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file() {
                    let relative_path = entry
                        .path()
                        .strip_prefix(&static_dir)
                        .map_err(|_| Error::DirectoryNotFound(static_dir.clone()))?;
                    let output_path = output_static.join(relative_path);

                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::copy(entry.path(), output_path)?;
                }
            }
        }

        Ok(())
    }

    pub fn generate_site(&self) -> Result<(), Error> {
        let pb = indicatif::ProgressBar::new_spinner();
        let style = indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap();
        pb.set_style(style);

        pb.set_message("Reading posts...");
        let mut posts = self.read_posts()?;

        pb.set_message("Sorting posts...");
        posts.sort_by(|a, b| {
            NaiveDate::parse_from_str(&b.metadata.date, "%Y-%m-%d")
                .unwrap()
                .cmp(&NaiveDate::parse_from_str(&a.metadata.date, "%Y-%m-%d").unwrap())
        });

        pb.set_message("Generating post pages...");
        for post in &posts {
            pb.set_message(format!("Generating post: {}", post.metadata.title));
            self.generate_post_page(post)?;
        }

        pb.set_message("Generating index page...");
        self.generate_index_page(&posts)?;

        pb.set_message("Copying static assets...");
        self.copy_static_assets()?;

        pb.finish_and_clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_site() -> Result<(TempDir, Config), Error> {
        let temp_dir = TempDir::new().unwrap();
        let site_dir = temp_dir.path();

        // Create necessary directories
        fs::create_dir_all(site_dir.join("posts"))?;
        fs::create_dir_all(site_dir.join("templates"))?;
        fs::create_dir_all(site_dir.join("static"))?;

        // Create minimal test templates that match our actual usage
        fs::write(
            site_dir.join("templates/base.html"),
            r#"<!DOCTYPE html>
<html>
<head><title>{% block title %}{{ title }}{% endblock %}</title></head>
<body>{% block content %}{% endblock %}</body>
</html>"#,
        )?;

        fs::write(
            site_dir.join("templates/post.html"),
            r#"{% extends "base.html" %}
{% block content %}
<article>
    <h1>{{ post.metadata.title }}</h1>
    {{ post.html_content | safe }}
</article>
{% endblock %}"#,
        )?;

        fs::write(
            site_dir.join("templates/index.html"),
            r#"{% extends "base.html" %}
{% block content %}
{% for post in posts %}
<article>
    <h2><a href="/posts/{{ post.metadata.slug }}">{{ post.metadata.title }}</a></h2>
    <p>{{ post.metadata.preview }}</p>
</article>
{% endfor %}
{% endblock %}"#,
        )?;

        // Create test config
        let config = Config::default();
        fs::write(
            site_dir.join("config.toml"),
            include_str!(concat!(env!("OUT_DIR"), "/templates/config.toml")),
        )?;

        Ok((temp_dir, config))
    }
    fn create_test_post(posts_dir: &Path, title: &str) -> Result<(), Error> {
        let date = Local::now().format("%Y-%m-%d");
        let slug = title.to_lowercase().replace(' ', "-");
        let content = format!(
            r#"---
title: "{}"
date: {}
author: "Test Author"
tags: ["test"]
preview: "Test preview"
slug: "{}"
---

# Test Content

This is a test post."#,
            title, date, slug
        );

        fs::write(posts_dir.join(format!("{}-{}.md", date, slug)), content)?;
        Ok(())
    }

    #[test]
    fn test_new_site_generator() -> Result<(), Error> {
        let (temp_dir, config) = setup_test_site()?;
        let generator = SiteGenerator::new(temp_dir.path(), Some(config))?;

        assert!(generator.posts_dir.exists());
        assert!(generator.templates_dir.exists());
        Ok(())
    }

    #[test]
    fn test_read_posts() -> Result<(), Error> {
        let (temp_dir, config) = setup_test_site()?;
        let generator = SiteGenerator::new(temp_dir.path(), Some(config))?;

        create_test_post(&generator.posts_dir, "Test Post")?;

        let posts = generator.read_posts()?;
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].metadata.title, "Test Post");
        assert_eq!(posts[0].metadata.tags, vec!["test"]);
        Ok(())
    }

    #[test]
    fn test_generate_post_page() -> Result<(), Error> {
        let (temp_dir, config) = setup_test_site()?;
        let generator = SiteGenerator::new(temp_dir.path(), Some(config))?;

        create_test_post(&generator.posts_dir, "Test Post")?;

        let posts = generator.read_posts()?;
        generator.generate_post_page(&posts[0])?;

        let output_path = generator
            .output_dir
            .join("posts")
            .join(&posts[0].metadata.slug)
            .join("index.html");

        assert!(output_path.exists());
        let content = fs::read_to_string(output_path)?;
        assert!(content.contains("Test Post"));
        Ok(())
    }

    #[test]
    fn test_generate_index_page() -> Result<(), Error> {
        let (temp_dir, config) = setup_test_site()?;
        let generator = SiteGenerator::new(temp_dir.path(), Some(config))?;

        create_test_post(&generator.posts_dir, "Test Post 1")?;
        create_test_post(&generator.posts_dir, "Test Post 2")?;

        let posts = generator.read_posts()?;
        generator.generate_index_page(&posts)?;

        let index_path = generator.output_dir.join("index.html");
        assert!(index_path.exists());

        let content = fs::read_to_string(index_path)?;
        assert!(content.contains("Test Post 1"));
        assert!(content.contains("Test Post 2"));
        Ok(())
    }

    #[test]
    fn test_copy_static_assets() -> Result<(), Error> {
        let (temp_dir, config) = setup_test_site()?;
        let generator = SiteGenerator::new(temp_dir.path(), Some(config))?;

        // Create a test static file
        let static_dir = generator.templates_dir.join("static");
        fs::create_dir_all(&static_dir)?;
        fs::write(static_dir.join("test.css"), "body { color: green; }")?;

        generator.copy_static_assets()?;

        let output_path = generator.output_dir.join("static").join("test.css");
        assert!(output_path.exists());

        let content = fs::read_to_string(output_path)?;
        assert_eq!(content, "body { color: green; }");
        Ok(())
    }

    #[test]
    fn test_generate_site() -> Result<(), Error> {
        let (temp_dir, config) = setup_test_site()?;
        let generator = SiteGenerator::new(temp_dir.path(), Some(config))?;

        create_test_post(&generator.posts_dir, "Test Post")?;
        generator.generate_site()?;

        assert!(generator.output_dir.exists());
        assert!(generator.output_dir.join("index.html").exists());
        assert!(generator.output_dir.join("posts").exists());
        Ok(())
    }
}
