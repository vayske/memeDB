mod db;
mod handlers;
mod models;
mod services;
use crate::db::init_db;
use std::env;
use axum::{
    Router,
    routing::{get, post, delete},
};
use handlers::{
    search::search_images,
    tags::{add_tags, get_image_tags, list_tags},
    upload::upload_image,
    delete::delete_image
};
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use dotenvy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Logger
    tracing_subscriber::fmt::init();

    // Initialize Database
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&db_url).await?;
    println!("Connected to Database");

    println!("Initializing Database...");
    init_db(&pool).await?;
    println!("Database Ready");

    // Configure Router
    let app = Router::new()
        .route("/", get(|| async { "MemeDB is running!" }))
        .route("/api/upload", post(upload_image))
        .route("/api/search", get(search_images))
        .route("/api/images/{id}/tags", post(add_tags).get(get_image_tags))
        .route("/api/tags", get(list_tags))
        .route("/api/images/{id}", delete(delete_image))
        .nest_service("/images", ServeDir::new("storage"))
        .with_state(pool);

    // Launch Service
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
