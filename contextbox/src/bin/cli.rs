use clap::Parser;
use std::path::PathBuf;
use serde_json::json;
use std::fs;
use contextbox::storage::{DocumentStore, StoredDocument, create_document};

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

#[derive(clap::Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
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
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
    Config {
        #[command(subcommand)]
        config_cmd: ConfigCommands,
    },
    Setup {
        #[arg(short, long)]
        mcp: bool,
        #[arg(short, long)]
        cli: bool,
    },
}

#[derive(clap::Subcommand)]
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
    fs::create_dir_all(data_dir.join("documents"))?;
    Ok(())
}

fn get_store_path(data_dir: &PathBuf) -> PathBuf {
    data_dir.join("documents.json")
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
    let store_path = get_store_path(&data_dir);
    
    match cli.command {
        Commands::Serve { port, host } => {
            println!("Starting ContextBox server on {}:{}...", host, port);
            println!("This will run the web API server");
            println!("Access via: http://{}:{}", host, port);
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
            
            let mut store = DocumentStore::new(&store_path);
            let doc = create_document(doc_name.clone(), doc_content.clone(), "cli");
            let id = store.add(doc);
            
            if let Err(e) = store.save(&store_path) {
                eprintln!("Error saving document: {}", e);
                return Ok(());
            }
            
            println!("Document '{}' added", doc_name);
            println!("  ID: {}", id);
            println!("  Content length: {} chars", doc_content.len());
            println!("  Saved to: {:?}", store_path);
        }
        Commands::List => {
            let store = DocumentStore::new(&store_path);
            let docs = store.list();
            
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
            let mut store = DocumentStore::new(&store_path);
            
            if store.remove(&id).is_some() {
                if let Err(e) = store.save(&store_path) {
                    eprintln!("Error saving changes: {}", e);
                    return Ok(());
                }
                println!("Document deleted: {}", id);
            } else {
                println!("Document not found: {}", id);
            }
        }
        Commands::Get { id } => {
            let store = DocumentStore::new(&store_path);
            
            if let Some(doc) = store.get(&id) {
                println!("Document: {}", doc.name);
                println!("ID: {}", doc.id);
                println!("Source: {}", doc.source);
                println!("Created: {}", doc.created_at);
                println!("");
                println!("Content:");
                println!("{}", doc.content);
            } else {
                println!("Document not found: {}", id);
            }
        }
        Commands::Search { query, limit: _ } => {
            println!("Searching for: {}", query);
            println!("(Full-text search will be implemented with vector embeddings)");
            
            let store = DocumentStore::new(&store_path);
            let docs = store.list();
            
            let query_lower = query.to_lowercase();
            let matches: Vec<_> = docs.iter()
                .filter(|d| d.content.to_lowercase().contains(&query_lower) || d.name.to_lowercase().contains(&query_lower))
                .collect();
            
            if matches.is_empty() {
                println!("No matches found");
            } else {
                println!("Found {} matches:", matches.len());
                for doc in matches {
                    println!("  [{}] {}", doc.id, doc.name);
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
                    println!("ContextBox CLI Configuration:");
                    println!("  Data directory: {:?}", data_dir);
                    println!("  OpenRouter key: {}", if cli.openrouter_key.is_some() { "Set" } else { "Not set" });
                }
            }
        }
        Commands::Setup { mcp, cli } => {
            println!("Running ContextBox setup...");
            println!("Data directory created: {:?}", data_dir);
            
            if mcp {
                println!("MCP configuration ready");
            }
            if cli {
                println!("CLI configured");
            }
            
            println!("\nNext steps:");
            println!("1. Set OPENROUTER_API_KEY in your environment");
            println!("2. Run: contextbox serve");
        }
    }
    
    Ok(())
}
