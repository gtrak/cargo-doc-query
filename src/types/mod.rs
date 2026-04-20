use serde::Serialize;

pub mod detail;
pub mod doc;
pub mod expand;
pub mod filter;
pub mod query;

/// Shared module item type - represents an item within a module or impl block.
/// 
/// This is the canonical definition used across the codebase. Previously,
/// three nearly identical struct definitions existed:
/// - `ModuleItemInfo` in `expand.rs` (with modifiers for function info)
/// - `ModuleItem` in `query.rs` (basic + signature only)  
/// - `NestedItemInfo` in `format/item.rs` (core fields only)
/// 
/// This unified type provides all possible fields; simpler use cases just don't set the optional ones.
#[derive(Serialize, Debug, Clone)]
pub struct ModuleItem {
    /// Item name
    pub name: String,
    /// Item kind: struct, enum, trait, function, type, macro, etc.
    pub kind: String,
    /// Fully qualified item path
    pub path: String,
    /// Function signature (for functions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Visibility modifier (pub, pub(crate), private)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    /// Generic parameters in Rust syntax (e.g., "<T, K: Clone>")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generics: Option<String>,
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
}

impl ModuleItem {
    /// Create a new module item with core fields only
    pub fn new(name: String, kind: String, path: String) -> Self {
        Self {
            name,
            kind,
            path,
            signature: None,
            visibility: None,
            generics: None,
            is_const: None,
            is_async: None,
            is_unsafe: None,
            abi: None,
        }
    }

    /// Set the function signature
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Set visibility
    pub fn with_visibility(mut self, visibility: impl Into<String>) -> Self {
        self.visibility = Some(visibility.into());
        self
    }

    /// Set generic parameters
    pub fn with_generics(mut self, generics: impl Into<String>) -> Self {
        self.generics = Some(generics.into());
        self
    }

    /// Set function modifiers (const, async, unsafe, abi)
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

    /// Convert to minimal representation (core fields only)
    pub fn to_minimal(&self) -> Self {
        Self {
            name: self.name.clone(),
            kind: self.kind.clone(),
            path: self.path.clone(),
            signature: None,
            visibility: None,
            generics: None,
            is_const: None,
            is_async: None,
            is_unsafe: None,
            abi: None,
        }
    }
}
