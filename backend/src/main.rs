mod db;
mod error;
mod models;
mod routes;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use axum::routing::{get, put};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::EnvFilter;

use crate::routes::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let db_path = db::db_path();
    let connection = db::open(&db_path)?;
    tracing::info!(path = %db_path.display(), "opened variable database");

    let state = AppState {
        db: Arc::new(Mutex::new(connection)),
    };

    let api = Router::new()
        .route("/health", get(routes::health))
        .route(
            "/variables",
            get(routes::list_variables).post(routes::create_variable),
        )
        .route(
            "/variables/{id}",
            put(routes::update_variable).delete(routes::delete_variable),
        )
        .route("/export", get(routes::export_variables))
        .with_state(state);

    let frontend_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend/dist");
    let spa = ServeDir::new(&frontend_dir)
        .not_found_service(ServeFile::new(frontend_dir.join("index.html")));

    let app = Router::new()
        .nest("/api", api)
        .fallback_service(spa)
        .layer(dev_cors());

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3001);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn dev_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://127.0.0.1:5173"),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE])
}
