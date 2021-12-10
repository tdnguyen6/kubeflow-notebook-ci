mod config;
mod services;
mod models;

use actix_files::Files;
use actix_web::{HttpServer, App, Responder, get};
use dotenv::dotenv;

#[get("/api")]
async fn index() -> actix_web::Result<impl Responder, Box<dyn std::error::Error>> {
    Ok("good")
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let config = config::Config::from_env()?;

    Ok(HttpServer::new(move || {
        App::new()
            .service(index)
            .service(Files::new("/", "frontend/dist").index_file("index.html"))
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await?)
}
