// Command execution and routing
//
// Provides high-level command execution that bridges CLI arguments
// with the query engines, handling DetailLevel propagation.

use anyhow::{Context, Result};
use std::time::Instant;

use crate::cache::store::CacheStore;
use crate::cli::args::{Args, Commands as ArgsCommands};
use crate::cli::build::BuildCommand;
use crate::cli::expand::ExpandCommand;
use crate::cli::Command;
use crate::error::errors::AppError;
use crate::types::detail::DetailLevel;
use crate::types::expand::TokenConfig;
use crate::types::filter::{FilterConfig, FilterEngine, FilterError};

/// Execute the appropriate command based on parsed CLI arguments
pub fn execute(args: Args, quiet: bool, no_color: bool) -> Result<(), AppError> {
    // Apply global flags
    if no_color {
        console::set_colors_enabled(false);
    }

    match args.command {
        ArgsCommands::Build => execute_build(args, quiet),
        ArgsCommands::Query {
            path,
            depth,
            crate_name,
            minimal,
            detailed,
            tokens,
            json,
            include,
            exclude,
            kind,
            crate_filter,
            visibility,
            only,
            help_filters,
        } => execute_query(
            path,
            depth,
            crate_name,
            minimal,
            detailed,
            tokens,
            json,
            include,
            exclude,
            kind,
            crate_filter,
            visibility,
            only,
            help_filters,
            quiet,
        ),
    }
}

/// Execute the build command
fn execute_build(args: Args, quiet: bool) -> Result<(), AppError> {
    // Convert manifest path to absolute path and resolve to Cargo.toml if needed
    let mut manifest_path = std::path::PathBuf::from(&args.manifest);
    if manifest_path.as_os_str().is_empty() {
        manifest_path = std::env::current_dir()
            .map_err(|e| AppError::Config(format!("Cannot get current directory: {}", e)))?;
    }

    // If it's a directory, resolve to Cargo.toml inside it
    if manifest_path.is_dir() {
        manifest_path.push("Cargo.toml");
    }

    let mut cmd = BuildCommand::new(manifest_path, args.all_features);
    cmd.set_quiet(quiet);
    cmd.execute()
        .map_err(|e| AppError::BuildFailed(e.to_string()))
}

/// Execute the query command with DetailLevel support
#[allow(clippy::too_many_arguments)]
fn execute_query(
    path: Option<String>,
    depth: u32,
    crate_name: Option<String>,
    minimal: bool,
    detailed: bool,
    tokens: Option<usize>,
    json: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    kind: Vec<String>,
    crate_filter: Vec<String>,
    visibility: Vec<String>,
    only: Option<String>,
    help_filters: bool,
    quiet: bool,
) -> Result<(), AppError> {
    // Handle --help-filters without requiring a path
    if help_filters {
        print_glob_syntax_help();
        return Ok(());
    }

    // Validate that path is provided
    let path = path.as_ref().ok_or_else(|| {
        AppError::Config("PATH argument is required. Use --help for usage information.".to_string())
    })?;

    // Compute DetailLevel from flags (minimal takes precedence)
    let detail_level = DetailLevel::from_flags(minimal, detailed);

    // Warn if both flags are specified
    if minimal && detailed && !quiet {
        eprintln!("Warning: --minimal takes precedence over --detailed");
    }

    // Always use expand (unified rendering)
    // Default depth is 1 to show submodules, depth=0 shows just the type
    let depth = if depth == 0 { 1 } else { depth };

    // Create token config with DetailLevel awareness
    let token_config = TokenConfig::new().with_budget(tokens).with_minimal(minimal);

    let mut cmd = ExpandCommand::from_args_with_detail(
        path.clone(),
        depth,
        crate_name,
        tokens,
        minimal,
        detail_level,
        include,
        exclude,
        kind,
        crate_filter,
        visibility,
        only,
        help_filters,
    );
    cmd.json = json;
    cmd.quiet = quiet;

    match cmd.execute_with_detail(detail_level) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Downcast to check for specific error types
            if let Some(expand_err) = e.downcast_ref::<crate::query::expand::ExpandError>() {
                match expand_err {
                    crate::query::expand::ExpandError::NoCache => Err(AppError::NoCache),
                    crate::query::expand::ExpandError::NotFound(p) => {
                        // Show suggestions for similar types
                        if !quiet && !json {
                            if let Ok(suggestions) = suggest_similar_types(path) {
                                if !suggestions.is_empty() {
                                    eprintln!("\nDid you mean:");
                                    for suggestion in suggestions {
                                        eprintln!("  • {}", suggestion);
                                    }
                                }
                            }
                        }
                        Err(AppError::NotFound(p.clone()))
                    }
                    crate::query::expand::ExpandError::Other(_) => Err(AppError::Other(e)),
                }
            } else {
                Err(AppError::Other(e))
            }
        }
    }
}

/// Find similar type names to suggest when a query fails
fn suggest_similar_types(path: &str) -> anyhow::Result<Vec<String>> {
    let cache_store = CacheStore::new()?;

    if let Some(index) = cache_store.load_current()? {
        let suggestions = crate::query::suggest::find_similar_types(&index, path, 5);
        Ok(suggestions)
    } else {
        Ok(vec![])
    }
}

/// Display glob pattern syntax help
fn print_glob_syntax_help() {
    println!("Filter Pattern Syntax (Glob Patterns)");
    println!("=====================================\n");

    println!("Special Characters:");
    println!("  *       Matches any sequence of characters (except path separator)");
    println!("  ?       Matches any single character");
    println!("  **      Matches any sequence including path separators");
    println!("  [...]   Matches any character in brackets");
    println!("  [!...]  Matches any character NOT in brackets\n");

    println!("Examples:");
    println!("  'std::*'           → All items in std crate");
    println!("  'std::vec::*'      → All items in std::vec module");
    println!("  '*::test*'         → Items with \"test\" in the name");
    println!("  '**::Display'      → Display trait anywhere");
    println!("  'crate::[A-Z]*'    → Items starting with capital letter");
    println!("  'serde::de::*'     → All items in serde::de module\n");

    println!("Tips:");
    println!("  - Use quotes around patterns with special characters");
    println!("  - Patterns are case-sensitive for paths");
    println!("  - Multiple --include flags = OR logic");
    println!("  - Different flag types = AND logic");
    println!("  - Run `cargo doc-query query --help` for filtering options");
}

/// Command executor that holds DetailLevel context
pub struct CommandExecutor {
    detail_level: DetailLevel,
}

impl CommandExecutor {
    /// Create new executor with specified detail level
    pub fn new(detail_level: DetailLevel) -> Self {
        Self { detail_level }
    }

    /// Get the current detail level
    pub fn detail_level(&self) -> DetailLevel {
        self.detail_level
    }

    /// Check if current detail level includes visibility
    pub fn includes_visibility(&self) -> bool {
        self.detail_level.includes_visibility()
    }

    /// Check if current detail level includes generics
    pub fn includes_generics(&self) -> bool {
        self.detail_level.includes_generics()
    }

    /// Check if current detail level includes attributes
    pub fn includes_attributes(&self) -> bool {
        self.detail_level.includes_attributes()
    }

    /// Check if current detail level includes deprecation
    pub fn includes_deprecation(&self) -> bool {
        self.detail_level.includes_deprecation()
    }

    /// Check if current detail level includes function modifiers
    pub fn includes_function_modifiers(&self) -> bool {
        self.detail_level.includes_function_modifiers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::Args;
    use clap::Parser;

    #[test]
    fn test_detail_level_from_args() {
        // Standard (neither flag)
        let args = Args::parse_from(["cmd", "query", "Vec"]);
        match args.command {
            ArgsCommands::Query {
                minimal, detailed, ..
            } => {
                let level = DetailLevel::from_flags(minimal, detailed);
                assert_eq!(level, DetailLevel::Standard);
            }
            _ => panic!("Expected Query command"),
        }

        // Minimal
        let args = Args::parse_from(["cmd", "query", "Vec", "--minimal"]);
        match args.command {
            ArgsCommands::Query {
                minimal, detailed, ..
            } => {
                let level = DetailLevel::from_flags(minimal, detailed);
                assert_eq!(level, DetailLevel::Minimal);
            }
            _ => panic!("Expected Query command"),
        }

        // Detailed
        let args = Args::parse_from(["cmd", "query", "Vec", "--detailed"]);
        match args.command {
            ArgsCommands::Query {
                minimal, detailed, ..
            } => {
                let level = DetailLevel::from_flags(minimal, detailed);
                assert_eq!(level, DetailLevel::Detailed);
            }
            _ => panic!("Expected Query command"),
        }

        // Minimal takes precedence
        let args = Args::parse_from(["cmd", "query", "Vec", "--minimal", "--detailed"]);
        match args.command {
            ArgsCommands::Query {
                minimal, detailed, ..
            } => {
                let level = DetailLevel::from_flags(minimal, detailed);
                assert_eq!(level, DetailLevel::Minimal);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_command_executor() {
        let executor = CommandExecutor::new(DetailLevel::Detailed);
        assert!(executor.includes_visibility());
        assert!(executor.includes_generics());
        assert!(executor.includes_attributes());
        assert!(executor.includes_deprecation());
        assert!(executor.includes_function_modifiers());

        let executor = CommandExecutor::new(DetailLevel::Standard);
        assert!(executor.includes_visibility());
        assert!(executor.includes_generics());
        assert!(!executor.includes_attributes());
        assert!(!executor.includes_deprecation());
        assert!(!executor.includes_function_modifiers());

        let executor = CommandExecutor::new(DetailLevel::Minimal);
        assert!(!executor.includes_visibility());
        assert!(!executor.includes_generics());
        assert!(!executor.includes_attributes());
        assert!(!executor.includes_deprecation());
        assert!(!executor.includes_function_modifiers());
    }
}
