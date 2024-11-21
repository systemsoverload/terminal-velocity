use crate::config::Config;
use crate::errors::Error;
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct Post {
    pub metadata: PostMetadata,
    pub content: String,
    pub html_content: String,
}

impl Post {
    // Get the assets directory for this post
    pub fn assets_dir(&self, config: &Config) -> PathBuf {
        config.posts_dir()
            .join(&self.metadata.slug)
            .join(&config.build.post_assets_dir)
    }

    // Get the output directory for this post's assets
    pub fn assets_output_dir(&self, config: &Config) -> PathBuf {
        config.output_dir()
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
            &format!("{}/{}/", post_url, config.build.post_assets_dir)
        );
        self.content = content.replace(
            &format!("./{}", config.build.post_assets_dir),
            &format!("{}/{}", post_url, config.build.post_assets_dir)
        );

        // Also update the HTML content
        let html_content = self.html_content.replace(
            &format!("./{}/", config.build.post_assets_dir),
            &format!("{}/{}/", post_url, config.build.post_assets_dir)
        );
        self.html_content = html_content.replace(
            &format!("./{}", config.build.post_assets_dir),
            &format!("{}/{}", post_url, config.build.post_assets_dir)
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
        title,
        date,
        slug,
        outline.as_ref().unwrap_or(&String::new()),
        if outline.is_some() {
            "<!-- Generated outline above. Replace with your content. -->"
        } else {
            "Write your post content here..."
        }
    );

    fs::write(filepath.clone(), template)?;
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
                port: 8000,
                verbose: false,
                output_dir: "dist".to_string(),
                posts_dir: "posts".to_string(),
                templates_dir: "templates".to_string(),
                static_dir: "static".to_string(),
                post_assets_dir: "assets".to_string()
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
                port: 8000,
                verbose: false,
                output_dir: "dist".to_string(),
                posts_dir: "posts".to_string(),
                templates_dir: "templates".to_string(),
                static_dir: "static".to_string(),
                post_assets_dir: "assets".to_string()
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
