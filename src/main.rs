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

// TODO - untangle the spaghetti that is target dir, dist dir, etc by referring to the config.toml instead

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
    },
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
        Commands::New {
            title,
            prompt,
            anthropic_key,
            dir,
        } => {
            let site_dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let config = Config::load(&site_dir)?;

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
            let config = Config::load(&site_dir)?;
            let absolute_site_dir = fs::canonicalize(&site_dir)?;

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

            let generator = SiteGenerator::new(&config)?;
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
            let site_dir = dir.unwrap_or_else(|| PathBuf::from("."));
            serve(site_dir, port, hot_reload)?;
        }
    }

    Ok(())
}
