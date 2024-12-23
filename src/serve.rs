use crate::generator::SiteGenerator;
use actix_files::Files;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use notify::{RecursiveMode, Watcher};

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::errors::Error;

pub struct Server {
    config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(self) -> std::io::Result<()> {
        println!(
            "Starting server on http://localhost:{}",
            self.config.server.port
        );

        if self.config.server.hot_reload {
            let (tx, rx) = mpsc::channel();
            let config_clone = self.config.clone();

            let mut watcher = notify::recommended_watcher(move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            })
            .unwrap();

            let watch_paths = [
                config_clone.posts_dir(),
                config_clone.templates_dir(),
                config_clone.static_dir(),
            ];

            for path in watch_paths.iter() {
                if path.exists() && watcher.watch(path, RecursiveMode::Recursive).is_ok() {
                    println!("Watching directory: {}", path.display());
                }
            }

            let config_for_rebuild = self.config.clone();
            let _watcher_handler = std::thread::spawn(move || {
                let mut last_build = Instant::now();
                let debounce_duration = Duration::from_millis(500);
                let _watcher = watcher;

                while let Ok(_event) = rx.recv() {
                    println!("Change detected...");

                    if last_build.elapsed() >= debounce_duration {
                        println!("🔄 Rebuilding site...");

                        // Use internal generator instead of spawning process
                        match rebuild_site(&config_for_rebuild) {
                            Ok(_) => {
                                println!("✨ Site rebuilt successfully!");
                                last_build = Instant::now();
                            }
                            Err(e) => eprintln!("Build failed: {}", e),
                        }
                    }
                }
            });
        }

        let output_dir = self.config.output_dir();

        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .service(
                    Files::new("/", output_dir.clone())
                        .index_file("index.html")
                        .use_last_modified(true)
                        .use_etag(true),
                )
                .default_service(
                    web::get()
                        .to(|| async { HttpResponse::NotFound().body("404 - Page not found") }),
                )
        })
        .bind(("127.0.0.1", self.config.server.port))?
        .run()
        .await
    }
}

fn rebuild_site(config: &Config) -> Result<(), Error> {
    let generator = SiteGenerator::new(config)?;
    generator.generate_site()
}

pub fn serve(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = config.output_dir();

    if !output_dir.exists() {
        return Err(format!(
            "Output directory not found: {}. Run `termv build` first.",
            output_dir.display()
        )
        .into());
    }

    let server = Server::new(config);
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(server.run())?;
    Ok(())
}
