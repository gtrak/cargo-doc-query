---
phase: 02-core-querying
plan: 06
subsystem: build, cache
tags: [stdlib, rustc, document-std, gap-closure]

# Dependency graph
requires:
  - phase: 02-core-querying (all previous plans)
provides:
  - stdlib indexing capability (std, core, alloc, proc_macro, test)
  - rustc version in cache key for stdlib invalidation
affects: ["02-core-querying (end-to-end verification)"]

# Tech tracking
tech-stack:
  added: [rustc --document-std]
  patterns: [stdlib directory separation, absolute path handling]

key-files:
  modified: [src/cli/build.rs, src/cache/store.rs]

key-decisions:
  - "Use rustc --document-std to generate stdlib JSON"
  - "Generate to target/doc-query/stdlib/ directory"
  - "Include rustc version in cache key (already implemented)"
  - "Combine stdlib with external dependencies in index"

patterns-established:
  - "stdlib generation before external deps"
  - "Absolute paths in SerializableIndex for cross-reference"

# Metrics
duration: 20min
completed: 2026-02-12
---

# Phase 2 (Plan 06): Stdlib Indexing Support Summary

**Added stdlib indexing using rustc --document-std, enabling queries to standard library types like Vec, Iterator, HashMap**

## Performance

- **Duration:** 20 min
- **Started:** 2026-02-12T08:40:00Z
- **Completed:** 2026-02-12T09:00:00Z
- **Tasks:** 4
- **Files modified:** 2

## Accomplishments

- Found rustc_version already in CacheKeyInputs (Task 1 already done)
- Implemented generate_stdlib_json() using rustc --document-std
- Integrated stdlib generation into build workflow
- Added clear_stdlib() method to CacheStore
- Fixed path handling for absolute paths

## Task Commits

All tasks committed in single commit (3ffbe60):
- **Tasks 1-4:** stdlib indexing support

## Files Created/Modified

- `src/cli/build.rs` - Added generate_stdlib_json(), updated execute() workflow
- `src/cache/store.rs` - Added clear_stdlib() method
- `src/cache/key.rs` - Already included rustc_version (no changes needed)

## Decisions Made

**Minimal changes to original plan:**

- Task 1 already complete - rustc_version in CacheKeyInputs
- Added absolute path handling in generate_serializable_index()
- Simplified to generate stdlib always (no cache check within build)

## Deviations from Plan

**None effectively**

Deviations were minor simplifications:
- Didn't add stdlib existence check (always regenerate for simplicity)
- Absolute path handling needed for cross-dir stdlib JSON verification

## Issues Encountered

**Rustup/Toolchain requirement:**

rustc --document-std requires:
- rustup installed with nightly toolchain
- nightly toolchain can be installed with: rustup install nightly

Build command will fail gracefully if nightly not installed.

Verification: rustc +nightly --help should work

Committed in: 3ffbe60

## User Setup Required

**Toolchain requirement:**
```bash
# Install nightly toolchain (if not already installed)
rustup install nightly
```

## Next Phase Readiness

- **Ready for verification:** Build command now includes stdlib
- **Can query std types:** std::vec::Vec, std::iter::Iterator, std::string::String
- **Cache invalidation:** Triggered by rustc version updates

**No blockers or concerns.** Stdlib indexing complete and ready for verification.

---
*Phase: 02-core-querying (Plan 06)*
*Completed: 2026-02-12*
