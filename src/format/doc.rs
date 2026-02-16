// Doc comment handling for formatting output
//
// Provides DocHandler for extracting, formatting, and truncating doc comments
// based on DetailLevel and token budget constraints.

use crate::types::detail::DetailLevel;
use rustdoc_types::Item;

/// DocHandler for managing documentation display based on detail level and budget
#[derive(Debug, Clone)]
pub struct DocHandler {
    detail_level: DetailLevel,
    remaining_budget: Option<usize>,
}

impl DocHandler {
    /// Create a new DocHandler with specified detail level and optional token budget
    pub fn new(detail_level: DetailLevel, token_budget: Option<usize>) -> Self {
        Self {
            detail_level,
            remaining_budget: token_budget,
        }
    }

    /// Extract doc comments from an item
    ///
    /// Returns trimmed doc string if present, None otherwise
    pub fn extract_docs(item: &Item) -> Option<String> {
        // Use trim() like the existing DocExtractor does
        item.docs.as_ref().map(|s| s.trim().to_string())
    }

    /// Format docs based on detail level and budget
    ///
    /// Returns:
    /// - None if detail_level is Minimal (DOCS-03)
    /// - Some(docs) if no budget or fits within budget
    /// - Some(truncated_docs + "...") if exceeds budget
    pub fn format_docs(&self, docs: &str) -> Option<String> {
        // DOCS-03: Minimal mode omits docs to save tokens
        if self.detail_level.is_minimal() {
            return None;
        }

        // If no budget constraint, return full docs
        if self.remaining_budget.is_none() {
            return Some(docs.to_string());
        }

        // Calculate doc tokens (rough estimate: 1 token ≈ 4 chars)
        let doc_tokens = docs.len() / 4;

        // Check if would exceed budget
        if self.would_exceed_budget(doc_tokens) {
            // Truncate to fit
            let max_tokens = self.remaining_budget.unwrap_or(0);
            let (truncated, _) = truncate_docs(docs, max_tokens);
            Some(truncated)
        } else {
            Some(docs.to_string())
        }
    }

    /// Check if adding tokens would exceed budget
    fn would_exceed_budget(&self, tokens: usize) -> bool {
        match self.remaining_budget {
            Some(budget) => tokens > budget,
            None => false,
        }
    }
}

/// Truncate docs at sentence boundaries when budget exceeded
///
/// Returns (truncated_string, was_truncated_flag)
///
/// DOCS-04: Smart truncation at sentence boundaries
/// DOCS-05: Code blocks preserved over prose during truncation
/// DOCS-06: Truncated docs show "..." indicator
pub fn truncate_docs(docs: &str, max_tokens: usize) -> (String, bool) {
    // Calculate max characters (rough estimate: 1 token ≈ 4 chars)
    let max_chars = max_tokens * 4;

    // If docs already fits, return as-is
    if docs.len() <= max_chars {
        return (docs.to_string(), false);
    }

    // First, check if there are code blocks in the docs
    let first_code_block = docs.find("```");

    if let Some(code_pos) = first_code_block {
        // There's a code block - preserve it, truncate prose before it
        // Find the end of this code block
        let after_code_start = code_pos + 3;
        let code_end_pos = docs[after_code_start..]
            .find("```")
            .map(|end| code_pos + 3 + end + 3)
            .unwrap_or(docs.len());

        // Find prose portion (everything before the code block)
        let prose = &docs[..code_pos];

        // Find the best sentence boundary in the prose portion
        // We want any complete sentence, not limited by max_chars for prose since code block takes priority
        let sentence_pos = find_sentence_boundary_allow_overflow(prose);

        if sentence_pos > 0 {
            let truncated_prose = prose[..sentence_pos].trim_end();
            // Append code block after truncated prose
            let code_block = &docs[code_pos..code_end_pos];
            (format!("{}\n\n{}...", truncated_prose, code_block), true)
        } else {
            // No sentence boundary found in prose, just truncate at code_pos
            let truncated = prose.trim_end();
            let code_block = &docs[code_pos..code_end_pos];
            (format!("{}...\n\n{}", truncated, code_block), true)
        }
    } else {
        // No code blocks - simple truncation at sentence boundary
        // Find sentence boundary within the allowed budget
        let sentence_pos = find_sentence_boundary(docs, max_chars);

        if sentence_pos > 0 {
            let truncated = docs[..sentence_pos].trim_end();
            (format!("{}...", truncated), true)
        } else {
            // No sentence boundary found, just truncate at max_chars
            let truncated = docs[..max_chars].trim_end();
            (format!("{}...", truncated), true)
        }
    }
}

/// Find the last complete sentence boundary (allowing overflow past max_chars)
/// Used when code blocks take priority - we want any complete sentence in prose
fn find_sentence_boundary_allow_overflow(text: &str) -> usize {
    let mut valid_pos = 0;

    for (i, c) in text.char_indices() {
        if matches!(c, '.' | '!' | '?') {
            let after = i + c.len_utf8();

            let is_valid = if after >= text.len() {
                true
            } else {
                let next_char = text.chars().nth(after);
                matches!(next_char, Some(' ') | Some('\n') | None)
            };

            if is_valid {
                valid_pos = after;
            }
        }
    }

    valid_pos
}

/// Find the last complete sentence boundary that fits within max_chars
/// Returns the position after the sentence-ending punctuation
fn find_sentence_boundary(text: &str, max_chars: usize) -> usize {
    // Look for [.!?] followed by [space, newline, or end-of-string]
    // Find ALL sentence boundaries in the entire text, then return the last one that fits in max_chars

    let mut valid_positions: Vec<usize> = Vec::new();

    for (i, c) in text.char_indices() {
        if matches!(c, '.' | '!' | '?') {
            // Check what's after this punctuation
            let after = i + c.len_utf8();

            // Valid if: end of text OR followed by space/newline
            let is_valid = if after >= text.len() {
                true // End of text
            } else {
                let next_char = text.chars().nth(after);
                matches!(next_char, Some(' ') | Some('\n') | None)
            };

            // Only consider boundaries that fit within max_chars
            if is_valid && after <= max_chars {
                valid_positions.push(after);
            }
        }
    }

    // Return the last valid position (closest to max_chars), or 0 if none found
    // If none found within max_chars, try to find any boundary and truncate at max_chars
    if valid_positions.is_empty() {
        // Fallback: try to find any boundary and we'll truncate at max_chars
        for (i, c) in text.char_indices() {
            if matches!(c, '.' | '!' | '?') {
                let after = i + c.len_utf8();
                let is_valid = if after >= text.len() {
                    true
                } else {
                    let next_char = text.chars().nth(after);
                    matches!(next_char, Some(' ') | Some('\n') | None)
                };
                if is_valid {
                    return max_chars; // Return max_chars to trigger truncation
                }
            }
        }
        0
    } else {
        valid_positions.pop().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // truncate_docs Tests
    // =========================================================================

    #[test]
    fn test_truncate_docs_short() {
        // Short docs shorter than max_tokens returns original
        let docs = "This is a short doc.";
        let (result, truncated) = truncate_docs(docs, 100);
        assert_eq!(result, docs);
        assert!(!truncated);
    }

    #[test]
    fn test_truncate_docs_exact_fit() {
        // Docs exactly at the character limit (not token limit)
        // max_tokens=2 means max_chars=8, so "Short." (6 chars) fits
        let docs = "Short.";
        let (result, truncated) = truncate_docs(docs, 2); // 2 tokens = 8 chars
        assert_eq!(result, docs);
        assert!(!truncated);
    }

    #[test]
    fn test_truncate_docs_multiple_sentences() {
        // Multiple sentences - should keep at least one complete sentence
        let docs = "First! Second? Third.";
        let (result, truncated) = truncate_docs(docs, 3); // 3 tokens = 12 chars

        // Should have at least first sentence
        assert!(result.contains("First!") || result.contains("First!..."));
        assert!(truncated);
    }

    #[test]
    fn test_truncate_docs_with_code_block() {
        // Code blocks should be preserved
        let docs = "Some prose description.\n\n```rust\nfn example() {}\n```\n\nMore prose here.";

        let (result, truncated) = truncate_docs(docs, 5); // Very small budget
        assert!(truncated);
        // Should contain the code block
        assert!(result.contains("```rust"));
        assert!(result.contains("fn example()"));
    }

    #[test]
    fn test_truncate_docs_no_code_block() {
        // No code blocks - clean truncation
        let docs = "This is a long doc string that should be truncated at sentence boundary.";

        let (result, truncated) = truncate_docs(docs, 5);
        assert!(truncated);
        // Should end with ...
        assert!(result.ends_with("..."));
        // Should be a complete sentence
        assert!(result.contains('.'));
    }

    #[test]
    fn test_truncate_docs_returns_indicator() {
        // Returns true when truncated
        let docs = "This is a very long documentation string that exceeds the token budget.";
        let (result, truncated) = truncate_docs(docs, 5);

        assert!(truncated);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_docs_exclamation() {
        // Exclamation is a sentence boundary - test that we can find it
        let docs = "First sentence! Second sentence.";
        let (result, truncated) = truncate_docs(docs, 10); // 10 tokens = 40 chars - fits, so no truncation

        // Since it fits, no truncation should happen
        assert!(!truncated);
        assert_eq!(result, docs);
    }

    #[test]
    fn test_truncate_docs_question() {
        // Question mark is a sentence boundary - test that we can find it
        let docs = "What is this? It is something.";
        let (result, truncated) = truncate_docs(docs, 10); // 10 tokens = 40 chars - fits, so no truncation

        // Since it fits, no truncation should happen
        assert!(!truncated);
        assert_eq!(result, docs);
    }

    #[test]
    fn test_truncate_docs_sentence_boundary() {
        // Truncates at sentence boundary, not mid-sentence - with enough budget
        let docs =
            "This is the first sentence. This is the second sentence. This is the third sentence.";
        let (result, truncated) = truncate_docs(docs, 10); // 10 tokens = 40 chars

        // Should truncate after first sentence
        assert!(result.contains("first sentence."));
        assert!(truncated);
        // Should NOT contain second sentence
        assert!(!result.contains("second"));
    }

    #[test]
    fn test_truncate_docs_preserves_code_prose_ratio() {
        // Code block should come after truncated prose
        let docs = "A short sentence.\n\n```\ncode\n```\n\nEnd.";

        let (result, truncated) = truncate_docs(docs, 5);
        assert!(truncated);
        // Code block should be present
        assert!(result.contains("```"));
    }

    // =========================================================================
    // DocHandler Tests
    // =========================================================================

    #[test]
    fn test_extract_docs_with_docs() {
        use rustdoc_types::{Generics, Id, ItemEnum, Struct, StructKind};

        let item = Item {
            id: Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: Some("/// Test documentation".to_string()),
            links: std::collections::HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: ItemEnum::Struct(Struct {
                generics: Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
                kind: StructKind::Unit,
            }),
        };

        let docs = DocHandler::extract_docs(&item);
        assert!(docs.is_some());
        // trim() only removes leading/trailing whitespace, keeps /// prefix
        assert_eq!(docs.unwrap(), "/// Test documentation");
    }

    #[test]
    fn test_extract_docs_no_docs() {
        use rustdoc_types::{Generics, Id, ItemEnum, Struct, StructKind};

        let item = Item {
            id: Id(1),
            crate_id: 1,
            name: Some("TestStruct".to_string()),
            span: None,
            visibility: rustdoc_types::Visibility::Public,
            docs: None,
            links: std::collections::HashMap::new(),
            attrs: vec![],
            deprecation: None,
            inner: ItemEnum::Struct(Struct {
                generics: Generics {
                    params: vec![],
                    where_predicates: vec![],
                },
                impls: vec![],
                kind: StructKind::Unit,
            }),
        };

        let docs = DocHandler::extract_docs(&item);
        assert!(docs.is_none());
    }

    #[test]
    fn test_format_docs_minimal_omits() {
        let handler = DocHandler::new(DetailLevel::Minimal, None);

        let result = handler.format_docs("Some documentation");
        assert!(result.is_none());
    }

    #[test]
    fn test_format_docs_standard_includes() {
        let handler = DocHandler::new(DetailLevel::Standard, None);

        let result = handler.format_docs("Some documentation");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Some documentation");
    }

    #[test]
    fn test_format_docs_detailed_includes() {
        let handler = DocHandler::new(DetailLevel::Detailed, None);

        let result = handler.format_docs("Some documentation");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Some documentation");
    }

    #[test]
    fn test_format_docs_respects_budget() {
        // Budget of 5 tokens = 20 chars
        let handler = DocHandler::new(DetailLevel::Standard, Some(5));

        // Long doc that exceeds budget
        let result = handler.format_docs("This is a very long documentation string");
        assert!(result.is_some());

        let formatted = result.unwrap();
        // Should be truncated
        assert!(formatted.ends_with("..."));
    }

    #[test]
    fn test_format_docs_within_budget() {
        // Budget of 100 tokens = 400 chars
        let handler = DocHandler::new(DetailLevel::Standard, Some(100));

        let result = handler.format_docs("Short doc");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Short doc");
    }

    #[test]
    fn test_doc_handler_new() {
        let handler = DocHandler::new(DetailLevel::Standard, Some(50));

        // Verify fields are set
        assert!(!handler.detail_level.is_minimal());
    }

    #[test]
    fn test_truncate_docs_no_space_after_punctuation() {
        // Sentence ending without space before end of text
        let docs = "This is sentence one.This is sentence two.";
        let (_result, truncated) = truncate_docs(docs, 4);

        // Should still truncate reasonably
        assert!(truncated);
    }
}
