use std::time::Duration;
use std::time::Instant;
use std::process::Command;

use actix_files::Files;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use notify::{RecursiveMode, Watcher};
use std::sync::mpsc;

use std::path::PathBuf;

pub struct Server {
    dist_dir: PathBuf,
    hot_reload: bool,
    port: u16,
}

impl Server {
    pub fn new(dist_dir: PathBuf, port: u16, hot_reload: bool) -> Self {
        Self {
            dist_dir,
            hot_reload,
            port,
        }
    }

    pub async fn run(self) -> std::io::Result<()> {
        println!("Starting server on http://localhost:{}", self.port);
        let dist_dir = self.dist_dir.clone();
        // TODO - clean this up
        let dist_dir_clone = self.dist_dir.clone();

        if self.hot_reload {
            let (tx, rx) = mpsc::channel();

            let mut watcher = notify::recommended_watcher(move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            }).unwrap();

            // Watch relevant directories
            for dir in ["posts", "templates", "static"].iter() {
                let path = self.dist_dir.join(dir);
                if path.exists() && watcher.watch(&path, RecursiveMode::Recursive).is_ok() {
                    println!("Watching directory: {}", path.display());
                }
            }

            let _watcher_handler = std::thread::spawn(move || {
                let mut last_build = Instant::now();
                let debounce_duration = Duration::from_millis(500);
                let _watcher = watcher; // Keep watcher alive in this thread

                while let Ok(event) = rx.recv() {
                    println!("Change detected: {:?}", event);

                    // Debounce builds
                    if last_build.elapsed() >= debounce_duration {
                        println!("🔄 Rebuilding site...");
                        match Command::new("termv")
                            .arg("build")
                            .arg("--path")
                            .arg(&dist_dir_clone)
                            .status()
                        {
                            Ok(status) if status.success() => {
                                println!("✨ Site rebuilt successfully!");
                                last_build = Instant::now();
                            },
                            Ok(status) => eprintln!("Build failed with status: {}", status),
                            Err(e) => eprintln!("Failed to execute build: {}", e),
                        }
                    }
                }
            });
        }

        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .service(
                    Files::new("/", dist_dir.clone())
                        .index_file("index.html")
                        .use_last_modified(true)
                        .use_etag(true),
                )
                .default_service(
                    web::get()
                        .to(|| async { HttpResponse::NotFound().body("404 - Page not found") }),
                )
        })
        .bind(("127.0.0.1", self.port))?
        .run()
        .await
    }
}

pub fn serve(dist_dir: PathBuf, port: u16, hot_reload: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !dist_dir.exists() {
        return Err(format!(
            "Directory not found: {}. Run `term-v build` first.",
            dist_dir.display()
        )
        .into());
    }

    let server = Server::new(dist_dir, port, hot_reload);
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(server.run())?;
    Ok(())
}
