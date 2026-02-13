//! Unified item formatting dispatcher
//!
//! Provides a single format_item() function that handles all ItemKind variants
//! with consistent formatting rules, controlled by DetailLevel.

use rustdoc_types::{Item, ItemEnum};

use crate::types::detail::{
    extract_deprecation_info, extract_semantic_attrs, visibility_to_string, DetailLevel,
};

/// Unified output structure for formatted items
#[derive(Debug, Clone)]
pub struct FormattedItem {
    /// Unique identifier
    pub id: String,
    /// Kind as string (e.g., "struct", "function")
    pub kind: String,
    /// Item name
    pub name: Option<String>,
    /// Full signature (for functions, methods, etc.)
    pub signature: Option<String>,
    /// Visibility string (e.g., "pub", "pub(crate)")
    pub visibility: Option<String>,
    /// Generics string (e.g., "<T, K, V>")
    pub generics: Option<String>,
    /// Documentation text
    pub docs: Option<String>,
    /// Fields for structs/unions
    pub fields: Vec<FieldInfo>,
    /// Variants for enums
    pub variants: Vec<VariantInfo>,
    /// Nested items (for modules, impls, etc.)
    pub items: Vec<NestedItemInfo>,
    /// Whether item is deprecated
    pub is_deprecated: bool,
    /// Deprecation note
    pub deprecation_note: Option<String>,
    /// Semantic attributes (#[must_use], #[non_exhaustive])
    pub attributes: Vec<String>,
    /// Function modifiers (const, async, unsafe, abi)
    pub modifiers: Option<FunctionModifiers>,
}

/// Field information for structs/unions
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_path: String,
    pub is_optional: bool,
}

/// Variant information for enums
#[derive(Debug, Clone)]
pub struct VariantInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

/// Nested item info (for modules, impls, traits)
#[derive(Debug, Clone)]
pub struct NestedItemInfo {
    pub name: String,
    pub kind: String,
    pub path: String,
}

/// Function modifiers
#[derive(Debug, Clone)]
pub struct FunctionModifiers {
    pub is_const: bool,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub abi: Option<String>,
}

/// Unified item formatter for all ItemKind variants
pub struct ItemFormatter {
    detail_level: DetailLevel,
    token_budget: Option<usize>,
    current_tokens: usize,
}

impl ItemFormatter {
    /// Create a new formatter with specified detail level and optional token budget
    pub fn new(detail_level: DetailLevel, token_budget: Option<usize>) -> Self {
        Self {
            detail_level,
            token_budget,
            current_tokens: 0,
        }
    }

    /// Main dispatcher - handles all ItemKind variants
    pub fn format_item(&mut self, item: &Item) -> FormattedItem {
        let kind_str = self.get_kind_string(item);

        // Extract common fields based on detail level
        let name = item.name.clone();
        let visibility = if self.detail_level.includes_visibility() {
            Some(visibility_to_string(&item.visibility))
        } else {
            None
        };

        let docs = if self.detail_level.is_minimal() {
            None
        } else {
            item.docs.as_ref().map(|s| s.trim().to_string())
        };
        let (is_deprecated, deprecation_note) = if self.detail_level.includes_deprecation() {
            extract_deprecation_info(item.deprecation.as_ref())
                .map(|(b, n)| (b, n))
                .unwrap_or((false, None))
        } else {
            (false, None)
        };
        let attributes = if self.detail_level.includes_attributes() {
            extract_semantic_attrs(&item.attrs)
        } else {
            vec![]
        };

        // Extract kind-specific data
        let (signature, fields, variants, items, modifiers, generics) =
            self.extract_kind_specific_data(item);

        FormattedItem {
            id: item.id.0.to_string(),
            kind: kind_str,
            name,
            signature,
            visibility,
            generics,
            docs,
            fields,
            variants,
            items,
            is_deprecated,
            deprecation_note,
            attributes,
            modifiers,
        }
    }

    /// Get kind string from item
    fn get_kind_string(&self, item: &Item) -> String {
        match &item.inner {
            ItemEnum::Module(_) => "module".to_string(),
            ItemEnum::Struct(_) => "struct".to_string(),
            ItemEnum::Enum(_) => "enum".to_string(),
            ItemEnum::Union(_) => "union".to_string(),
            ItemEnum::Trait(_) => "trait".to_string(),
            ItemEnum::Function(_) => "function".to_string(),
            ItemEnum::TypeAlias(_) => "type".to_string(),
            ItemEnum::Constant { .. } => "constant".to_string(),
            ItemEnum::Static(_) => "static".to_string(),
            ItemEnum::Macro(_) => "macro".to_string(),
            ItemEnum::Use(_) => "use".to_string(),
            ItemEnum::Impl(_) => "impl".to_string(),
            ItemEnum::AssocConst { .. } => "associated_constant".to_string(),
            ItemEnum::AssocType { .. } => "associated_type".to_string(),
            ItemEnum::Variant(_) => "variant".to_string(),
            ItemEnum::StructField(_) => "field".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Extract kind-specific data based on item inner type
    fn extract_kind_specific_data(
        &self,
        item: &Item,
    ) -> (
        Option<String>,
        Vec<FieldInfo>,
        Vec<VariantInfo>,
        Vec<NestedItemInfo>,
        Option<FunctionModifiers>,
        Option<String>,
    ) {
        match &item.inner {
            // Module
            ItemEnum::Module(m) => {
                let items: Vec<NestedItemInfo> = m
                    .items
                    .iter()
                    .map(|id| NestedItemInfo {
                        name: "".to_string(),
                        kind: "".to_string(),
                        path: "".to_string(),
                    })
                    .collect();
                (None, vec![], vec![], items, None, None)
            }
            // Struct
            ItemEnum::Struct(_) => (None, vec![], vec![], vec![], None, None),
            // Enum
            ItemEnum::Enum(_) => (None, vec![], vec![], vec![], None, None),
            // Function
            ItemEnum::Function(f) => {
                let sig = format!("fn {}(...)", item.name.as_deref().unwrap_or(""));
                let modifiers = if self.detail_level.includes_function_modifiers() {
                    Some(FunctionModifiers {
                        is_const: false,
                        is_async: false,
                        is_unsafe: false,
                        abi: None,
                    })
                } else {
                    None
                };
                let generics = if self.detail_level.includes_generics() {
                    format_generics(&f.generics)
                } else {
                    None
                };
                (Some(sig), vec![], vec![], vec![], modifiers, generics)
            }
            // Trait
            ItemEnum::Trait(t) => {
                let items: Vec<NestedItemInfo> = t
                    .items
                    .iter()
                    .map(|id| NestedItemInfo {
                        name: "".to_string(),
                        kind: "".to_string(),
                        path: "".to_string(),
                    })
                    .collect();
                let generics = if self.detail_level.includes_generics() {
                    format_generics(&t.generics)
                } else {
                    None
                };
                (None, vec![], vec![], items, None, generics)
            }
            // Impl
            ItemEnum::Impl(i) => {
                let items: Vec<NestedItemInfo> = i
                    .items
                    .iter()
                    .map(|id| NestedItemInfo {
                        name: "".to_string(),
                        kind: "".to_string(),
                        path: "".to_string(),
                    })
                    .collect();
                (None, vec![], vec![], items, None, None)
            }
            // TypeAlias
            ItemEnum::TypeAlias(ta) => {
                let sig = format!("type {}", item.name.as_deref().unwrap_or(""));
                let generics = if self.detail_level.includes_generics() {
                    format_generics(&ta.generics)
                } else {
                    None
                };
                (Some(sig), vec![], vec![], vec![], None, generics)
            }
            // Constant
            ItemEnum::Constant { .. } => {
                let sig = format!("const {}", item.name.as_deref().unwrap_or(""));
                (Some(sig), vec![], vec![], vec![], None, None)
            }
            // Static
            ItemEnum::Static(_) => {
                let sig = format!("static {}", item.name.as_deref().unwrap_or(""));
                (Some(sig), vec![], vec![], vec![], None, None)
            }
            // Macro
            ItemEnum::Macro(_) => {
                let sig = format!("{}!", item.name.as_deref().unwrap_or(""));
                (Some(sig), vec![], vec![], vec![], None, None)
            }
            // Union
            ItemEnum::Union(_) => (None, vec![], vec![], vec![], None, None),
            // Use
            ItemEnum::Use(_) => {
                let sig = format!("use ...");
                (Some(sig), vec![], vec![], vec![], None, None)
            }
            // Default case
            _ => (None, vec![], vec![], vec![], None, None),
        }
    }

    /// Check if formatting would exceed token budget
    fn would_exceed_budget(&self, _additional_tokens: usize) -> bool {
        if let Some(budget) = self.token_budget {
            self.current_tokens + _additional_tokens > budget
        } else {
            false
        }
    }
}

/// Format an item with default Standard detail level
pub fn format_item(item: &Item) -> FormattedItem {
    let mut formatter = ItemFormatter::new(DetailLevel::default(), None);
    formatter.format_item(item)
}

/// Format an item with specific detail level
pub fn format_item_with_detail(item: &Item, detail_level: DetailLevel) -> FormattedItem {
    let mut formatter = ItemFormatter::new(detail_level, None);
    formatter.format_item(item)
}

/// Format an item with specific detail level and token budget
pub fn format_item_with_budget(
    item: &Item,
    detail_level: DetailLevel,
    budget: usize,
) -> FormattedItem {
    let mut formatter = ItemFormatter::new(detail_level, Some(budget));
    formatter.format_item(item)
}

/// Helper function for formatting generics (imported from detail)
fn format_generics(generics: &rustdoc_types::Generics) -> Option<String> {
    use crate::types::detail::format_generics as fg;
    fg(generics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustdoc_types::Id;

    /// Helper to create a minimal Item for testing - uses simplest possible inner types
    fn create_test_item() -> Item {
        Item {
            id: Id(1),
            crate_id: 1,
            name: Some("TestItem".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: Some("Test documentation".to_string()),
            links: std::collections::HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: ItemEnum::Struct(rustdoc_types::Struct {
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
                kind: rustdoc_types::StructKind::Unit,
            }),
        }
    }

    #[test]
    fn test_format_item_struct() {
        let item = create_test_item();
        let mut formatter = ItemFormatter::new(DetailLevel::Standard, None);
        let result = formatter.format_item(&item);

        assert_eq!(result.kind, "struct");
        assert_eq!(result.name, Some("TestItem".to_string()));
        assert!(result.visibility.is_some());
    }

    #[test]
    fn test_format_item_respects_detail_level_minimal() {
        let item = create_test_item();

        let mut formatter_minimal = ItemFormatter::new(DetailLevel::Minimal, None);
        let result_minimal = formatter_minimal.format_item(&item);

        // Minimal should omit visibility and docs
        assert_eq!(result_minimal.visibility, None);
        assert_eq!(result_minimal.docs, None);

        let mut formatter_standard = ItemFormatter::new(DetailLevel::Standard, None);
        let result_standard = formatter_standard.format_item(&item);

        // Standard should include visibility and docs
        assert!(result_standard.visibility.is_some());
        assert!(result_standard.docs.is_some());
    }

    #[test]
    fn test_format_item_with_budget() {
        let item = create_test_item();

        // Should not panic with budget
        let mut formatter = ItemFormatter::new(DetailLevel::Standard, Some(1000));
        let result = formatter.format_item(&item);
        assert!(!result.kind.is_empty());
    }
}
