---
phase: 09-unified-rendering
plan: 05
subsystem: cli
tags: [unified-formatter, cli, expand-command, detail-level, token-budget]

# Dependency graph
requires:
  - phase: 09-01
    provides: ItemFormatter and format_item() dispatcher
  - phase: 09-02
    provides: DocHandler for doc comment formatting
  - phase: 09-03
    provides: BudgetTracker for token budget control
affects: [future CLI integration, rendering consistency]

# Tech tracking
tech-stack:
  added: []
  patterns: [unified formatter wired to CLI command]

key-files:
  created: []
  modified:
    - src/cli/expand.rs - Wired unified formatter into expand command

key-decisions:
  - "Pass DetailLevel from CLI flags to format_expand_result_with_formatter"
  - "Pass token_budget from CLI --tokens flag to formatter"

patterns-established:
  - "Unified formatter integration pattern"

# Metrics
duration: 1min
completed: 2026-02-13
---

# Phase 09 Plan 05: Wire Unified Formatter into CLI Expand Summary

**Wired unified ItemFormatter into CLI expand command, passing DetailLevel and token budget**

## Performance

- **Duration:** 1 min
- **Tasks:** 1/1
- **Files modified:** 1

## Accomplishments
- Added import for format_expand_result_with_formatter and format_with_item_formatter
- Replaced format_expand_result call with unified formatter variant
- Passed detail_level (from DetailLevel::from_flags) and token_budget (self.tokens)
- All 308 tests pass

## Task Commits

1. **Task 1: Wire unified formatter into CLI** - `17a9c78` (feat)

**Plan metadata:** (included in task commit)

## Files Created/Modified
- `src/cli/expand.rs` - Added import and replaced format_expand_result with format_expand_result_with_formatter

## Decisions Made
None - plan executed exactly as specified

## Deviations from Plan
None - plan executed exactly as written

## Issues Encountered
None

## Next Phase Readiness
- Unified formatter is now wired into CLI expand command
- Ready for further integration or enhancements

---
*Phase: 09-unified-rendering*
*Completed: 2026-02-13*
