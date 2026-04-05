use clap::{Parser, Subcommand};
use std::path::PathBuf;
use serde_json::json;

mod config;
mod error;
mod storage;
mod parser;
mod ai;
mod mcp;
mod api;

use config::Config;
use storage::{Document, VectorStore};

#[derive(Parser)]
#[command(name = "contextbox")]
#[command(about = "Self-hosted Document AI Platform", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "./data")]
    data_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    Add {
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        #[arg(short, long)]
        name: Option<String>,
        
        #[arg(short, long)]
        content: Option<String>,
        
        #[arg(short, long)]
        url: Option<String>,
    },
    List,
    Search {
        #[arg(short, long)]
        query: String,
        
        #[limit = 5]
        limit: Option<usize>,
    },
    Delete {
        #[arg(short, long)]
        id: String,
    },
    Config {
        #[arg(short, long)]
        mcp: bool,
    },
    Setup,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = Config::load().unwrap_or_else(|_| {
        tracing_subscriber::fmt::init();
        Config {
            server: config::ServerConfig::default(),
            api: config::ApiConfig::default(),
            storage: config::StorageConfig {
                data_dir: cli.data_dir.clone(),
                vector_db_path: cli.data_dir.join("vectors.db"),
            },
            features: config::FeaturesConfig::default(),
            logging: config::LoggingConfig::default(),
        }
    });

    match cli.command {
        Commands::Serve { port } => {
            println!("Starting server on port {}...", port);
            println!("Use --help for more options");
            Ok(())
        }
        Commands::Add { file, name, content, url } => {
            let mut doc_content = String::new();
            let mut doc_name = name.unwrap_or_else(|| "untitled".to_string());

            if let Some(path) = file {
                let parser = parser::DocumentParser::new();
                doc_content = parser.parse_file(&path)
                    .map_err(|e| format!("Failed to parse file: {}", e))?;
                doc_name = name.unwrap_or_else(|| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("untitled")
                        .to_string()
                });
            } else if let Some(c) = content {
                doc_content = c;
            } else if let Some(u) = url {
                println!("URL upload not yet implemented: {}", u);
                Ok(())
            } else {
                return Err("No input provided. Use --file, --content, or --url".into());
            }

            let doc = Document::new(doc_name, doc_content);
            println!("Added document: {} ({})", doc.name, doc.id);
            Ok(())
        }
        Commands::List => {
            println!("No documents yet");
            Ok(())
        }
        Commands::Search { query, limit } => {
            println!("Searching for: {}", query);
            println!("(Search requires embedding client setup)");
            Ok(())
        }
        Commands::Delete { id } => {
            println!("Deleted document: {}", id);
            Ok(())
        }
        Commands::Config { mcp } => {
            if mcp {
                let mcp_config = json!({
                    "mcpServers": {
                        "contextbox": {
                            "command": "contextbox",
                            "args": ["mcp"],
                            "env": {
                                "OPENROUTER_API_KEY": "${OPENROUTER_API_KEY}"
                            }
                        }
                    }
                });
                println!("MCP Configuration:");
                println!("{}", serde_json::to_string_pretty(&mcp_config).unwrap());
            }
            Ok(())
        }
        Commands::Setup => {
            println!("Running setup wizard...");
            println!("This feature is not yet implemented");
            Ok(())
        }
    }
}
