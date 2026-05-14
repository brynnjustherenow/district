mod db;
mod handlers;
mod admin_html;
mod auth;

use axum::Router;
use axum::routing::{get, post, put, delete};
use axum::extract::DefaultBodyLimit;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub type AppState = Arc<Mutex<rusqlite::Connection>>;

const API_KEY: &str = "jjjshop-district-2026";

#[tokio::main]
async fn main() {
    let conn = db::init_db().expect("Failed to init database");
    let state: AppState = Arc::new(Mutex::new(conn));

    let public_routes = Router::new()
        .route("/districts", get(handlers::list_districts))
        .route("/districts/{city}/{area}", get(handlers::get_district))
        .with_state(state.clone());

    let protected_routes = Router::new()
        .route("/districts", post(handlers::create_district))
        .route("/districts/{city}/{area}", put(handlers::update_district))
        .route("/districts/{city}/{area}", delete(handlers::delete_district))
        .route("/districts/{city}/{area}/image", post(handlers::upload_image))
        .route("/init-hebei", post(handlers::init_hebei_data))
        .route_layer(axum::middleware::from_fn(auth::require_api_key))
        .with_state(state.clone());

    let app = Router::new()
        .route("/", get(admin_html::admin_page))
        .nest("/api", public_routes.merge(protected_routes))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("District service running on http://localhost:3000");
    println!("Admin page: http://localhost:3000/");
    println!("API: http://localhost:3000/api/districts");
    println!("API Key: {}", API_KEY);
    axum::serve(listener, app).await.unwrap();
}
