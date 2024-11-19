use crate::errors::Error;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostMetadata {
    pub title: String,
    pub date: String,
    pub author: String,
    pub tags: Vec<String>,
    pub preview: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct Post {
    pub metadata: PostMetadata,
    pub content: String,
    pub html_content: String,
}

pub fn create_new_post(title: &str) -> Result<(), Error> {
    let date = Local::now().format("%Y-%m-%d");
    let slug = slugify(title);
    let posts_dir = PathBuf::from("posts");

    if !posts_dir.exists() {
        fs::create_dir_all(&posts_dir)?;
    }

    let filename = format!("{}-{}.md", date, slug);
    let filepath = posts_dir.join(filename);

    let template = format!(
        r#"---
title: "{}"
date: {}
author: "Anonymous"
tags: []
preview: "Add a preview of your post here"
slug: "{}"
---

Write your post content here...
"#,
        title, date, slug
    );

    fs::write(filepath, template)?;
    Ok(())
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| match c {
            'a'..='z' | '0'..='9' => Some(c),
            ' ' | '-' | '_' => Some('-'),
            _ => None,
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
