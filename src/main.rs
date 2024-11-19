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
#[command(about = "A blazingly fast static site generator for tech blogs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new blog
    Init {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Create a new blog post
    New { title: String },
    /// Serve the site locally
    Serve {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Build the site
    Build {
        /// Path to the site directory
        #[arg(short, long)]
        path: Option<PathBuf>,

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
    println!("{}", accent.apply_to(BANNER));
    match cli.command {
        Commands::Init { path } => {
            create_directory_structure(&path.clone().unwrap_or_else(|| PathBuf::from(".")))?;
            println!("✨ Created new blog at {}", path.expect("REASON").display());
        }
        Commands::New { title } => {
            create_new_post(&title)?;
            println!("📝 Created new post: {}", title);
        }
        Commands::Build {
            path,
            output_path,
            verbose,
        } => {
            let site_dir = path.unwrap_or_else(|| PathBuf::from("."));
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

        Commands::Serve { path } => {
            serve(path)?;
        }
    }

    Ok(())
}
