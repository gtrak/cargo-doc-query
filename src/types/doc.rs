// Documentation extraction utilities

use rustdoc_types::Item;

/// Extracts documentation and metadata from rustdoc_types::Item
pub struct DocExtractor;

impl DocExtractor {
    /// Extract doc comments from an item
    ///
    /// Returns trimmed doc string if present, None otherwise
    pub fn extract_docs(item: &Item) -> Option<String> {
        item.docs.as_ref().map(|s| s.trim().to_string())
    }

    /// Determine if an item is public
    ///
    /// Public items have default visibility.
    /// TODO: Handle specific visibility flags when needed for --include=private
    pub fn extract_visibility(_item: &Item) -> bool {
        // Public items have default visibility
        // This is a placeholder that may need refinement based on rustdoc-types visibility field
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_extract_docs_with_docs() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: Some("/// This is a test struct".to_string()),
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let docs = DocExtractor::extract_docs(&item);
        assert!(docs.is_some());
        assert_eq!(docs.unwrap(), "/// This is a test struct");
    }

    #[test]
    fn test_extract_docs_empty_string() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: Some("".to_string()),
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let docs = DocExtractor::extract_docs(&item);
        assert!(docs.is_some());
        assert_eq!(docs.unwrap(), "");
    }

    #[test]
    fn test_extract_docs_none() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: None,
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let docs = DocExtractor::extract_docs(&item);
        assert!(docs.is_none());
    }

    #[test]
    fn test_extract_docs_with_multiline() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: Some("/// First line\n///\n/// Second line".to_string()),
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let docs = DocExtractor::extract_docs(&item);
        assert!(docs.is_some());
        assert_eq!(docs.unwrap(), "/// First line\n///\n/// Second line");
    }

    #[test]
    fn test_extract_docs_with_doc_macro() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: Some("#[doc = \"Doc attribute\"]".to_string()),
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let docs = DocExtractor::extract_docs(&item);
        assert!(docs.is_some());
        assert_eq!(docs.unwrap(), "#[doc = \"Doc attribute\"]");
    }

    #[test]
    fn test_extract_docs_with_tabs() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: Some("\t/// Indented doc comment".to_string()),
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let docs = DocExtractor::extract_docs(&item);
        assert!(docs.is_some());
        assert_eq!(docs.unwrap(), "/// Indented doc comment");
    }

    #[test]
    fn test_extract_visibility_public() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: None,
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let visibility = DocExtractor::extract_visibility(&item);
        assert!(visibility);
    }

    #[test]
    fn test_extract_visibility_with_docs() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: None,
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let visibility = DocExtractor::extract_visibility(&item);
        assert!(visibility);
    }

    #[test]
    fn test_extract_visibility_default() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: None,
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let visibility = DocExtractor::extract_visibility(&item);
        assert!(visibility);
    }

    #[test]
    fn test_extract_docs_none_crate_version() {
        let item = Item {
            id: rustdoc_types::Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: None,
            links: HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: rustdoc_types::ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Plain {
                    fields: vec![],
                    has_stripped_fields: false,
                },
                generics: rustdoc_types::Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
            }),
        };

        let docs = DocExtractor::extract_docs(&item);
        assert!(docs.is_none());
    }
}
