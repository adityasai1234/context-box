use std::net::SocketAddr;
use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod storage;
mod parser;
mod ai;
mod mcp;
mod api;

use api::{create_router, AppState};
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .init();

    tracing::info!("Starting ContextBox...");

    let config = Config::load()
        .map_err(|e| format!("Failed to load config: {}", e))?;

    tracing::info!("Loaded configuration");
    tracing::info!("Features: MCP={}, WebUI={}, CLI={}, Chat={}, URLCrawl={}",
        config.features.enable_mcp,
        config.features.enable_web_ui,
        config.features.enable_cli,
        config.features.enable_chat,
        config.features.enable_url_crawl,
    );

    std::fs::create_dir_all(&config.storage.data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let state = AppState::new(config);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app: Router = create_router(state)
        .layer(cors);

    let addr = SocketAddr::from((
        [127, 0, 0, 1],
        8080,
    ));

    tracing::info!("ContextBox listening on http://{}", addr);
    tracing::info!("Health check: http://{}/health", addr);
    tracing::info!("API docs: http://{}/api/documents", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
