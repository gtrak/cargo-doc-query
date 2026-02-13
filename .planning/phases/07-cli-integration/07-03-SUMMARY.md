---
phase: 07-cli-integration
plan: 03
type: summary
subsystem: cli
completed: 2026-02-13
duration: "45 minutes"
tags: [cli, validation, error-handling, help-text, filters]

decisions:
  - "PATH argument made optional in Query command to support --help-filters without requiring a query target"
  - "Validation errors return early with helpful messages before processing"
  - "Error messages follow consistent format: what, why, example, reference"

tech-stack:
  added: []
  patterns:
    - "Early validation with anyhow::bail! for clear error messages"
    - "Case-insensitive kind matching via to_lowercase()"
    - "Glob pattern syntax help extracted to standalone function"

key-files:
  created: []
  modified:
    - src/cli/expand.rs
    - src/main.rs
    - src/types/filter.rs

artifacts:
  - path: "src/cli/expand.rs"
    lines: "~200"
    description: "Enhanced validation and error handling for filter flags"
  - path: "src/main.rs"
    lines: "~60"
    description: "--help-filters support without PATH requirement"

deviations:
  - type: none
    description: "Plan executed exactly as written"
---

# Phase 7 Plan 3: Filter Validation and Help Text Summary

## Overview

Completed comprehensive validation, help text, and error handling for filter flags in the cargo-doc-query CLI.

## What Was Built

### Task 1: Validation for Conflicting Filter Flags

**Implementation:**
- Enhanced `validate()` method in `ExpandCommand` to detect and report conflicts
- Added check for `--include` + `--only` mutual exclusivity with actionable error message
- Implemented visibility validation supporting `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`, `private`
- Added detection for exact pattern conflicts between include and exclude
- All validation errors include helpful guidance and reference to `--help-filters`

**Key Features:**
```
Error: Cannot use --include with --only.

--only is shorthand for 'include this and exclude everything else'.
Use either:
  --only 'pattern'          (include only matching items)
Or:
  --include 'pattern'       (include matching items alongside others)

For more help, run: cargo doc-query query --help-filters
```

### Task 2: FILTERING Section in --help

**Status:** Already present in base code

**Enhancement Made:**
- Updated visibility help text to include `pub(in path)` option
- FILTERING section displays 5 examples:
  - `--include "std::*"` - Show only items from std crate
  - `--exclude "*::test*"` - Exclude items with "test" in path
  - `--kind function` - Show only functions
  - `--only "serde::*"` - Show only serde items (shorthand)
  - `--include "std::*" --kind fn` - Show only std functions (AND logic)

### Task 3: --help-filters Flag

**Implementation:**
- Made PATH argument optional in Query command variant
- Added early check in `run()` function before path validation
- Created `print_glob_syntax_help()` function with comprehensive documentation

**Output:**
```
Filter Pattern Syntax (Glob Patterns)
=====================================

Special Characters:
  *       Matches any sequence of characters (except path separator)
  ?       Matches any single character
  **      Matches any sequence including path separators
  [...]   Matches any character in brackets
  [!...]  Matches any character NOT in brackets

Examples:
  'std::*'           → All items in std crate
  'std::vec::*'      → All items in std::vec module
  '*::test*'         → Items with "test" in the name
  '**::Display'      → Display trait anywhere
  'crate::[A-Z]*'    → Items starting with capital letter
  'serde::de::*'     → All items in serde::de module

Tips:
  - Use quotes around patterns with special characters
  - Patterns are case-sensitive for paths
  - Multiple --include flags = OR logic
  - Different flag types = AND logic
```

### Task 4: Enhanced Error Messages

**Implementation:**
- Invalid glob patterns show specific error with pattern and explanation
- Empty patterns display clear error with valid example
- All errors reference `--help-filters` for more information
- Error format: what went wrong, why, example, reference

**Example Error Output:**
```
Error: Invalid glob pattern '[invalid'

The pattern is not valid: Pattern syntax error near position 0: invalid range pattern

Example of a valid pattern: 'std::vec::*'

For glob syntax reference, run:
  cargo doc-query query --help-filters
```

## Verification Results

✅ All validation scenarios tested:
- `--include "x" --only "y"` shows mutual exclusivity error
- `--visibility invalid` shows valid options list
- `--include "[invalid"` shows pattern syntax error with explanation
- `--include ""` shows empty pattern error
- `--kind FUNCTION` works (case-insensitive)
- `--help-filters` displays without requiring PATH

## Technical Details

### Error Message Format
All error messages follow a consistent structure:
1. **What went wrong** - Clear statement of the problem
2. **Why** - Brief explanation of the issue
3. **Example** - Valid usage example
4. **Reference** - Pointer to `--help-filters` for more info

### Visibility Validation
Supports all Rust visibility modifiers:
- `pub` - Public visibility
- `pub(crate)` - Crate-visible
- `pub(super)` - Parent module visible
- `pub(in path)` - Specific path visible
- `private` - Non-public items

### Case-Insensitive Kind Matching
FilterEngine normalizes kinds to lowercase for comparison:
```rust
// In FilterEngine::matches()
if !self.kinds.iter().any(|k| k == &kind.to_lowercase()) {
    return false;
}
```

## Commits

1. `62a2f6f` - feat(07-03): add validation for conflicting filter flags
2. `a5b2120` - feat(07-03): update visibility help text to include pub(in path)
3. `d7efb64` - feat(07-03): implement --help-filters flag without requiring PATH
4. `2393b43` - fix(07-03): remove cfg feature gate from Filterable impl for TypeNode

## Self-Check: PASSED

- ✅ Validation detects --include + --only conflict
- ✅ Visibility validation works for all valid options
- ✅ FILTERING section has 5 examples in --help
- ✅ --help-filters displays without PATH argument
- ✅ Invalid patterns show specific error with example
- ✅ Case-insensitive kind matching works (test passes)
- ✅ All existing tests pass (212 tests)
