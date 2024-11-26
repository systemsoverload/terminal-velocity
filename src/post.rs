use chrono::{Local, NaiveDate};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use yaml_front_matter::{Document, YamlFrontMatter};

use crate::config::Config;
use crate::errors::Error;
use crate::markdown::MarkdownProcessor;

#[derive(Debug, Serialize)]
pub struct Post {
    pub metadata: PostMetadata,
    pub content: String,
    pub html_content: String,
}

impl Post {
    pub fn new_from_path(path: &Path, md_proc: &MarkdownProcessor) -> Result<Self, Error> {
        let content = fs::read_to_string(path)?;
        let file_name = path.display().to_string();

        let doc: Document<PostMetadata> =
            YamlFrontMatter::parse(&content).map_err(|e| Error::Frontmatter {
                file: file_name,
                message: e.to_string(),
            })?;

        let mut post = Self {
            metadata: doc.metadata,
            content: doc.content.clone(),
            html_content: md_proc.render(&doc.content),
        };

        post.metadata.read_time = calculate_read_time(&doc.content);
        Ok(post)
    }
    // Get the assets directory for this post
    pub fn assets_dir(&self, config: &Config) -> PathBuf {
        config
            .posts_dir()
            .join(&self.metadata.slug)
            .join(&config.build.post_assets_dir)
    }

    // Get the output directory for this post's assets
    pub fn assets_output_dir(&self, config: &Config) -> PathBuf {
        config
            .output_dir()
            .join("posts")
            .join(&self.metadata.slug)
            .join(&config.build.post_assets_dir)
    }

    // Process post content to update asset paths
    pub fn process_asset_paths(&mut self, config: &Config) {
        // Update markdown image/video paths to point to the correct output location
        let post_url = format!("/posts/{}", self.metadata.slug);

        // Replace relative asset paths with absolute paths
        let content = self.content.replace(
            &format!("./{}/", config.build.post_assets_dir),
            &format!("{}/{}/", post_url, config.build.post_assets_dir),
        );
        self.content = content.replace(
            &format!("./{}", config.build.post_assets_dir),
            &format!("{}/{}", post_url, config.build.post_assets_dir),
        );

        // Also update the HTML content
        let html_content = self.html_content.replace(
            &format!("./{}/", config.build.post_assets_dir),
            &format!("{}/{}/", post_url, config.build.post_assets_dir),
        );
        self.html_content = html_content.replace(
            &format!("./{}", config.build.post_assets_dir),
            &format!("{}/{}", post_url, config.build.post_assets_dir),
        );
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostMetadata {
    pub title: String,
    #[serde(deserialize_with = "validate_date")]
    pub date: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub preview: String,
    #[serde(deserialize_with = "validate_and_slugify")]
    pub slug: String,
    #[serde(default)]
    pub read_time: u32,
}

fn default_author() -> String {
    "Anonymous".to_string()
}

fn validate_date<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let date_str = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|_| serde::de::Error::custom("Invalid date format. Expected YYYY-MM-DD"))?;
    Ok(date_str)
}

fn validate_and_slugify<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    let slug = slugify(&text);
    if slug.is_empty() {
        return Err(serde::de::Error::custom("Slug cannot be empty"));
    }
    Ok(slug)
}

// Calculate read time in minutes based on word count
fn calculate_read_time(content: &str) -> u32 {
    const WORDS_PER_MINUTE: u32 = 200; // Average adult reading speed

    // Create a markdown parser with basic options
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);

    // Count words only in actual content, excluding code blocks and YAML frontmatter
    let mut word_count = 0;
    let mut in_code_block = false;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(_)) => in_code_block = true,
            Event::End(TagEnd::CodeBlock) => in_code_block = false,
            Event::Text(text) if !in_code_block => {
                // Count words in text content
                word_count += text.split_whitespace().count();
            }
            _ => {}
        }
    }

    // Calculate minutes rounded up to the nearest minute
    let minutes = (word_count as f32 / WORDS_PER_MINUTE as f32).ceil() as u32;
    if minutes == 0 {
        1
    } else {
        minutes
    }
}

pub fn slugify(text: &str) -> String {
    let slug = text
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            'a'..='z' | '0'..='9' => Some(c),
            ' ' | '-' | '_' => Some('-'),
            _ => None,
        })
        .collect::<String>();

    // Remove consecutive hyphens and trim
    slug.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub async fn create_new_post(
    config: &Config,
    title: &str,
    prompt: Option<String>,
    api_key: Option<String>,
) -> Result<PathBuf, Error> {
    let date = Local::now().format("%Y-%m-%d");
    let slug = slugify(title);

    if slug.is_empty() {
        return Err(Error::Other(
            "Post title must contain at least one alphanumeric character".into(),
        ));
    }

    let posts_dir = config.posts_dir();
    let assets_dir = posts_dir.join(&config.build.post_assets_dir);

    if !posts_dir.exists() {
        fs::create_dir_all(&posts_dir)?;
    }
    fs::create_dir_all(&assets_dir)?;

    let filename = format!("{}-{}.md", date, slug);
    let filepath = posts_dir.join(filename);

    // Generate outline if requested
    let outline = if let (Some(prompt), Some(key)) = (prompt, api_key) {
        Some(crate::anthropic::generate_outline(&prompt, Some(&key)).await?)
    } else {
        None
    };

    let template = format!(
        r#"---
title: "{}"
date: {}
author: "Anonymous"
tags: []
preview: "Add a preview of your post here"
slug: "{}"
---

{}

{}"#,
        title.trim(),
        date,
        slug.trim(),
        outline.as_ref().map(|o| o.trim()).unwrap_or(""),
        if outline.is_some() {
            "<!-- Generated outline above. Replace with your content. -->"
        } else {
            "Write your post content here..."
        }
    );

    fs::write(filepath.clone(), template)?;

    println!("📝 Created new post: {}", title);

    Ok(filepath)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use tempfile::TempDir;

    #[test]
    fn test_slugify() {
        let test_cases = vec![
            ("Hello World!", "hello-world"),
            ("Test 123", "test-123"),
            ("Multiple   Spaces", "multiple-spaces"),
            ("special#@!characters", "specialcharacters"),
            ("trailing-dash-", "trailing-dash"),
            ("-leading-dash", "leading-dash"),
            ("mixed_CASE_123", "mixed-case-123"),
            ("", ""),
            ("   ", ""),
            ("###", ""),
        ];

        for (input, expected) in test_cases {
            assert_eq!(slugify(input), expected);
        }
    }

    #[test]
    fn test_validate_date() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct TestDate {
            #[serde(deserialize_with = "validate_date")]
            date: String,
        }

        // Test valid date
        let valid = toml::from_str::<TestDate>("date = '2024-01-01'").unwrap();
        assert_eq!(valid.date, "2024-01-01");

        // Test invalid dates
        let invalid_cases = [
            "date = '2024/01/01'",
            "date = '01-01-2024'",
            "date = '2024-13-01'",
            "date = 'not-a-date'",
        ];

        for invalid in invalid_cases {
            assert!(toml::from_str::<TestDate>(invalid).is_err());
        }
    }

    #[tokio::test]
    async fn test_create_new_post() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            site_dir: temp_dir.path().to_path_buf(),
            base_url: "http://test.com".to_string(),
            title: "Test Blog".to_string(),
            description: "Test Description".to_string(),
            author: config::Author {
                name: "Test Author".to_string(),
                email: "test@example.com".to_string(),
            },
            build: config::BuildConfig {
                verbose: false,

                output_dir: "dist".to_string(),
                posts_dir: "posts".to_string(),
                templates_dir: "templates".to_string(),
                static_dir: "static".to_string(),
                post_assets_dir: "assets".to_string(),
            },
            server: config::ServerConfig {
                auto_build: true,
                port: 8000,
                hot_reload: true,
            },
        };

        // Create a new post
        let title = "Test Post Title";
        let filepath = create_new_post(&config, title, None, None).await.unwrap();

        // Verify file was created
        assert!(filepath.exists());

        // Check content
        let content = fs::read_to_string(filepath).unwrap();
        assert!(content.contains("title: \"Test Post Title\""));
        assert!(content.contains("slug: \"test-post-title\""));
        assert!(content.contains("date: "));
        assert!(content.contains("Write your post content here..."));
    }

    #[test]
    fn test_empty_title_error() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            site_dir: temp_dir.path().to_path_buf(),
            base_url: "http://test.com".to_string(),
            title: "Test Blog".to_string(),
            description: "Test Description".to_string(),
            author: config::Author {
                name: "Test Author".to_string(),
                email: "test@example.com".to_string(),
            },
            build: config::BuildConfig {
                verbose: false,

                output_dir: "dist".to_string(),
                posts_dir: "posts".to_string(),
                templates_dir: "templates".to_string(),
                static_dir: "static".to_string(),
                post_assets_dir: "assets".to_string(),
            },
            server: config::ServerConfig {
                auto_build: true,
                port: 8000,
                hot_reload: true,
            },
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(create_new_post(&config, "###", None, None));
        assert!(result.is_err());
    }

    #[test]
    fn test_post_metadata_deserialization() {
        let yaml = r#"---
title: "Test Post"
date: "2024-01-01"
author: "Test Author"
tags: ["tag1", "tag2"]
preview: "Test preview"
slug: "test-post"
---"#;

        let doc: yaml_front_matter::Document<PostMetadata> =
            yaml_front_matter::YamlFrontMatter::parse(yaml).unwrap();

        assert_eq!(doc.metadata.title, "Test Post");
        assert_eq!(doc.metadata.date, "2024-01-01");
        assert_eq!(doc.metadata.author, "Test Author");
        assert_eq!(doc.metadata.tags, vec!["tag1", "tag2"]);
        assert_eq!(doc.metadata.preview, "Test preview");
        assert_eq!(doc.metadata.slug, "test-post");
    }

    #[test]
    fn test_post_metadata_defaults() {
        let yaml = r#"---
title: "Test Post"
date: "2024-01-01"
slug: "test-post"
---"#;

        let doc: yaml_front_matter::Document<PostMetadata> =
            yaml_front_matter::YamlFrontMatter::parse(yaml).unwrap();

        assert_eq!(doc.metadata.author, "Anonymous");
        assert!(doc.metadata.tags.is_empty());
        assert_eq!(doc.metadata.preview, "");
    }
}

#[test]
fn test_read_time_in_post_metadata() {
    let content = r#"---
title: "Test Post"
date: "2024-01-01"
slug: "test-post"
---

This is a test post with enough words to make it take at least one minute to read.
Let's add some more text to make sure we have enough content to test the read time calculation.
We'll keep adding words until we have enough for a proper test of the functionality."#;

    let temp_dir = tempfile::TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-post.md");
    fs::write(&test_file, content).unwrap();

    let md_proc = MarkdownProcessor::new();
    let post = Post::new_from_path(&test_file, &md_proc).unwrap();

    assert!(
        post.metadata.read_time > 0,
        "Read time should be calculated and greater than 0"
    );
}
