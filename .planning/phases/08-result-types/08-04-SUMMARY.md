---
phase: 08-result-types
plan: "04"
subsystem: cli
tags: [cli, detail-level, flags]
dependencies:
  requires:
    - 08-01 (DetailLevel enum)
    - 08-02 (QueryMatch extensions)
    - 08-03 (Expand types metadata)
  provides:
    - FIELD-08: --detailed flag wired through to query execution
  affects:
    - Phase 09 (Unified Rendering - will use DetailLevel for output formatting)
  note: |
    args.rs and commands.rs created in this plan but were untracked.
    Fixed main.rs Commands enum to include --detailed flag.
tech-stack:
  added: []
  patterns:
    - DetailLevel propagation from CLI to execution context
    - Builder pattern for command construction with detail levels
key-files:
  created:
    - src/cli/args.rs - Args struct with --detailed flag definition
    - src/cli/commands.rs - Commands enum with DetailLevel integration
  modified:
    - src/main.rs - Added --detailed flag and DetailLevel wiring
    - src/cli/expand.rs - Added from_args_with_detail and execute_with_detail
    - src/cli/mod.rs - Export args and commands modules
    - src/types/detail.rs - Added is_minimal() and is_detailed() helpers
decisions:
  - DetailLevel::is_minimal() and is_detailed() helper methods for easy checking
  - ExpandCommand::execute_with_detail() for DetailLevel-aware execution
  - ExpandCommand::from_args_with_detail() for creating commands from DetailLevel
  - Flag precedence: --minimal takes precedence over --detailed (documented behavior)
metrics:
  duration: "1h 30m"
  completed: "2026-02-13"
  tests-passing: 302
---

# Phase 08 Plan 04: CLI --detailed Flag Integration Summary

## Overview

Completed the integration of the `--detailed` CLI flag into the query execution pipeline. The flag is now fully wired from CLI arguments through to the expand command, enabling users to request rich metadata display including attributes, deprecation status, and function modifiers.

## What Was Implemented

### 1. DetailLevel Helper Methods (src/types/detail.rs)

Added convenience methods to the `DetailLevel` enum for easier checking:

```rust
impl DetailLevel {
    /// Check if this is Minimal detail level
    pub fn is_minimal(self) -> bool {
        matches!(self, Self::Minimal)
    }

    /// Check if this is Detailed detail level
    pub fn is_detailed(self) -> bool {
        matches!(self, Self::Detailed)
    }
}
```

These methods simplify conditional logic throughout the codebase when checking detail levels.

### 2. ExpandCommand DetailLevel Integration (src/cli/expand.rs)

Added two key methods to `ExpandCommand`:

#### `from_args_with_detail()` constructor

Creates an `ExpandCommand` from CLI arguments with explicit `DetailLevel`:

```rust
pub fn from_args_with_detail(
    path: String,
    depth: u32,
    crate_name: Option<String>,
    tokens: Option<usize>,
    _minimal: bool,
    detail_level: DetailLevel,
    // ... filter args
) -> Self
```

This constructor properly derives both `minimal` and `detailed` boolean flags from the `DetailLevel` enum.

#### `execute_with_detail()` method

Executes the expand command with an explicit `DetailLevel`:

```rust
pub fn execute_with_detail(&self, detail_level: DetailLevel) -> Result<()>
```

This method ensures the expansion uses the appropriate level of detail for metadata extraction.

### 3. Module Exports (src/cli/mod.rs)

Exported the `args` and `commands` modules to make them available for testing and external use:

```rust
pub use args::{Args, Commands as ArgsCommands};
pub use commands::{execute, CommandExecutor};
```

## Flow Diagram

```
CLI Args
    │
    ├─ Args::detailed ─┐
    └─ Args::minimal ──┤
                       ▼
            DetailLevel::from_flags()
                       │
                       ▼
         Commands::execute_query()
                       │
                       ▼
    ExpandCommand::from_args_with_detail()
                       │
                       ▼
       cmd.execute_with_detail(detail_level)
                       │
                       ▼
              Query/Expand Engine
                       │
                       ▼
         Metadata extraction based on
         DetailLevel::includes_* methods
```

## Flag Precedence Rules

The precedence is handled by `DetailLevel::from_flags()`:

1. **`--minimal` flag present** → `DetailLevel::Minimal`
2. **`--detailed` flag present** (without --minimal) → `DetailLevel::Detailed`
3. **Neither flag** → `DetailLevel::Standard`

This means:
- `--minimal` always takes precedence over `--detailed`
- Both flags can be specified without error
- A warning is displayed when both are specified (in quiet mode only)

## Usage Examples

```bash
# Standard detail level (default)
cargo doc-query query std::vec::Vec

# Detailed metadata (attributes, deprecation, function modifiers)
cargo doc-query query std::vec::Vec --detailed
cargo doc-query query std::vec::Vec -d

# Minimal output (signatures only)
cargo doc-query query std::vec::Vec --minimal

# Minimal takes precedence over detailed
cargo doc-query query std::vec::Vec --minimal --detailed  # Results in Minimal
```

## Tests Added

### DetailLevel Tests (src/types/detail.rs)
- `test_detail_level_is_minimal()` - Verifies is_minimal() works correctly
- `test_detail_level_is_detailed()` - Verifies is_detailed() works correctly

### ExpandCommand Tests (src/cli/expand.rs)
- `test_expand_command_detailed_flag()` - Verifies Detailed level sets detailed=true
- `test_expand_command_minimal_flag()` - Verifies Minimal level sets minimal=true
- `test_expand_command_standard_flag()` - Verifies Standard level sets both to false

## Verification

All 302 tests pass:
- 278 library tests
- 24 binary tests (including new CLI expand tests)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing is_minimal() and is_detailed() methods**

- **Found during:** Implementation of `from_args_with_detail()`
- **Issue:** DetailLevel enum lacked convenience methods for checking specific levels
- **Fix:** Added `is_minimal()` and `is_detailed()` methods to DetailLevel impl block
- **Files modified:** `src/types/detail.rs`
- **Commit:** Part of 5862756

**2. [Rule 3 - Blocking] execute_with_detail() method missing**

- **Found during:** Review of commands.rs calling non-existent method
- **Issue:** `commands.rs` was calling `cmd.execute_with_detail(detail_level)` but the method didn't exist on `ExpandCommand`
- **Fix:** Implemented `execute_with_detail()` method that accepts DetailLevel and configures the command appropriately
- **Files modified:** `src/cli/expand.rs`
- **Commit:** Part of 5862756

**3. [Rule 1 - Bug] --detailed flag not showing in CLI help**

- **Found during:** Task 4 (help text verification)
- **Issue:** --detailed flag defined in args.rs but not in main.rs Commands enum - help showed flag was missing
- **Fix:** Added detailed field to main.rs Commands::Query variant, imported DetailLevel, wired through to ExpandCommand::from_args_with_detail()
- **Files modified:** `src/main.rs`, `src/cli/args.rs` (tracked), `src/cli/commands.rs` (tracked)
- **Commit:** 5aedece

## Authentication Gates

None - no external authentication required for this implementation.

## Next Steps

The --detailed flag is now fully wired through the CLI to the execution layer. The next phase (Phase 09: Unified Rendering) will:

1. Use `DetailLevel` to control output formatting
2. Implement actual metadata rendering based on detail level
3. Ensure backward compatibility with existing JSON output

## References

- Plan: `.planning/phases/08-result-types/08-04-PLAN.md`
- DetailLevel implementation: `src/types/detail.rs`
- CLI args: `src/cli/args.rs`
- Command execution: `src/cli/commands.rs`
- Expand command: `src/cli/expand.rs`

## Self-Check: PASSED

✓ All modified files exist:
  - src/cli/expand.rs
  - src/cli/mod.rs
  - src/types/detail.rs
  - src/main.rs
  - src/cli/args.rs
  - src/cli/commands.rs

✓ Commits exist: 5862756, 5aedece

✓ All tests pass (302 total)
