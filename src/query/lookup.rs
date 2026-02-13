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
#[cfg(test)]
mod tests {
    use super::*;
    use rustdoc_types::Crate;

    #[test]
    fn test_find_by_path_with_filter_excludes_non_matching() {
        let mut krate = Crate {
            root: Id(1),
            crate_version: None,
            includes_private: true,
            index: HashMap::new(),
            paths: HashMap::new(),
            external_crates: HashMap::new(),
            target: rustdoc_types::Target {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                target_features: vec![],
            },
            format_version: 1,
        };
        krate.paths.insert(
            Id(1),
            rustdoc_types::ItemSummary {
                crate_id: 1,
                path: vec!["std".to_string()],
                kind: rustdoc_types::ItemKind::Module,
            },
        );
        krate.index.insert(
            Id(1),
            rustdoc_types::Item {
                id: Id(1),
                crate_id: 1,
                name: Some("std".to_string()),
                span: None,
                visibility: rustdoc_types::Visibility::Public,
                docs: None,
                links: std::collections::HashMap::new(),
                attrs: vec![],
                deprecation: None,
                inner: rustdoc_types::ItemEnum::Module(rustdoc_types::Module {
                    is_crate: true,
                    items: vec![],
                    is_stripped: false,
                }),
            },
        );

        let crates: HashMap<String, Crate> = [("std".to_string(), krate)].into_iter().collect();
        let results = PathResolver::find_by_path_in_crates(&crates, "std", Some("std"));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_find_by_path_with_filter_excludes_filtered_crates() {
        let mut krate1 = Crate {
            root: Id(1),
            crate_version: None,
            includes_private: true,
            index: HashMap::new(),
            paths: HashMap::new(),
            external_crates: HashMap::new(),
            target: rustdoc_types::Target {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                target_features: vec![],
            },
            format_version: 1,
        };
        let mut krate2 = Crate {
            root: Id(2),
            crate_version: None,
            includes_private: true,
            index: HashMap::new(),
            paths: HashMap::new(),
            external_crates: HashMap::new(),
            target: rustdoc_types::Target {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                target_features: vec![],
            },
            format_version: 1,
        };

        krate1.paths.insert(
            Id(1),
            rustdoc_types::ItemSummary {
                crate_id: 1,
                path: vec!["std".to_string()],
                kind: rustdoc_types::ItemKind::Module,
            },
        );

        krate2.paths.insert(
            Id(2),
            rustdoc_types::ItemSummary {
                crate_id: 2,
                path: vec!["serde".to_string()],
                kind: rustdoc_types::ItemKind::Module,
            },
        );

        let crates: HashMap<String, Crate> =
            [("std".to_string(), krate1), ("serde".to_string(), krate2)]
                .into_iter()
                .collect();
        let results = PathResolver::find_by_path_in_crates(&crates, "std", Some("serde"));
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_item_from_crates_nonexistent_id() {
        let mut krate = Crate {
            root: Id(1),
            crate_version: None,
            includes_private: true,
            index: HashMap::new(),
            paths: HashMap::new(),
            external_crates: HashMap::new(),
            target: rustdoc_types::Target {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                target_features: vec![],
            },
            format_version: 1,
        };
        let id = Id(1);
        let crates: HashMap<String, Crate> = [("std".to_string(), krate)].into_iter().collect();
        let item = PathResolver::get_item_from_crates(&crates, "std", id);
        assert!(item.is_none());
    }
}
