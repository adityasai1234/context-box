use clap::{Parser, Subcommand};
use std::path::PathBuf;
use serde_json::json;
use std::fs;
use contextbox::storage::SqliteStorage;
use contextbox::crypto::{self, get_key_path, ensure_key, load_key};

#[derive(Parser)]
#[command(name = "contextbox")]
#[command(about = "Self-hosted Document AI Platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, default_value = "./data")]
    data_dir: PathBuf,
    
    #[arg(short, long, env = "OPENROUTER_API_KEY")]
    openrouter_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
    },
    Keygen,
    Key {
        #[command(subcommand)]
        subcommand: KeyCommands,
    },
    Add {
        #[arg(short, long)]
        file: Option<PathBuf>,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        content: Option<String>,
    },
    List,
    Delete {
        id: String,
    },
    Get {
        id: String,
    },
    Search {
        query: String,
    },
    Config {
        #[command(subcommand)]
        config_cmd: ConfigCommands,
    },
    Setup,
}

#[derive(Subcommand)]
enum KeyCommands {
    Show,
    Path,
}

#[derive(Subcommand)]
enum ConfigCommands {
    Mcp,
    Cli,
}

fn get_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("contextbox"))
        .unwrap_or_else(|| PathBuf::from("./data"))
}

fn ensure_data_dir(data_dir: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(data_dir)?;
    Ok(())
}

fn get_db_path(data_dir: &PathBuf) -> PathBuf {
    data_dir.join("documents.db")
}

fn open_storage(data_dir: &PathBuf) -> Result<SqliteStorage, String> {
    let key = load_key().map_err(|e| e.to_string())?;
    let db_path = get_db_path(data_dir);
    SqliteStorage::new(&db_path, &key).map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    let data_dir = if cli.data_dir.as_os_str().is_empty() {
        get_data_dir()
    } else {
        cli.data_dir
    };
    
    ensure_data_dir(&data_dir)?;
    
    match cli.command {
        Commands::Serve { port, host } => {
            println!("Starting ContextBox server on {}:{}...", host, port);
            println!("This will run the web API server");
            println!("Access via: http://{}:{}", host, port);
        }
        
        Commands::Keygen => {
            let (key, is_new) = ensure_key().map_err(|e| e.to_string())?;
            
            if is_new {
                println!("New encryption key generated");
            } else {
                println!("Encryption key already exists");
            }
            
            let key_path = get_key_path();
            println!("Key saved to: {:?}", key_path);
            println!("");
            println!("WARNING: Keep this key safe!");
            println!("- Anyone with this key can read your documents");
            println!("- If you lose the key, your documents cannot be recovered");
            println!("- Set permissions: chmod 600 {}", key_path.display());
        }
        
        Commands::Key { subcommand } => {
            match subcommand {
                KeyCommands::Show => {
                    let key_path = get_key_path();
                    if key_path.exists() {
                        let content = fs::read_to_string(&key_path)?;
                        println!("{}", content.trim());
                    } else {
                        println!("No key found. Run 'cb keygen' first.");
                    }
                }
                KeyCommands::Path => {
                    println!("{}", get_key_path().display());
                }
            }
        }
        
        Commands::Add { file, name, content } => {
            let doc_content = if let Some(f) = file {
                match fs::read_to_string(&f) {
                    Ok(text) => text,
                    Err(e) => {
                        eprintln!("Error reading file: {}", e);
                        return Ok(());
                    }
                }
            } else if let Some(c) = content {
                c
            } else {
                eprintln!("No input provided. Use --file or --content");
                return Ok(());
            };
            
            let doc_name = name.unwrap_or_else(|| {
                file.as_ref()
                    .and_then(|f| f.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("Untitled")
                    .to_string()
            });
            
            let storage = open_storage(&data_dir)?;
            let id = storage.add(doc_name.clone(), doc_content.clone(), "cli")
                .map_err(|e| e.to_string())?;
            
            println!("Document '{}' added (encrypted)", doc_name);
            println!("  ID: {}", id);
            println!("  Content length: {} chars", doc_content.len());
        }
        
        Commands::List => {
            let storage = open_storage(&data_dir)?;
            let docs = storage.list().map_err(|e| e.to_string())?;
            
            if docs.is_empty() {
                println!("No documents found");
            } else {
                println!("Documents ({} total):", docs.len());
                for doc in docs {
                    println!("  [{}] {}", doc.id, doc.name);
                }
            }
        }
        
        Commands::Delete { id } => {
            let storage = open_storage(&data_dir)?;
            
            if storage.delete(&id).map_err(|e| e.to_string())? {
                println!("Document deleted: {}", id);
            } else {
                println!("Document not found: {}", id);
            }
        }
        
        Commands::Get { id } => {
            let storage = open_storage(&data_dir)?;
            
            match storage.get(&id).map_err(|e| e.to_string()) {
                Ok(doc) => {
                    println!("Document: {}", doc.name);
                    println!("ID: {}", doc.id);
                    println!("Source: {}", doc.source);
                    println!("Created: {}", doc.created_at);
                    println!("");
                    println!("Content (decrypted):");
                    println!("{}", doc.content);
                }
                Err(_) => {
                    println!("Document not found: {}", id);
                }
            }
        }
        
        Commands::Search { query } => {
            let storage = open_storage(&data_dir)?;
            let results = storage.search(&query).map_err(|e| e.to_string())?;
            
            if results.is_empty() {
                println!("No matches found");
            } else {
                println!("Found {} matches:", results.len());
                for doc in results {
                    println!("");
                    println!("=== [{}] {} ===", doc.id, doc.name);
                    println!("{}", doc.content);
                }
            }
        }
        
        Commands::Config { config_cmd } => {
            match config_cmd {
                ConfigCommands::Mcp => {
                    let mcp_config = json!({
                        "mcpServers": {
                            "contextbox": {
                                "command": "contextbox",
                                "args": ["mcp"]
                            }
                        }
                    });
                    println!("{}", serde_json::to_string_pretty(&mcp_config)?);
                }
                ConfigCommands::Cli => {
                    let key_path = get_key_path();
                    println!("ContextBox CLI Configuration:");
                    println!("  Data directory: {:?}", data_dir);
                    println!("  Key path: {:?}", key_path);
                    println!("  Key exists: {}", key_path.exists());
                    println!("  OpenRouter key: {}", if cli.openrouter_key.is_some() { "Set" } else { "Not set" });
                }
            }
        }
        
        Commands::Setup => {
            println!("Running ContextBox setup...");
            println!("");
            
            let (key, is_new) = ensure_key().map_err(|e| e.to_string())?;
            println!("Encryption key: {}", if is_new { "generated" } else { "exists" });
            
            let db_path = get_db_path(&data_dir);
            println!("Database: {:?}", db_path);
            
            println!("");
            println!("Next steps:");
            println!("1. Set OPENROUTER_API_KEY in your environment");
            println!("2. Run: contextbox serve");
            println!("3. Add documents: cb add --file doc.md");
        }
    }
    
    Ok(())
}
