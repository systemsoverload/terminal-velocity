#![deny(unused_crate_dependencies)]

mod anthropic;
mod config;
mod errors;
mod generator;
mod git;
mod init;
mod markdown;
mod post;
mod serve;

use clap::Parser;
use clap::Subcommand;
use console::Style;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::errors::Error;
use crate::generator::SiteGenerator;
use crate::git::open_editor;
use crate::init::{create_directory_structure, validate_site_directory};
use crate::post::create_new_post;
use crate::serve::serve;

const BANNER: &str = r#"
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
"#;

#[derive(Parser)]
#[command(name = "termv")]
#[command(about = "A blazingly fast static site generator for dorks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new blog
    Init {
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    /// Create a new blog post
    New {
        title: String,

        #[arg(short, long = "target-dir", default_value = ".")]
        dir: Option<PathBuf>,

        #[arg(long, requires = "anthropic_key")]
        prompt: Option<String>,

        #[arg(long, env = "ANTHROPIC_API_KEY")]
        anthropic_key: Option<String>,

        #[arg(short, long)]
        author: Option<String>,
    },
    /// Serve the site locally
    Serve {
        #[arg(short, long = "target-dir", default_value = ".")]
        dir: Option<PathBuf>,

        #[arg(long)]
        port: Option<u16>,

        #[arg(long)]
        hot_reload: Option<bool>,

        #[arg(short, long)]
        verbose: Option<bool>,
    },
    /// Build the site
    Build {
        #[arg(short, long = "target-dir", default_value = ".")]
        dir: Option<PathBuf>,

        #[arg(short, long)]
        output_path: Option<PathBuf>,

        #[arg(short, long)]
        verbose: Option<bool>,
    },
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let accent = Style::new().cyan();

    match cli.command {
        Commands::Init { dir } => {
            println!("{}", accent.apply_to(BANNER));

            create_directory_structure(&dir)?;
            println!("✨ Created new site at {}", &dir.display());
        }
        Commands::New {
            title,
            prompt,
            anthropic_key,
            dir,
            author,
        } => {
            let site_dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let mut config = Config::load(&site_dir)?;
            config.site_dir = fs::canonicalize(&site_dir)?;

            // Override author if provided
            if let Some(cli_author) = author {
                config.author.name = cli_author;
            }

            let rt = tokio::runtime::Runtime::new()?;
            let filepath = rt.block_on(async {
                create_new_post(&config, &title, prompt, anthropic_key).await
            })?;
            println!("📝 Created new post: {}", title);

            open_editor(&filepath)?;
        }
        Commands::Build {
            dir,
            output_path,
            verbose,
        } => {
            let site_dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let mut config = Config::load(&site_dir)?;
            config.site_dir = fs::canonicalize(&site_dir)?;

            // Override config with CLI args if provided
            if let Some(path) = output_path {
                // If the path is absolute, use it as-is
                // If relative, make it relative to the current working directory, not the site dir
                let output_dir = if path.is_absolute() {
                    path
                } else {
                    std::env::current_dir()?.join(path)
                };
                config.set_output_dir(output_dir);
            }

            if let Some(v) = verbose {
                config.build.verbose = v;
            }

            validate_site_directory(&config.site_dir)?;

            if config.build.verbose {
                println!("Site directory: {}", config.site_dir.display());
                println!("Posts directory: {}", config.posts_dir().display());
                println!("Templates directory: {}", config.templates_dir().display());
                println!("Output directory: {}", config.output_dir().display());
            }

            println!("{}", accent.apply_to("\nGenerating site..."));

            let generator = SiteGenerator::new(&config)?;
            generator.generate_site()?;

            println!(
            "{}",
            accent.apply_to(if config.build.verbose {
                format!(
                    "\nSite generation complete!\nOutput directory: {}\nYou can serve the site locally with:\n  termv serve --directory {}",
                    config.output_dir().display(),
                    config.output_dir().display()
                )
            } else {
                "\nSite generated successfully! 🚀".to_string()
            })
        );
        }
        Commands::Serve {
            dir,
            port,
            hot_reload,
            verbose,
        } => {
            let site_dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let mut config = Config::load(&site_dir)?;
            config.site_dir = fs::canonicalize(&site_dir)?;

            // Override config with CLI args if provided
            if let Some(p) = port {
                config.build.port = p;
            }
            if let Some(v) = verbose {
                config.build.verbose = v;
            }
            if let Some(h) = hot_reload {
                config.build.hot_reload = h;
            }

            serve(config)?;
        }
    }

    Ok(())
}
