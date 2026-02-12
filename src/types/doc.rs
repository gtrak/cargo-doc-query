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
    pub fn extract_visibility(item: &Item) -> bool {
        // Public items have default visibility
        // This is a placeholder that may need refinement based on rustdoc-types visibility field
        true
    }
}
