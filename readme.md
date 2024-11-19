# Terminal Velocity 🚀

[![Crates.io](https://img.shields.io/crates/v/terminal-velocity.svg)](https://crates.io/crates/terminal-velocity)
[![Downloads](https://img.shields.io/crates/d/terminal-velocity.svg)](https://crates.io/crates/terminal-velocity)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> Because your blog should load faster than a `cd` command.

Terminal Velocity is a blazingly fast static site generator that turns your markdown files into a sleek, retro-terminal styled tech blog. Perfect for developers who think RGB keyboards aren't enough.

## Features ⚡

- 🖥️ Retro terminal aesthetic that would make GNU proud
- ⌨️ Markdown-driven because WYSIWYG is for mortals
- 🚄 Blazingly fast™ because it's written in Rust
- 🎨 Built-in cyberpunk theme that would make Gibson proud
- 🏷️ Tag support that puts your Gmail filters to shame
- 📱 Responsive design because even hackers use phones
- 🔧 Zero config because ain't nobody got time for that

## Installation 💾

```bash
cargo install terminal-velocity
```

Or clone and build from source if you're one of those people:

```bash
git clone https://github.com/yourusername/terminal-velocity.git
cd terminal-velocity
cargo build --release
```

## Usage 🔧

### Initialize a new blog

```bash
termv init my-cyber-blog
```

### Create a new post

```bash
termv new "Why Mechanical Keyboards Are Actually Time Machines"
```

### Build your site

```bash
termv build
```

### Deploy to the mainframe (or just serve locally)

```bash
termv serve
```

## File Structure 📁

```
your-blog/
├── _posts/
│   └── 2024-11-18-why-vim-is-better.md
├── _templates/
│   └── they-work-fine-out-of-the-box.html
└── _config.yml (optional, we're not Jekyll)
```

## Post Format 📝

```markdown
---
title: "Why I Rebuilt Git in Rust (and Why You Shouldn't)"
date: 2024-11-18
tags: ["rust", "git", "bad-ideas", "over-engineering"]
---

Here's why I spent 6 months rebuilding Git in Rust...
```

## Performance 📊

| Generator          | Build Time | Coolness Factor |
|-------------------|------------|-----------------|
| Terminal Velocity | 0.3s       | Over 9000      |
| Jekyll            | 3.2s       | Meh            |
| Hugo             | 0.8s       | Pretty good    |
| Writing by hand  | ∞          | Maximum        |

## FAQ 🤔

**Q: Why another static site generator?**
A: Because the world needed a static site generator that makes your blog look like you're hacking the mainframe.

**Q: Is it production ready?**
A: If you have to ask, you're not ready for the aesthetic.

**Q: Why Rust?**
A: Have you tried telling people you wrote something in Rust? It's better than CrossFit.

## Contributing 🤝

1. Fork it
2. Create your feature branch
3. Make it more blazingly fast
4. Push to the branch
5. Create a Pull Request

## License 📜

MIT License - Because even hackers need lawyers.

## Acknowledgments 🙏

- The Rust community, for making "blazingly fast" a personality trait
- The 1980s, for the aesthetic
- Coffee, for obvious reasons

---

Made with ⚡ by developers who type really fast on mechanical keyboards
