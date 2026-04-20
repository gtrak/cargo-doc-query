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
    /// Crate name for filtering
    pub crate_name: String,
    /// Visibility for filtering
    pub visibility: String,
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
    /// Whether the item is deprecated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_deprecated: Option<bool>,
    /// Deprecation note/replacement hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation_note: Option<String>,
    /// Key attributes: #[must_use], #[non_exhaustive]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<String>,
    /// Function modifier: const
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_const: Option<bool>,
    /// Function modifier: async
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,
    /// Function modifier: unsafe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_unsafe: Option<bool>,
    /// Function ABI (None for Rust ABI)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<String>,
    /// Documentation comments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

/// Minimal version of TypeNode for reduced output
#[derive(Serialize, Debug, Clone)]
pub struct MinimalTypeNode {
    /// Fully qualified path
    pub id: String,
    /// Type kind
    pub kind: String,
    /// Crate name for filtering
    pub crate_name: String,
    /// Visibility for filtering
    pub visibility: String,
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

/// Re-export the shared ModuleItem type from types module.
/// 
/// This is an alias for `crate::types::ModuleItem` to maintain backward
/// compatibility with existing code that references `ModuleItemInfo`.
pub use super::ModuleItem as ModuleItemInfo;

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
        Self::with_crate_visibility(id, kind, depth, String::new(), String::new())
    }

    /// Create a new type node with crate name and visibility
    pub fn with_crate_visibility(
        id: String,
        kind: String,
        depth: u32,
        crate_name: String,
        visibility: String,
    ) -> Self {
        Self {
            id,
            kind,
            crate_name,
            visibility,
            fields: Vec::new(),
            variants: Vec::new(),
            items: Vec::new(),
            generic_params: Vec::new(),
            depth,
            field_count: None,
            variant_count: None,
            is_deprecated: None,
            deprecation_note: None,
            attributes: Vec::new(),
            is_const: None,
            is_async: None,
            is_unsafe: None,
            abi: None,
            docs: None,
        }
    }

    /// Set deprecation information
    pub fn with_deprecation(mut self, is_deprecated: bool, note: Option<String>) -> Self {
        self.is_deprecated = Some(is_deprecated);
        self.deprecation_note = note;
        self
    }

    /// Set semantic attributes
    pub fn with_attributes(mut self, attrs: Vec<String>) -> Self {
        self.attributes = attrs;
        self
    }

    /// Set function modifiers
    pub fn with_function_modifiers(
        mut self,
        is_const: bool,
        is_async: bool,
        is_unsafe: bool,
        abi: Option<String>,
    ) -> Self {
        self.is_const = Some(is_const);
        self.is_async = Some(is_async);
        self.is_unsafe = Some(is_unsafe);
        self.abi = abi;
        self
    }

    /// Set documentation from rustdoc Item::docs
    pub fn with_docs(mut self, docs: Option<String>) -> Self {
        self.docs = docs.map(|s| s.trim().to_string());
        self
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
            crate_name: self.crate_name.clone(),
            visibility: self.visibility.clone(),
            fields: Vec::new(),         // Omit field details
            variants: Vec::new(),       // Omit variant details
            items: self.items.clone(),     // Keep module contents in minimal
            generic_params: self.generic_params.clone(), // Keep generics in minimal
            depth: self.depth,
            field_count: Some(self.fields.len()),
            variant_count: Some(self.variants.len()),
            is_deprecated: None,
            deprecation_note: None,
            attributes: Vec::new(),
            is_const: None,
            is_async: None,
            is_unsafe: None,
            abi: None,
            docs: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_node_creation() {
        let node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0);
        assert_eq!(node.id, "test::Type");
        assert_eq!(node.kind, "struct");
        assert_eq!(node.depth, 0);
        assert!(node.fields.is_empty());
        assert!(node.variants.is_empty());
        assert!(node.items.is_empty());
        assert!(node.generic_params.is_empty());
    }

    #[test]
    fn test_type_node_with_field() {
        let mut node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0);
        let field = FieldInfo::new("x".to_string(), "i32".to_string(), false);
        node.add_field(field);

        assert_eq!(node.fields.len(), 1);
        assert_eq!(node.fields[0].name, "x");
        assert_eq!(node.fields[0].type_path, "i32");
        assert!(!node.fields[0].is_optional);
        assert!(node.fields[0].nested_type_id.is_none());
    }

    #[test]
    fn test_type_node_with_variant() {
        let mut node = TypeNode::new("test::Enum".to_string(), "enum".to_string(), 0);
        let variant = VariantInfo::new("Variant1".to_string());
        node.add_variant(variant);

        assert_eq!(node.variants.len(), 1);
        assert_eq!(node.variants[0].name, "Variant1");
        assert!(node.variants[0].fields.is_empty());
        assert!(node.variants[0].discriminant.is_none());
    }

    #[test]
    fn test_type_node_with_generic_param() {
        let mut node = TypeNode::new("test::Type".to_string(), "type".to_string(), 0);
        node.add_generic_param("T = String".to_string());

        assert_eq!(node.generic_params.len(), 1);
        assert_eq!(node.generic_params[0], "T = String");
    }

    #[test]
    fn test_type_node_add_item() {
        let mut node = TypeNode::new("test::Module".to_string(), "module".to_string(), 0);
        let item = ModuleItemInfo::new(
            "Func".to_string(),
            "function".to_string(),
            "test::Func".to_string(),
        );
        node.add_item(item);

        assert_eq!(node.items.len(), 1);
        assert_eq!(node.items[0].name, "Func");
        assert_eq!(node.items[0].kind, "function");
    }

    #[test]
    fn test_type_node_estimate_tokens() {
        let mut node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0);
        node.add_field(FieldInfo::new("x".to_string(), "i32".to_string(), false));
        node.add_field(FieldInfo::new("y".to_string(), "f64".to_string(), true));

        let tokens = node.estimate_tokens();
        assert!(tokens > 0);
    }

    #[test]
    fn test_type_node_to_minimal() {
        let mut node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0);
        node.add_field(FieldInfo::new("x".to_string(), "i32".to_string(), false));

        let minimal = node.to_minimal();

        assert_eq!(minimal.id, node.id);
        assert_eq!(minimal.kind, node.kind);
        assert_eq!(minimal.depth, node.depth);
        assert_eq!(minimal.field_count, Some(1));
        assert_eq!(minimal.variant_count, Some(0));
        assert!(minimal.fields.is_empty());
        assert!(minimal.generic_params.is_empty());
    }

    #[test]
    fn test_type_graph_creation() {
        let graph = TypeGraph::new("test".to_string(), 10);
        assert_eq!(graph.root, "test");
        assert_eq!(graph.depth_limit, 10);
        assert!(graph.nodes.is_empty());
        assert!(graph.cycles_detected.is_empty());
    }

    #[test]
    fn test_type_graph_add_node() {
        let mut graph = TypeGraph::new("test".to_string(), 10);
        let node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0);
        let node_id = graph.add_node(node);

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, "test::Type");
        assert_eq!(node_id, "test::Type");
    }

    #[test]
    fn test_type_graph_estimate_tokens() {
        let mut graph = TypeGraph::new("test".to_string(), 10);
        graph.add_node(TypeNode::new(
            "test::Type".to_string(),
            "struct".to_string(),
            0,
        ));

        let tokens = graph.estimate_tokens();
        assert!(tokens > 0);
    }

    #[test]
    fn test_type_graph_to_minimal() {
        let mut graph = TypeGraph::new("test".to_string(), 10);
        let mut node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0);
        node.add_field(FieldInfo::new("x".to_string(), "i32".to_string(), false));
        graph.add_node(node);

        let minimal = graph.to_minimal();

        assert_eq!(minimal.root, graph.root);
        assert_eq!(minimal.depth_limit, graph.depth_limit);
        assert_eq!(minimal.nodes.len(), 1);
        assert!(minimal.nodes[0].fields.is_empty());
    }

    #[test]
    fn test_expansion_result_creation() {
        let graph = TypeGraph::new("test".to_string(), 10);
        let result = ExpansionResult::new(graph);

        assert_eq!(result.graph.root, "test");
        assert_eq!(result.graph.nodes.len(), 0);
        assert!(result.cycles_detected.is_empty());
        assert_eq!(result.token_count, 10);
        assert!(!result.budget_exceeded);
        assert!(result.truncated_paths.is_empty());
    }

    #[test]
    fn test_expansion_result_with_truncation() {
        let graph = TypeGraph::new("test".to_string(), 10);
        let mut result = ExpansionResult::new(graph);
        result = result.with_truncation(vec!["path1".to_string(), "path2".to_string()]);

        assert_eq!(result.truncated_paths.len(), 2);
        assert!(result.budget_exceeded);
    }

    #[test]
    fn test_expansion_result_update_token_count() {
        let mut graph = TypeGraph::new("test".to_string(), 10);
        graph.add_node(TypeNode::new(
            "test::Type".to_string(),
            "struct".to_string(),
            0,
        ));

        let mut result = ExpansionResult::new(graph);
        let initial_count = result.token_count;

        result.update_token_count();
        assert_eq!(result.token_count, initial_count);
    }

    #[test]
    fn test_field_info_creation() {
        let field = FieldInfo::new("x".to_string(), "i32".to_string(), false);
        assert_eq!(field.name, "x");
        assert_eq!(field.type_path, "i32");
        assert!(!field.is_optional);
        assert!(field.nested_type_id.is_none());
    }

    #[test]
    fn test_field_info_with_nested_type() {
        let field = FieldInfo::new("x".to_string(), "HashMap".to_string(), true)
            .with_nested_type("HashMap".to_string());

        assert_eq!(field.nested_type_id, Some("HashMap".to_string()));
    }

    #[test]
    fn test_variant_info_creation() {
        let variant = VariantInfo::new("Variant1".to_string());
        assert_eq!(variant.name, "Variant1");
        assert!(variant.fields.is_empty());
        assert!(variant.discriminant.is_none());
    }

    #[test]
    fn test_variant_info_with_discriminant() {
        let variant = VariantInfo::new("Variant1".to_string()).with_discriminant("1".to_string());
        assert_eq!(variant.discriminant, Some("1".to_string()));
    }

    #[test]
    fn test_variant_info_add_field() {
        let mut variant = VariantInfo::new("Variant1".to_string());
        let field = FieldInfo::new("x".to_string(), "i32".to_string(), false);
        variant.add_field(field);

        assert_eq!(variant.fields.len(), 1);
        assert_eq!(variant.fields[0].name, "x");
    }

    #[test]
    fn test_module_item_info_creation() {
        let item = ModuleItemInfo::new(
            "Function".to_string(),
            "function".to_string(),
            "test::Function".to_string(),
        );
        assert_eq!(item.name, "Function");
        assert_eq!(item.kind, "function");
        assert_eq!(item.path, "test::Function");
    }

    #[test]
    fn test_token_config_builder() {
        let config = TokenConfig::new()
            .with_budget(Some(100))
            .with_minimal(true)
            .with_threshold(0.5);

        assert_eq!(config.budget, Some(100));
        assert!(config.minimal_mode);
        assert_eq!(config.warning_threshold, 0.5);
    }

    #[test]
    fn test_token_config_chaining() {
        let config = TokenConfig::default()
            .with_budget(Some(200))
            .with_minimal(false)
            .with_threshold(0.7);

        assert_eq!(config.budget, Some(200));
        assert!(!config.minimal_mode);
        assert_eq!(config.warning_threshold, 0.7);
    }

    #[test]
    fn test_estimate_tokens_valid_json() {
        let mut graph = TypeGraph::new("test".to_string(), 10);
        graph.add_node(TypeNode::new(
            "test::Type".to_string(),
            "struct".to_string(),
            0,
        ));

        let tokens = graph.estimate_tokens();
        assert!(tokens > 0);
    }

    // =========================================================================
    // Backward Compatibility Tests
    // =========================================================================

    #[test]
    fn test_typenode_optional_fields_omitted() {
        let node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0);
        let json = serde_json::to_string(&node).unwrap();

        // Should not contain the new optional field keys when not set
        assert!(!json.contains("\"is_deprecated\""));
        assert!(!json.contains("\"deprecation_note\""));
        assert!(!json.contains("\"attributes\""));
        assert!(!json.contains("\"is_const\""));
        assert!(!json.contains("\"is_async\""));
        assert!(!json.contains("\"is_unsafe\""));
        assert!(!json.contains("\"abi\""));
    }

    #[test]
    fn test_typenode_optional_fields_present_when_set() {
        let node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0)
            .with_deprecation(true, Some("Use NewType instead".to_string()))
            .with_attributes(vec!["#[must_use]".to_string()])
            .with_function_modifiers(true, false, true, Some("C".to_string()));
        let json = serde_json::to_string(&node).unwrap();

        // Should contain the fields when set
        assert!(json.contains("\"is_deprecated\":true"));
        assert!(json.contains("\"deprecation_note\":\"Use NewType instead\""));
        assert!(json.contains("\"attributes\""));
        assert!(json.contains("\"is_const\":true"));
        assert!(json.contains("\"is_unsafe\":true"));
        assert!(json.contains("\"abi\":\"C\""));
    }

    #[test]
    fn test_typenode_minimal_smaller() {
        let mut node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0);
        node.add_field(FieldInfo::new("x".to_string(), "i32".to_string(), false));
        node.add_field(FieldInfo::new("y".to_string(), "f64".to_string(), true));
        node.add_generic_param("T".to_string());
        node.is_deprecated = Some(true);
        node.attributes = vec!["#[must_use]".to_string()];
        node.is_const = Some(true);

        let full_json = serde_json::to_string(&node).unwrap();
        let minimal = node.to_minimal();
        let minimal_json = serde_json::to_string(&minimal).unwrap();

        // Minimal should be smaller
        assert!(
            minimal_json.len() < full_json.len(),
            "Minimal JSON should be smaller: {} < {}",
            minimal_json.len(),
            full_json.len()
        );

        // Minimal should have counts but no details
        assert!(!minimal_json.contains("\"x\"")); // No field names
        assert!(!minimal_json.contains("\"y\""));
        // Minimal now keeps generic params per user request
        assert!(!minimal_json.contains("\"is_deprecated\"")); // Cleared in minimal
        assert!(!minimal_json.contains("\"attributes\""));
    }

    #[test]
    fn test_module_item_info_backward_compat() {
        let item = ModuleItemInfo::new(
            "func".to_string(),
            "function".to_string(),
            "test::func".to_string(),
        )
        .with_signature("fn() -> i32".to_string())
        .with_visibility("pub")
        .with_generics("<T>".to_string())
        .with_function_modifiers(false, true, false, None);

        let json = serde_json::to_string(&item).unwrap();

        // Should contain all new fields when set
        assert!(json.contains("\"visibility\":\"pub\""));
        assert!(json.contains("\"generics\":\"<T>\""));
        assert!(json.contains("\"is_async\":true"));

        // Old code without new fields should still deserialize
        #[derive(serde::Deserialize)]
        struct OldItem {
            name: String,
            kind: String,
            path: String,
        }

        let old: OldItem = serde_json::from_str(&json).unwrap();
        assert_eq!(old.name, "func");
        assert_eq!(old.kind, "function");
        assert_eq!(old.path, "test::func");
    }

    #[test]
    fn test_module_item_info_minimal() {
        let item = ModuleItemInfo::new(
            "func".to_string(),
            "function".to_string(),
            "test::func".to_string(),
        )
        .with_signature("fn() -> i32".to_string())
        .with_visibility("pub")
        .with_generics("<T>".to_string())
        .with_function_modifiers(true, false, false, Some("C".to_string()));

        let full_json = serde_json::to_string(&item).unwrap();
        let minimal = item.to_minimal();
        let minimal_json = serde_json::to_string(&minimal).unwrap();

        // Minimal should be smaller
        assert!(
            minimal_json.len() < full_json.len(),
            "Minimal JSON should be smaller: {} < {}",
            minimal_json.len(),
            full_json.len()
        );

        // Minimal should not contain optional fields
        assert!(!minimal_json.contains("\"signature\""));
        assert!(!minimal_json.contains("\"visibility\""));
        assert!(!minimal_json.contains("\"generics\""));
        assert!(!minimal_json.contains("\"is_const\""));
    }

    #[test]
    fn test_module_item_info_omits_none_fields() {
        let item = ModuleItemInfo::new(
            "type".to_string(),
            "type".to_string(),
            "test::Type".to_string(),
        );
        let json = serde_json::to_string(&item).unwrap();

        // Should not contain optional fields when None
        assert!(!json.contains("\"signature\""));
        assert!(!json.contains("\"visibility\""));
        assert!(!json.contains("\"generics\""));
        assert!(!json.contains("\"is_const\""));
        assert!(!json.contains("\"is_async\""));
        assert!(!json.contains("\"is_unsafe\""));
        assert!(!json.contains("\"abi\""));

        // Should contain required fields
        assert!(json.contains("\"name\":\"type\""));
        assert!(json.contains("\"kind\":\"type\""));
        assert!(json.contains("\"path\":\"test::Type\""));
    }

    #[test]
    fn test_type_graph_nodes_minimal() {
        let mut graph = TypeGraph::new("test".to_string(), 10);
        let mut node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0);
        node.add_field(FieldInfo::new("x".to_string(), "i32".to_string(), false));
        node.add_generic_param("T".to_string());
        node.is_deprecated = Some(true);
        graph.add_node(node);

        let full_json = serde_json::to_string(&graph).unwrap();
        let minimal = graph.to_minimal();
        let minimal_json = serde_json::to_string(&minimal).unwrap();

        // Minimal graph should be smaller
        assert!(
            minimal_json.len() < full_json.len(),
            "Minimal graph JSON should be smaller: {} < {}",
            minimal_json.len(),
            full_json.len()
        );

        // Minimal should have field_count but no fields
        assert!(minimal_json.contains("\"field_count\":1"));
        assert!(!minimal_json.contains("\"is_deprecated\""));
    }

    #[test]
    fn test_builder_chaining() {
        let node = TypeNode::new("test::Type".to_string(), "struct".to_string(), 0)
            .with_deprecation(true, Some("old".to_string()))
            .with_attributes(vec!["#[must_use]".to_string()])
            .with_function_modifiers(false, false, false, None);

        assert_eq!(node.is_deprecated, Some(true));
        assert_eq!(node.deprecation_note, Some("old".to_string()));
        assert_eq!(node.attributes, vec!["#[must_use]"]);
        assert_eq!(node.is_const, Some(false));

        let item = ModuleItemInfo::new(
            "func".to_string(),
            "function".to_string(),
            "test::func".to_string(),
        )
        .with_signature("fn()".to_string())
        .with_visibility("pub")
        .with_generics("<T>".to_string())
        .with_function_modifiers(true, true, false, Some("C".to_string()));

        assert_eq!(item.visibility, Some("pub".to_string()));
        assert_eq!(item.generics, Some("<T>".to_string()));
        assert_eq!(item.is_const, Some(true));
        assert_eq!(item.is_async, Some(true));
        assert_eq!(item.abi, Some("C".to_string()));
    }
}
