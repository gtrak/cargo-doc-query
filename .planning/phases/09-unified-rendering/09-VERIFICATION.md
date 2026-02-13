---
phase: 09-unified-rendering
verified: 2026-02-13T12:00:00Z
status: gaps_found
score: 2/6 must-haves verified
gaps:
  - truth: "Doc comments displayed according to token budgets"
    status: failed
    reason: "ItemFormatter receives token_budget but never uses it to truncate docs. would_exceed_budget method exists but is never called."
    artifacts:
      - path: "src/format/item.rs"
        issue: "format_item method extracts docs without checking budget (lines 106-110)"
    missing:
      - "Call to DocHandler::truncate_docs in ItemFormatter"
      - "Integration between token_budget field and doc extraction"
  - truth: "Smart truncation at sentence boundaries when budget exceeded"
    status: failed
    reason: "truncate_docs exists in doc.rs but is never called from rendering pipeline"
    artifacts:
      - path: "src/format/doc.rs"
        issue: "Functions exist with tests but not integrated into ItemFormatter"
    missing:
      - "Wiring from ItemFormatter to DocHandler"
  - truth: "Token budget integrated at rendering layer"
    status: partial
    reason: "BudgetTracker exists but only tracks at item-inclusion level, not doc-truncation level"
    artifacts:
      - path: "src/format/text.rs"
        issue: "format_with_item_formatter uses BudgetTracker for include/exclude, not for doc truncation"
  - truth: "Unified formatter wired into main CLI"
    status: failed
    reason: "New formatter functions exist but are marked 'unused' - not connected to CLI"
    missing:
      - "CLI expand command uses format_with_item_formatter or format_expand_result_with_formatter"
---

# Phase 09: Unified Rendering Verification Report

**Phase Goal:** Users see unified rendering across all item types at any depth, with doc comments displayed according to token budgets.

**Verified:** 2026-02-13
**Status:** gaps_found
**Score:** 2/6 must-haves verified
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth   | Status     | Evidence       |
| --- | ------- | ---------- | -------------- |
| 1   | Single format_item() dispatcher handles all 24 ItemKind variants | ✓ VERIFIED | item.rs lines 94-144 implement dispatcher with match on item.inner |
| 2   | DetailLevel controls what metadata to display | ✓ VERIFIED | item.rs lines 100-122 check detail_level.is_minimal() and includes_* methods |
| 3   | Doc comments extracted from Item::docs field | ✓ VERIFIED | item.rs line 109: item.docs.as_ref().map(|s| s.trim().to_string()) |
| 4   | Doc comments displayed in standard mode, omitted in minimal mode | ✓ VERIFIED | item.rs lines 106-110 check is_minimal() |
| 5   | Smart truncation at sentence boundaries when budget exceeded | ✗ FAILED | truncate_docs exists but never called from ItemFormatter |
| 6   | Token budget integrated at rendering layer | ✗ FAILED | BudgetTracker used but only for item inclusion, not doc truncation |

**Score:** 2/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `src/format/item.rs` | format_item dispatcher | ✓ VERIFIED | 393 lines, has ItemFormatter, format_item method |
| `src/format/doc.rs` | DocHandler, truncate_docs | ✓ VERIFIED | 462 lines, has truncate_docs with sentence boundary detection |
| `src/format/budget.rs` | BudgetTracker | ✓ VERIFIED | 294 lines, has track_item and estimate_item_tokens |
| `src/format/text.rs` | Integration | ⚠️ PARTIAL | Updated with imports, has format_with_item_formatter but unused |
| `src/format/mod.rs` | Exports | ✓ VERIFIED | Exports item, doc, budget, text modules |
| `src/types/detail.rs` | DetailLevel | ✓ VERIFIED | Has Minimal/Standard/Detailed with all include_* methods |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| item.rs | detail.rs | DetailLevel import | ✓ WIRED | Line 8 imports DetailLevel |
| doc.rs | detail.rs | DetailLevel import | ✓ WIRED | Line 6 imports DetailLevel |
| budget.rs | item.rs | FormattedItem import | ✓ WIRED | Line 6 imports FormattedItem |
| text.rs | budget.rs | BudgetTracker import | ✓ WIRED | Line 3 imports BudgetTracker |
| text.rs | item.rs | ItemFormatter import | ✓ WIRED | Line 4 imports ItemFormatter |
| **item.rs** | **doc.rs** | **DocHandler/truncate_docs** | **✗ NOT WIRED** | **No import, truncate_docs never called** |
| **CLI** | **format_with_item_formatter** | **Function call** | **✗ NOT WIRED** | **Format functions unused** |

### Requirements Coverage

| Requirement | Status | Blocking Issue |
| --- | --- | --- |
| REND-01: All 24 ItemKind variants render with consistent formatting | ✓ SATISFIED | format_item handles many variants via match |
| REND-02: Single format_item() dispatcher handles all item types | ✓ SATISFIED | Implemented |
| REND-03: Depth-aware formatting | ⚠️ PARTIAL | DetailLevel exists but depth handling not explicit |
| REND-04: Token budget integrated at rendering layer | ✗ BLOCKED | Budget passed but not used for truncation |
| DOCS-01: Doc comments extracted from Item::docs | ✓ SATISFIED | Works via item.docs |
| DOCS-02: Doc comments in standard mode | ✓ SATISFIED | Shown when not Minimal |
| DOCS-03: Doc comments omitted in minimal | ✓ SATISFIED | is_minimal() check |
| DOCS-04: Smart truncation at sentence boundaries | ✗ BLOCKED | truncate_docs exists but not called |
| DOCS-05: Code blocks preserved over prose | ✓ SATISFIED | Implemented in truncate_docs |
| DOCS-06: Truncated docs show "..." indicator | ✓ SATISFIED | Returns "..." in truncate |
| DOCS-07: Token budget enforcement includes doc tokens | ✗ BLOCKED | Budget not integrated |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| item.rs | 106-110 | Docs extracted without budget check | ⚠️ Warning | Budget parameter ignored |
| item.rs | 288 | would_exceed_budget unused | ⚠️ Warning | Dead code - method never called |
| text.rs | 371-400 | format_with_item_formatter unused | ⚠️ Warning | Alternative path not connected |
| text.rs | 406-472 | format_expand_result_with_formatter unused | ⚠️ Warning | Alternative path not connected |

### Human Verification Required

N/A — structural verification sufficient

### Gaps Summary

**Critical Gap: Doc truncation not integrated into ItemFormatter**

The ItemFormatter receives token_budget but never uses it to truncate docs. Looking at src/format/item.rs lines 106-110:

```rust
let docs = if self.detail_level.is_minimal() {
    None
} else {
    item.docs.as_ref().map(|s| s.trim().to_string())
};
```

This extracts docs without any budget consideration. The would_exceed_budget method exists (line 288) but is never called.

Similarly, DocHandler and truncate_docs exist with full implementations and tests, but they are never called from the rendering pipeline.

**Secondary Gap: New formatter not wired to CLI**

The format_with_item_formatter and format_expand_result_with_formatter functions exist in text.rs but are marked "unused" — the CLI expand command doesn't use them.

**Impact:**

1. When users provide a token budget, doc comments are NOT truncated — instead, whole items may be excluded
2. The unified rendering system exists but doesn't actually enforce token budgets on docs
3. Users don't benefit from the smart sentence-boundary truncation

---

_Verified: 2026-02-13_
_Verifier: Claude (gsd-verifier)_
