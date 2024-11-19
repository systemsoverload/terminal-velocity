use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("templates");
    fs::create_dir_all(&out_path).unwrap();

    // Base templates that contain only the bare minimum
    let base_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}{% endblock %}</title>
</head>
<body>
    {% block content %}{% endblock %}
</body>
</html>"#;

    let index_html = r#"{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
    {% for post in posts %}
    <article>
        <h2><a href="/posts/{{ post.metadata.slug }}">{{ post.metadata.title }}</a></h2>
        <p>{{ post.metadata.preview }}</p>
    </article>
    {% endfor %}
{% endblock %}"#;

    let post_html = r#"{% extends "base.html" %}

{% block title %}{{ post.metadata.title }}{% endblock %}

{% block content %}
    <article>
        <h1>{{ post.metadata.title }}</h1>
        {{ post.html_content | safe }}
    </article>
{% endblock %}"#;

    let example_md = r#"---
title: "Welcome to Terminal Velocity"
date: 2024-01-01
author: "Terminal Velocity"
tags: ["welcome", "getting-started"]
preview: "Get started with your new Terminal Velocity blog"
slug: "welcome"
---

# Welcome to Terminal Velocity

This is your first blog post. Edit or delete it and start writing!"#;

    let config_toml = r#"title = "My Terminal Velocity Blog"
description = "A blazingly fast tech blog"
base_url = "http://localhost:8000"

[author]
name = "Anonymous"
email = "author@example.com"

[build]
output_dir = "dist"
port = 8000"#;

    // Write templates to the output directory
    fs::write(out_path.join("base.html"), base_html).unwrap();
    fs::write(out_path.join("index.html"), index_html).unwrap();
    fs::write(out_path.join("post.html"), post_html).unwrap();
    fs::write(out_path.join("example.md"), example_md).unwrap();
    fs::write(out_path.join("config.toml"), config_toml).unwrap();

    // Tell Cargo to rerun this script if any of these files change
    println!("cargo:rerun-if-changed=build.rs");
}
