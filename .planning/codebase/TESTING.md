# Testing Patterns

**Analysis Date:** 2026-02-12

## Test Framework

**Runner:**
- **Tool:** Built-in `cargo test`
- **Version:** Rust 1.93.0
- **Config:** No custom configuration (no `Cargo.toml` test configuration)

**Assertion Library:**
- Standard library assertions: `assert!`, `assert_eq!`, `assert_ne!`
- `std::panic::catch_unwind` for panic testing
- Consider `pretty_assertions` for better diff output in tests

**Run Commands:**
```bash
# Run all tests
cargo test

# Run tests with output visible
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests matching pattern
cargo test query_

# Run with release optimizations
cargo test --release

# Run ignored tests (e.g., integration tests)
cargo test -- --ignored
```

## Test File Organization

**Current State:**
- No test files exist yet
- Source structure: `src/main.rs` only

**Recommended Structure:**
```
src/
├── main.rs           # Binary entry point
├── lib.rs            # Library root (allows testing)
├── index.rs          # Index module
├── index/
│   └── mod.rs        # Alternative module layout
├── query.rs          # Query module
├── cache.rs          # Cache module
└── types.rs          # Type definitions

tests/
├── integration_tests.rs   # Integration tests
└── fixtures/              # Test data
    └── sample_crate/
```

**Naming:**
- Unit tests: Inline in source files or `src/{module}_test.rs`
- Integration tests: `tests/{feature}_test.rs`
- Test functions: `snake_case` with `test_` prefix

## Test Structure

**Inline Unit Tests (in source files):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_index_builds() {
        let index = CrateIndex::new("test_crate");
        assert_eq!(index.crate_name, "test_crate");
        assert!(index.items.is_empty());
    }

    #[test]
    fn test_type_info_methods() {
        let type_info = TypeInfo {
            path: "std::fs::File".to_string(),
            methods: vec![],
            traits: vec![],
        };
        assert_eq!(type_info.path, "std::fs::File");
    }
}
```

**Integration Tests (in `tests/` directory):**
```rust
// tests/query_tests.rs
use cargo_doc_query::index::CrateIndex;

#[test]
fn test_query_methods_for_file() {
    // Build index
    let index = CrateIndex::from_rustdoc_json("tests/fixtures/sample.json")
        .expect("Failed to parse");
    
    // Query
    let methods = index.query_methods("std::fs::File");
    
    // Assert
    assert!(!methods.is_empty(), "Expected methods for File type");
}
```

**Test Organization Pattern:**
- Group related tests in modules
- Use descriptive test names that explain the scenario
- One assertion concept per test (may have multiple `assert!` calls)

## Mocking

**Current State:**
- No mocking framework configured

**Recommended Options:**
1. **mockall** - Powerful mocking framework for Rust traits
2. **Manual mocks** - Implement traits with test doubles
3. **Stub implementations** - Simple structs with controlled behavior

**Mock Pattern (with mockall):**
```rust
use mockall::mock;

#[cfg(test)]
mock! {
    pub Cache {}
    
    impl Cache for Cache {
        fn get(&self, key: &str) -> Option<Vec<u8>>;
        fn set(&self, key: &str, value: Vec<u8>);
    }
}

#[test]
fn test_uses_cached_value() {
    let mut mock_cache = MockCache::new();
    mock_cache
        .expect_get()
        .with(mockall::predicate::eq("test_crate"))
        .return_once(|_| Some(vec![1, 2, 3]));
    
    let result = query_with_cache(&mock_cache, "test_crate");
    assert!(result.is_ok());
}
```

**What to Mock:**
- File system operations (use `tempfile` crate for real files)
- External command execution (cargo, rustdoc)
- Network requests (not applicable yet)
- Time-based operations

**What NOT to Mock:**
- Pure data structures
- Internal utility functions
- Value objects (structs with no side effects)

## Fixtures and Factories

**Test Data:**

**Location:**
- `tests/fixtures/` - Static test data files
- `src/test_utils.rs` - Test helper functions and builders

**JSON Fixtures:**
Store sample rustdoc JSON outputs for testing:
```
tests/
├── fixtures/
│   ├── simple_crate/
│   │   └── doc.json
│   ├── complex_generics/
│   │   └── doc.json
│   └── empty_crate/
│       └── doc.json
```

**Builder Pattern for Test Data:**
```rust
#[cfg(test)]
pub mod builders {
    use crate::types::{CrateIndex, TypeInfo, MethodInfo};

    pub struct CrateIndexBuilder {
        name: String,
    }

    impl CrateIndexBuilder {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }

        pub fn with_type(mut self, path: &str) -> Self {
            // Add type to index
            self
        }

        pub fn build(self) -> CrateIndex {
            CrateIndex {
                crate_name: self.name,
                items: HashMap::new(),
            }
        }
    }
}
```

## Coverage

**Current State:**
- No coverage requirements configured
- No coverage tool integrated

**View Coverage:**
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# Or with llvm-cov (requires nightly)
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

**Recommended Coverage Targets:**
- Core indexing logic: 90%+
- Query parsing: 85%+
- CLI argument handling: 70%+
- Cache operations: 80%+

## Test Types

**Unit Tests:**
- Location: Inline in source files (`#[cfg(test)]` modules)
- Scope: Individual functions and methods
- Run: `cargo test`

**Integration Tests:**
- Location: `tests/` directory
- Scope: Full workflow from CLI to output
- Run: `cargo test --test integration`

**Example Integration Test:**
```rust
// tests/cli_tests.rs
use std::process::Command;

#[test]
fn test_build_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "build"])
        .current_dir("tests/fixtures/sample_project")
        .output()
        .expect("Failed to execute");
    
    assert!(output.status.success());
}

#[test]
fn test_methods_query_output_format() {
    // First build the index
    // Then query and verify JSON output structure
}
```

**E2E Tests:**
- Not currently implemented
- Consider for full dependency projects
- Can use `assert_cmd` and `predicates` crates

## Common Patterns

**Async Testing:**
If async code is introduced:
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

**Error Testing:**
```rust
#[test]
fn test_invalid_type_path_errors() {
    let result = query_methods("not_a_valid::path::");
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid type path"));
}

#[test]
#[should_panic(expected = "index not built")]
fn test_query_without_build_panics() {
    let index = CrateIndex::new("unbuilt");
    index.query_methods("std::fs::File"); // Should panic
}
```

**Temp Directory Pattern:**
```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_cache_write() {
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("test_cache.bin");
    
    // Write to cache
    let cache = Cache::new(&cache_path);
    cache.store("key", b"value").unwrap();
    
    // Verify file exists
    assert!(cache_path.exists());
    
    // Cleanup handled automatically by TempDir drop
}
```

**Snapshot Testing (for JSON output):**
Consider using `insta` for snapshot testing CLI output:
```rust
use insta::assert_json_snapshot;

#[test]
fn test_methods_output() {
    let index = build_test_index();
    let output = index.query_methods("std::fs::File");
    
    assert_json_snapshot!(output);
}
```

## Test Data Management

**Sample Projects:**
Create minimal Rust projects in `tests/fixtures/` for testing:
```
tests/fixtures/
├── hello_world/           # Simple binary
│   ├── Cargo.toml
│   └── src/main.rs
├── library_crate/         # Library with public API
│   ├── Cargo.toml
│   └── src/lib.rs
└── generic_heavy/         # Tests generic handling
    ├── Cargo.toml
    └── src/lib.rs
```

**Generating Fixtures:**
```rust
// tests/generate_fixtures.rs (helper, not a test)
use std::process::Command;

pub fn generate_rustdoc_json(project_path: &str) {
    Command::new("cargo")
        .args(["rustdoc", "--", "-Z", "unstable-options", "--output-format", "json"])
        .current_dir(project_path)
        .status()
        .expect("Failed to generate rustdoc");
}
```

---

*Testing analysis: 2026-02-12*
