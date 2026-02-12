mod cache;
mod cargo;
mod cli;
mod index;
mod parser;
mod query;
mod types;

use clap::{Parser, Subcommand};
use cli::build::BuildCommand;
use cli::query::QueryCommand;
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

    /// Query methods and traits for a type
    #[command(name = "query")]
    Query {
        /// The path to query (e.g., std::vec::Vec)
        path: String,

        /// Limit to specific crate
        #[arg(long)]
        crate_name: Option<String>,

        /// What to include in output
        #[arg(
            long,
            value_parser = clap::builder::PossibleValuesParser::new([
                "docs",
                "private",
                "trait_parameterization",
            ])
        )]
        include: Vec<String>,

        /// Which kind of query (methods, traits, types, all)
        #[arg(long, default_value = "all")]
        kind: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Build => {
            // Convert manifest path to absolute path and resolve to Cargo.toml if needed
            let mut manifest_path = std::path::PathBuf::from(&cli.manifest);
            if manifest_path.as_os_str().is_empty() {
                manifest_path = std::env::current_dir().expect("Cannot get current directory");
            }

            // If it's a directory, resolve to Cargo.toml inside it
            if manifest_path.is_dir() {
                manifest_path.push("Cargo.toml");
            }

            BuildCommand::new(manifest_path, cli.all_features)
                .execute()
                .expect("Build failed");
        }
        Commands::Query {
            path,
            crate_name,
            include,
            kind,
        } => {
            let kind = match kind.to_lowercase().as_str() {
                "methods" => cli::query::QueryKindArg::Methods,
                "traits" => cli::query::QueryKindArg::Traits,
                "types" => cli::query::QueryKindArg::Types,
                _ => cli::query::QueryKindArg::All,
            };

            QueryCommand::new(path.clone(), crate_name.clone(), include.clone(), kind)
                .execute()
                .expect("Query failed");
        }
    }
}
