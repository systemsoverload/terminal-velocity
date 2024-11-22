use chrono::NaiveDate;
use std::fs::{self};
use tera::{Context, Tera};
use walkdir::WalkDir;
use yaml_front_matter::{Document, YamlFrontMatter};

use crate::config::Config;
use crate::errors::Error;
use crate::markdown::MarkdownProcessor;
use crate::post::Post;
use crate::post::PostMetadata;

pub struct SiteGenerator {
    config: Config,
    tera: Tera,
    markdown: MarkdownProcessor,
}

impl SiteGenerator {
    pub fn new(config: &Config) -> Result<Self, Error> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(config.output_dir())?;

        // Initialize Tera with templates
        let templates_dir = config.templates_dir();
        if !templates_dir.exists() {
            return Err(Error::Template(tera::Error::msg(
                "Templates directory does not exist",
            )));
        }

        let template_pattern = format!("{}/**/*.html", templates_dir.display());
        let tera = Tera::new(&template_pattern).map_err(Error::Template)?;

        Ok(Self {
            config: config.clone(),
            tera,
            markdown: MarkdownProcessor::new(),
        })
    }

    fn read_posts(&self) -> Result<Vec<Post>, Error> {
        let mut posts = Vec::new();
        let posts_dir = self.config.posts_dir();

        for entry in WalkDir::new(posts_dir)
            .min_depth(1)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            let content = fs::read_to_string(entry.path())?;
            let file_name = entry.path().display().to_string();

            let doc: Document<PostMetadata> =
                YamlFrontMatter::parse(&content).map_err(|e| Error::Frontmatter {
                    file: file_name,
                    message: e.to_string(),
                })?;

            posts.push(Post {
                metadata: doc.metadata,
                content: doc.content.clone(),
                html_content: self.markdown.render(&doc.content),
            });
        }

        Ok(posts)
    }

    fn copy_post_assets(&self, post: &Post) -> Result<(), Error> {
        let assets_dir = post.assets_dir(&self.config);

        if !assets_dir.exists() {
            return Ok(());
        }

        let output_dir = post.assets_output_dir(&self.config);

        if self.config.build.verbose {
            println!("Copying assets for post: {}", post.metadata.title);
            println!("  From: {}", assets_dir.display());
            println!("  To: {}", output_dir.display());
        }

        for entry in WalkDir::new(&assets_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let rel_path = entry
                .path()
                .strip_prefix(&assets_dir)
                .map_err(|_| Error::DirectoryNotFound(assets_dir.clone()))?;

            let dest_path = output_dir.join(rel_path);

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(entry.path(), &dest_path)?;

            if self.config.build.verbose {
                println!("    Copied: {}", rel_path.display());
            }
        }

        Ok(())
    }

    fn generate_post_page(&self, post: &mut Post) -> Result<(), Error> {
        // Process asset paths before rendering
        post.process_asset_paths(&self.config);

        // Copy post assets
        self.copy_post_assets(post)?;

        let mut context = Context::new();
        context.insert("post", post);
        context.insert("config", &self.config);
        context.insert("title", &post.metadata.title);

        let html = self.tera.render("post.html", &context)?;

        let output_path = self
            .config
            .output_dir()
            .join("posts")
            .join(&post.metadata.slug)
            .join("index.html");

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(output_path, html)?;
        Ok(())
    }

    fn generate_index_page(&self, posts: &[Post]) -> Result<(), Error> {
        let mut context = Context::new();
        context.insert("posts", posts);
        context.insert("config", &self.config);
        context.insert("title", &self.config.title);

        let html = self.tera.render("index.html", &context)?;
        fs::write(self.config.output_dir().join("index.html"), html)?;
        Ok(())
    }

    fn copy_static_files(&self) -> Result<(), Error> {
        let static_dir = self.config.static_dir();
        if static_dir.exists() {
            for entry in WalkDir::new(&static_dir)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let relative_path = entry
                    .path()
                    .strip_prefix(&static_dir)
                    .map_err(|_| Error::DirectoryNotFound(static_dir.clone()))?;

                let dest_path = self.config.output_dir().join(relative_path);

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

        // Ensure the output directory exists
        fs::create_dir_all(self.config.output_dir())?;

        pb.set_message("Reading posts...");
        let mut posts = self.read_posts()?;

        pb.set_message("Sorting posts...");
        posts.sort_by(|a, b| {
            let parse_date = |date: &str| NaiveDate::parse_from_str(date, "%Y-%m-%d");
            match (parse_date(&a.metadata.date), parse_date(&b.metadata.date)) {
                (Ok(date_a), Ok(date_b)) => date_b.cmp(&date_a),
                _ => b.metadata.date.cmp(&a.metadata.date),
            }
        });

        pb.set_message("Generating post pages and copying assets...");
        for post in &mut posts {
            pb.set_message(format!("Processing post: {}", post.metadata.title));

            // First copy assets, then generate the page with updated paths
            self.copy_post_assets(post)?;
            self.generate_post_page(post)?;
        }

        pb.set_message("Generating index page...");
        self.generate_index_page(&posts)?;

        pb.set_message("Copying static assets...");
        self.copy_static_files()?;

        pb.finish_and_clear();

        if self.config.build.verbose {
            println!("\nSite generation complete!");
            println!("Output directory: {}", self.config.output_dir().display());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Author, BuildConfig};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> Config {
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
            build: BuildConfig {
                port: 8000,
                verbose: false,
                hot_reload: true,
                output_dir: "dist".to_string(),
                posts_dir: "posts".to_string(),
                templates_dir: "templates".to_string(),
                static_dir: "static".to_string(),
                post_assets_dir: "assets".to_string(),
            },
        }
    }

    fn setup_test_site(temp_dir: &TempDir) -> std::io::Result<()> {
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

    #[test]
    fn test_site_generator_initialization() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        setup_test_site(&temp_dir)?;
        let config = create_test_config(&temp_dir);

        let generator = SiteGenerator::new(&config)?;
        assert!(generator.config.output_dir().exists());
        Ok(())
    }

    #[test]
    fn test_read_posts() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        setup_test_site(&temp_dir)?;
        let config = create_test_config(&temp_dir);
        let generator = SiteGenerator::new(&config)?;

        let posts = generator.read_posts()?;
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].metadata.title, "Test Post");
        assert_eq!(posts[0].metadata.date, "2024-01-01");
        Ok(())
    }

    #[test]
    fn test_generate_post_page() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        setup_test_site(&temp_dir)?;
        let config = create_test_config(&temp_dir);
        let generator = SiteGenerator::new(&config)?;

        let mut posts = generator.read_posts()?;
        generator.generate_post_page(&mut posts[0])?;

        let post_path = config.output_dir().join("posts/test-post/index.html");
        assert!(post_path.exists());
        Ok(())
    }

    #[test]
    fn test_generate_index_page() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        setup_test_site(&temp_dir)?;
        let config = create_test_config(&temp_dir);
        let generator = SiteGenerator::new(&config)?;

        let posts = generator.read_posts()?;
        generator.generate_index_page(&posts)?;

        let index_path = generator.config.output_dir().join("index.html");
        assert!(index_path.exists());
        Ok(())
    }

    #[test]
    fn test_copy_static_files() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        setup_test_site(&temp_dir)?;
        let config = create_test_config(&temp_dir);
        let generator = SiteGenerator::new(&config)?;

        generator.copy_static_files()?;

        let css_path = generator.config.output_dir().join("css/style.css");
        assert!(css_path.exists());
        Ok(())
    }

    #[test]
    fn test_generate_site() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        setup_test_site(&temp_dir)?;
        let config = create_test_config(&temp_dir);
        let generator = SiteGenerator::new(&config)?;

        generator.generate_site()?;

        assert!(generator
            .config
            .output_dir()
            .join("posts/test-post/index.html")
            .exists());
        assert!(generator.config.output_dir().join("index.html").exists());
        assert!(generator.config.output_dir().join("css/style.css").exists());
        Ok(())
    }

    #[test]
    fn test_post_sorting() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        setup_test_site(&temp_dir)?;

        // Create additional test posts with different dates
        fs::write(
            temp_dir.path().join("posts/old-post.md"),
            r#"---
title: "Old Post"
date: 2023-12-31
slug: "old-post"
---
Old content"#,
        )?;

        fs::write(
            temp_dir.path().join("posts/new-post.md"),
            r#"---
title: "New Post"
date: 2024-01-02
slug: "new-post"
---
New content"#,
        )?;

        let config = create_test_config(&temp_dir);
        let generator = SiteGenerator::new(&config)?;
        let posts = generator.read_posts()?;

        assert_eq!(posts.len(), 3);
        let sorted_posts = {
            let mut posts = posts;
            posts.sort_by(|a, b| {
                let parse_date = |date: &str| NaiveDate::parse_from_str(date, "%Y-%m-%d");
                match (parse_date(&a.metadata.date), parse_date(&b.metadata.date)) {
                    (Ok(date_a), Ok(date_b)) => date_b.cmp(&date_a),
                    _ => b.metadata.date.cmp(&a.metadata.date),
                }
            });
            posts
        };

        assert_eq!(sorted_posts[0].metadata.date, "2024-01-02");
        assert_eq!(sorted_posts[1].metadata.date, "2024-01-01");
        assert_eq!(sorted_posts[2].metadata.date, "2023-12-31");
        Ok(())
    }

    #[test]
    fn test_error_handling_missing_template() -> Result<(), Error> {
        let temp_dir = TempDir::new()?;
        fs::create_dir_all(temp_dir.path().join("templates"))?;

        // Create config without templates
        fs::write(
            temp_dir.path().join("config.toml"),
            r#"
            title = "Test Blog"
            description = "Test Description"
            base_url = "http://localhost:8000"

            [author]
            name = "Test Author"
            email = "test@example.com"

            [build]
            port = 8000
            verbose = false
            output_dir = "dist"
            posts_dir = "posts"
            templates_dir = "dir_that_doesnt_exist"
            static_dir = "static"
            "#,
        )?;

        let config = Config::load(temp_dir.path())?;
        let result = SiteGenerator::new(&config);

        assert!(matches!(result, Err(Error::Template(_))));
        if let Err(Error::Template(e)) = result {
            assert!(e.to_string().contains("Templates directory does not exist"));
        } else {
            panic!("Expected Template error");
        }

        Ok(())
    }
}
