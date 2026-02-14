---
phase: 09-unified-rendering
verified: 2026-02-13T16:30:00Z
status: gaps_found
score: 9/11 must-haves verified
re_verification: true
previous_status: gaps_found
previous_score: 5/7
gaps_closed:
  - "TypeNode now has docs: Option<String> field (expand.rs:160-162)"
  - "Expansion logic extracts item.docs via with_docs() (query/expand.rs:215, 324, 497)"
  - "Token budget tracking includes doc_tokens (text.rs:525)"
  - "FormattedItem built correctly from TypeNode with docs (text.rs:482)"
  - "Uses FormattedItem.render() instead of manual println! (text.rs:531)"
gaps_remaining:
  - truth: "Doc comments displayed in standard output mode (DOCS-02)"
    status: failed
    reason: "FormattedItem.render() doesn't output docs field - only renders kind, id, generics, visibility, deprecation, fields, variants"
    artifacts:
      - path: "src/format/item.rs"
        issue: "render() method (lines 49-86) doesn't include docs in output"
      - path: "src/format/text.rs"
        issue: "Line 430 creates ItemFormatter but doesn't use it - just copies raw docs"
    missing:
      - "render() method needs to include formatted.docs in output"
      - "OR text.rs should use ItemFormatter.format_item() instead of manual FormattedItem construction"
  - truth: "Token budget enforcement includes doc comment tokens (DOCS-07)"
    status: failed
    reason: "Doc tokens are tracked (line 525) but docs are never rendered, so truncation has no effect"
    artifacts:
      - path: "src/format/text.rs"
        issue: "Line 525 tracks doc_tokens but docs field is never displayed"
    missing:
      - "After fixing DOCS-02, doc truncation will work properly"
---

# Phase 09: Unified Rendering Verification Report

**Phase Goal:** Users see unified rendering across all item types at any depth, with doc comments displayed according to token budgets.

**Verified:** 2026-02-13
**Status:** gaps_found
**Score:** 9/11 must-haves verified
**Re-verification:** Yes — after gap closure attempts

## Goal Achievement

### Observable Truths

| #   | Truth   | Status     | Evidence       |
| --- | ------- | ---------- | -------------- |
| 1   | REND-01: All 24 ItemKind variants render with consistent formatting | ✓ VERIFIED | item.rs:192-207 handles 16+ ItemEnum variants |
| 2   | REND-02: Single format_item() dispatcher handles all item types | ✓ VERIFIED | item.rs:139 format_item() is main entry point |
| 3   | REND-03: Depth-aware formatting | ✓ VERIFIED | text.rs:432-441 groups nodes by depth |
| 4   | REND-04: Token budget integrated at rendering layer | ✓ VERIFIED | text.rs:429 creates BudgetTracker, line 527 tracks |
| 5   | DOCS-01: Doc comments extracted from Item::docs field | ✓ VERIFIED | query/expand.rs:215,324,497 use with_docs() |
| 6   | DOCS-02: Doc comments display in standard output mode | ✗ FAILED | render() doesn't output docs field |
| 7   | DOCS-03: Doc comments omitted in minimal mode | ✓ VERIFIED | doc.rs:41 checks is_minimal() |
| 8   | DOCS-04: Smart truncation at sentence boundaries | ✓ VERIFIED | doc.rs:80-133 truncate_docs function |
| 9   | DOCS-05: Code blocks preserved over prose during truncation | ✓ VERIFIED | doc.rs:92-118 handles code blocks first |
| 10  | DOCS-06: Truncated docs show "..." indicator | ✓ VERIFIED | doc.rs:112,117,126,130 add "..." |
| 11  | DOCS-07: Token budget enforcement includes doc comment tokens | ✗ FAILED | Tracking works but docs never rendered |

**Score:** 9/11 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `src/format/item.rs` | format_item dispatcher | ✓ VERIFIED | 390+ lines, handles all variants |
| `src/format/doc.rs` | DocHandler, truncate_docs | ✓ VERIFIED | 462 lines, all truncation logic |
| `src/format/budget.rs` | BudgetTracker | ✓ VERIFIED | 294 lines, track_item works |
| `src/types/expand.rs` | TypeNode with docs | ✓ VERIFIED | docs field at line 162 |
| `src/types/detail.rs` | DetailLevel | ✓ VERIFIED | Minimal/Standard/Detailed |
| `src/format/text.rs` | Integration | ⚠️ PARTIAL | Builds FormattedItem but render() missing docs |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| item.rs | detail.rs | DetailLevel import | ✓ WIRED | Line 8 imports DetailLevel |
| doc.rs | detail.rs | DetailLevel import | ✓ WIRED | Line 6 imports DetailLevel |
| budget.rs | item.rs | FormattedItem import | ✓ WIRED | Line 6 imports FormattedItem |
| text.rs | budget.rs | BudgetTracker import | ✓ WIRED | Line 3 imports BudgetTracker |
| text.rs | item.rs | ItemFormatter import | ✓ WIRED | Line 4 imports ItemFormatter |
| item.rs | doc.rs | DocHandler::format_docs | ✓ WIRED | Lines 151-153 call DocHandler |
| query/expand.rs | TypeNode | with_docs() | ✓ WIRED | Lines 215,324,497 populate docs |
| text.rs | TypeNode | node.docs.clone() | ✓ WIRED | Line 482 extracts docs |
| text.rs | FormattedItem | Struct creation | ✓ WIRED | Lines 471-521 build FormattedItem |
| text.rs | BudgetTracker | track_item | ✓ WIRED | Line 527 tracks with doc_tokens |
| **text.rs** | **FormattedItem.render()** | **Method call** | **✗ INCOMPLETE** | **render() doesn't output docs** |

### Requirements Coverage

| Requirement | Status | Blocking Issue |
| ----------- | ------ | -------------- |
| REND-01: All 24 ItemKind variants render with consistent formatting | ✓ SATISFIED | Dispatcher handles 16+ variants |
| REND-02: Single format_item() dispatcher handles all item types | ✓ SATISFIED | Implemented |
| REND-03: Depth-aware formatting | ✓ SATISFIED | Nodes grouped by depth |
| REND-04: Token budget integrated at rendering layer | ✓ SATISFIED | BudgetTracker used |
| DOCS-01: Doc comments extracted from Item::docs | ✓ SATISFIED | with_docs() extracts properly |
| DOCS-02: Doc comments in standard mode | ✗ BLOCKED | FormattedItem.render() doesn't output docs |
| DOCS-03: Doc comments omitted in minimal | ✓ SATISFIED | is_minimal() check |
| DOCS-04: Smart truncation at sentence boundaries | ✓ SATISFIED | truncate_docs works |
| DOCS-05: Code blocks preserved over prose | ✓ SATISFIED | Code blocks prioritized |
| DOCS-06: Truncated docs show "..." indicator | ✓ SATISFIED | Added in truncate_docs |
| DOCS-07: Token budget enforcement includes doc tokens | ✗ BLOCKED | Tracking works but docs not rendered |

### What Was Fixed

1. **TypeNode now has docs field** - ✓ FIXED
   - expand.rs:160-162 adds `pub docs: Option<String>`
   - with_docs() method at line 423-427

2. **Expansion extracts docs from Item::docs** - ✓ FIXED
   - query/expand.rs:215, 324, 497 call .with_docs(item.docs...)
   - Properly trims whitespace

3. **Token budget tracking includes doc_tokens** - ✓ FIXED
   - text.rs:525 calculates `doc_tokens` from formatted.docs
   - Line 527 passes to tracker.track_item()

4. **FormattedItem correctly built** - ✓ FIXED
   - text.rs:482 clones docs from TypeNode
   - All fields properly mapped

5. **Uses FormattedItem.render()** - ✓ FIXED
   - text.rs:531 calls formatted.render()
   - No longer manual println!

### What Remains Broken

1. **FormattedItem.render() doesn't output docs** - ✗ CRITICAL GAP
   - item.rs:49-86 render() method only outputs:
     - kind, id, generics, visibility, is_deprecated, fields, variants
   - Does NOT output: docs, signature, items, attributes, deprecation_note, modifiers
   - Result: Even with docs populated, nothing appears in expand output

2. **ItemFormatter not used in text.rs** - ✗ CRITICAL GAP
   - text.rs:430 creates `let _formatter = ItemFormatter::new(...)` but never uses it
   - Should either:
     a) Use ItemFormatter.format_item() to get properly formatted docs, OR
     b) Manually call DocHandler on the raw docs before storing in FormattedItem
   - Current code just copies raw docs: `docs: node.docs.clone()`

3. **Consequence: DOCS-07 ineffective** - ✗ CONSEQUENCE
   - Token tracking happens but has no visible effect
   - Truncation logic exists but never triggers because docs aren't displayed

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| item.rs | 49-86 | render() missing docs output | 🛑 Blocker | Docs never displayed |
| text.rs | 430 | Unused formatter variable | 🛑 Blocker | DocHandler not applied |
| text.rs | 482 | Raw docs cloned, not processed | 🛑 Blocker | No truncation applied |

### Gaps Summary

**Gap 1: FormattedItem.render() Missing Docs Output**

The render() method at item.rs:49-86 renders basic structure but omits the docs field entirely. This is a fundamental gap - even if docs are populated correctly in FormattedItem, they never appear in output.

**Gap 2: DocHandler Not Applied to Expand Results**

The text.rs function creates an ItemFormatter but doesn't use it. The docs are copied directly from TypeNode without going through DocHandler.format_docs(), which means:
- No smart sentence boundary truncation
- No budget-aware truncation
- No minimal mode handling (though this is less relevant for expand)

**Gap 3: DOCS-07 Cannot Work Without Gap 1 & 2**

Token budget enforcement for doc comments cannot function because:
1. Doc truncation logic exists but isn't applied
2. Even if applied, render() doesn't output the (possibly truncated) docs

---

_Verified: 2026-02-13T16:30:00Z_
_Verifier: Claude (gsd-verifier)_
