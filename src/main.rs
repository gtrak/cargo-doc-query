use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo-doc-query")]
#[command(version = "0.1.0")]
#[command(about = "Fast, structured API queries over Rust dependency documentation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate documentation index from Rust dependencies
    Build,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Build => {
            println!("Build command invoked");
        }
    }
}
