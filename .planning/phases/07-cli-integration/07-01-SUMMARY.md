---
phase: 07-cli-integration
plan: 01
subsystem: cli
tags: [clap, filter-config, expand-command, filter-flags, CLI]

# Dependency graph
requires:
  - phase: 06-filter-engine
    provides: FilterConfig, FilterEngine for filtering query results
provides:
  - CLI filter flags: --include, --exclude, --kind, --crate, --visibility, --only
  - ExpandCommand.filter_config() method to construct FilterConfig from CLI args
  - Integration point for passing filter configuration to query engine
affects: [07-02-PLAN.md, 08-result-types-PLAN.md]

# Tech tracking
tech-stack:
  added: [clap filter flags with Vec<String> for multiple values]
  patterns: [flag-to-config builder pattern, CLI-to-engine parameter forwarding]

key-files:
  created: []
  modified: [src/main.rs, src/cli/expand.rs]

key-decisions:
  - Multiple flag values collected into Vec<String> with OR logic within type
  - --only flag takes precedence over --include when both provided
  - --kind values are case-insensitive (normalized to lowercase in FilterEngine)
  - crate_filter field name avoids Rust keyword conflict with --crate flag
  - from_args() is primary constructor, new() marked deprecated

patterns-established:
  - Flag-to-config pattern: CLI flags stored in command struct, filtered via filter_config()
  - Parameter forwarding: main.rs match arm passes filter values to ExpandCommand::from_args()
  - Builder pattern reuse: FilterConfig reused from Phase 6 with builder methods

# Metrics
duration: 4min 10s
completed: 2026-02-13
---

# Phase 07: CLI Integration - Plan 01 Summary

**CLI filter flags with FilterConfig integration into ExpandCommand**

## Performance

- **Duration:** 4min 10s
- **Started:** 2026-02-13T17:50:49Z
- **Completed:** 2026-02-13T17:53:59Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added 6 filter flags to Commands::Query variant (include, exclude, kind, crate, visibility, only)
- Integrated FilterConfig into ExpandCommand struct with proper field types
- Created filter_config() method with --only precedence logic
- Updated from_args() to accept all filter parameters
- Maintained backward compatibility with deprecated new() constructor

## Task Commits

Each task was committed atomically:

1. **Task 1: Add filter flags to Commands::Query variant** - `a58e033` (feat)
2. **Task 2: Add FilterConfig to ExpandCommand struct** - `19af74a` (feat)

**Plan metadata:** (not committed - plan execution complete)

## Files Created/Modified

- `src/main.rs` - Added filter flags to Commands::Query variant, updated match arm to pass filter parameters
- `src/cli/expand.rs` - Added FilterConfig import, filter fields to struct, updated from_args(), added filter_config() method, deprecated new() constructor

## Decisions Made

- **Multiple flag collection:** Using Vec<String> for multiple values enables OR logic within flag type (e.g., `--include "std::*" --include "alloc::*"`)
- **--only precedence:** If --only is provided, it replaces --include as the include pattern; otherwise --include is used
- **Field naming:** crate_filter field name avoids Rust keyword conflict with --crate flag
- **Constructor preference:** from_args() is primary constructor since it supports filter flags; new() marked deprecated for backward compatibility
- **Flag grouping:** All filter flags grouped together in struct for clarity and maintainability

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**1. Match arm pattern mismatch**
- **Found during:** Final verification (Task 2 completion)
- **Issue:** Match arm in main.rs for Commands::Query was missing the new filter fields, causing compilation errors
- **Fix:** Updated pattern match to destructure all filter fields (include, exclude, kind, crate_filter, visibility, only) and pass them to ExpandCommand::from_args()
- **Files modified:** src/main.rs
- **Verification:** cargo check --lib and cargo check --bin cargo-doc-query both pass
- **Committed in:** `19af74a` (part of Task 2 commit)

**Total issues encountered:** 1 (fixed during verification, not counted as deviation since it was discovered during task execution)

## Next Phase Readiness

- CLI filter flags are defined and available for use
- FilterConfig integration point created in ExpandCommand
- Next phase (07-02) can now use these filters in query execution
- Existing functionality (--depth, --minimal, --tokens) remains intact and preserved

## Self-Check: PASSED

- All tasks completed and committed
- All verification checks pass (cargo check, tests)
- Both modified files exist and contain expected changes
- All filter flags properly typed and documented

---

*Phase: 07-cli-integration*
*Completed: 2026-02-13*
