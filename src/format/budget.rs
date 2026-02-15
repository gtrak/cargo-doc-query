//! Token budget tracking for rendering layer
//!
//! Provides BudgetTracker for tracking token usage during item formatting,
//! enabling truncation decisions based on budget constraints.

use crate::format::item::FormattedItem;

/// Action to take when tracking an item against budget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruncationAction {
    /// Include the item in output
    Include,
    /// Truncate/skip the item due to budget
    Truncate,
}

/// Token budget tracker for rendering layer
///
/// Tracks cumulative token usage and provides truncation decisions
/// based on configured budget constraints.
#[derive(Debug, Clone)]
pub struct BudgetTracker {
    /// Optional total token budget
    total_budget: Option<usize>,
    /// Current token count used
    current_tokens: usize,
    /// Warning threshold (default 0.8 = 80%)
    warning_threshold: f32,
}

impl BudgetTracker {
    /// Create a new BudgetTracker with optional token budget
    pub fn new(budget: Option<usize>) -> Self {
        Self {
            total_budget: budget,
            current_tokens: 0,
            warning_threshold: 0.8,
        }
    }

    /// Track an item's token usage
    ///
    /// Returns the tokens used and the action to take (Include or Truncate)
    pub fn track_item(
        &mut self,
        item_tokens: usize,
        doc_tokens: usize,
    ) -> (usize, TruncationAction) {
        let total_item_tokens = item_tokens + doc_tokens;

        // Check if this would exceed budget
        if self.would_exceed(total_item_tokens) {
            (0, TruncationAction::Truncate)
        } else {
            self.current_tokens += total_item_tokens;
            (total_item_tokens, TruncationAction::Include)
        }
    }

    /// Check if adding the given tokens would exceed the budget
    pub fn would_exceed(&self, tokens: usize) -> bool {
        match self.total_budget {
            None => false,
            Some(budget) => self.current_tokens + tokens > budget,
        }
    }

    /// Get remaining tokens in budget
    ///
    /// Returns None if no budget is set
    pub fn remaining(&self) -> Option<usize> {
        self.total_budget
            .map(|b| b.saturating_sub(self.current_tokens))
    }

    /// Check if warning should be displayed (threshold reached)
    pub fn is_warning_needed(&self) -> bool {
        match self.total_budget {
            None => false,
            Some(budget) => {
                if budget == 0 {
                    false
                } else {
                    let ratio = self.current_tokens as f32 / budget as f32;
                    ratio >= self.warning_threshold
                }
            }
        }
    }
}

/// Estimate token count for a formatted item
///
/// Provides a rough estimate based on item complexity:
/// - Base: 20 tokens per item (path overhead)
/// - +5 tokens per field
/// - +5 tokens per variant
/// - +10 tokens per module item
/// - +doc_tokens (docs.len() / 4)
pub fn estimate_item_tokens(formatted: &FormattedItem) -> usize {
    let mut tokens = 20; // Base overhead per item

    // Add for fields
    tokens += formatted.fields.len() * 5;

    // Add for variants
    tokens += formatted.variants.len() * 5;

    // Add for nested items
    tokens += formatted.items.len() * 10;

    // Add for doc tokens
    if let Some(docs) = &formatted.docs {
        tokens += docs.len() / 4; // Rough estimate: 1 token per 4 chars
    }

    // Add for signature
    if formatted.signature.is_some() {
        tokens += 10;
    }

    // Add for generics
    if formatted.generics.is_some() {
        tokens += 5;
    }

    // Add for visibility
    if formatted.visibility.is_some() {
        tokens += 3;
    }

    // Add for attributes
    tokens += formatted.attributes.len() * 3;

    // Add for modifiers
    if formatted.modifiers.is_some() {
        tokens += 5;
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::item::FieldInfo;

    fn create_test_formatted_item() -> FormattedItem {
        FormattedItem {
            id: "test::Struct".to_string(),
            kind: "struct".to_string(),
            name: Some("Struct".to_string()),
            signature: None,
            visibility: Some("pub".to_string()),
            generics: Some("<T>".to_string()),
            docs: Some("Test documentation".to_string()),
            fields: vec![FieldInfo {
                name: "field1".to_string(),
                type_path: "i32".to_string(),
                is_optional: false,
            }],
            variants: vec![],
            items: vec![],
            is_deprecated: false,
            deprecation_note: None,
            attributes: vec!["#[must_use]".to_string()],
            modifiers: None,
        }
    }

    #[test]
    fn test_budget_tracker_no_limit() {
        let mut tracker = BudgetTracker::new(None);

        let (tokens, action) = tracker.track_item(100, 50);

        assert_eq!(tokens, 150);
        assert_eq!(action, TruncationAction::Include);
        assert!(!tracker.is_warning_needed());
    }

    #[test]
    fn test_budget_tracker_exceeds() {
        let mut tracker = BudgetTracker::new(Some(100));

        // First item fits
        let (tokens, action) = tracker.track_item(50, 30);
        assert_eq!(tokens, 80);
        assert_eq!(action, TruncationAction::Include);

        // Second item exceeds
        let (tokens, action) = tracker.track_item(30, 20);
        assert_eq!(tokens, 0);
        assert_eq!(action, TruncationAction::Truncate);
    }

    #[test]
    fn test_budget_tracker_remaining() {
        let mut tracker = BudgetTracker::new(Some(100));

        assert_eq!(tracker.remaining(), Some(100));

        tracker.track_item(30, 20);
        assert_eq!(tracker.remaining(), Some(50));

        // At budget
        tracker.track_item(50, 0);
        assert_eq!(tracker.remaining(), Some(0));
    }

    #[test]
    fn test_estimate_item_tokens() {
        let item = create_test_formatted_item();

        let tokens = estimate_item_tokens(&item);

        // Base: 20
        // + field: 5
        // + docs: 18 / 4 = 4
        // + visibility: 3
        // + generics: 5
        // + attributes: 1 * 3 = 3
        // Total: 20 + 5 + 4 + 3 + 5 + 3 = 40
        assert_eq!(tokens, 40);
    }

    #[test]
    fn test_track_item_includes_docs() {
        let mut tracker = BudgetTracker::new(Some(50));

        // Item tokens: 20, Doc tokens: 30 = 50 total
        let (tokens, action) = tracker.track_item(20, 30);

        assert_eq!(tokens, 50); // Total = item + doc
        assert_eq!(action, TruncationAction::Include);

        // Remaining should be 0
        assert_eq!(tracker.remaining(), Some(0));
    }

    #[test]
    fn test_would_exceed() {
        let tracker = BudgetTracker::new(Some(100));

        assert!(!tracker.would_exceed(50));
        assert!(!tracker.would_exceed(100));
        assert!(tracker.would_exceed(101));
    }

    #[test]
    fn test_no_budget_never_exceeds() {
        let tracker = BudgetTracker::new(None);

        assert!(!tracker.would_exceed(1_000_000));
    }
}
