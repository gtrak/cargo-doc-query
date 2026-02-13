# Phase 9: Unified Rendering and Documentation - Research

**Researched:** 2026-02-13
**Domain:** Text rendering, documentation extraction, token budget management
**Confidence:** HIGH

## Summary

This phase implements unified rendering across all 24 rustdoc ItemKind variants and doc comment extraction with token-aware display. The key architectural insight is that rendering consistency is primarily a dispatcher design problem: all item types should flow through a single `format_item()` function that delegates to type-specific formatters based on ItemKind, while respecting DetailLevel for what metadata to include.

The codebase already has substantial foundations:
- `DetailLevel` enum (Minimal/Standard/Detailed) in `src/types/detail.rs`
- `DocExtractor` for extracting docs from `Item::docs` field in `src/types/doc.rs`
- `TokenConfig` for budget tracking in `src/types/expand.rs`
- Text formatting in `src/format/text.rs` (needs refactoring for unified dispatcher)

**Primary recommendation:** Create a new unified dispatcher `format_item()` in `src/format/mod.rs` that handles all 24 ItemKind variants with consistent formatting rules, delegating to existing type formatters while adding doc comment handling and token budget integration.

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Unified formatting regardless of position — Root-level items and nested items follow the same formatting rules
- All 24 ItemKind variants use the same dispatcher (`format_item()`)
- Depth affects detail level, but not format consistency
- Doc comments extracted from rustdoc JSON `Item::docs` field
- Displayed in standard mode, omitted in minimal mode
- Smart truncation at sentence boundaries when budget exceeded
- Code blocks preserved over prose during truncation
- Token budget integrated at rendering layer
- Tracks per-item overhead including doc comments
- Truncation triggers with "..." indicator

### Claude's Discretion
- Exact formatting for each ItemKind variant
- Specific truncation algorithm details
- Sentence boundary detection approach
- Per-item overhead calculation method

### Deferred Ideas (OUT OF SCOPE)
None — within phase scope

---

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rustdoc-types | latest | ItemKind, ItemEnum definitions | Official rustdoc JSON schema |
| console | 0.21+ | Colored/styled terminal output | Used throughout for `style()` |
| serde | 1.0+ | JSON serialization | Existing in project |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| console::style | - | Terminal styling | All text output formatting |
| DetailLevel | - | Metadata control | Already exists in types/detail.rs |
| TokenConfig | - | Budget tracking | Already exists in types/expand.rs |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| New formatting crate | termcolor, ansi_term | Console already used, avoid new dependency |
| Custom token counting | tiktoken tokenizers | Simple heuristic (chars/4) sufficient for estimates |

**Installation:**
```bash
# No new dependencies needed - all functionality exists
# Console crate already in Cargo.toml
```

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── format/                    # New unified rendering module
│   ├── mod.rs                 # Re-exports and dispatcher
│   ├── text.rs                # Existing - refactor for unified API
│   ├── item.rs                # NEW: format_item() dispatcher
│   ├── doc.rs                # NEW: doc comment handling
│   └── budget.rs             # NEW: token budget integration
├── types/
│   ├── detail.rs             # Existing DetailLevel
│   ├── doc.rs                # Existing DocExtractor
│   └── expand.rs             # Existing TokenConfig
```

### Pattern 1: Unified Dispatcher (`format_item()`)

**What:** Single entry point for rendering all 24 ItemKind variants

**When to use:** Any text rendering of rustdoc items

**Design:**
```rust
// src/format/item.rs

use crate::types::detail::DetailLevel;
use rustdoc_types::{Item, ItemKind};

/// Unified formatter for all ItemKind variants
pub struct ItemFormatter {
    detail_level: DetailLevel,
    token_budget: Option<usize>,
    current_tokens: usize,
}

impl ItemFormatter {
    pub fn new(detail_level: DetailLevel, token_budget: Option<usize>) -> Self {
        Self {
            detail_level,
            token_budget,
            current_tokens: 0,
        }
    }

    /// Main dispatcher - handles all 24 ItemKind variants
    pub fn format_item(&mut self, item: &Item) -> FormattedItem {
        let kind = item.kind;
        
        // Check budget before processing
        if self.would_exceed_budget_for(item) {
            return self.format_truncated(item);
        }

        match kind {
            ItemKind::Module => self.format_module(item),
            ItemKind::Struct => self.format_struct(item),
            ItemKind::Enum => self.format_enum(item),
            ItemKind::Union => self.format_union(item),
            ItemKind::Trait => self.format_trait(item),
            ItemKind::Function => self.format_function(item),
            ItemKind::TypeAlias => self.format_type_alias(item),
            ItemKind::Constant => self.format_constant(item),
            ItemKind::Static => self.format_static(item),
            ItemKind::Macro => self.format_macro(item),
            // ... handle all 24 variants
            _ => self.format_generic_item(item),
        }
    }
}
```

**Key principle:** "Consistency wherever the node lands" — same formatting rules apply regardless of depth/position.

### Pattern 2: Doc Comment Truncation

**What:** Smart truncation that preserves sentence boundaries and code blocks

**When to use:** When doc comments exceed token budget

**Design:**
```rust
// src/format/doc.rs

/// Truncate documentation to fit token budget
pub fn truncate_docs(docs: &str, max_tokens: usize) -> (String, bool) {
    let max_chars = max_tokens * 4; // Approximate: 1 token ≈ 4 chars
    if docs.len() <= max_chars {
        return (docs.to_string(), false);
    }

    // Find last sentence boundary before max_chars
    let truncation_point = find_sentence_boundary(docs, max_chars);
    let truncated = &docs[..truncation_point];
    
    // Check if we should preserve code blocks
    if truncated.contains("```") {
        // Preserve code block, truncate prose before it
        let code_start = truncated.find("```").unwrap();
        return (format!("{}...", &truncated[..code_start]), true);
    }
    
    (format!("{}...", truncated.trim()), true)
}

/// Find the last sentence-ending punctuation before max_len
fn find_sentence_boundary(text: &str, max_len: usize) -> usize {
    let slice = &text[..max_len.min(text.len())];
    
    // Look for sentence endings: . ! ? followed by space or end
    let candidates = [". ", "! ", "? "];
    
    for &sep in &candidates {
        if let Some(pos) = slice.rfind(sep) {
            return pos + sep.len();
        }
    }
    
    // Fallback to word boundary
    if let Some(pos) = slice.rfind(' ') {
        return pos;
    }
    
    max_len
}
```

### Pattern 3: Depth-Aware Rendering

**What:** Adjust detail based on tree depth while maintaining format consistency

**When to use:** When rendering nested items at different depths

**Design:**
```rust
impl ItemFormatter {
    /// Format with depth consideration - same format, different detail
    pub fn format_item_at_depth(&mut self, item: &Item, depth: u32) -> FormattedItem {
        // Base format is always the same (REND-01, REND-02)
        let mut formatted = self.format_item(item);
        
        // But detail level increases with depth (REND-03)
        let effective_detail = match depth {
            0 => DetailLevel::Detailed,     // Root: full details
            1 => DetailLevel::Standard,      // Direct fields: standard
            _ => DetailLevel::Minimal,      // Deep nested: minimal
        };
        
        // Adjust based on effective detail
        formatted.apply_detail_level(effective_detail);
        
        formatted
    }
}
```

### Anti-Patterns to Avoid

- **Separate formatters per kind:** Would violate REND-01 (consistent formatting) and REND-02 (single dispatcher)
- **Hardcoding depth logic in formatters:** Depth is a rendering concern, not a formatting concern
- **Truncating during extraction:** Must happen at rendering layer per REND-04 (token budget integrated at rendering layer)
- **Losing code blocks during truncation:** DOCS-05 explicitly requires preserving code blocks

---

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ItemKind string conversion | Custom match statements | `rustdoc_types::ItemKind` variant names directly | Already defined, canonical |
| Doc extraction from Item | String parsing | `DocExtractor::extract_docs()` | Already exists in types/doc.rs |
| Token budget tracking | New implementation | `TokenConfig` from types/expand.rs | Already tracks current_token_count |
| Detail level handling | Custom enum | `DetailLevel` from types/detail.rs | Already has from_flags(), includes_*() methods |
| Terminal styling | Raw ANSI codes | `console::style()` | Already used in format/text.rs |

**Key insight:** The Phase 8 result types already have all the data structures needed. The Phase 9 work is primarily about:
1. Unifying the rendering path (dispatcher design)
2. Adding doc comment handling to existing types
3. Integrating token budget at the rendering layer

---

## Common Pitfalls

### Pitfall 1: Inconsistent Formatting at Different Depths

**What goes wrong:** Root items get detailed formatting, nested items get different format entirely

**Why it happens:** Previous code treated depth as format change, not detail change

**How to avoid:** Use `format_item()` dispatcher for ALL items, use depth only to determine DetailLevel

**Warning signs:** Different formatting functions called based on whether item is root or nested

### Pitfall 2: Doc Comments Not Respected in Minimal Mode

**What goes wrong:** DOCS-03 requires omitting docs in minimal mode, but they still appear

**Why it happens:** Doc extraction happens at wrong layer (extraction vs rendering)

**How to avoid:** Check DetailLevel::is_minimal() BEFORE adding docs to FormattedItem

**Warning signs:** Docs field present in QueryMatch regardless of mode

### Pitfall 3: Token Budget Ignores Doc Comments

**What goes wrong:** REND-04 requires tracking per-item overhead including docs, but budget only tracks structural elements

**Why it happens:** Doc handling added after token budget was implemented

**How to avoid:** Add doc token estimate when calculating item overhead in formatter

**Warning signs:** Token count doesn't increase when docs are displayed

### Pitfall 4: Truncation Loses Code Blocks

**What goes wrong:** DOCS-05 requires preserving code blocks, but simple length truncation removes them

**Why it happens:** Truncation algorithm only looks at character count

**How to avoid:** Detect code blocks (``` markers) before truncating, preserve them even if prose is cut

**Warning signs:** Code examples missing from truncated output

### Pitfall 5: Sentence Boundary Detection Wrong

**What goes wrong:** DOCS-04 requires truncation at sentence boundaries, but implementation cuts mid-sentence

**Why it happens:** Simple character-based truncation without punctuation analysis

**How to avoid:** Scan for . ! ? followed by space/end before truncation point

**Warning signs:** Truncated docs end with partial sentences like "This method retu..."

---

## Code Examples

Verified patterns from existing code:

### Current Text Formatting (needs refactoring)

Source: `src/format/text.rs` lines 44-58

```rust
fn format_query_match(match_: &QueryMatch) {
    // Header: crate::Type (kind)
    let header = format!(
        "{}::{} ({}",
        match_.crate_name, match_.fully_qualified_path, match_.kind
    );
    println!("{}", style(header).bold().cyan());
    
    match &match_.content {
        QueryContent::Type(type_result) => format_type_result(type_result),
        QueryContent::Trait(trait_result) => format_trait_result(trait_result),
        QueryContent::Module(module_result) => format_module_result(module_result),
    }
}
```

### Doc Extraction (already exists)

Source: `src/types/doc.rs` lines 12-14

```rust
pub fn extract_docs(item: &Item) -> Option<String> {
    item.docs.as_ref().map(|s| s.trim().to_string())
}
```

### Token Budget Tracking (already exists)

Source: `src/types/expand.rs` lines 124-132

```rust
fn would_exceed_budget(&self, additional_tokens: usize) -> bool {
    match self.token_config.budget {
        None => false,
        Some(budget) => self.current_token_count + additional_tokens > budget,
    }
}
```

### DetailLevel Methods (already exists)

Source: `src/types/detail.rs` lines 38-71

```rust
impl DetailLevel {
    pub fn is_minimal(self) -> bool {
        matches!(self, Self::Minimal)
    }
    
    pub fn includes_visibility(self) -> bool {
        matches!(self, Self::Standard | Self::Detailed)
    }
    
    pub fn includes_generics(self) -> bool {
        matches!(self, Self::Standard | Self::Detailed)
    }
}
```

### ItemKind Handling (existing, needs unification)

Source: `src/query/engine.rs` lines 295-312

```rust
fn item_kind_to_string(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::Module => "module",
        ItemKind::Struct => "struct",
        ItemKind::Enum => "enum",
        ItemKind::Union => "union",
        ItemKind::Trait => "trait",
        ItemKind::Function => "function",
        ItemKind::TypeAlias => "type",
        ItemKind::Constant { .. } => "constant",
        ItemKind::Static(_) => "static",
        ItemKind::Macro(_) => "macro",
        // ... (not all 24 variants handled)
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate formatters per QueryContent type | Unified format_item() dispatcher | Phase 9 (planned) | Consistent formatting across all ItemKind |
| Docs extracted ad-hoc | DocExtractor in types/doc.rs | Phase 8 (complete) | Centralized doc handling |
| Token budget only for expansion | Token budget at rendering layer | Phase 9 (planned) | Accurate per-item overhead |
| DetailLevel for extraction only | DetailLevel for rendering | Phase 9 (planned) | Display respects mode |

**Deprecated/outdated:**
- QueryContent-specific formatting functions (format_type_result, format_trait_result, format_module_result): Should be replaced with unified dispatcher approach

---

## Open Questions

### Question 1: Exact ItemKind Coverage
**What we know:** rustdoc-types defines 24 ItemKind variants
**What's unclear:** Which variants are most common vs. rarely used in practice?
**Recommendation:** Implement all 24, but prioritize struct/enum/function/trait/module as they're most common

### Question 2: Per-Item Overhead Calculation
**What we know:** Token budget exists, needs to track doc comment tokens
**What's unclear:** Exact formula for "per-item overhead"
**Recommendation:** Use heuristic: base_item_tokens + doc_tokens, where doc_tokens = doc_chars / 4

### Question 3: Code Block Detection
**What we know:** DOCS-05 requires preserving code blocks
**What's unclear:** How to handle inline code (`code`) vs. code blocks (```)?
**Recommendation:** Preserve ``` blocks, truncate inline code as prose

---

## Sources

### Primary (HIGH confidence)
- Context7: rustdoc-types crate - ItemKind, ItemEnum definitions
- `src/types/detail.rs` - DetailLevel implementation
- `src/types/doc.rs` - DocExtractor implementation  
- `src/types/expand.rs` - TokenConfig implementation

### Secondary (MEDIUM confidence)
- `src/format/text.rs` - Current text formatting patterns
- `src/query/engine.rs` - Current ItemKind handling
- GitHub rustdoc-types CHANGELOG - Recent ItemKind changes

### Tertiary (LOW confidence)
- Web search results on rustdoc JSON format - Need verification against actual rustdoc-types crate

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All needed types exist, no new dependencies required
- Architecture: HIGH - Dispatcher pattern clearly defined, existing code supports it
- Pitfalls: HIGH - Identified from existing code patterns and requirements

**Research date:** 2026-02-13
**Valid until:** 2026-03-13 (30 days - stable domain)
