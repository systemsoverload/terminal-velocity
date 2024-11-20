use crate::post::Post;
use crate::post::PostMetadata;
use chrono::NaiveDate;
use pulldown_cmark::{html, Options, Parser as MarkdownParser};
use std::{
    fs::{self},
    path::PathBuf,
};
use tera::{Context, Tera};
use walkdir::WalkDir;
use yaml_front_matter::{Document, YamlFrontMatter};

use crate::config::Config;
use crate::errors::Error;

pub struct SiteGenerator {
    #[allow(dead_code)]
    site_dir: PathBuf,

    posts_dir: PathBuf,
    output_dir: PathBuf,
    static_dir: PathBuf,
    tera: Tera,
    config: Config,
}

impl SiteGenerator {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let posts_dir = config.posts_dir();
        let output_dir = config.output_dir();
        let templates_dir = config.templates_dir();
        let static_dir = config.static_dir();
        let site_dir = config.site_dir();

        fs::create_dir_all(&output_dir)?;

        let tera = Tera::new(&format!("{}/**/*.html", templates_dir.display()))
            .map_err(Error::Template)?;

        Ok(Self {
            site_dir: site_dir.to_path_buf(),
            posts_dir,
            output_dir,
            static_dir,
            tera,
            // XXX - This feels clunky, maybe refactor?
            config: config.clone(),
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

        // TODO - Handle the case where the index template doesn't exist (yet?)
        let html = self.tera.render("index.html", &context)?;
        fs::write(self.output_dir.join("index.html"), html)?;
        Ok(())
    }

    fn copy_static_files(&self) -> Result<(), Error> {
        if self.static_dir.exists() {
            for entry in WalkDir::new(&self.static_dir)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let relative_path = entry
                    .path()
                    .strip_prefix(&self.static_dir)
                    .map_err(|_| Error::DirectoryNotFound(self.static_dir.clone()))?;
                let dest_path = self.output_dir.join("static").join(relative_path);
                // Create parent directories if they don't exist
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::copy(entry.path(), &dest_path)?;

                if self.config.build.verbose {
                    println!("Copied static file: {}", relative_path.display());
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
            match (
                NaiveDate::parse_from_str(&a.metadata.date, "%Y-%m-%d"),
                NaiveDate::parse_from_str(&b.metadata.date, "%Y-%m-%d"),
            ) {
                (Ok(date_a), Ok(date_b)) => date_b.cmp(&date_a),
                (Ok(_), Err(_)) => std::cmp::Ordering::Less,
                (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
                (Err(_), Err(_)) => a.metadata.date.cmp(&b.metadata.date), // Fall back to string comparison
            }
        });

        pb.set_message("Generating post pages...");
        for post in &posts {
            pb.set_message(format!("Generating post: {}", post.metadata.title));
            self.generate_post_page(post)?;
        }

        pb.set_message("Generating index page...");
        self.generate_index_page(&posts)?;

        pb.set_message("Copying static assets...");
        self.copy_static_files()?;

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

    fn setup_test_site() -> Result<Config, Error> {
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

        Ok(config)
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
        let config = setup_test_site()?;
        let generator = SiteGenerator::new(&config)?;

        assert!(generator.posts_dir.exists());
        Ok(())
    }

    #[test]
    fn test_read_posts() -> Result<(), Error> {
        let config = setup_test_site()?;
        let generator = SiteGenerator::new(&config)?;

        create_test_post(&generator.posts_dir, "Test Post")?;

        let posts = generator.read_posts()?;
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].metadata.title, "Test Post");
        assert_eq!(posts[0].metadata.tags, vec!["test"]);
        Ok(())
    }

    #[test]
    fn test_generate_post_page() -> Result<(), Error> {
        let config = setup_test_site()?;
        let generator = SiteGenerator::new(&config)?;

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
        let config = setup_test_site()?;
        let generator = SiteGenerator::new(&config)?;

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
        let config = setup_test_site()?;
        let generator = SiteGenerator::new(&config)?;

        // Create a test static file
        let static_dir = generator.site_dir.join("static");
        fs::create_dir_all(&static_dir)?;
        fs::write(static_dir.join("test.css"), "body { color: green; }")?;

        generator.copy_static_files()?;

        let output_path = generator.output_dir.join("static").join("test.css");
        assert!(output_path.exists());

        let content = fs::read_to_string(output_path)?;
        assert_eq!(content, "body { color: green; }");
        Ok(())
    }

    #[test]
    fn test_generate_site() -> Result<(), Error> {
        let config = setup_test_site()?;
        let generator = SiteGenerator::new(&config)?;

        create_test_post(&generator.posts_dir, "Test Post")?;
        generator.generate_site()?;

        assert!(generator.output_dir.exists());
        assert!(generator.output_dir.join("index.html").exists());
        assert!(generator.output_dir.join("posts").exists());
        Ok(())
    }
}
