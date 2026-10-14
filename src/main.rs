mod bench;
mod expand;
mod mcp;
mod model;
mod parser;
mod router;
mod scanner;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "doc-router",
    version,
    about = "Route agent documentation reads by token cost"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Route {
        #[arg(short, long)]
        task: String,
        #[arg(short, long, default_value = ".")]
        root: PathBuf,
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    Expand {
        #[arg(short, long)]
        id: String,
        #[arg(short, long, default_value = ".")]
        root: PathBuf,
    },
    Bench {
        #[arg(short, long)]
        corpus: PathBuf,
        #[arg(short, long)]
        tasks: PathBuf,
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    Mcp {
        #[arg(short, long, default_value = ".")]
        root: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Route { task, root, format } => {
            let docs = scanner::scan_docs(&root)?;
            let tree = parser::build_tree(&docs)?;
            let r = router::route(&task, &tree)?;
            match format.as_str() {
                "human" => router::print_human(&r),
                _ => println!("{}", serde_json::to_string_pretty(&r)?),
            }
        }
        Commands::Expand { id, root } => {
            println!("{}", expand::expand_section(&root, &id)?);
        }
        Commands::Bench {
            corpus,
            tasks,
            format,
        } => {
            bench::run(&corpus, &tasks, &format)?;
        }
        Commands::Mcp { root } => {
            mcp::serve(&root)?;
        }
    }
    Ok(())
}
