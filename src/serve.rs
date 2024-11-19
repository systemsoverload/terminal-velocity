use actix_files as fs;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use std::path::PathBuf;

pub struct Server {
    dist_dir: PathBuf,
    port: u16,
}

impl Server {
    pub fn new(dist_dir: PathBuf, port: u16) -> Self {
        Self { dist_dir, port }
    }

    pub async fn run(self) -> std::io::Result<()> {
        println!("Starting server on http://localhost:{}", self.port);
        let dist_dir = self.dist_dir.clone();

        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .service(
                    fs::Files::new("/", dist_dir.clone())
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

pub fn serve(dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let dist_dir = dir.unwrap_or_else(|| PathBuf::from("dist"));
    if !dist_dir.exists() {
        return Err(format!(
            "Directory not found: {}. Run `term-v build` first.",
            dist_dir.display()
        )
        .into());
    }

    let server = Server::new(dist_dir, 8000);
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(server.run())?;
    Ok(())
}
