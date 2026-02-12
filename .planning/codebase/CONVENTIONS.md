# Coding Conventions

**Analysis Date:** 2026-02-12

## Overview

This is a new Rust project (`cargo-doc-query`) in its initial stages. Currently using default Rust conventions with minimal custom configuration.

## Naming Patterns

**Files:**
- Source files: `snake_case.rs` (Rust standard)
- Main entry: `src/main.rs` for binary crate
- Module files: Expected to follow `src/{module_name}.rs` or `src/{module_name}/mod.rs`

**Functions:**
- Use `snake_case` for functions and methods (Rust standard)
- Example: `fn main()`, `fn query_methods()`

**Variables:**
- Use `snake_case` for local variables
- Use `SCREAMING_SNAKE_CASE` for constants

**Types:**
- Use `PascalCase` for structs, enums, traits, and type aliases
- Examples from plan: `CrateIndex`, `TypeInfo`, `MethodInfo`

**Modules:**
- Use `snake_case` for module names
- Example: `cache`, `query`, `index`

## Code Style

**Formatting:**
- **Tool:** `rustfmt` (version 1.8.0-stable)
- **Configuration:** No custom `.rustfmt.toml` - using defaults
- **Standard:** Rust 2024 edition formatting rules

**To format code:**
```bash
rustfmt src/**/*.rs
# or
cargo fmt
```

**Linting:**
- **Tool:** Clippy (implied, no custom configuration)
- **Configuration:** No custom `.clippy.toml` or `clippy.toml`
- Run with: `cargo clippy`

**Edition:**
- Rust 2024 Edition (`Cargo.toml` line 4)

## Import Organization

**Order:**
1. Standard library imports (`std::`, `core::`, `alloc::`)
2. External crate imports
3. Local module imports (`crate::`, `super::`, `self::`)

**Grouping:**
- Separate groups with blank lines
- Within groups, sort alphabetically

**Example (expected pattern):**
```rust
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::index::CrateIndex;
use crate::types::TypeInfo;
```

**Path Aliases:**
- Not currently configured
- Standard crate-relative paths expected: `crate::`

## Error Handling

**Current State:**
- Project uses basic `println!` (not yet implemented)

**Expected Patterns:**
- Use `Result<T, E>` for fallible operations
- Custom error types should implement `std::error::Error`
- Consider using `thiserror` or `anyhow` for error handling

**CLI Error Pattern (recommended):**
```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}
```

## Logging

**Framework:** Not yet implemented

**Recommended Options:**
- `tracing` - structured logging, good for LLM agent tools
- `env_logger` - simple environment-based logging
- `log` + `stderrlog` - standard ecosystem

**Pattern (recommended):**
```rust
use tracing::{info, debug, error};

debug!("Building index for crate: {}", crate_name);
info!("Cache hit for {}", crate_name);
error!("Failed to parse rustdoc JSON: {}", e);
```

## Comments

**When to Comment:**
- Document public APIs with `///` doc comments
- Explain non-obvious algorithmic choices
- Mark TODO/FIXME items clearly

**Doc Comments:**
- Use `///` for item documentation
- Use `//!` for module-level documentation
- Follow Rustdoc conventions

**Example:**
```rust
/// A cached index of crate documentation.
///
/// The index maps item IDs to their metadata, enabling
/// sub-100ms queries over dependency APIs.
pub struct CrateIndex {
    /// Name of the indexed crate.
    pub crate_name: String,
    /// Map of item IDs to their information.
    pub items: HashMap<ItemId, Item>,
}
```

## Function Design

**Size:**
- Keep functions under 50 lines where possible
- Extract complex logic into well-named helper functions

**Parameters:**
- Prefer borrowing over ownership when possible (`&T` vs `T`)
- Use `&str` over `String` for read-only parameters
- Group related parameters into structs for clarity

**Return Values:**
- Return `Result<T, E>` for operations that can fail
- Use `Option<T>` for nullable returns
- Consider custom types for complex return values

## Module Design

**Visibility:**
- Default to `pub(crate)` for internal APIs
- Use `pub` only for truly public interfaces
- Use `#[doc(hidden)]` sparingly

**Exports:**
- Use `pub use` to re-export commonly used items at module boundaries
- Keep module hierarchy shallow (max 3-4 levels)

**Barrel Files:**
- Use `mod.rs` to organize submodules
- Re-export public items for cleaner imports

## CLI Conventions

**Cargo Subcommand Pattern:**
This crate implements a Cargo subcommand, following conventions:
- Binary name: `cargo-doc-query`
- Entry point: `src/main.rs`
- Subcommands as enum variants (recommended `clap`)

**Recommended CLI Structure:**
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo-doc-query")]
#[command(about = "Query Rust documentation APIs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the documentation index
    Build,
    /// Query methods for a type
    Methods { type_path: String },
    /// Expand return type graph
    Expand { type_path: String },
}
```

## JSON Handling

**Serialization:**
- Use `serde` for JSON serialization/deserialization
- Derive `Serialize` and `Deserialize` for data structures
- Use `#[serde(rename_all = "camelCase")]` for JSON naming conventions

**Output:**
- Provide pretty-printed JSON in verbose mode
- Provide compact JSON for machine consumption (default)

---

*Convention analysis: 2026-02-12*
