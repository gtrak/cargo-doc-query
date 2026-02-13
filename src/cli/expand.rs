// Expand command for type hierarchy exploration

use anyhow::{Context, Result};
use clap::Parser;
use std::time::Instant;

use crate::cli::Command;
use crate::types::expand::TokenConfig;
use crate::types::filter::{FilterConfig, FilterEngine, FilterError};

/// Check if a visibility string is valid
fn is_valid_visibility(vis: &str) -> bool {
    matches!(vis, "pub" | "pub(crate)" | "pub(super)" | "private") || vis.starts_with("pub(in ")
}

#[derive(Parser, Debug)]
pub struct ExpandCommand {
    /// The type path to expand (e.g., anyhow::Error)
    path: String,

    /// Maximum recursion depth (default: 3)
    #[arg(long, default_value = "3")]
    depth: u32,

    /// Limit to specific crate
    #[arg(long)]
    crate_name: Option<String>,

    /// Maximum tokens in output (approximate, default: unlimited)
    #[arg(long)]
    tokens: Option<usize>,

    /// Output minimal representation (signatures only, no field details)
    #[arg(long)]
    minimal: bool,

    /// Output as JSON instead of human-readable text
    #[arg(long)]
    pub json: bool,

    /// Suppress progress indicators and timing info
    #[arg(skip)]
    pub quiet: bool,

    /// Include patterns (glob) - items must match at least one
    #[arg(short, long, value_name = "PATTERN")]
    include: Vec<String>,

    /// Exclude patterns (glob) - items must not match any
    #[arg(short, long, value_name = "PATTERN")]
    exclude: Vec<String>,

    /// Filter by item kind (struct, enum, trait, function, etc.) (multiple allowed)
    #[arg(short, long, value_name = "KIND")]
    kind: Vec<String>,

    /// Filter by crate name (multiple allowed)
    #[arg(long, value_name = "CRATE")]
    crate_filter: Vec<String>,

    /// Filter by visibility (pub, pub(crate), pub(super), private)
    #[arg(long, value_name = "VIS")]
    visibility: Vec<String>,

    /// Include only matching items, exclude everything else
    #[arg(long, value_name = "PATTERN")]
    only: Option<String>,

    /// Display glob syntax help and exit
    #[arg(long)]
    help_filters: bool,
}

impl ExpandCommand {
    #[deprecated(note = "Use from_args() instead - supports filter flags")]
    pub fn new(path: String, depth: u32, crate_name: Option<String>) -> Self {
        Self {
            path,
            depth,
            crate_name,
            tokens: None,
            minimal: false,
            json: false,
            quiet: false,
            include: Vec::new(),
            exclude: Vec::new(),
            kind: Vec::new(),
            crate_filter: Vec::new(),
            visibility: Vec::new(),
            only: None,
            help_filters: false,
        }
    }

    /// Create from parsed arguments
    pub fn from_args(
        path: String,
        depth: u32,
        crate_name: Option<String>,
        tokens: Option<usize>,
        minimal: bool,
        include: Vec<String>,
        exclude: Vec<String>,
        kind: Vec<String>,
        crate_filter: Vec<String>,
        visibility: Vec<String>,
        only: Option<String>,
        help_filters: bool,
    ) -> Self {
        Self {
            path,
            depth,
            crate_name,
            tokens: None,
            minimal: false,
            json: false,
            quiet: false,
            include,
            exclude,
            kind,
            crate_filter,
            visibility,
            only,
            help_filters,
        }
    }

    /// Set quiet mode
    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    /// Validate filter arguments and return any errors
    ///
    /// Checks for mutually exclusive flags, invalid visibility values,
    /// and contradictory patterns.
    fn validate(&self) -> anyhow::Result<()> {
        // Check for --include and --only both being specified
        if !self.include.is_empty() && self.only.is_some() {
            anyhow::bail!(
                "Cannot use --include with --only.\n\n\
                --only is shorthand for 'include this and exclude everything else'.\n\
                Use either:\n\
                  --only 'pattern'          (include only matching items)\n\
                Or:\n\
                  --include 'pattern'       (include matching items alongside others)\n\n\
                For more help, run: cargo doc-query query --help-filters"
            );
        }

        // Validate visibility values
        for vis in &self.visibility {
            if !is_valid_visibility(vis) {
                anyhow::bail!(
                    "Invalid visibility value '{}'\n\n\
                    Valid options:\n\
                      pub, pub(crate), pub(super), pub(in path), private\n\n\
                    Example: --visibility pub\n\n\
                    For more help, run: cargo doc-query query --help-filters",
                    vis
                );
            }
        }

        // Check for same pattern in both include and exclude (warning only)
        for pattern in &self.include {
            if self.exclude.contains(pattern) {
                eprintln!(
                    "Warning: Pattern '{}' appears in both --include and --exclude. Exclude takes precedence.",
                    pattern
                );
            }
        }

        // Check for completely contradictory patterns (same pattern in both)
        // This is different from overlapping - this is exact match
        let exact_conflicts: Vec<_> = self
            .include
            .iter()
            .filter(|p| self.exclude.contains(p))
            .collect();

        if !exact_conflicts.is_empty() {
            anyhow::bail!(
                "Contradictory filter patterns detected.\n\n\
                The following patterns appear in both --include and --exclude:\n\
                {}\n\n\
                An item cannot be both included and excluded.\n\
                Remove conflicting patterns or use --only for exclusive filtering.\n\n\
                For filter syntax help, run: cargo doc-query query --help-filters",
                exact_conflicts
                    .iter()
                    .map(|p| format!("  - {}", p))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }

        Ok(())
    }

    /// Create FilterConfig from CLI filter arguments
    ///
    /// If --only is specified, it takes precedence over --include.
    /// Otherwise, --include is used as the include pattern.
    fn filter_config(&self) -> crate::types::filter::FilterConfig {
        let include_pattern = self.only.as_ref().map(|s| s.as_str());

        let mut config = crate::types::filter::FilterConfig::default();
        if let Some(pattern) = include_pattern {
            config = config.with_include(pattern.to_string());
        } else {
            for pattern in &self.include {
                config = config.with_include(pattern.clone());
            }
        }

        for pattern in &self.exclude {
            config = config.with_exclude(pattern.clone());
        }

        for kind in &self.kind {
            config = config.with_kind(kind.clone());
        }

        for crate_name in &self.crate_filter {
            config = config.with_crate(crate_name.clone());
        }

        for vis in &self.visibility {
            config = config.with_visibility(vis.clone());
        }

        config
    }

    /// Check if any filter flags are configured
    fn has_filters(&self) -> bool {
        !self.include.is_empty()
            || !self.exclude.is_empty()
            || !self.kind.is_empty()
            || !self.crate_filter.is_empty()
            || !self.visibility.is_empty()
            || self.only.is_some()
    }

    /// Validate filter arguments and return any errors
    pub fn execute(&self) -> Result<()> {
        // Display glob syntax help if requested
        if self.help_filters {
            self.display_glob_syntax_help();
            return Ok(());
        }

        // Validate filter arguments
        self.validate()?;

        // Validate token budget
        if let Some(tokens) = self.tokens {
            if tokens < 100 {
                return Err(anyhow::anyhow!(
                    "Token budget too small, minimum is 100 (got {})",
                    tokens
                ));
            }
        }

        // Discover manifest path (default to current directory)
        let manifest_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("Cargo.toml");

        // Generate expected cache key from manifest files
        let cache_inputs = crate::cache::key::CacheKeyInputs::from_project(&manifest_path)
            .context("Failed to create cache key inputs")?;
        let expected_key = cache_inputs.generate_key();

        // Check if rebuild is needed
        let cache_store =
            crate::cache::store::CacheStore::new().context("Failed to initialize cache store")?;

        let index = if let Some(current_index) = cache_store.load_current()? {
            // Compare cache keys
            if current_index.cache_key != expected_key {
                if !self.quiet {
                    println!("Manifest changed, rebuilding index...");
                }
                let build_cmd = crate::cli::build::BuildCommand::new(
                    manifest_path.with_file_name("Cargo.toml"),
                    false,
                );
                build_cmd.execute().context("Rebuild failed")?;
                cache_store
                    .load(&expected_key)
                    .context("Failed to load rebuilt index")?
                    .ok_or_else(|| anyhow::anyhow!("No cached index found after rebuild"))?
            } else {
                current_index
            }
        } else {
            // No cache exists, need to build
            if !self.quiet {
                println!("No index found, building...");
            }
            let build_cmd = crate::cli::build::BuildCommand::new(
                manifest_path.with_file_name("Cargo.toml"),
                false,
            );
            build_cmd.execute().context("Build failed")?;
            cache_store
                .load(&expected_key)
                .context("Failed to load built index")?
                .ok_or_else(|| anyhow::anyhow!("No cached index found after build"))?
        };

        if !self.quiet {
            println!("Loaded index ({} crates)", index.nodes.len());
        }

        // Create token config
        let token_config = TokenConfig::new()
            .with_budget(self.tokens)
            .with_minimal(self.minimal);

        // Time the expansion execution
        let start = Instant::now();

        // Use expand_type_with_config for token budgeting
        let mut expansion = crate::query::expand::expand_type_with_config(
            &self.path,
            self.depth,
            self.crate_name.as_deref(),
            token_config,
        )
        .context(format!("Expansion failed for path: {}", self.path))?;

        let duration = start.elapsed();

        // Print warnings if budget exceeded
        if expansion.budget_exceeded && !self.quiet {
            eprintln!("⚠ Warning: Token budget exceeded. Some types were truncated.");
            if !expansion.truncated_paths.is_empty() {
                eprintln!("  Truncated: {:?}", expansion.truncated_paths);
            }
        }

        // Print token count and timing to stderr (unless quiet)
        if !self.quiet {
            eprintln!(
                "Expansion completed in {}ms ({} tokens)",
                duration.as_millis(),
                expansion.token_count
            );
        }

        // Apply filters if configured
        let config = self.filter_config();
        if self.has_filters() {
            self.apply_filters(&mut expansion);
        }

        // Output based on format preference
        if self.json {
            // Output JSON
            let json_output = serde_json::to_string_pretty(&expansion)
                .context("Failed to serialize expansion as JSON")?;
            println!("{}", json_output);
        } else {
            // Output human-readable text
            crate::format::text::format_expand_result(&expansion, &self.path);
        }

        Ok(())
    }

    /// Apply filters to expansion results (mutates in place)
    fn apply_filters(&self, expansion: &mut crate::types::expand::ExpansionResult) {
        // Get filter configuration from CLI args
        let config = self.filter_config();

        // Only apply filters if any are configured
        if !self.has_filters() {
            return;
        }

        // Compile filter engine
        let engine = match FilterEngine::compile(&config) {
            Ok(e) => e,
            Err(FilterError::InvalidGlob { pattern, message }) => {
                eprintln!("Error: Invalid glob pattern '{}'", pattern);
                eprintln!();
                eprintln!("The pattern is not valid: {}", message);
                eprintln!();
                eprintln!("Example of a valid pattern: 'std::vec::*'");
                eprintln!();
                eprintln!("For glob syntax reference, run:");
                eprintln!("  cargo doc-query query --help-filters");
                return;
            }
            Err(FilterError::EmptyPattern) => {
                eprintln!("Error: Empty filter pattern");
                eprintln!();
                eprintln!("Filter patterns cannot be empty. Check your --include, --exclude, or --only flags.");
                eprintln!();
                eprintln!("Example: --include \"std::*\"");
                eprintln!();
                eprintln!("For filter syntax help, run:");
                eprintln!("  cargo doc-query query --help-filters");
                return;
            }
            Err(FilterError::ConflictingFilters { item }) => {
                eprintln!("Error: Conflicting filter patterns");
                eprintln!();
                eprintln!(
                    "The item '{}' matches both include and exclude patterns.",
                    item
                );
                eprintln!("Review your --include and --exclude flags to resolve the conflict.");
                return;
            }
            Err(e) => {
                eprintln!("Error: Failed to compile filters: {}", e);
                eprintln!();
                eprintln!("For filter syntax help, run:");
                eprintln!("  cargo doc-query query --help-filters");
                return;
            }
        };

        // Filter the type graph nodes
        let (filtered_nodes, stats) = engine.filter_with_stats(&expansion.graph.nodes);

        // Update expansion with filtered nodes
        expansion.graph.nodes = filtered_nodes
            .into_iter()
            .map(|node| node.clone())
            .collect();

        // Display stats if not in quiet mode
        if !self.quiet {
            println!();
            println!("{}", stats.summary());
        }
    }

    /// Display glob pattern syntax help
    fn display_glob_syntax_help(&self) {
        println!("Filter Pattern Syntax (Glob Patterns)");
        println!("====================================\n");

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
}
