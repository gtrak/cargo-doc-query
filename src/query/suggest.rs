//! Type suggestion engine for "did you mean?" functionality

use crate::cache::store::SerializableIndex;

/// Calculate string similarity using simple case-insensitive substring matching
/// Returns score from 0.0 to 1.0, higher is more similar
fn similarity_score(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    // Exact match
    if a_lower == b_lower {
        return 1.0;
    }

    // One contains the other
    if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
        return 0.8;
    }

    // Calculate character-based similarity
    let a_chars: Vec<char> = a_lower.chars().collect();
    let b_chars: Vec<char> = b_lower.chars().collect();

    let max_len = a_chars.len().max(b_chars.len());
    if max_len == 0 {
        return 0.0;
    }

    // Count matching characters at same positions
    let min_len = a_chars.len().min(b_chars.len());
    let matches = (0..min_len).filter(|&i| a_chars[i] == b_chars[i]).count();

    matches as f64 / max_len as f64
}

/// Find similar crate names in the index
pub fn find_similar_types(
    index: &SerializableIndex,
    query_path: &str,
    max_suggestions: usize,
) -> Vec<String> {
    let query_lower = query_path.to_lowercase();

    let mut scored_crates: Vec<(String, f64)> = Vec::new();

    // Search through all nodes (crates) in the index
    for node in &index.nodes {
        let crate_score = similarity_score(&node.name, &query_lower);
        if crate_score > 0.3 {
            scored_crates.push((node.name.clone(), crate_score));
        }

        // Also check if the query matches the crate name as a prefix
        if query_lower.starts_with(&node.name.to_lowercase()) {
            scored_crates.push((node.name.clone(), 0.9));
        }
    }

    // Sort by score (highest first)
    scored_crates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Remove duplicates and take top N
    let mut seen = std::collections::HashSet::new();
    scored_crates
        .into_iter()
        .filter_map(|(path, _)| {
            if seen.insert(path.clone()) {
                Some(path)
            } else {
                None
            }
        })
        .take(max_suggestions)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_exact_match() {
        assert_eq!(similarity_score("anyhow", "anyhow"), 1.0);
    }

    #[test]
    fn test_similarity_case_insensitive() {
        assert_eq!(similarity_score("Anyhow", "anyhow"), 1.0);
    }

    #[test]
    fn test_similarity_contains() {
        let score = similarity_score("serde", "ser");
        assert!(score > 0.5);
    }

    #[test]
    fn test_similarity_prefix() {
        let score = similarity_score("anyhow", "any");
        assert!(score > 0.5);
    }
}
