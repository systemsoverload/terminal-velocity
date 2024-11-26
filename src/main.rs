use clap::Parser;
use clap::Subcommand;
use console::Style;

use std::path::PathBuf;

use terminal_velocity::config::{Config, ConfigOverrides};
use terminal_velocity::constants::BANNER;
use terminal_velocity::errors::Error;
use terminal_velocity::generator::SiteGenerator;
use terminal_velocity::git::open_editor;
use terminal_velocity::init::{create_directory_structure, validate_site_directory};
use terminal_velocity::post::create_new_post;
use terminal_velocity::serve::serve;

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

    match cli.command {
        Commands::Init { dir } => {
            println!("{}", ACCENT_STYLE.apply_to(BANNER));
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
            let config = Config::load(&site_dir)?.with_overrides(ConfigOverrides {
                author,
                ..Default::default()
            });

            let rt = tokio::runtime::Runtime::new()?;
            let filepath = rt.block_on(async {
                create_new_post(&config, &title, prompt, anthropic_key).await
            })?;

            open_editor(&filepath)?;
        }

        Commands::Build {
            dir,
            output_path,
            verbose,
        } => {
            let site_dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let config = Config::load(&site_dir)?.with_overrides(ConfigOverrides {
                output_dir: output_path,
                verbose,
                ..Default::default()
            });

            validate_site_directory(&config.site_dir)?;

            let generator = SiteGenerator::new(&config)?;
            generator.generate_site()?;

            println!(
            "{}",
            ACCENT_STYLE.apply_to(if config.build.verbose {
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
            let config = Config::load(&site_dir)?.with_overrides(ConfigOverrides {
                port,
                verbose,
                hot_reload,
                ..Default::default()
            });

            serve(config)?;
        }
    }

    Ok(())
}
