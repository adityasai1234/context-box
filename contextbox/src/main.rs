use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::net::SocketAddr;
use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod storage;
mod parser;
mod ai;
mod mcp;
mod api;
mod crypto;

use api::AppState;

#[derive(Parser)]
#[command(name = "context-box")]
#[command(about = "ContextBox - Self-hosted Document AI Platform")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, env = "DATA_DIR", default_value = "./data")]
    data_dir: PathBuf,
    
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        
        #[arg(long)]
        api_key: Option<String>,
        
        #[arg(long)]
        cors: Option<String>,
    },
    Version,
}

fn get_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("contextbox"))
        .unwrap_or_else(|| PathBuf::from("./data"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    let data_dir = if cli.data_dir.as_os_str().is_empty() {
        get_data_dir()
    } else {
        cli.data_dir
    };
    
    match cli.command {
        Commands::Version => {
            println!("ContextBox v{}", env!("CARGO_PKG_VERSION"));
            println!("License: MIT");
            println!("Self-hosted Document AI Platform");
        }
        
        Commands::Serve { port, host, api_key, cors } => {
            let log_level = cli.log_level.clone();
            
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::new(log_level))
                .init();
            
            println!("Starting ContextBox server...");
            println!("Data directory: {:?}", data_dir);
            println!("");
            
            std::fs::create_dir_all(&data_dir)?;
            
            let cfg = config::Config {
                server: config::ServerConfig {
                    host: host.clone(),
                    port,
                },
                api: config::ApiConfig {
                    key: api_key,
                    cors_origins: cors
                        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                        .unwrap_or_else(|| vec!["http://localhost:3000".to_string()]),
                    openrouter_api_key: std::env::var("OPENROUTER_API_KEY").ok(),
                },
                storage: config::StorageConfig {
                    data_dir: data_dir.clone(),
                    vector_db_path: data_dir.join("vectors.db"),
                },
                features: config::FeaturesConfig::default(),
                logging: config::LoggingConfig {
                    level: cli.log_level,
                },
            };
            
            let state = AppState::new(cfg);
            
            let cors_origins = state.config.api.cors_origins.clone();
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            
            let app: Router = api::create_router(state)
                .layer(cors)
                .layer(TraceLayer::new_for_http());
            
            let addr: SocketAddr = if host == "0.0.0.0" {
                SocketAddr::from(([0, 0, 0, 0], port))
            } else {
                let parts: Vec<&str> = host.split('.').collect();
                if parts.len() == 4 {
                    let ip: [u8; 4] = (
                        parts[0].parse().unwrap_or(127),
                        parts[1].parse().unwrap_or(0),
                        parts[2].parse().unwrap_or(0),
                        parts[3].parse().unwrap_or(1),
                    );
                    SocketAddr::from((ip, port))
                } else {
                    SocketAddr::from(([127, 0, 0, 1], port))
                }
            };
            
            println!("Server starting on http://{}", addr);
            println!("");
            println!("Endpoints:");
            println!("  Health:     http://{}/health", addr);
            println!("  Documents:  http://{}/api/documents", addr);
            println!("  Search:     http://{}/api/search?query=...", addr);
            println!("  Chat:       http://{}/api/chat", addr);
            println!("");
            
            if state.config.api.key.is_some() {
                println!("API Key: Enabled (set via --api-key or API_KEY env)");
            } else {
                println!("API Key: Not set (WARNING: Anyone can access)");
            }
            
            println!("");
            println!("CORS origins: {:?}", cors_origins);
            println!("");
            println!("To add documents via CLI:");
            println!("  cb add --file document.md");
            println!("");
            
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
    }
    
    Ok(())
}
