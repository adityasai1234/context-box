use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "contextbox")]
#[command(about = "Self-hosted Document AI Platform", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, default_value = "./data")]
    pub data_dir: PathBuf,
}

#[derive(Subcommand)]
pub enum Commands {
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
        
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
    Delete {
        #[arg(short, long)]
        id: String,
    },
    Get {
        #[arg(short, long)]
        id: String,
    },
    Config {
        #[command(subcommand)]
        config_cmd: ConfigCommands,
    },
    Setup,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Mcp,
}
