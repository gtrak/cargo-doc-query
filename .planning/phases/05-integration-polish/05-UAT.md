---
status: testing
phase: 05-integration-polish
source: 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md, 05-04-SUMMARY.md, 05-05-SUMMARY.md
started: 2026-02-13T00:00:00Z
updated: 2026-02-13T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Error Codes Display
expected: |
  Query without cache shows "No cached index found. Run `cargo doc-query build` first." and exits with code 2
result: pass

### 2. Help Text - Main
expected: |
  Running --help shows EXAMPLES section with sample commands and EXIT CODES section documenting codes 0-9
result: pass

### 3. Progress Indicators
expected: |
  Running build shows spinner/progress bar during rustdoc JSON generation with stage labels like "Generating rustdoc JSON..."
result: pass

### 4. Timing Output
expected: |
  Query shows timing info: "Query completed in Xms" printed to stderr
result: [pending]

### 2. Help Text - Main
expected: |
  Running --help shows EXAMPLES section with sample commands and EXIT CODES section documenting codes 0-9
result: [pending]

### 3. Help Text - Subcommands
expected: |
  cargo run -- query --help shows examples and flag descriptions
  cargo run -- build --help explains documentation generation
result: [pending]

### 4. Progress Indicators
expected: |
  Running build shows spinner/progress bar during rustdoc JSON generation with stage labels like "Generating rustdoc JSON..."
result: pass

### 5. Timing Output
expected: |
  Query shows timing info: "Query completed in Xms" printed to stderr
result: pass

### 6. --quiet Flag
expected: |
  Running with --quiet suppresses progress bars, timing info, and "Loaded index" messages (but still outputs results)
result: pass

### 7. --no-color Flag
expected: |
  Running with --no-color disables ANSI colors in output (useful for piping)
result: pass

### 8. Missing Cache Auto-Build
expected: |
  Query without existing cache automatically triggers build with message "No index found, building..." then completes the query
result: pass

### 9. Manifest Change Detection
expected: |
  After modifying Cargo.toml, next query shows "Manifest changed, rebuilding index..." and auto-rebuilds
result: pass

### 10. Type Suggestions
expected: |
  Query with typo (e.g., "anywhow") shows "No results found" followed by "Did you mean:" with suggestions like "anyhow"
result: issue
reported: "Type suggestions not showing - code was implemented in QueryCommand but Query now uses ExpandCommand which bypasses the suggestion logic"
severity: major

### 11. Corrupt Cache Detection
expected: |
  If cache file is corrupt, shows warning "Cache file appears corrupt, will rebuild..." and automatically rebuilds
result: issue
reported: "Shows generic deserialization error instead of specific warning and auto-rebuild"
severity: minor

### 12. No Dependencies Message
expected: |
  In a project with no external dependencies, build shows helpful message explaining tool needs dependencies
result: [pending]

### 13. Ctrl+C Handling
expected: |
  Pressing Ctrl+C during build/query interrupts gracefully with exit code 130
result: [pending]

### 14. Module Expansion
expected: |
  Can expand module paths (not just types): cargo run -- expand anyhow --depth 1 shows module contents grouped by kind
result: pass

### 15. Function Signatures in Expansion
expected: |
  Module expansion shows complete function signatures with parameters and return types
result: pass

## Summary

total: 15
passed: 10
issues: 2
pending: 3
skipped: 0

## Gaps

- truth: "Type suggestions show when query returns no results"
  status: failed
  reason: "Type suggestions implemented in QueryCommand.execute() but query subcommand now delegates to ExpandCommand which bypasses the suggestion logic"
  severity: major
  test: 10
  missing:
    - "Move suggestion logic from QueryCommand to ExpandCommand or main.rs"
    - "Ensure suggestions display when expansion fails with 'No items found'"

- truth: "Corrupt cache shows warning and auto-rebuilds"
  status: failed
  reason: "CacheStore.load() returns error on deserialization failure instead of None, bypassing auto-rebuild logic"
  severity: minor
  test: 11
  missing:
    - "Fix CacheStore.load() to catch postcard errors and return None instead of Err"

- truth: "Expand subcommand exists"
  status: failed
  reason: "ExpandCommand exists but is not registered in Commands enum, so 'cargo doc-query expand' doesn't work"
  severity: major
  test: 14
  missing:
    - "Add Expand variant to Commands enum in main.rs"
