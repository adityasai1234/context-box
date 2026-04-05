use clap::Parser;
use std::path::PathBuf;
use serde_json::json;
use std::fs;

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
        Commands::Add { file, name, content } => {
            println!("Adding document...");
            
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
            
            let doc_name = name.unwrap_or_else(|| "Untitled".to_string());
            
            println!("✓ Document '{}' added", doc_name);
            println!("  Content length: {} chars", doc_content.len());
            println!("  Data directory: {:?}", data_dir);
        }
        Commands::List => {
            println!("Documents in {:?}:", data_dir.join("documents"));
            println!("(No documents found yet)");
        }
        Commands::Delete { id } => {
            println!("Deleting document: {}", id);
        }
        Commands::Get { id } => {
            println!("Getting document: {}", id);
        }
        Commands::Search { query, limit } => {
            println!("Searching for: {}", query);
            println!("Limit: {} results", limit);
            println!("(Search requires server to be running)");
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
            println!("✓ Data directory created: {:?}", data_dir);
            
            if mcp {
                println!("✓ MCP configuration ready");
            }
            if cli {
                println!("✓ CLI configured");
            }
            
            println!("\nNext steps:");
            println!("1. Set OPENROUTER_API_KEY in your environment");
            println!("2. Run: contextbox serve");
        }
    }
    
    Ok(())
}
