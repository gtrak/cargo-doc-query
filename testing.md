# Testing Strategy for cargo-doc-query

## Overview

This document outlines the comprehensive test coverage strategy for cargo-doc-query, targeting **90%+ code coverage** using unit tests, integration tests, and property-based tests with proptest.

## Test Infrastructure

### Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
# ... existing dependencies ...

# Testing dependencies
proptest = "1.4"
tarpaulin = "0.29"
```

### Directory Structure

```
cargo-doc-query/
├── src/
│   ├── error/
│   ├── cache/
│   ├── cli/
│   ├── query/
│   ├── types/
│   ├── format/
│   ├── cargo/
│   └── parser/
├── tests/
│   ├── fixtures/
│   │   └── index/           # Generated from building repo
│   └── integration/
│       └── cli.rs
├── proptest/                # Property-based tests
│   ├── mod.rs
│   └── types/
├── testing.md
└── Cargo.toml
```

### Test Data Strategy

Instead of using mock data, we'll use the **repo itself as the test fixture**:

1. Build the documentation index for this repo itself
2. Use the generated rustdoc JSON and cached index for all tests
3. This ensures tests always reflect actual rustdoc format and structure

**Building the fixture:**
```bash
# Build and build index
cargo build --release
cargo run --release build

# Verify cache
ls -la target/doc-query/
```

## Test Categories

### 1. Unit Tests (Tier 1)

Located in `src/*/xxx.rs` as `#[cfg(test)] mod tests { ... }`

#### Priority Modules

**High Priority:**
- `src/error/errors.rs` - Error handling and exit codes
- `src/cache/key.rs` - Cache key generation
- `src/cache/store.rs` - Cache storage operations
- `src/cli/build.rs` - Build command logic
- `src/cli/query.rs` - Query command parsing and execution

**Medium Priority:**
- `src/query/engine.rs` - Core query engine
- `src/query/lookup.rs` - Path resolution
- `src/types/query.rs` - Query response types

**Lower Priority:**
- `src/types/expand.rs` - Expansion types
- `src/format/text.rs` - Output formatting
- `src/cargo/dependencies.rs` - Cargo metadata parsing
- `src/parser/validate.rs` - Format validation

#### Unit Test Coverage Goals

| Module | Coverage Target |
|--------|-----------------|
| Error handling | 95%+ |
| Cache operations | 95%+ |
| CLI commands | 90%+ |
| Query engine | 85%+ |
| Type formatting | 90%+ |
| Expansion logic | 85%+ |
| Output formatting | 85%+ |
| Documentation extraction | 80%+ |

### 2. Integration Tests (Tier 2)

Located in `tests/integration/cli.rs`

These tests exercise complete workflows:
- Build → Query → Expand pipeline
- Cache invalidation on manifest changes
- Token budget enforcement across commands
- Multiple crate queries
- JSON output validation
- Minimal mode integration

**Key test scenarios:**
```rust
#[test]
fn test_full_workflow() {
    // Build index for repo
    // Query known types
    // Expand modules
    // Verify results
}

#[test]
fn test_cache_invalidation() {
    // Query with cache
    // Modify Cargo.toml
    // Query again (should rebuild)
    // Verify new index used
}
```

### 3. Property-Based Tests (Tier 3)

Located in `proptest/mod.rs` and `proptest/types/`

Use proptest to test invariants and edge cases:

#### Example Property Tests

**Cache Key Determinism:**
```rust
#[proptest]
fn prop_cache_key_deterministic(manifest_path: &str) {
    let key1 = CacheKeyInputs::from_project(manifest_path).unwrap().generate_key();
    let key2 = CacheKeyInputs::from_project(manifest_path).unwrap().generate_key();
    prop_assert_eq!(key1, key2);
}
```

**Type Formatter Coverage:**
```rust
#[proptest]
fn prop_type_formatter_coverage(
    base_type in "\"String\" | \"Vec<T>\" | \"Option<T>\" | \"Result<T,E>\""
) {
    let type = parse_type(base_type).unwrap();
    let formatted = TypeFormatter::format_type(&type);
    prop_assert!(!formatted.is_empty());
    // Verify format is valid
}
```

**Expansion Invariants:**
```rust
#[proptest]
fn prop_expansion_no_cycles(path: String, depth: u8) {
    let mut expander = TypeExpander::new(index, depth);
    let result = expander.expand(&path, None).unwrap();

    prop_assert!(result.truncated_paths.is_empty() || result.budget_exceeded);
    prop_assert!(result.graph.nodes.len() <= calculate_max_nodes(path, depth));
}
```

### 4. Edge Case Tests

Special focus on error paths and edge cases:

- Empty collections (Vec::is_empty checks)
- Missing files (cache store error handling)
- Invalid JSON (parsing errors)
- Malformed paths (query validation)
- Overflow scenarios (depth limits, token budgets)
- Concurrent access (cache race conditions)

## Test Execution

### Run All Tests

```bash
# Run all unit tests
cargo test --verbose

# Run with output
cargo test -- --nocapture

# Run with test output visible
cargo test -- --nocapture -- --show-output

# Run tests in parallel (default)
cargo test

# Run tests with output
cargo test -- --nocapture -- --show-output
```

### Run Specific Modules

```bash
# Run tests for a specific module
cargo test --lib query::engine

# Run tests for a specific file
cargo test --lib cli::build

# Run tests for a specific function
cargo test --lib query::engine::query

# Run tests matching a pattern
cargo test methods
cargo test expand
```

### Run Property Tests

```bash
# Run proptest tests specifically
cargo test proptest:: -vvv

# Run with proptest specific options
cargo test -- --test-threads=1 --nocapture
```

### Generate Coverage Report

```bash
# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# Generate XML coverage for CI
cargo tarpaulin --out Xml --output-dir coverage

# Generate both HTML and JSON
cargo tarpaulin --out Html --out Json --output-dir coverage

# Run with specific threads
cargo tarpaulin --threads 8 --out Html

# Run with timeout
cargo tarpaulin --timeout 120 --out Html
```

### View Coverage Report

```bash
# Open HTML report in browser
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

## Coverage Goals

### Overall Target: 90%+ Line Coverage

Breakdown by category:
- **Unit tests**: 85-95%
- **Integration tests**: Additional 5-10% edge cases
- **Property tests**: Additional invariants verification
- **Total**: 90%+ combined

### Coverage Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Line coverage | 90%+ | 🔄 Pending |
| Branch coverage | 85%+ | 🔄 Pending |
| Function coverage | 95%+ | 🔄 Pending |
| Test speed (unit) | < 5s | 🔄 Pending |
| Test speed (integration) | < 30s | 🔄 Pending |

## Task List (21 Tasks)

### Phase 1: Setup & Infrastructure

1. **Add test dependencies** to Cargo.toml (proptest, tarpaulin)
2. **Create test directories** (tests/, tests/fixtures/, proptest/)
3. **Build repo fixture** index: `cargo run build`
4. **Verify cache structure** and cargo test compilation

### Phase 2: Core Module Tests

5. **Error module tests** (src/error/errors.rs) - exit codes, messages
6. **Cache module tests** (src/cache/key.rs, store.rs) - key generation, store ops
7. **CLI Build Command tests** (src/cli/build.rs) - caching, features, errors
8. **CLI Query Command tests** (src/cli/query.rs) - parsing, validation, output

### Phase 3: Query Engine Tests

9. **Path Resolver tests** (src/query/lookup.rs) - matching, cross-crate
10. **Query Engine tests** (src/query/engine.rs) - querying, extraction
11. **Type Formatter tests** (src/query/format.rs) - all Type variants, signatures
12. **Query Types tests** (src/types/query.rs) - response types, minimal mode

### Phase 4: Expansion & Output Tests

13. **Type Expander tests** (src/query/expand.rs) - expansion, cycles, budgets
14. **Type System tests** (src/types/expand.rs) - TypeNode, expansion results
15. **Output Formatting tests** (src/format/text.rs) - text output, edge cases
16. **Documentation tests** (src/types/doc.rs) - doc extraction, visibility

### Phase 5: Integration & Property Tests

17. **Integration tests** (tests/integration/cli.rs) - end-to-end workflows
18. **Property tests** (proptest/mod.rs) - invariants, edge cases

### Phase 6: Coverage & Final

19. **Generate coverage report** with tarpaulin
20. **Fix coverage gaps** - add tests for uncovered paths
21. **Final validation** - all tests pass, 90%+ coverage achieved

## Test Coverage Checklist

### Phase 1: Setup ✓

- [ ] Dependencies added to Cargo.toml
- [ ] Test directories created
- [ ] Repo index built
- [ ] Cache verified
- [ ] cargo test compiles

### Phase 2: Core Tests ✓

- [ ] Error module tests written and passing
- [ ] Cache tests passing (100% coverage)
- [ ] Build command tests passing
- [ ] Query command tests passing

### Phase 3: Query Engine ✓

- [ ] Path resolver tests (all matching modes)
- [ ] Query engine core tests
- [ ] Type formatter comprehensive tests
- [ ] Query types and minimal mode tests

### Phase 4: Expansion ✓

- [ ] Type expander tests
- [ ] Expansion types tests
- [ ] Output formatting tests
- [ ] Documentation extraction tests

### Phase 5: Integration & Property ✓

- [ ] Integration tests pass
- [ ] Property tests pass
- [ ] No test warnings

### Phase 6: Coverage ✓

- [ ] Coverage report generated
- [ ] 90%+ line coverage achieved
- [ ] Coverage gaps documented
- [ ] Final validation complete

## Testing Guidelines

### Writing Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";

        // Act
        let result = function_to_test(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "specific error")]
    fn test_panics_on_error() {
        function_that_panics("bad input");
    }

    #[test]
    fn test_with_valid_and_invalid() {
        assert!(function(true));
        assert!(!function(false));
    }
}
```

### Writing Property Tests

```rust
#[proptest]
fn prop_function_property(input: String) {
    // Use proptest::prelude::* for generators
    let input = generate_test_input();
    let result = function(input);
    assert!(validate_result(result));
}
```

### Writing Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use std::fs;

    #[test]
    fn test_end_to_end_workflow() {
        // Setup
        build_index();

        // Execute
        let result = query("known::type");

        // Verify
        assert!(result.is_ok());
        assert!(result.unwrap().matches.len() > 0);
    }
}
```

### Test Organization

- **Small, focused tests** - test one thing at a time
- **Clear names** - `test_cache_key_generation_is_deterministic`
- **Arrange-Act-Assert** - standard test pattern
- **Skip expensive tests** with `#[ignore]` if needed
- **Use `#[should_panic]`** for error handling
- **Avoid global state** - use fixtures or setup/teardown

## Common Testing Patterns

### Testing Cache Operations

```rust
#[test]
fn test_cache_save_and_load() {
    let store = CacheStore::new().unwrap();
    let cache_key = "test-key";

    let test_index = SerializableIndex {
        format_version: 1,
        cache_key: cache_key.to_string(),
        nodes: vec![],
        edges: vec![],
    };

    store.save(cache_key, &test_index).unwrap();

    let loaded = store.load(cache_key).unwrap().unwrap();
    assert_eq!(loaded.cache_key, cache_key);
}
```

### Testing Query Engine

```rust
#[test]
fn test_query_known_type() {
    let index = load_test_index();
    let mut engine = QueryEngine::new(index);
    let options = QueryOptions::new(QueryKind::All);

    let result = engine.query("std::vec::Vec", &options, None);

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.matches.is_empty());
}
```

### Testing Token Budgeting

```rust
#[test]
fn test_token_budget_enforcement() {
    let config = TokenConfig::new()
        .with_budget(Some(100))
        .with_threshold(0.8);

    let mut expander = TypeExpander::with_config(index, 3, config);
    let result = expander.expand("known::Type", None).unwrap();

    assert!(result.token_count <= 100);
}
```

## Troubleshooting

### Tests Fail to Compile

```bash
# Check for test flag
cargo test --no-run

# Check dependencies
cargo tree | grep proptest

# Clean and rebuild
cargo clean && cargo build
```

### Tests Too Slow

```bash
# Run tests in parallel (default)
cargo test

# Run with reduced parallelism
cargo test -- --test-threads=1

# Skip slow tests
cargo test -- --exclude-slow
```

### Coverage Gaps

```bash
# Run tarpaulin with coverage output
cargo tarpaulin --out Html --output-dir coverage

# View coverage in browser
open coverage/index.html

# Look for red lines in coverage report
```

### Property Test Flakiness

```bash
# Run proptest with more iterations
cargo test -- --ignored --nocapture

# Increase proptest iterations
# Add to proptest.toml:
iter_min = 100
iter_max = 1000
```

## Success Criteria

- ✅ All 21 tasks completed
- ✅ All unit tests passing (no failures)
- ✅ All integration tests passing
- ✅ All property tests passing
- ✅ **90%+ line coverage** achieved
- ✅ **85%+ branch coverage** achieved
- ✅ No test warnings
- ✅ Build succeeds with warnings as errors disabled for tests
- ✅ Test suite completes in reasonable time (< 5 minutes)
- ✅ Documentation in `AGENTS.md`

## Next Steps

1. **Start with Task 1**: Add test dependencies to Cargo.toml
2. **Proceed through Task 21**: Execute each task sequentially
3. **Run coverage regularly**: After each phase, generate coverage report
4. **Fix gaps immediately**: Don't defer coverage improvements
5. **Validate at the end**: Ensure 90%+ coverage is achieved

## Questions or Issues

If you encounter issues:
1. Check test output for specific error messages
2. Verify dependencies are installed
3. Ensure cache fixture is built
4. Review test code for common mistakes
5. Consult troubleshooting section above

Good luck with achieving 90%+ test coverage! 🚀
