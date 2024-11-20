#![deny(unused_crate_dependencies)]

mod config;
mod errors;
mod generator;
mod init;
mod post;
mod serve;

use clap::Parser;
use clap::Subcommand;
use console::Style;
use std::fs;
use std::path::PathBuf;

use crate::config::{BuildConfig, Config};
use crate::errors::Error;
use crate::generator::SiteGenerator;
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

// TODO - untangle the spaghetti that is target dir, dist dir, etc by referring to the config.toml instead

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new blog
    Init {
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    /// Create a new blog post
    New { title: String },
    /// Serve the site locally
    Serve {
        #[arg(short, long = "target-dir", default_value = ".")]
        dir: Option<PathBuf>,

        #[arg(long, default_value_t = 8080)]
        port: u16,

        #[arg(long)]
        hot_reload: bool,
    },
    /// Build the site
    Build {
        /// Path to the site directory
        #[arg(short, long = "target-dir", default_value = ".")]
        dir: Option<PathBuf>,

        /// Output directory for the generated site
        #[arg(short, long, default_value = "dist")]
        output_path: PathBuf,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
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
        Commands::New { title } => {
            create_new_post(&title)?;
            println!("📝 Created new post: {}", title);
        }
        Commands::Build {
            dir,
            output_path,
            verbose,
        } => {
            let site_dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let absolute_site_dir = fs::canonicalize(&site_dir)?;
            let config = Config::load(&site_dir)?;

            // Override output directory if specified
            let config = if output_path != PathBuf::from("dist") {
                Config {
                    build: BuildConfig {
                        output_dir: output_path.to_string_lossy().to_string(),
                        ..config.build
                    },
                    ..config
                }
            } else {
                config
            };

            validate_site_directory(&absolute_site_dir)?;

            let posts_dir = absolute_site_dir.join("posts");
            let templates_dir = absolute_site_dir.join("templates");
            let output_dir = output_path;

            if verbose {
                println!("Site directory: {}", absolute_site_dir.display());
                println!("Posts directory: {}", posts_dir.display());
                println!("Templates directory: {}", templates_dir.display());
                println!("Output directory: {}", output_dir.display());
            }

            println!("{}", accent.apply_to("\nGenerating site..."));

            let generator = SiteGenerator::new(&site_dir, Some(config))?;
            generator.generate_site()?;

            if verbose {
                println!("\nSite generation complete!");
                println!("Output directory: {}", output_dir.display());
                println!("You can serve the site locally with:");
                println!("  termv serv --directory {}", output_dir.display());
            } else {
                println!("{}", accent.apply_to("\nSite generated successfully! 🚀"));
            }
        }

        Commands::Serve {
            dir,
            port,
            hot_reload,
        } => {
            let dist_dir = dir.unwrap_or_else(|| PathBuf::from("./dist"));
            serve(dist_dir, port, hot_reload)?;
        }
    }

    Ok(())
}
