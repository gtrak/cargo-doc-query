// JSON output schema types for query responses

use serde::ser::SerializeSeq;
use serde::Serialize;

/// Top-level query response
#[derive(Serialize, Debug)]
pub struct QueryResponse {
    pub query: String,
    pub matches: Vec<QueryMatch>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

/// Individual match from a query
#[derive(Serialize, Debug)]
pub struct QueryMatch {
    pub crate_name: String,
    pub version: String,
    pub fully_qualified_path: String,
    pub kind: String, // "type", "trait", "module", etc.
    pub content: QueryContent,
}

/// Content of a match - type, trait, or module result
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum QueryContent {
    Type(TypeResult),
    Trait(TraitResult),
    Module(ModuleResult),
}

/// Result of a type query
#[derive(Serialize, Debug)]
pub struct TypeResult {
    pub kind: String, // "struct", "enum", "type alias", etc.
    pub methods: Vec<MethodOutput>,
    pub trait_implementations: Vec<TraitImplOutput>,
}

/// Result of a trait query
#[derive(Serialize, Debug)]
pub struct TraitResult {
    pub name: String,
    pub path: String,
    pub methods: Vec<MethodOutput>,
    pub associated_types: Vec<AssociatedTypeOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub provided_methods: Vec<String>,
}

/// Result of a module query
#[derive(Serialize, Debug)]
pub struct ModuleResult {
    /// Module items (types, traits, functions, etc.)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ModuleItem>,
    /// Submodules
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub submodules: Vec<String>,
}

/// Item within a module
#[derive(Serialize, Debug)]
pub struct ModuleItem {
    pub name: String,
    pub kind: String, // "struct", "enum", "trait", "function", "type", etc.
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>, // For functions
}

/// Method output for queries
#[derive(Serialize, Debug)]
pub struct MethodOutput {
    pub name: String,
    pub signature: String,
    pub return_type: String,
    pub visibility: String,
    pub is_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub is_trait_method: bool,
}

/// Helper function for skipping false values
pub(crate) fn is_false(b: &bool) -> bool {
    !b
}

/// Associated type output for trait queries
#[derive(Serialize, Debug, Clone)]
pub struct AssociatedTypeOutput {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Trait implementation output
#[derive(Serialize, Debug)]
pub struct TraitImplOutput {
    pub trait_name: String,
    pub trait_path: String,
    pub methods: Vec<MethodOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub provided_methods: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_args: Option<String>,
}

impl QueryResponse {
    /// Create a new query response
    pub fn new(query: String) -> Self {
        Self {
            query,
            matches: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add a match to the response
    pub fn add_match(&mut self, match_: QueryMatch) {
        self.matches.push(match_);
    }

    /// Add an error to the response
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Convert to minimal representation
    pub fn to_minimal(&self) -> Self {
        Self {
            query: self.query.clone(),
            matches: self.matches.iter().map(|m| m.to_minimal()).collect(),
            errors: self.errors.clone(),
        }
    }

    /// Estimate token count (JSON length / 4)
    pub fn estimate_tokens(&self) -> usize {
        match serde_json::to_string(self) {
            Ok(json) => json.len() / 4,
            Err(_) => 0,
        }
    }
}

impl QueryMatch {
    /// Create a new query match
    pub fn new(
        crate_name: String,
        version: String,
        fully_qualified_path: String,
        kind: String,
        content: QueryContent,
    ) -> Self {
        Self {
            crate_name,
            version,
            fully_qualified_path,
            kind,
            content,
        }
    }

    /// Convert to minimal representation
    pub fn to_minimal(&self) -> Self {
        Self {
            crate_name: self.crate_name.clone(),
            version: self.version.clone(),
            fully_qualified_path: self.fully_qualified_path.clone(),
            kind: self.kind.clone(),
            content: match &self.content {
                QueryContent::Type(t) => QueryContent::Type(t.to_minimal()),
                QueryContent::Trait(t) => QueryContent::Trait(t.to_minimal()),
                QueryContent::Module(m) => QueryContent::Module(m.to_minimal()),
            },
        }
    }
}

impl TypeResult {
    /// Create a new type result
    pub fn new(kind: String) -> Self {
        Self {
            kind,
            methods: Vec::new(),
            trait_implementations: Vec::new(),
        }
    }

    /// Add a method to the type result
    pub fn add_method(&mut self, method: MethodOutput) {
        self.methods.push(method);
    }

    /// Add a trait implementation to the type result
    pub fn add_trait_impl(&mut self, trait_impl: TraitImplOutput) {
        self.trait_implementations.push(trait_impl);
    }

    /// Convert to minimal representation
    pub fn to_minimal(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            methods: self.methods.iter().map(|m| m.to_minimal()).collect(),
            trait_implementations: self
                .trait_implementations
                .iter()
                .map(|t| t.to_minimal())
                .collect(),
        }
    }
}

impl TraitResult {
    /// Create a new trait result
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            methods: Vec::new(),
            associated_types: Vec::new(),
            provided_methods: Vec::new(),
        }
    }

    /// Add a method to the trait result
    pub fn add_method(&mut self, method: MethodOutput) {
        self.methods.push(method);
    }

    /// Add an associated type to the trait result
    pub fn add_associated_type(&mut self, assoc_type: AssociatedTypeOutput) {
        self.associated_types.push(assoc_type);
    }

    /// Add a provided method to the trait result
    pub fn add_provided_method(&mut self, method: String) {
        self.provided_methods.push(method);
    }

    /// Convert to minimal representation
    pub fn to_minimal(&self) -> Self {
        Self {
            name: self.name.clone(),
            path: self.path.clone(),
            methods: self.methods.iter().map(|m| m.to_minimal()).collect(),
            associated_types: self.associated_types.clone(),
            provided_methods: Vec::new(), // Omit provided methods
        }
    }
}

impl MethodOutput {
    /// Create a new method output
    pub fn new(
        name: String,
        signature: String,
        return_type: String,
        visibility: String,
        is_public: bool,
    ) -> Self {
        Self {
            name,
            signature,
            return_type,
            visibility,
            is_public,
            docs: None,
            is_trait_method: false,
        }
    }

    /// Set the documentation string
    pub fn with_docs(mut self, docs: Option<String>) -> Self {
        self.docs = docs;
        self
    }

    /// Set whether this is a trait method
    pub fn with_is_trait_method(mut self, is_trait_method: bool) -> Self {
        self.is_trait_method = is_trait_method;
        self
    }

    /// Convert to minimal representation
    pub fn to_minimal(&self) -> Self {
        Self {
            name: self.name.clone(),
            signature: self.signature.clone(),
            return_type: self.return_type.clone(),
            visibility: self.visibility.clone(),
            is_public: self.is_public,
            docs: None,                                                    // Omit docs
            is_trait_method: self.is_trait_method && self.is_trait_method, // Only keep if true
        }
    }
}

impl AssociatedTypeOutput {
    /// Create a new associated type output
    pub fn new(name: String) -> Self {
        Self {
            name,
            bounds: None,
            default: None,
        }
    }

    /// Set the bounds
    pub fn with_bounds(mut self, bounds: Option<String>) -> Self {
        self.bounds = bounds;
        self
    }

    /// Set the default value
    pub fn with_default(mut self, default: Option<String>) -> Self {
        self.default = default;
        self
    }
}

impl ModuleResult {
    /// Create a new module result
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            submodules: Vec::new(),
        }
    }

    /// Add an item to the module
    pub fn add_item(&mut self, item: ModuleItem) {
        self.items.push(item);
    }

    /// Add a submodule
    pub fn add_submodule(&mut self, name: String) {
        self.submodules.push(name);
    }

    /// Convert to minimal representation
    pub fn to_minimal(&self) -> Self {
        Self {
            items: self.items.iter().map(|i| i.to_minimal()).collect(),
            submodules: self.submodules.clone(),
        }
    }
}

impl ModuleItem {
    /// Create a new module item
    pub fn new(name: String, kind: String, path: String) -> Self {
        Self {
            name,
            kind,
            path,
            signature: None,
        }
    }

    /// Set the signature (for functions)
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Convert to minimal representation
    pub fn to_minimal(&self) -> Self {
        Self {
            name: self.name.clone(),
            kind: self.kind.clone(),
            path: self.path.clone(),
            signature: None, // Omit signatures in minimal mode
        }
    }
}

impl TraitImplOutput {
    /// Create a new trait implementation output
    pub fn new(trait_name: String, trait_path: String) -> Self {
        Self {
            trait_name,
            trait_path,
            methods: Vec::new(),
            provided_methods: Vec::new(),
            generic_args: None,
        }
    }

    /// Add a method to the trait implementation
    pub fn add_method(&mut self, method: MethodOutput) {
        self.methods.push(method);
    }

    /// Add a provided method to the trait implementation
    pub fn add_provided_method(&mut self, method: String) {
        self.provided_methods.push(method);
    }

    /// Set the generic arguments
    pub fn with_generic_args(mut self, generic_args: Option<String>) -> Self {
        self.generic_args = generic_args;
        self
    }

    /// Convert to minimal representation
    pub fn to_minimal(&self) -> Self {
        Self {
            trait_name: self.trait_name.clone(),
            trait_path: self.trait_path.clone(),
            methods: self.methods.iter().map(|m| m.to_minimal()).collect(),
            provided_methods: Vec::new(), // Omit provided methods
            generic_args: None,           // Omit generic args
        }
    }
}
