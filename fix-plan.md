# Fix Plan: Deterministic Output Path and Broken Build Support

## Problem Summary

The cargo-doc-query tool has two issues:

1. **Non-deterministic JSON output location**: The build was looking for JSON files in `target/{triple}/doc` but JSON files are actually placed in `target/doc` (depending on project configuration)

2. **Build failure blocks indexing**: If the project has compile errors, we can't generate documentation and therefore can't index dependencies

## Solution

### Part 1: Deterministic Output Path (BUILD-PATH-01)

**Issue**: Different projects configure cargo/rustdoc differently, leading to JSON files in different locations

**Fix**: Use `CARGO_TARGET_DIR` environment variable to force output to a deterministic location

**Implementation**:
- Set `CARGO_TARGET_DIR=target/.cargo-doc-query/` when running `cargo doc`
- This creates a tool-specific directory that doesn't interfere with the project's normal target
- JSON files will always be at `target/.cargo-doc-query/doc/*.json`
- Update `generate_rustdoc_json()` to scan this deterministic path

**Files to modify**:
- `src/cli/build.rs`:
  - Remove hardcoded target triple path logic
  - Add `CARGO_TARGET_DIR` to the cargo command environment
  - Update `scan_json_files()` to use the new deterministic path

### Part 2: Support Broken Builds (BUILD-BROKEN-01)

**Issue**: Currently requires successful project build to index dependencies

**Fix**: Document only external dependencies without building the local workspace

**Implementation**:
- Parse `Cargo.lock` or use `cargo metadata` to discover dependencies (works even with broken builds)
- Instead of `cargo doc --workspace`, use `cargo doc -p <crate1> -p <crate2> ...` for each external dependency
- This generates docs for deps without requiring the local project to compile

**Files to modify**:
- `src/cli/build.rs`:
  - Modify `generate_rustdoc_json()` to accept list of dependency crates
  - Build docs for each dependency individually using `-p` flags
  - Parse Cargo.lock if cargo metadata fails

- `src/cargo/dependencies.rs` (new or existing):
  - Add function to parse Cargo.lock for dependency names/versions
  - Ensure this works even when cargo metadata fails

## Acceptance Criteria

1. **Deterministic Path**:
   - ✅ JSON files are always generated to `target/.cargo-doc-query/doc/`
   - ✅ Works regardless of project target configuration
   - ✅ Doesn't interfere with project's normal `target/` directory

2. **Broken Build Support**:
   - ✅ Can index `bevy::input::ButtonInput` even when tetris project has compile errors
   - ✅ Parses dependencies from Cargo.lock when cargo metadata fails
   - ✅ Documents only external crates, not local workspace

3. **Query Works**:
   - ✅ `cargo-doc-query query bevy_input::button_input::ButtonInput` returns results
   - ✅ Correct path is `bevy_input::button_input::ButtonInput` (not `bevy::input::ButtonInput`)

## Implementation Steps

1. [ ] Revert incorrect path fix in `src/cli/build.rs` (restore original logic)
2. [ ] Add `CARGO_TARGET_DIR` environment variable to cargo doc command
3. [ ] Update `scan_json_files()` path to use deterministic location
4. [ ] Modify dependency discovery to use Cargo.lock fallback
5. [ ] Change cargo doc invocation to use `-p` flags for each dependency
6. [ ] Test with tetris project:
   - Break a file in tetris to cause compile error
   - Verify `cargo-doc-query build` still works
   - Verify `cargo-doc-query query bevy_input::button_input::ButtonInput` works
7. [ ] Run `cargo test` to ensure no regressions
8. [ ] Install updated binary with `cargo install --path .`

## Testing

```bash
# Test 1: Normal operation
cd ../tetris
cargo-doc-query build
cargo-doc-query query bevy_input::button_input::ButtonInput

# Test 2: Broken build
cd ../tetris
# Introduce compile error in src/main.rs
cargo-doc-query build  # Should still work
cargo-doc-query query bevy_input::button_input::ButtonInput  # Should still work

# Test 3: Regression
cargo test  # All tests pass
```

## Notes

- The correct path for ButtonInput is `bevy_input::button_input::ButtonInput`, not `bevy::input::ButtonInput`
- This is because ButtonInput is defined in the `bevy_input` crate, not re-exported at `bevy::input`
- Users should use the actual module path from the defining crate
