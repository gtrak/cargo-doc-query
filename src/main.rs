mod cache;
mod cargo;
mod cli;
mod index;
mod parser;

use clap::{Parser, Subcommand};
use cli::build::BuildCommand;
use cli::Command;

#[derive(Parser)]
#[command(name = "cargo-doc-query")]
#[command(version = "0.1.0")]
#[command(about = "Fast, structured API queries over Rust dependency documentation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to Cargo.toml manifest (default: current directory)
    #[arg(short, long, default_value = ".")]
    manifest: String,

    /// Include all features when generating documentation
    #[arg(long)]
    all_features: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate documentation index from Rust dependencies
    #[command(name = "build")]
    Build,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Build => {
            let manifest_path = std::path::PathBuf::from(&cli.manifest);
            BuildCommand::new(manifest_path, cli.all_features)
                .execute()
                .expect("Build failed");
        }
    }
}
