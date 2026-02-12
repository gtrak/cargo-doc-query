// Expansion output types for type hierarchy exploration

use serde::Serialize;

/// Top-level expansion result containing type graph and metadata
#[derive(Serialize, Debug, Default)]
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
    /// Type kind: struct, enum, primitive, generic, etc.
    pub kind: String,
    /// Fields (for struct-like types)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldInfo>,
    /// Variants (for enums)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<VariantInfo>,
    /// Generic parameters
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub generic_params: Vec<String>,
    /// Depth from root type
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
    pub fn add_node(&mut self, mut node: TypeNode) -> String {
        let id = node.id.clone();
        self.nodes.push(node);
        id
    }

    /// Record a detected cycle
    pub fn add_cycle(&mut self, path: String) {
        self.cycles_detected.push(path);
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
            generic_params: Vec::new(),
            depth,
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

    /// Add a generic parameter
    pub fn add_generic_param(&mut self, param: String) {
        self.generic_params.push(param);
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
