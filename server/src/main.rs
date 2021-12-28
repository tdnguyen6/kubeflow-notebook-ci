use actix_web::{web, HttpServer};
use sqlx::postgres::PgPoolOptions;

mod config;
mod models;
mod services;
mod utils;

#[actix_web::get("/api")]
async fn index() -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    Ok("good")
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let config = config::Config::from_env()?;
    let config2 = config.clone();

    let pool = PgPoolOptions::new()
        .max_connections(config.database_maxcon)
        .connect(config.database_url.as_str())
        .await?;
    let pool2 = pool.clone();

    sqlx::migrate!().run(&pool).await?;
    HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_cors::Cors::permissive())
            .app_data(web::Data::new(pool2.clone()))
            .app_data(web::Data::new(config2.clone()))
            .service(index)
            .service(services::repo::add_track_log)
            .service(services::repo::add_build_log)
            .service(services::repo::end_update)
            .service(services::repo::get_map)
            .service(services::repo::get_repo)
            .service(services::repo::get_track_log)
            .service(services::repo::get_build_log)
            .service(services::repo::get_image_digest)
            .service(services::repo::start_update)
            .service(services::repo::revert_update)
            .service(services::repo::updating)
            .service(services::repo::set_image_digest)
            .service(services::repo::should_track)
            .service(services::notebook::put)
            .service(services::notebook::remove)
            .service(services::notebook::restart_pod)
            .service(services::notebook::get)
            .service(services::notebook::reset_push_log)
            .service(services::notebook::add_push_log)
            .service(services::notebook::get_push_log)
            .service(services::frontend_data)
            .service(services::frontend)
            .service(services::stop_all_updates_endpoint)
            // .service(services::notebook::get_all)
            .service(services::clean::reconcile)
            .service(actix_files::Files::new("/", "../frontend/dist").index_file("index.html"))
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await?;

    services::stop_all_updates_and_syncing(&pool).await?;

    Ok(())
}
