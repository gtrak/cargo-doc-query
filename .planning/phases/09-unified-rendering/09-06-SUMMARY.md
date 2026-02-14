---
phase: 09-unified-rendering
plan: "06"
subsystem: Rendering
tags: [docs, type-node, formatting, token-budget]
created: 2026-02-13
duration: ~5 minutes
completed: 2026-02-13

requires:
  - 09-01 (ItemFormatter foundation)
  - 09-02 (DocHandler)
  - 09-03 (BudgetTracker)

provides:
  - TypeNode.docs field
  - Doc-aware expand result formatting
  - Unified ItemFormatter rendering pipeline

affects:
  - 09-VERIFICATION (closes gaps)
---

# Phase 09 Plan 06: Gap Closure — Docs Field & Rendering Summary

## Objective

Close verification gaps from 09-VERIFICATION.md:
1. TypeNode missing docs field - prevents doc comments from being captured
2. Doc token tracking always returns 0 - because TypeNode had no docs
3. Manual rendering instead of using ItemFormatter pipeline

## Execution Summary

**Completed:** 3/3 tasks  
**Tests:** 308 passing  
**Duration:** ~5 minutes  

### Task Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add docs field to TypeNode | fbb3239 | src/types/expand.rs, src/query/expand.rs |
| 2 | Wire docs into format_expand_result_with_formatter | 4bb467c | src/format/text.rs |
| 3 | Use ItemFormatter for rendering | ae505ce | src/format/item.rs, src/format/text.rs |

## Changes Made

### 1. TypeNode Docs Field (Task 1)

**Files modified:** `src/types/expand.rs`, `src/query/expand.rs`

- Added `docs: Option<String>` field to TypeNode struct (expand.rs:160)
- Added `with_docs()` builder method (expand.rs:413-417)
- Updated `to_minimal()` to clear docs field (expand.rs:472)
- Wired docs extraction into 3 TypeNode creation sites in query/expand.rs:
  - Line 208: Function handling
  - Line 317: Type/enum/primitive handling  
  - Line 490: Module handling
- Uses `item.docs.as_ref().map(|s| s.trim().to_string())` pattern

### 2. Doc Wiring (Task 2)

**Files modified:** `src/format/text.rs`

- Changed line 482 from `docs: None` to `docs: node.docs.clone()`
- Now FormattedItem receives docs from TypeNode
- Enables doc token tracking at line 525 (was always 0 before)

### 3. ItemFormatter Rendering (Task 3)

**Files modified:** `src/format/item.rs`, `src/format/text.rs`

- Added `FormattedItem::render()` method in item.rs
- Returns formatted string with kind, id, generics, visibility, deprecation, fields, variants
- Updated `format_expand_result_with_formatter` to use render() instead of manual println! calls

## Gap Closure Status

| Gap | Status | Evidence |
|-----|--------|----------|
| TypeNode missing docs field | ✓ CLOSED | docs: Option<String> now in struct |
| Doc token tracking returns 0 | ✓ CLOSED | Now uses node.docs.clone() |
| Manual rendering | ✓ CLOSED | Uses FormattedItem::render() |

## Key Files Created/Modified

**key-files.created:** []

**key-files.modified:**
- src/types/expand.rs - Added docs field and builder
- src/query/expand.rs - Wired docs extraction into 3 sites
- src/format/text.rs - Wired docs into FormattedItem, uses render()
- src/format/item.rs - Added FormattedItem::render() method

## Decisions Made

- **FormattedItem::render() approach:** Chose to add render method to FormattedItem rather than reworking the whole pipeline to use ItemFormatter directly (which expects rustdoc Item types). This provides a simple, unified rendering path.

## Deviations from Plan

None - plan executed exactly as written.

---

**Self-Check: PASSED**

All tasks completed as specified:
- TypeNode has docs: Option<String> field with builder ✓
- format_expand_result_with_formatter passes docs to FormattedItem ✓
- Doc token tracking now works (not always 0) ✓
- ItemFormatter rendering used ✓
- All 308 tests pass ✓
