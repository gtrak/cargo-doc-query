// Query command implementation

use anyhow::{Context, Result};
use clap::Parser;
use std::str::FromStr;
use std::time::Instant;

use crate::cache::key::CacheKeyInputs;
use crate::cache::store::CacheStore;
use crate::cli::build::BuildCommand;
use crate::cli::Command;
use crate::query::engine::{QueryEngine, QueryKind, QueryOptions};

#[derive(Parser, Debug)]
pub struct QueryCommand {
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
    kind: QueryKindArg,

    /// Output minimal representation (signatures only, no docs)
    #[arg(long)]
    minimal: bool,

    /// Maximum tokens in output (approximate)
    #[arg(long)]
    tokens: Option<usize>,

    /// Output as JSON instead of human-readable text
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryKindArg {
    Methods,
    Traits,
    Types,
    All,
}

impl FromStr for QueryKindArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "methods" => Ok(QueryKindArg::Methods),
            "traits" => Ok(QueryKindArg::Traits),
            "types" => Ok(QueryKindArg::Types),
            "all" => Ok(QueryKindArg::All),
            _ => Err(format!(
                "Invalid query kind: {}. Expected: methods, traits, types, all",
                s
            )),
        }
    }
}

impl std::fmt::Display for QueryKindArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryKindArg::Methods => write!(f, "methods"),
            QueryKindArg::Traits => write!(f, "traits"),
            QueryKindArg::Types => write!(f, "types"),
            QueryKindArg::All => write!(f, "all"),
        }
    }
}

impl QueryCommand {
    pub fn new(
        path: String,
        crate_name: Option<String>,
        include: Vec<String>,
        kind: QueryKindArg,
        minimal: bool,
        tokens: Option<usize>,
        json: bool,
    ) -> Self {
        Self {
            path,
            crate_name,
            include,
            kind,
            minimal,
            tokens,
            json,
        }
    }

    fn parse_options(&self) -> QueryOptions {
        let include_docs = self.include.contains(&"docs".to_string());
        let include_private = self.include.contains(&"private".to_string());
        let _include_trait_param = self.include.contains(&"trait_parameterization".to_string());

        let kind = match self.kind {
            QueryKindArg::Methods => QueryKind::Methods,
            QueryKindArg::Traits => QueryKind::Traits,
            QueryKindArg::Types => QueryKind::Types,
            QueryKindArg::All => QueryKind::All,
        };

        QueryOptions::new(kind)
            .with_docs(include_docs)
            .with_private(include_private)
            .with_minimal(self.minimal)
            .with_token_budget(self.tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_query_command_creation() {
        let query_cmd = QueryCommand::new(
            "std::vec::Vec".to_string(),
            None,
            vec!["docs".to_string()],
            QueryKindArg::All,
            false,
            None,
            false,
        );

        assert_eq!(query_cmd.path, "std::vec::Vec");
        assert!(query_cmd.include.contains(&"docs".to_string()));
        assert_eq!(query_cmd.kind, QueryKindArg::All);
        assert!(!query_cmd.minimal);
        assert_eq!(query_cmd.tokens, None);
        assert!(!query_cmd.json);
    }

    #[test]
    fn test_query_command_with_crate_name() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            Some("std".to_string()),
            vec![],
            QueryKindArg::All,
            false,
            None,
            false,
        );

        assert_eq!(query_cmd.crate_name, Some("std".to_string()));
    }

    #[test]
    fn test_query_command_with_json_output() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            None,
            vec![],
            QueryKindArg::All,
            false,
            None,
            true,
        );

        assert!(query_cmd.json);
    }

    #[test]
    fn test_query_command_with_minimal_mode() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            None,
            vec![],
            QueryKindArg::All,
            true,
            None,
            false,
        );

        assert!(query_cmd.minimal);
    }

    #[test]
    fn test_query_command_with_token_budget() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            None,
            vec![],
            QueryKindArg::All,
            false,
            Some(500),
            false,
        );

        assert_eq!(query_cmd.tokens, Some(500));
    }

    #[test]
    fn test_query_kind_from_str_valid() {
        let tests = vec![
            ("methods", QueryKindArg::Methods),
            ("traits", QueryKindArg::Traits),
            ("types", QueryKindArg::Types),
            ("all", QueryKindArg::All),
            ("METHODS", QueryKindArg::Methods),
            ("Methods", QueryKindArg::Methods),
        ];

        for (input, expected) in tests {
            let result = QueryKindArg::from_str(input).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_query_kind_from_str_invalid() {
        let invalid = vec!["invalid", "foo", "methods_and_traits"];

        for input in invalid {
            let result = QueryKindArg::from_str(input);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_query_kind_display() {
        let tests = vec![
            (QueryKindArg::Methods, "methods"),
            (QueryKindArg::Traits, "traits"),
            (QueryKindArg::Types, "types"),
            (QueryKindArg::All, "all"),
        ];

        for (kind, expected) in tests {
            let display = kind.to_string();
            assert_eq!(display, expected);
        }
    }

    #[test]
    fn test_query_kind_equality() {
        assert_eq!(QueryKindArg::Methods, QueryKindArg::Methods);
        assert_ne!(QueryKindArg::Methods, QueryKindArg::Traits);
        assert_ne!(QueryKindArg::All, QueryKindArg::Types);
    }

    #[test]
    fn test_parse_options_with_docs() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            None,
            vec!["docs".to_string()],
            QueryKindArg::All,
            false,
            None,
            false,
        );

        let options = query_cmd.parse_options();
        assert!(options.include_docs);
    }

    #[test]
    fn test_parse_options_with_private() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            None,
            vec!["private".to_string()],
            QueryKindArg::All,
            false,
            None,
            false,
        );

        let options = query_cmd.parse_options();
        assert!(options.include_private);
    }

    #[test]
    fn test_parse_options_with_minimal() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            None,
            vec![],
            QueryKindArg::All,
            true,
            None,
            false,
        );

        let options = query_cmd.parse_options();
        assert!(options.minimal);
    }

    #[test]
    fn test_parse_options_with_token_budget() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            None,
            vec![],
            QueryKindArg::All,
            false,
            Some(500),
            false,
        );

        let options = query_cmd.parse_options();
        assert_eq!(options.token_budget, Some(500));
    }

    #[test]
    fn test_parse_options_default_values() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            None,
            vec![],
            QueryKindArg::All,
            false,
            None,
            false,
        );

        let options = query_cmd.parse_options();
        assert_eq!(options.kind, QueryKind::All);
        assert!(!options.include_docs);
        assert!(!options.include_private);
        assert!(!options.minimal);
        assert_eq!(options.token_budget, None);
    }

    #[test]
    fn test_token_budget_validation_minimum() {
        let query_cmd = QueryCommand::new(
            "Vec".to_string(),
            None,
            vec![],
            QueryKindArg::All,
            false,
            Some(50),
            false,
        );

        assert!(query_cmd.tokens.is_some_and(|t| t < 100));
    }

    #[test]
    fn test_token_budget_validation_multiple() {
        let budgets = vec![100, 150, 500, 1000];

        for budget in budgets {
            let query_cmd = QueryCommand::new(
                "Vec".to_string(),
                None,
                vec![],
                QueryKindArg::All,
                false,
                Some(budget),
                false,
            );

            assert_eq!(query_cmd.tokens, Some(budget));
            assert!(budget >= 100);
        }
    }

    #[test]
    fn test_query_command_path_variations() {
        let paths = vec![
            "std::vec::Vec",
            "Vec",
            "anyhow::Error",
            "serde_json::Value",
            "some::module::Type",
        ];

        for path in paths {
            let query_cmd = QueryCommand::new(
                path.to_string(),
                None,
                vec![],
                QueryKindArg::All,
                false,
                None,
                false,
            );
            assert_eq!(query_cmd.path, path);
        }
    }

    #[test]
    fn test_include_options_combinations() {
        let combinations = vec![
            vec!["docs".to_string()],
            vec!["private".to_string()],
            vec!["docs", "private"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            vec![],
            vec!["trait_parameterization".to_string()],
        ];

        for combination in combinations {
            let query_cmd = QueryCommand::new(
                "Vec".to_string(),
                None,
                combination.clone(),
                QueryKindArg::All,
                false,
                None,
                false,
            );

            assert_eq!(query_cmd.include, combination);
        }
    }

    #[test]
    fn test_query_command_with_all_query_kinds() {
        let query_kinds = vec![
            QueryKindArg::Methods,
            QueryKindArg::Traits,
            QueryKindArg::Types,
            QueryKindArg::All,
        ];

        for kind in query_kinds {
            let query_cmd = QueryCommand::new(
                "Vec".to_string(),
                None,
                vec![],
                kind.clone(),
                false,
                None,
                false,
            );

            assert_eq!(query_cmd.kind, kind);
        }
    }
}

impl Command for QueryCommand {
    fn execute(&self) -> Result<()> {
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
        let cache_inputs = CacheKeyInputs::from_project(&manifest_path)
            .context("Failed to create cache key inputs")?;
        let expected_key = cache_inputs.generate_key();

        // Check if rebuild is needed
        let cache_store = CacheStore::new().context("Failed to initialize cache store")?;

        let index = if let Some(current_index) = cache_store.load_current()? {
            // Compare cache keys
            if current_index.cache_key != expected_key {
                println!("Manifest changed, rebuilding index...");
                let build_cmd = BuildCommand::new(
                    manifest_path.with_file_name("Cargo.toml"),
                    false, // Use default features
                );
                build_cmd.execute().context("Rebuild failed")?;
                // Reload index after rebuild
                cache_store
                    .load(&expected_key)
                    .context("Failed to load rebuilt index")?
                    .ok_or_else(|| anyhow::anyhow!("No cached index found after rebuild"))?
            } else {
                // Cache key matches, use current index
                current_index
            }
        } else {
            // No cache exists, need to build
            println!("No index found, building...");
            let build_cmd = BuildCommand::new(
                manifest_path.with_file_name("Cargo.toml"),
                false, // Use default features
            );
            build_cmd.execute().context("Build failed")?;
            cache_store
                .load(&expected_key)
                .context("Failed to load built index")?
                .ok_or_else(|| anyhow::anyhow!("No cached index found after build"))?
        };

        println!("Loaded index ({} crates)", index.nodes.len());

        // Time the query execution
        let start = Instant::now();

        // Create query engine
        let mut engine = QueryEngine::new(index);

        // Parse options
        let options = self.parse_options();

        // Execute query
        let response = engine
            .query(&self.path, &options, self.crate_name.as_deref())
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let duration = start.elapsed();

        // Check token budget if set
        if let Some(budget) = self.tokens {
            let token_count = response.estimate_tokens();
            if token_count > budget {
                eprintln!(
                    "⚠ Warning: Token budget exceeded ({} > {} tokens)",
                    token_count, budget
                );
            }
            eprintln!(
                "Query completed in {}ms ({} tokens)",
                duration.as_millis(),
                token_count
            );
        } else {
            eprintln!("Query completed in {}ms", duration.as_millis());
        }

        // Output based on format preference
        if self.json {
            // Output JSON
            let json_output = serde_json::to_string_pretty(&response)
                .context("Failed to serialize response as JSON")?;
            println!("{}", json_output);
        } else {
            // Output human-readable text
            crate::format::text::format_query_response(&response, &self.path);
        }

        Ok(())
    }
}
