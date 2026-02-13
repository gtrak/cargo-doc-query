// Expansion output types for type hierarchy exploration

use serde::Serialize;

/// Configuration for token budget and output mode
#[derive(Debug, Clone)]
pub struct TokenConfig {
    /// Maximum token budget (None = unlimited)
    pub budget: Option<usize>,
    /// Output minimal representation
    pub minimal_mode: bool,
    /// Warning threshold (0.0-1.0, default 0.8)
    pub warning_threshold: f32,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            budget: None,
            minimal_mode: false,
            warning_threshold: 0.8,
        }
    }
}

impl TokenConfig {
    /// Create new config with unlimited budget
    pub fn new() -> Self {
        Self::default()
    }

    /// Set token budget
    pub fn with_budget(mut self, budget: Option<usize>) -> Self {
        self.budget = budget;
        self
    }

    /// Set minimal mode
    pub fn with_minimal(mut self, minimal: bool) -> Self {
        self.minimal_mode = minimal;
        self
    }

    /// Set warning threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.warning_threshold = threshold;
        self
    }
}

/// Top-level expansion result
#[derive(Serialize, Debug)]
pub struct ExpansionResult {
    /// Type graph with all discovered types
    pub graph: TypeGraph,
    /// List of paths that hit cycle detection limits
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cycles_detected: Vec<String>,
    /// Estimated token count for this result
    pub token_count: usize,
    /// Whether the budget was exceeded
    pub budget_exceeded: bool,
    /// Paths that were truncated due to budget
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub truncated_paths: Vec<String>,
}

impl ExpansionResult {
    /// Create new expansion result
    pub fn new(graph: TypeGraph) -> Self {
        let token_count = graph.estimate_tokens();
        Self {
            graph,
            cycles_detected: Vec::new(),
            token_count,
            budget_exceeded: false,
            truncated_paths: Vec::new(),
        }
    }

    /// Set budget exceeded flag and truncated paths
    pub fn with_truncation(mut self, truncated: Vec<String>) -> Self {
        self.budget_exceeded = !truncated.is_empty();
        self.truncated_paths = truncated;
        self
    }

    /// Update token count after modification
    pub fn update_token_count(&mut self) {
        self.token_count = self.graph.estimate_tokens();
    }
}

/// Top-level expansion result containing type graph and metadata
#[derive(Serialize, Debug, Default, Clone)]
pub struct TypeGraph {
    /// The path that was expanded
    pub root: String,
    /// Maximum recursion depth
    pub depth_limit: u32,
    /// All types discovered in the expansion
    pub nodes: Vec<TypeNode>,
    /// Paths that hit cycle detection limits
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cycles_detected: Vec<String>,
}

/// Individual type in the expansion graph
#[derive(Serialize, Debug, Clone)]
pub struct TypeNode {
    /// Fully qualified path
    pub id: String,
    /// Type kind: struct, enum, module, primitive, generic, etc.
    pub kind: String,
    /// Fields (for struct-like types)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldInfo>,
    /// Variants (for enums)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<VariantInfo>,
    /// Module items (for modules: types, functions, traits, etc.)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ModuleItemInfo>,
    /// Generic parameters
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub generic_params: Vec<String>,
    /// Depth from root type
    pub depth: u32,
    /// Field count (for minimal mode reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_count: Option<usize>,
    /// Variant count (for minimal mode reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_count: Option<usize>,
}

/// Minimal version of TypeNode for reduced output
#[derive(Serialize, Debug, Clone)]
pub struct MinimalTypeNode {
    /// Fully qualified path
    pub id: String,
    /// Type kind
    pub kind: String,
    /// Number of fields (if struct-like)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_count: Option<usize>,
    /// Number of variants (if enum)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_count: Option<usize>,
    /// Generic parameter count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_count: Option<usize>,
    /// Depth from root
    pub depth: u32,
}

/// Field information for struct/enum variants
#[derive(Serialize, Debug, Clone)]
pub struct FieldInfo {
    /// Field name
    pub name: String,
    /// Type path (e.g., String, std::collections::HashMap)
    pub type_path: String,
    /// Is the field optional?
    pub is_optional: bool,
    /// Reference to another node (if this field is a complex type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested_type_id: Option<String>,
}

/// Variant information for enums
#[derive(Serialize, Debug, Clone)]
pub struct VariantInfo {
    /// Variant name
    pub name: String,
    /// Variant fields
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldInfo>,
    /// Variant discriminant value (for C-like enums)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminant: Option<String>,
}

/// Module item information (types, functions, traits, etc.)
#[derive(Serialize, Debug, Clone)]
pub struct ModuleItemInfo {
    /// Item name
    pub name: String,
    /// Item kind: type, function, trait, macro, etc.
    pub kind: String,
    /// Item path
    pub path: String,
}

impl TypeGraph {
    /// Create new type graph
    pub fn new(root: String, depth_limit: u32) -> Self {
        Self {
            root,
            depth_limit,
            nodes: Vec::new(),
            cycles_detected: Vec::new(),
        }
    }

    /// Add a type node to the graph
    /// Returns the node ID for reference linking
    pub fn add_node(&mut self, node: TypeNode) -> String {
        let id = node.id.clone();
        self.nodes.push(node);
        id
    }

    /// Record a detected cycle
    pub fn add_cycle(&mut self, path: String) {
        self.cycles_detected.push(path);
    }

    /// Estimate token count (rough approximation: JSON string length / 4)
    pub fn estimate_tokens(&self) -> usize {
        match serde_json::to_string(self) {
            Ok(json) => json.len() / 4,
            Err(_) => 0,
        }
    }

    /// Convert to minimal representation
    pub fn to_minimal(&self) -> Self {
        Self {
            root: self.root.clone(),
            depth_limit: self.depth_limit,
            nodes: self.nodes.iter().map(|n| n.to_minimal()).collect(),
            cycles_detected: self.cycles_detected.clone(),
        }
    }
}

impl TypeNode {
    /// Create a new type node
    pub fn new(id: String, kind: String, depth: u32) -> Self {
        Self {
            id,
            kind,
            fields: Vec::new(),
            variants: Vec::new(),
            items: Vec::new(),
            generic_params: Vec::new(),
            depth,
            field_count: None,
            variant_count: None,
        }
    }

    /// Add a field to this type
    pub fn add_field(&mut self, field: FieldInfo) {
        self.fields.push(field);
    }

    /// Add a variant to this type
    pub fn add_variant(&mut self, variant: VariantInfo) {
        self.variants.push(variant);
    }

    /// Add an item to this module
    pub fn add_item(&mut self, item: ModuleItemInfo) {
        self.items.push(item);
    }

    /// Add a generic parameter
    pub fn add_generic_param(&mut self, param: String) {
        self.generic_params.push(param);
    }

    /// Convert to minimal representation (counts only, no details)
    pub fn to_minimal(&self) -> Self {
        Self {
            id: self.id.clone(),
            kind: self.kind.clone(),
            fields: Vec::new(),         // Omit field details
            variants: Vec::new(),       // Omit variant details
            items: Vec::new(),          // Omit item details
            generic_params: Vec::new(), // Omit generic details
            depth: self.depth,
            field_count: Some(self.fields.len()),
            variant_count: Some(self.variants.len()),
        }
    }

    /// Estimate tokens for this node
    pub fn estimate_tokens(&self) -> usize {
        match serde_json::to_string(self) {
            Ok(json) => json.len() / 4,
            Err(_) => 50, // Default estimate
        }
    }
}

impl FieldInfo {
    /// Create a new field info
    pub fn new(name: String, type_path: String, is_optional: bool) -> Self {
        Self {
            name,
            type_path,
            is_optional,
            nested_type_id: None,
        }
    }

    /// Set the nested type ID for linking
    pub fn with_nested_type(mut self, id: String) -> Self {
        self.nested_type_id = Some(id);
        self
    }
}

impl VariantInfo {
    /// Create a new variant info
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            discriminant: None,
        }
    }

    /// Add a field to this variant
    pub fn add_field(&mut self, field: FieldInfo) {
        self.fields.push(field);
    }

    /// Set the discriminant value
    pub fn with_discriminant(mut self, discriminant: String) -> Self {
        self.discriminant = Some(discriminant);
        self
    }
}

impl ModuleItemInfo {
    /// Create a new module item info
    pub fn new(name: String, kind: String, path: String) -> Self {
        Self { name, kind, path }
    }
}
