# Quick Task 001: Implement Fix Plan for Deterministic Output Path and Broken Build Support

## Overview
Implement the fixes described in fix-plan.md to:
1. Use deterministic output path via CARGO_TARGET_DIR
2. Support broken builds by documenting only external dependencies

## Tasks

### Task 1: Add CARGO_TARGET_DIR for deterministic output
**File**: `src/cli/build.rs`
- Add `CARGO_TARGET_DIR=target/.cargo-doc-query/` environment variable to cargo doc command
- Update `scan_json_files()` to use the new deterministic path
- Remove any hardcoded target triple path logic

### Task 2: Support broken builds with dependency-only documentation
**File**: `src/cli/build.rs`
- Parse Cargo.lock to discover dependencies (fallback when cargo metadata fails)
- Use `cargo doc -p <crate>` for each external dependency instead of `--workspace`
- Generate docs only for external crates, not local workspace

### Task 3: Test and verify
- Test with tetris project (normal and broken build scenarios)
- Run cargo test to ensure no regressions
- Install updated binary

## Success Criteria
- JSON files always generated to `target/.cargo-doc-query/doc/`
- Can index dependencies even when project has compile errors
- `cargo-doc-query query bevy_input::button_input::ButtonInput` works
- All tests pass
