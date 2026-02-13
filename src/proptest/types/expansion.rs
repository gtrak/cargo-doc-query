use crate::query::expand::{TokenConfig, TypeExpander};
use crate::types::expand::{ExpansionResult, TypeGraph, TypeNode};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_expansion_result_creation(original: u32, token_budget: u32) {
        let config = TokenConfig {
            max_depth: 5,
            token_budget,
            include_docs: true,
        };

        let mut graph = TypeGraph::new(config.clone());
        let node = TypeNode::new(format!("test-{}", original), vec![], vec![], 0);
        graph.add_node(node);

        let result = ExpansionResult::new(graph, vec![], None, 0);

        prop_assert_eq!(result.graph.nodes.len(), 1);
        prop_assert_eq!(result.token_count, 0);
    }

    #[test]
    fn prop_expansion_no_duplicate_nodes(original_paths: Vec<String>) {
        if original_paths.len() < 2 {
            return Ok(());
        }

        let config = TokenConfig {
            max_depth: 10,
            token_budget: 1000,
            include_docs: false,
        };

        let mut graph = TypeGraph::new(config);
        let mut expander = TypeExpander::new(graph);

        for path in original_paths.iter().take(5) {
            let _ = expander.expand(path, None);
        }

        let node_ids: Vec<usize> = expander.graph.nodes.iter()
            .map(|n| n.id)
            .collect();

        // All node IDs should be unique
        prop_assert!(node_ids.len() == node_ids.into_iter().collect::<std::collections::HashSet<_>>().len());
    }

    #[test]
    fn prop_expansion_depth_limits(max_depth: u8) {
        if max_depth == 0 {
            return Ok(());
        }

        let config = TokenConfig {
            max_depth: max_depth as usize,
            token_budget: 1000,
            include_docs: false,
        };

        let mut graph = TypeGraph::new(config);
        let mut expander = TypeExpander::new(graph);

        // Expand a path that should have depth constraints
        let _ = expander.expand("std::collections::HashMap<String, String>", None);

        // Verify depth limits are enforced
        prop_assert!(!expander.truncated_paths.is_empty() || expander.budget_exceeded);
    }

    #[test]
    fn prop_type_graph_max_nodes(token_budget: u32) {
        if token_budget < 100 {
            return Ok(());
        }

        let config = TokenConfig {
            max_depth: 3,
            token_budget,
            include_docs: false,
        };

        let mut graph = TypeGraph::new(config);
        let mut expander = TypeExpander::new(graph);

        // Expand multiple complex types
        for path in ["Vec<String>", "Option<Vec<i32>>", "Result<String, String>"].iter() {
            let _ = expander.expand(path, None);
        }

        let max_nodes = graph.config.token_budget as usize / 10; // Rough estimate
        prop_assert!(graph.nodes.len() <= max_nodes + 10);
    }

    #[test]
    fn prop_expansion_result_token_count(token_budget: u32) {
        let config = TokenConfig {
            max_depth: 5,
            token_budget,
            include_docs: false,
        };

        let mut graph = TypeGraph::new(config);
        let node = TypeNode::new("test".to_string(), vec![], vec![], 100);
        graph.add_node(node);

        let result = ExpansionResult::new(graph.clone(), vec![], None, 100);

        prop_assert_eq!(result.token_count, 100);
        prop_assert_eq!(result.graph.nodes.len(), 1);
    }

    #[test]
    fn prop_type_node_estimate_tokens() {
        let base_tokens: u32 = 50;

        let node = TypeNode::new(
            "test".to_string(),
            vec![],
            vec![],
            base_tokens,
        );

        let estimated = node.estimate_tokens();

        prop_assert!(estimated >= base_tokens);
        prop_assert!(estimated <= base_tokens * 2);
    }
}
