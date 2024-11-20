# Terminal Velocity

[![Main Branch Status](https://github.com/systemsoverload/terminal-velocity/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/your-username/terminal-velocity/actions/workflows/rust.yml)

A blazingly fast static site generator for developers who want to write things down. Built with Rust for performance and efficiency.

```
████████╗███████╗██████╗ ███╗   ███╗██╗███╗   ██╗ █████╗ ██╗      
╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██║████╗  ██║██╔══██╗██║      
   ██║   █████╗  ██████╔╝██╔████╔██║██║██╔██╗ ██║███████║██║      
   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║██║╚██╗██║██╔══██║██║      
   ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║██║ ╚████║██║  ██║███████╗ 
   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝ 

██╗   ██╗███████╗██╗      ██████╗  ██████╗██╗████████╗██╗   ██╗   
██║   ██║██╔════╝██║     ██╔═══██╗██╔════╝██║╚══██╔══╝╚██╗ ██╔╝   
██║   ██║█████╗  ██║     ██║   ██║██║     ██║   ██║    ╚████╔╝    
╚██╗ ██╔╝██╔══╝  ██║     ██║   ██║██║     ██║   ██║     ╚██╔╝     
 ╚████╔╝ ███████╗███████╗╚██████╔╝╚██████╗██║   ██║      ██║      
  ╚═══╝  ╚══════╝╚══════╝ ╚═════╝  ╚═════╝╚═╝   ╚═╝      ╚═╝      
```

## Features

- 🚀 Lightning fast builds with Rust
- 📝 Markdown support with YAML frontmatter
- 🎨 Customizable templates using Tera
- 🔥 Hot reloading during development
- 📁 Static file handling
- 🏷️ Tag support for posts
- 🎯 Simple and intuitive CLI

## Installation

### From Source

1. Make sure you have Rust and Cargo installed
2. Clone the repository
3. Build and install:

```bash
cargo install terminal-velocity
```

## Quick Start

1. Create a new blog:
```bash
termv init hello-world
```

2. Create a new post:
```bash
termv new "My First Post"
```

3. Build the site:
```bash
termv build
```

4. Serve locally:
```bash
termv serve
```

## Project Structure

After initialization, your project will have the following structure:

```
my-blog/
├── posts/          # Your markdown posts go here
├── templates/      # Tera templates
│   ├── base.html
│   ├── index.html
│   └── post.html
├── static/         # Static assets (CSS, images, etc.)
├── components/     # Reusable template components
└── config.toml     # Site configuration
```

## Command Reference

### `init`

Initialize a new blog site:

```bash
termv init [path]
```

Options:
- `path`: Directory to create the new blog in (default: current directory)

### `new`

Create a new blog post:

```bash
termv new "Your Post Title"
```

This will create a new markdown file in the `posts` directory with the following format:
- Filename: `YYYY-MM-DD-your-post-title.md`
- Pre-populated frontmatter
- Slugified title for URLs

### `build`

Build your site:

```bash
termv build [options]
```

Options:
- `--target-dir, -t`: Source directory containing your site (default: current directory)
- `--output-path, -o`: Output directory for the built site (default: "dist")
- `--verbose, -v`: Show verbose output during build

### `serve`

Serve your site locally:

```bash
termv serve [options]
```

Options:
- `--target-dir, -t`: Directory containing the built site (default: "./dist")
- `--port`: Port to serve on (default: 8080)
- `--hot-reload`: Enable hot reloading on file changes

## Post Format

Posts should be written in Markdown with YAML frontmatter:

```markdown
---
title: "Your Post Title"
date: 2024-11-19
author: "Your Name"
tags: ["rust", "blog"]
preview: "A brief preview of your post"
slug: "your-post-slug"
---

Your post content here...
```

## Configuration

The `config.toml` file contains your site's configuration:

```toml
title = "Your Site Title"
description = "Your site description"
base_url = "https://your-site.com"

[author]
name = "Your Name"
email = "your@email.com"

[build]
output_dir = "dist"
port = 8080
```

## Development

### Requirements

- Rust 1.70+
- Cargo

### Building from Source

1. Clone the repository
2. Install dependencies and build:
```bash
cargo build
```

### Running Tests

```bash
cargo test
```


## Publishing

To publish a new version:

1. Update the version in `Cargo.toml`
2. Update CHANGELOG.md
3. Commit the changes
4. Create a new version tag:
   ```bash
   git tag -a v0.1.0 -m "Release version 0.1.0"
   ```
5. Push the tag:
   ```bash
   git push origin v0.1.0
   ```

The GitHub Action will automatically:
1. Verify the version matches the tag
2. Run all tests
3. Publish to crates.io
4. Create a GitHub release

### Publishing Manually

If you need to publish manually:

```bash
# Verify everything works
cargo test
cargo publish --dry-run

# Publish to crates.io
cargo publish
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
