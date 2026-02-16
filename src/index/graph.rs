use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;

/// Node representing a crate in the documentation index
#[derive(Debug, Clone)]
pub struct CrateNode {
    pub name: String,
    pub version: String,
    pub json_path: std::path::PathBuf,
    // rustdoc: Option<rustdoc_types::Crate>, // Populated later
}

/// Edge types for dependency relationships
#[derive(Debug, Clone)]
pub enum DependencyEdge {
    Normal, // Normal dependency
    Dev,    // Dev dependency
    Build,  // Build dependency
}

/// Graph-based documentation index
#[derive(Debug)]
pub struct CrateGraph {
    graph: Graph<CrateNode, DependencyEdge>,
    name_index: HashMap<(String, String), NodeIndex>, // (name, version) -> NodeIndex
}

impl CrateGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            name_index: HashMap::new(),
        }
    }

    pub fn add_crate(&mut self, node: CrateNode) -> NodeIndex {
        let idx = self.graph.add_node(node.clone());
        self.name_index
            .insert((node.name.clone(), node.version.clone()), idx);
        idx
    }

    // TODO: what is this for?
    pub fn add_dependency(&mut self, from: NodeIndex, to: NodeIndex, kind: DependencyEdge) {
        self.graph.add_edge(from, to, kind);
    }

    /// Returns the number of crates in the index
    pub fn crate_count(&self) -> usize {
        self.graph.node_count()
    }
}
