// JSON output schema types for query responses

use serde::ser::SerializeSeq;
use serde::Serialize;

/// Top-level query response
#[derive(Serialize, Debug, Clone)]
pub struct QueryResponse {
    pub query: String,
    pub matches: Vec<QueryMatch>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

/// Individual match from a query
#[derive(Serialize, Debug, Clone)]
pub struct QueryMatch {
    pub crate_name: String,
    pub version: String,
    pub fully_qualified_path: String,
    pub kind: String,
    pub content: QueryContent,

    // New fields for FIELD-01..04
    /// Visibility modifier (FIELD-01)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    /// Generic parameters in Rust syntax (FIELD-03)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generics: Option<String>,

    /// Whether the item is deprecated (FIELD-02)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_deprecated: Option<bool>,

    /// Deprecation note/replacement hint (FIELD-02)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation_note: Option<String>,

    /// Key attributes: #[must_use], #[non_exhaustive] (FIELD-04)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<String>,
}

/// Content of a match - type, trait, or module result
#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum QueryContent {
    Type(TypeResult),
    Trait(TraitResult),
    Module(ModuleResult),
}

/// Result of a type query
#[derive(Serialize, Debug, Clone)]
pub struct TypeResult {
    pub kind: String, // "struct", "enum", "type alias", etc.
    pub methods: Vec<MethodOutput>,
    pub trait_implementations: Vec<TraitImplOutput>,
}

/// Result of a trait query
#[derive(Serialize, Debug, Clone)]
pub struct TraitResult {
    pub name: String,
    pub path: String,
    pub methods: Vec<MethodOutput>,
    pub associated_types: Vec<AssociatedTypeOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub provided_methods: Vec<String>,
}

/// Result of a module query
#[derive(Serialize, Debug, Clone)]
pub struct ModuleResult {
    /// Module items (types, traits, functions, etc.)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ModuleItem>,
    /// Submodules
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub submodules: Vec<String>,
}

/// Item within a module
#[derive(Serialize, Debug, Clone)]
pub struct ModuleItem {
    pub name: String,
    pub kind: String, // "struct", "enum", "trait", "function", "type", etc.
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>, // For functions
}

/// Method output for queries
#[derive(Serialize, Debug, Clone)]
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

    // New fields for FIELD-05
    /// Whether the function is const (FIELD-05)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_const: Option<bool>,

    /// Whether the function is async (FIELD-05)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,

    /// Whether the function is unsafe (FIELD-05)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_unsafe: Option<bool>,

    /// ABI string for non-Rust ABIs (FIELD-05)
    /// Only present when ABI is not "Rust"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<String>,
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

/// Trait implementation output for type queries
#[derive(Serialize, Debug, Clone)]
pub struct TraitImplOutput {
    pub trait_name: String,
    pub trait_path: String,
    pub methods: Vec<MethodOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub provided_methods: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_args: Option<String>,
}

/// Helper function for skipping false values
pub(crate) fn is_false(b: &bool) -> bool {
    !b
}

/// Helper function for skipping empty vectors
pub(crate) fn is_empty<T>(v: &[T]) -> bool {
    v.is_empty()
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
            visibility: None,
            generics: None,
            is_deprecated: None,
            deprecation_note: None,
            attributes: Vec::new(),
        }
    }

    /// Set the visibility
    pub fn with_visibility(mut self, vis: impl Into<String>) -> Self {
        self.visibility = Some(vis.into());
        self
    }

    /// Set the generics
    pub fn with_generics(mut self, generics: impl Into<String>) -> Self {
        self.generics = Some(generics.into());
        self
    }

    /// Set the deprecation information
    pub fn with_deprecation(mut self, note: Option<String>) -> Self {
        self.is_deprecated = Some(true);
        self.deprecation_note = note;
        self
    }

    /// Set the attributes
    pub fn with_attributes(mut self, attrs: Vec<String>) -> Self {
        self.attributes = attrs;
        self
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
            visibility: None, // FIELD-06: omitted in minimal mode
            generics: None,
            is_deprecated: None,
            deprecation_note: None,
            attributes: Vec::new(),
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
            is_const: None,
            is_async: None,
            is_unsafe: None,
            abi: None,
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

    /// Set whether this is a const function
    pub fn with_is_const(mut self, is_const: bool) -> Self {
        self.is_const = Some(is_const);
        self
    }

    /// Set whether this is an async function
    pub fn with_is_async(mut self, is_async: bool) -> Self {
        self.is_async = Some(is_async);
        self
    }

    /// Set whether this is an unsafe function
    pub fn with_is_unsafe(mut self, is_unsafe: bool) -> Self {
        self.is_unsafe = Some(is_unsafe);
        self
    }

    /// Set the ABI string
    pub fn with_abi(mut self, abi: Option<String>) -> Self {
        self.abi = abi;
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
            is_const: None, // FIELD-06: omitted in minimal mode
            is_async: None,
            is_unsafe: None,
            abi: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_response_creation() {
        let response = QueryResponse::new("std::vec::Vec".to_string());
        assert_eq!(response.query, "std::vec::Vec");
        assert_eq!(response.matches.len(), 0);
        assert_eq!(response.errors.len(), 0);
    }

    #[test]
    fn test_query_response_add_match() {
        let mut response = QueryResponse::new("test".to_string());
        let match_ = QueryMatch::new(
            "std".to_string(),
            "1.0.0".to_string(),
            "std::Vec".to_string(),
            "type".to_string(),
            QueryContent::Module(ModuleResult::new()),
        );
        response.add_match(match_);
        assert_eq!(response.matches.len(), 1);
    }

    #[test]
    fn test_query_response_add_error() {
        let mut response = QueryResponse::new("test".to_string());
        response.add_error("test error".to_string());
        assert_eq!(response.errors.len(), 1);
        assert_eq!(response.errors[0], "test error");
    }

    #[test]
    fn test_query_response_estimate_tokens() {
        let mut response = QueryResponse::new("test".to_string());
        response.add_match(QueryMatch::new(
            "std".to_string(),
            "1.0.0".to_string(),
            "std::Vec".to_string(),
            "type".to_string(),
            QueryContent::Module(ModuleResult::new()),
        ));
        let tokens = response.estimate_tokens();
        assert!(tokens > 0);
    }

    #[test]
    fn test_query_response_to_minimal() {
        let mut response = QueryResponse::new("test".to_string());
        response.add_match(QueryMatch::new(
            "std".to_string(),
            "1.0.0".to_string(),
            "std::Vec".to_string(),
            "type".to_string(),
            QueryContent::Module(ModuleResult::new()),
        ));

        let minimal = response.to_minimal();
        assert_eq!(minimal.query, response.query);
        assert_eq!(minimal.matches.len(), response.matches.len());
    }

    #[test]
    fn test_query_match_creation() {
        let match_ = QueryMatch::new(
            "std".to_string(),
            "1.0.0".to_string(),
            "std::Vec".to_string(),
            "type".to_string(),
            QueryContent::Module(ModuleResult::new()),
        );
        assert_eq!(match_.crate_name, "std");
        assert_eq!(match_.version, "1.0.0");
        assert_eq!(match_.fully_qualified_path, "std::Vec");
        assert_eq!(match_.kind, "type");
    }

    #[test]
    fn test_query_match_to_minimal() {
        let match_ = QueryMatch::new(
            "std".to_string(),
            "1.0.0".to_string(),
            "std::Vec".to_string(),
            "type".to_string(),
            QueryContent::Module(ModuleResult::new()),
        );

        let minimal = match_.to_minimal();
        assert_eq!(minimal.crate_name, match_.crate_name);
        assert_eq!(minimal.version, match_.version);
    }

    #[test]
    fn test_type_result_creation() {
        let result = TypeResult::new("struct".to_string());
        assert_eq!(result.kind, "struct");
        assert_eq!(result.methods.len(), 0);
        assert_eq!(result.trait_implementations.len(), 0);
    }

    #[test]
    fn test_type_result_add_method() {
        let mut result = TypeResult::new("type".to_string());
        result.add_method(MethodOutput::new(
            "new".to_string(),
            "fn()".to_string(),
            "()".to_string(),
            "pub".to_string(),
            true,
        ));
        assert_eq!(result.methods.len(), 1);
    }

    #[test]
    fn test_type_result_add_trait_impl() {
        let mut result = TypeResult::new("type".to_string());
        result.add_trait_impl(TraitImplOutput::new(
            "Display".to_string(),
            "std::fmt::Display".to_string(),
        ));
        assert_eq!(result.trait_implementations.len(), 1);
    }

    #[test]
    fn test_type_result_to_minimal() {
        let mut result = TypeResult::new("struct".to_string());
        result.add_method(MethodOutput::new(
            "new".to_string(),
            "fn()".to_string(),
            "()".to_string(),
            "pub".to_string(),
            true,
        ));

        let minimal = result.to_minimal();
        assert_eq!(minimal.kind, result.kind);
        assert_eq!(minimal.methods.len(), result.methods.len());
    }

    #[test]
    fn test_trait_result_creation() {
        let result = TraitResult::new("Display".to_string(), "std::fmt::Display".to_string());
        assert_eq!(result.name, "Display");
        assert_eq!(result.path, "std::fmt::Display");
        assert_eq!(result.methods.len(), 0);
        assert_eq!(result.associated_types.len(), 0);
        assert_eq!(result.provided_methods.len(), 0);
    }

    #[test]
    fn test_trait_result_add_method() {
        let mut result = TraitResult::new("Display".to_string(), "std::fmt::Display".to_string());
        result.add_method(MethodOutput::new(
            "fmt".to_string(),
            "fn(&self) -> std::fmt::Result".to_string(),
            "Result".to_string(),
            "pub".to_string(),
            true,
        ));
        assert_eq!(result.methods.len(), 1);
    }

    #[test]
    fn test_trait_result_add_associated_type() {
        let mut result = TraitResult::new("Display".to_string(), "std::fmt::Display".to_string());
        result.add_associated_type(AssociatedTypeOutput::new("Output".to_string()));
        assert_eq!(result.associated_types.len(), 1);
    }

    #[test]
    fn test_trait_result_add_provided_method() {
        let mut result = TraitResult::new("Display".to_string(), "std::fmt::Display".to_string());
        result.add_provided_method("to_string".to_string());
        assert_eq!(result.provided_methods.len(), 1);
    }

    #[test]
    fn test_trait_result_to_minimal() {
        let mut result = TraitResult::new("Display".to_string(), "std::fmt::Display".to_string());
        result.add_method(MethodOutput::new(
            "fmt".to_string(),
            "fn(&self) -> std::fmt::Result".to_string(),
            "Result".to_string(),
            "pub".to_string(),
            true,
        ));

        let minimal = result.to_minimal();
        assert_eq!(minimal.name, result.name);
        assert_eq!(minimal.methods.len(), result.methods.len());
        assert_eq!(minimal.provided_methods.len(), 0); // Should be empty in minimal
    }

    #[test]
    fn test_module_result_creation() {
        let result = ModuleResult::new();
        assert_eq!(result.items.len(), 0);
        assert_eq!(result.submodules.len(), 0);
    }

    #[test]
    fn test_module_result_add_item() {
        let mut result = ModuleResult::new();
        let item = ModuleItem::new(
            "function1".to_string(),
            "function".to_string(),
            "path1".to_string(),
        );
        result.add_item(item);
        assert_eq!(result.items.len(), 1);
    }

    #[test]
    fn test_module_result_add_submodule() {
        let mut result = ModuleResult::new();
        result.add_submodule("std".to_string());
        assert_eq!(result.submodules.len(), 1);
    }

    #[test]
    fn test_module_result_to_minimal() {
        let mut result = ModuleResult::new();
        result.add_item(ModuleItem::new(
            "test".to_string(),
            "type".to_string(),
            "path".to_string(),
        ));

        let minimal = result.to_minimal();
        assert_eq!(minimal.items.len(), result.items.len());
        assert_eq!(minimal.submodules.len(), result.submodules.len());
    }

    #[test]
    fn test_module_item_creation() {
        let item = ModuleItem::new(
            "Function".to_string(),
            "function".to_string(),
            "path::to::function".to_string(),
        );
        assert_eq!(item.name, "Function");
        assert_eq!(item.kind, "function");
        assert_eq!(item.path, "path::to::function");
        assert!(item.signature.is_none());
    }

    #[test]
    fn test_module_item_with_signature() {
        let item = ModuleItem::new(
            "Function".to_string(),
            "function".to_string(),
            "path".to_string(),
        )
        .with_signature("fn(x: i32) -> i32".to_string());
        assert_eq!(item.signature, Some("fn(x: i32) -> i32".to_string()));
    }

    #[test]
    fn test_module_item_to_minimal() {
        let item = ModuleItem::new(
            "Function".to_string(),
            "function".to_string(),
            "path".to_string(),
        )
        .with_signature("fn(x: i32) -> i32".to_string());

        let minimal = item.to_minimal();
        assert_eq!(minimal.name, item.name);
        assert_eq!(minimal.kind, item.kind);
        assert_eq!(minimal.path, item.path);
        assert!(minimal.signature.is_none()); // Should be None in minimal
    }

    #[test]
    fn test_method_output_creation() {
        let method = MethodOutput::new(
            "new".to_string(),
            "fn()".to_string(),
            "()".to_string(),
            "pub".to_string(),
            true,
        );
        assert_eq!(method.name, "new");
        assert_eq!(method.signature, "fn()");
        assert_eq!(method.return_type, "()");
        assert_eq!(method.visibility, "pub");
        assert!(method.is_public);
        assert!(method.docs.is_none());
        assert!(!method.is_trait_method);
    }

    #[test]
    fn test_method_output_with_docs() {
        let method = MethodOutput::new(
            "new".to_string(),
            "fn()".to_string(),
            "()".to_string(),
            "pub".to_string(),
            true,
        )
        .with_docs(Some("Creates a new instance".to_string()));

        assert_eq!(method.docs, Some("Creates a new instance".to_string()));
    }

    #[test]
    fn test_method_output_with_trait_flag() {
        let method = MethodOutput::new(
            "fmt".to_string(),
            "fn(&self)".to_string(),
            "()".to_string(),
            "pub".to_string(),
            true,
        )
        .with_is_trait_method(true);

        assert!(method.is_trait_method);
    }

    #[test]
    fn test_method_output_to_minimal() {
        let method = MethodOutput::new(
            "new".to_string(),
            "fn()".to_string(),
            "()".to_string(),
            "pub".to_string(),
            true,
        )
        .with_docs(Some("Docs".to_string()))
        .with_is_trait_method(true);

        let minimal = method.to_minimal();
        assert_eq!(minimal.name, method.name);
        assert_eq!(minimal.docs, None); // Should be None in minimal
        assert!(minimal.is_trait_method);
    }

    #[test]
    fn test_associated_type_output_creation() {
        let assoc_type = AssociatedTypeOutput::new("Output".to_string());
        assert_eq!(assoc_type.name, "Output");
        assert!(assoc_type.bounds.is_none());
        assert!(assoc_type.default.is_none());
    }

    #[test]
    fn test_associated_type_with_bounds() {
        let assoc_type = AssociatedTypeOutput::new("Output".to_string())
            .with_bounds(Some("T: Clone".to_string()));
        assert_eq!(assoc_type.bounds, Some("T: Clone".to_string()));
    }

    #[test]
    fn test_associated_type_with_default() {
        let assoc_type =
            AssociatedTypeOutput::new("Output".to_string()).with_default(Some("None".to_string()));
        assert_eq!(assoc_type.default, Some("None".to_string()));
    }

    #[test]
    fn test_trait_impl_output_creation() {
        let impl_output =
            TraitImplOutput::new("Display".to_string(), "std::fmt::Display".to_string());
        assert_eq!(impl_output.trait_name, "Display");
        assert_eq!(impl_output.trait_path, "std::fmt::Display");
        assert_eq!(impl_output.methods.len(), 0);
        assert_eq!(impl_output.provided_methods.len(), 0);
        assert!(impl_output.generic_args.is_none());
    }

    #[test]
    fn test_trait_impl_output_add_method() {
        let mut impl_output =
            TraitImplOutput::new("Display".to_string(), "std::fmt::Display".to_string());
        impl_output.add_method(MethodOutput::new(
            "fmt".to_string(),
            "fn(&self)".to_string(),
            "()".to_string(),
            "pub".to_string(),
            true,
        ));
        assert_eq!(impl_output.methods.len(), 1);
    }

    #[test]
    fn test_trait_impl_output_add_provided_method() {
        let mut impl_output =
            TraitImplOutput::new("Display".to_string(), "std::fmt::Display".to_string());
        impl_output.add_provided_method("to_string".to_string());
        assert_eq!(impl_output.provided_methods.len(), 1);
    }

    #[test]
    fn test_trait_impl_output_with_generic_args() {
        let impl_output =
            TraitImplOutput::new("Display".to_string(), "std::fmt::Display".to_string())
                .with_generic_args(Some("T".to_string()));
        assert_eq!(impl_output.generic_args, Some("T".to_string()));
    }

    #[test]
    fn test_trait_impl_output_to_minimal() {
        let mut impl_output =
            TraitImplOutput::new("Display".to_string(), "std::fmt::Display".to_string());
        impl_output.add_method(MethodOutput::new(
            "fmt".to_string(),
            "fn(&self)".to_string(),
            "()".to_string(),
            "pub".to_string(),
            true,
        ));

        let minimal = impl_output.to_minimal();
        assert_eq!(minimal.trait_name, impl_output.trait_name);
        assert_eq!(minimal.methods.len(), impl_output.methods.len());
        assert_eq!(minimal.provided_methods.len(), 0); // Should be empty in minimal
    }
}
