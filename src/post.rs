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

    if !posts_dir.exists() {
        fs::create_dir_all(&posts_dir)?;
    }

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
