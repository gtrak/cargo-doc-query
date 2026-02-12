// Path resolution and ID lookup utilities

use rustdoc_types::{Crate, Id, Item};
use std::collections::HashMap;

pub struct PathResolver;

impl PathResolver {
    /// Find item by fully qualified path
    pub fn find_by_path<'a>(krate: &'a Crate, path: &str) -> Vec<(Id, &'a Item)> {
        krate
            .paths
            .iter()
            .filter(|(_, summary)| Self::path_matches(&summary.path, path))
            .filter_map(|(id, _)| krate.index.get(id).map(|item| (*id, item)))
            .collect()
    }

    /// Find items by path across all loaded crates
    pub fn find_by_path_in_crates<'a>(
        crates: &'a HashMap<String, Crate>,
        path: &str,
        crate_filter: Option<&str>,
    ) -> Vec<(String, Id, &'a Item)> {
        let mut matches = Vec::new();

        for (crate_name, krate) in crates {
            if let Some(filter) = crate_filter {
                if crate_name != filter {
                    continue;
                }
            }

            for (id, item) in Self::find_by_path(krate, path) {
                matches.push((crate_name.clone(), id, item));
            }
        }

        matches
    }

    /// Check if a path matches the query
    /// Path is Vec<String> but query is &str (e.g., "Vec" or "std::vec::Vec")
    fn path_matches(item_path: &[String], query_path: &str) -> bool {
        // Convert Vec<String> to a path string
        let path_str = item_path.join("::");

        if path_str == query_path {
            return true;
        }

        // Suffix match: query "Vec" matches "std::vec::Vec"
        if path_str.ends_with(&format!("::{}", query_path)) {
            return true;
        }

        false
    }

    /// Get item by Id from a specific crate
    pub fn get_item(krate: &Crate, id: Id) -> Option<&Item> {
        krate.index.get(&id)
    }

    /// Get item by Id from multiple crates
    pub fn get_item_from_crates<'a>(
        crates: &'a HashMap<String, Crate>,
        crate_name: &str,
        id: Id,
    ) -> Option<&'a Item> {
        crates
            .get(crate_name)
            .and_then(|krate| krate.index.get(&id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_matches_exact() {
        let item_path = vec!["std".to_string(), "vec".to_string(), "Vec".to_string()];
        assert!(PathResolver::path_matches(&item_path, "std::vec::Vec"));
    }

    #[test]
    fn test_path_matches_suffix() {
        let item_path = vec!["std".to_string(), "vec".to_string(), "Vec".to_string()];
        assert!(PathResolver::path_matches(&item_path, "vec::Vec"));
        assert!(PathResolver::path_matches(&item_path, "Vec"));
    }

    #[test]
    fn test_path_matches_no_match() {
        let item_path = vec!["std".to_string(), "vec".to_string(), "Vec".to_string()];
        assert!(!PathResolver::path_matches(
            &item_path,
            "std::string::String"
        ));
        assert!(!PathResolver::path_matches(&item_path, "Option"));
    }

    #[test]
    fn test_path_matches_partial_suffix() {
        let item_path = vec!["anyhow".to_string(), "Error".to_string()];
        assert!(PathResolver::path_matches(&item_path, "anyhow::Error"));
        assert!(PathResolver::path_matches(&item_path, "Error"));
    }
}
