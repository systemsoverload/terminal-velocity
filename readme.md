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

## LLM Integration

Terminal Velocity includes integration with Claude, Anthropic's large language model, to help you get started with blog post writing. When creating a new post, you can provide a prompt to generate an initial outline.

### Using the LLM Features

To generate a blog post outline using Claude, use the `--prompt` flag with the `new` command:

```bash
# Create a new post with AI-generated outline
termv new "My Post Title" --prompt "Write about the history and impact of the Rust programming language"

# You can also set your API key via environment variable
export ANTHROPIC_API_KEY=your_key_here
termv new "My Post Title" --prompt "Explain WebAssembly and its use cases"
```

The `--prompt` flag requires an Anthropic API key, which you can provide in two ways:
1. Set the `ANTHROPIC_API_KEY` environment variable
2. Pass it directly using the `--anthropic-key` flag

### Example

```bash
# Create a new post about distributed systems
termv new "Understanding Distributed Systems" --prompt "Explain key concepts in distributed systems including consensus, replication, and fault tolerance"
```

This will create a new post with:
- Standard frontmatter (title, date, etc.)
- An AI-generated outline based on your prompt
- A placeholder for your content

The file will automatically open in your configured editor, where you can begin writing using the generated outline as a guide.

### Tips for Good Prompts

For best results with the outline generation:
- Be specific about the topics you want to cover
- Mention your target audience if relevant
- Include any specific aspects or angles you want to explore
- Note if you want a particular style (technical, beginner-friendly, etc.)

Example prompt: "Write an outline for a technical blog post explaining WebAssembly to experienced JavaScript developers, focusing on real-world use cases and performance benefits"

## Configuration

The `config.toml` file contains your site's configuration:

```toml
title = "My Terminal Velocity Blog"
description = "A blazingly fast tech blog"
base_url = "http://localhost:8000"

[author]
name = "Anonymous"
email = "author@example.com"

[build]
port = 8000
verbose = true
# Relative to the site directory
output_dir = "dist"
posts_dir = "posts"
templates_dir = "templates"
static_dir = "static"
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
