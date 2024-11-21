---
title: "Welcome to Terminal Velocity"
date: 2024-01-01
author: "Terminal Velocity"
tags: ["welcome", "getting-started"]
preview: "Get started with your new Terminal Velocity blog"
slug: "example"
---

Terminal Velocity (termv) is a blazingly fast static site generator built in Rust. It supports markdown posts, per-post assets, live reload, and AI-assisted content generation.

![Terminal Velocity Demo](./assets/terminal_velocity.png)

## Quick Start

Create a new blog:
```bash
termv init my-blog
cd my-blog
```

Create your first post:
```bash
termv new "Hello World"
```

Build the site:
```bash
termv build
```

Serve locally with hot reload:
```bash
termv serve --hot-reload
```

## Project Structure

After running `init`, your blog will have this structure:
```
my-blog/
├── config.toml          # Site configuration
├── posts/              # Blog posts directory
│   └── example/        # Each post has its own directory
│       ├── post.md     # Post content
│       └── assets/     # Post-specific images/files
├── static/             # Global static assets
│   └── css/            # Stylesheets
├── templates/          # HTML templates
└── components/         # Reusable components
```

## Configuration

The `config.toml` file controls your site's settings:

```toml
title = "My Blog"
description = "A blog about interesting things"
base_url = "https://example.com"

[author]
name = "Your Name"
email = "you@example.com"

[build]
port = 8080
verbose = false
output_dir = "dist"
posts_dir = "posts"
templates_dir = "templates"
static_dir = "static"
```

## Writing Posts

Posts are written in Markdown with YAML front matter:

```
---
title: "My First Post"
date: 2024-03-20
author: "Your Name"
tags: ["rust", "blogging"]
preview: "A short preview of the post"
slug: "my-first-post"
---

# Post Content Here

Regular markdown content...
```

### Using Post Assets

Each post can have its own assets:

1. Place files in your post's `assets/` directory:
   ```
   posts/
     my-first-post/
       post.md
       assets/
         terminal_velocity.png
   ```

2. Reference them in your markdown:
   ```markdown
   ![My Diagram](./assets/terminal_velocity.png)
   ```

## CLI Commands

| Command | Description | Example |
|---------|-------------|---------|
| `init` | Create new blog | `termv init my-blog` |
| `new` | Create new post | `termv new "Post Title"` |
| `build` | Generate site | `termv build` |
| `serve` | Start dev server | `termv serve --hot-reload` |

### AI-Assisted Post Generation

Generate post outlines using Claude:

```bash
export ANTHROPIC_API_KEY="your-key-here"
termv new "My Post" --prompt "Write about Rust web frameworks"
```

## Development Server

Run the development server with hot reload:

```bash
termv serve --hot-reload --port 8080
```

This will:
- Serve your site at `http://localhost:8080`
- Watch for changes in posts and templates
- Automatically rebuild when changes are detected

## Building for Production

Build your site for production:

```bash
termv build --verbose
```

The generated site will be in the `dist/` directory (or wherever `output_dir` is set in your config).

## Contributing

Contributions are welcome! Check out our contribution guidelines and feel free to:

- Report bugs
- Request features
- Submit pull requests

## License

Terminal Velocity is distributed under the MIT license. See LICENSE for details.
