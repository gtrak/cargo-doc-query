mod cache;
mod cargo;
mod cli;
mod error;
mod format;
mod index;
mod parser;
mod query;
mod types;

use clap::{Parser, Subcommand};
use cli::build::BuildCommand;
use cli::expand::ExpandCommand;
use cli::query::QueryCommand;
use cli::Command;
use error::errors::AppError;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "cargo-doc-query")]
#[command(version = "0.1.0")]
#[command(about = "Fast, structured API queries over Rust dependency documentation")]
#[command(long_about = r#"
cargo-doc-query is a tool for querying Rust crate documentation.

It generates an index from your dependencies' rustdoc JSON output and allows
you to quickly query methods, traits, and types with sub-100ms response times.

EXAMPLES:
    # Build the documentation index (run first)
    cargo doc-query build

    # Query a type's methods and traits
    cargo doc-query query std::vec::Vec
    cargo doc-query query anyhow::Error --minimal

    # Query with nested type expansion
    cargo doc-query query anyhow::Error --depth 2
    cargo doc-query query std::collections::HashMap --depth 1

    # Query with token budget for LLM contexts
    cargo doc-query query serde_json::Value --tokens 500

EXIT CODES:
    0   Success
    1   General error
    2   No cache found (run 'build' first)
    3   Query returned no results
    4   Build failed
    5   Invalid query
    6   Cache error
    7   IO error
    8   JSON parsing error
    9   Configuration error
"#)]
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
    ///
    /// This command generates rustdoc JSON for all workspace dependencies
    /// and builds a searchable index. Run this first before using query/expand.
    #[command(name = "build")]
    Build,

    /// Query a type's methods, traits, and optionally expand nested types
    ///
    /// Queries the documentation index for a specific type path and returns
    /// methods, trait implementations, and optionally expands nested types.
    ///
    /// EXAMPLES:
    ///     cargo doc-query query std::vec::Vec
    ///     cargo doc-query query anyhow::Error --depth 2
    ///     cargo doc-query query std::collections::HashMap --depth 1
    #[command(name = "query")]
    Query {
        /// The path to query (e.g., std::vec::Vec)
        path: String,

        /// Maximum recursion depth for expanding nested types (default: 0)
        ///
        /// - depth 0: Show methods and traits only (no nested types)
        /// - depth 1: Expand direct field types
        /// - depth 2+: Recursively expand nested types
        #[arg(long, default_value = "0", value_name = "N")]
        depth: u32,

        /// Limit to specific crate
        #[arg(long, value_name = "CRATE")]
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

        /// Output minimal representation (signatures only, no docs)
        #[arg(long)]
        minimal: bool,

        /// Maximum tokens in output (approximate)
        #[arg(long, value_name = "N")]
        tokens: Option<usize>,

        /// Output as JSON instead of human-readable text
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            e.exit_code()
        }
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    match &cli.command {
        Commands::Build => {
            // Convert manifest path to absolute path and resolve to Cargo.toml if needed
            let mut manifest_path = std::path::PathBuf::from(&cli.manifest);
            if manifest_path.as_os_str().is_empty() {
                manifest_path = std::env::current_dir().map_err(|e| {
                    AppError::Config(format!("Cannot get current directory: {}", e))
                })?;
            }

            // If it's a directory, resolve to Cargo.toml inside it
            if manifest_path.is_dir() {
                manifest_path.push("Cargo.toml");
            }

            BuildCommand::new(manifest_path, cli.all_features)
                .execute()
                .map_err(|e| AppError::BuildFailed(e.to_string()))
        }
        Commands::Query {
            path,
            depth,
            crate_name,
            include,
            kind,
            minimal,
            tokens,
            json,
        } => {
            let kind = match kind.to_lowercase().as_str() {
                "methods" => cli::query::QueryKindArg::Methods,
                "traits" => cli::query::QueryKindArg::Traits,
                "types" => cli::query::QueryKindArg::Types,
                _ => cli::query::QueryKindArg::All,
            };

            // Always use expand (unified rendering)
            // Default depth is 1 to show submodules, depth=0 shows just the type
            let depth = if *depth == 0 { 1 } else { *depth };
            let mut cmd = ExpandCommand::from_args(
                path.clone(),
                depth,
                crate_name.clone(),
                *tokens,
                *minimal,
            );
            cmd.json = *json;
            cmd.execute().map_err(|e| {
                let msg = e.to_string();
                if msg.contains("No cached index found") {
                    AppError::NoCache
                } else if msg.contains("No items found") {
                    AppError::NotFound(path.clone())
                } else {
                    AppError::Other(e)
                }
            })
        }
    }
}
