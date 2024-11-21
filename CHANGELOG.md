# Changelog

All notable changes to Terminal Velocity will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).'
## [0.1.6] - 2024-11-20

### Added
   - Per-post asset collection/hosting
   - Update example post to demonstrate

## [0.1.4] - 2024-11-20

### Added
   - Server side syntax highlighting

## [0.1.4] - 2024-11-20

### Added
   - Claude integration
### Changed
   - Vastly simplified path management by relying solely on the config.toml data

## [0.1.2] - 2024-11-20

### Added
- Static file handling with proper directory structure preservation
- Hot reloading support for local development
- Progress indicators during site generation
- Default theme with terminal-inspired styling
- Automatic slug generation for posts
- Tag support for blog posts
- Preview text support in post metadata
- GitHub Actions for CI/CD and publishing
- Comprehensive test suite

### Changed
- Improved post date handling with better error messages
- More robust frontmatter parsing
- Better error handling for file operations
- Simplified initialization command (`termv init [path]`)

### Fixed
- Port validation for development server
- URL path normalization to prevent double slashes
- Empty slug generation edge cases
- Static file copying to respect output directory configuration
- Date parsing validation in frontmatter
- Added port range validation (1024-65535)

## [0.1.1] - 2024-11-20

### Added
- Basic static site generation
- Markdown support with frontmatter
- Template system using Tera
- Local development server
- CLI interface with basic commands
- Post creation wizard

## [0.1.0] - 2024-11-20

### Added
- Initial release
- Basic project structure
- Core CLI framework
