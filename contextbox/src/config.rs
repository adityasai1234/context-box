use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Missing required configuration: {0}")]
    Missing(String),
    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub api: ApiConfig,
    pub storage: StorageConfig,
    pub features: FeaturesConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub key: Option<String>,
    pub cors_origins: Vec<String>,
    pub openrouter_api_key: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            key: None,
            cors_origins: vec!["http://localhost:3000".to_string()],
            openrouter_api_key: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub vector_db_path: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            vector_db_path: PathBuf::from("./data/vectors.db"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub enable_mcp: bool,
    pub enable_web_ui: bool,
    pub enable_cli: bool,
    pub enable_chat: bool,
    pub enable_url_crawl: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            enable_mcp: true,
            enable_web_ui: false,
            enable_cli: true,
            enable_chat: false,
            enable_url_crawl: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();

        let server = ServerConfig {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
        };

        let api = ApiConfig {
            key: std::env::var("API_KEY").ok(),
            cors_origins: std::env::var("CORS_ORIGINS")
                .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                .unwrap_or_else(|_| vec!["http://localhost:3000".to_string()]),
            openrouter_api_key: std::env::var("OPENROUTER_API_KEY").ok(),
        };

        let data_dir = std::env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data"));
        
        let storage = StorageConfig {
            data_dir: data_dir.clone(),
            vector_db_path: data_dir.join("vectors.db"),
        };

        let features = FeaturesConfig {
            enable_mcp: std::env::var("ENABLE_MCP")
                .map(|v| v == "true")
                .unwrap_or(true),
            enable_web_ui: std::env::var("ENABLE_WEB_UI")
                .map(|v| v == "true")
                .unwrap_or(false),
            enable_cli: std::env::var("ENABLE_CLI")
                .map(|v| v == "true")
                .unwrap_or(true),
            enable_chat: std::env::var("ENABLE_CHAT")
                .map(|v| v == "true")
                .unwrap_or(false),
            enable_url_crawl: std::env::var("ENABLE_URL_CRAWL")
                .map(|v| v == "true")
                .unwrap_or(false),
        };

        let logging = LoggingConfig {
            level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        };

        Ok(Self {
            server,
            api,
            storage,
            features,
            logging,
        })
    }

    pub fn load() -> Result<Self, ConfigError> {
        Self::from_env()
    }
}
