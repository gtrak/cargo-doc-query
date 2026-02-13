// CLI argument definitions for cargo-doc-query
//
// Provides centralized argument parsing with clap, including
// global flags and per-command options.

use clap::{Parser, Subcommand};

/// Global CLI arguments
#[derive(Parser, Debug)]
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
    cargo doc-query query serde::Serialize --detailed

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
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to Cargo.toml manifest (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub manifest: String,

    /// Include all features when generating documentation
    #[arg(long)]
    pub all_features: bool,

    /// Disable colored output (useful for CI or piping)
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Suppress progress indicators and timing info
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

/// CLI subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
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
    /// DETAIL LEVEL:
    /// Control how much metadata is displayed:
    ///   --minimal    Signatures only, no metadata (smallest output)
    ///   --detailed   Full metadata including attributes and deprecation
    ///   (default)    Standard metadata (visibility, generics)
    ///
    /// Note: --minimal takes precedence if both flags are specified.
    ///
    /// FILTERING:
    /// Filter results using glob patterns and criteria. Multiple filters combine with AND logic.
    /// Multiple values for the same flag combine with OR logic.
    ///
    ///     --include "std::*"              Show only items from std crate
    ///     --exclude "*::test*"            Exclude items with "test" in path
    ///     --kind function                 Show only functions
    ///     --only "serde::*"               Show only serde items (shorthand)
    ///     --include "std::*" --kind fn    Show only std functions (AND logic)
    ///
    /// EXAMPLES:
    ///     cargo doc-query query Vec --include "std::*"
    ///     cargo doc-query query Error --exclude "*test*" --kind struct
    ///     cargo doc-query query Serialize --only "serde::*"
    ///     cargo doc-query query Vec --detailed --depth 2
    ///
    /// For detailed glob pattern syntax, run: cargo doc-query query --help-filters
    #[command(name = "query")]
    Query {
        /// The path to query (e.g., std::vec::Vec)
        path: Option<String>,

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

        /// Output minimal representation (signatures only, no metadata)
        #[arg(long)]
        minimal: bool,

        /// Display detailed metadata for each item (attributes, deprecation, etc.)
        #[arg(long, short = 'd')]
        detailed: bool,

        /// Maximum tokens in output (approximate)
        #[arg(long, value_name = "N")]
        tokens: Option<usize>,

        /// Output as JSON instead of human-readable text
        #[arg(long)]
        json: bool,

        /// Include items matching glob pattern. Multiple allowed (OR logic).
        #[arg(short, long, value_name = "PATTERN")]
        include: Vec<String>,

        /// Exclude items matching glob pattern. Multiple allowed (OR logic).
        #[arg(short, long, value_name = "PATTERN")]
        exclude: Vec<String>,

        /// Filter by item kind (struct, enum, trait, function, etc.). Case-insensitive.
        #[arg(short, long, value_name = "KIND")]
        kind: Vec<String>,

        /// Filter by crate name. Multiple allowed.
        #[arg(long, value_name = "CRATE")]
        crate_filter: Vec<String>,

        /// Filter by visibility: pub, pub(crate), pub(super), pub(in path), private
        #[arg(long, value_name = "VIS")]
        visibility: Vec<String>,

        /// Include only matching items, exclude everything else. Mutually exclusive with --include.
        #[arg(long, value_name = "PATTERN")]
        only: Option<String>,

        /// Display glob syntax help and exit
        #[arg(long)]
        help_filters: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_detailed_flag_parsing() {
        let args = Args::parse_from(["cmd", "query", "Vec", "--detailed"]);
        match args.command {
            Commands::Query {
                detailed, minimal, ..
            } => {
                assert!(detailed);
                assert!(!minimal);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_detailed_short_flag() {
        let args = Args::parse_from(["cmd", "query", "Vec", "-d"]);
        match args.command {
            Commands::Query { detailed, .. } => {
                assert!(detailed);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_minimal_flag_parsing() {
        let args = Args::parse_from(["cmd", "query", "Vec", "--minimal"]);
        match args.command {
            Commands::Query {
                detailed, minimal, ..
            } => {
                assert!(!detailed);
                assert!(minimal);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_both_flags_parsing() {
        let args = Args::parse_from(["cmd", "query", "Vec", "--minimal", "--detailed"]);
        match args.command {
            Commands::Query {
                detailed, minimal, ..
            } => {
                assert!(detailed);
                assert!(minimal);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_default_flags() {
        let args = Args::parse_from(["cmd", "query", "Vec"]);
        match args.command {
            Commands::Query {
                detailed, minimal, ..
            } => {
                assert!(!detailed);
                assert!(!minimal);
            }
            _ => panic!("Expected Query command"),
        }
    }
}
