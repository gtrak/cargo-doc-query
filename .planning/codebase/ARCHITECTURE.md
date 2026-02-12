# Architecture

**Analysis Date:** 2026-02-12

## Pattern Overview

**Overall:** CLI Tool with Modular Command Structure

**Key Characteristics:**
- Stateless CLI application (no daemon)
- Cargo subcommand invocation pattern
- Graph-based internal data model
- Disk-based caching with incremental rebuilds
- Machine-readable JSON output optimized for LLM agents

## Layers

**CLI Layer:**
- Purpose: Command parsing and dispatch
- Location: `src/main.rs`
- Contains: CLI argument parsing, subcommand routing, output formatting
- Depends on: Commands layer
- Used by: End users via `cargo doc-query`

**Commands Layer:**
- Purpose: Implements each subcommand's logic
- Location: `src/commands/` (planned)
- Contains: `build`, `methods`, `expand`, `traits` command handlers
- Depends on: Index layer, Cache layer
- Used by: CLI layer

**Index Layer:**
- Purpose: Graph representation of rustdoc JSON data
- Location: `src/index/` (planned)
- Contains: `CrateIndex`, `TypeInfo`, `MethodInfo`, `TraitInfo` structures
- Depends on: Parser layer
- Used by: Commands layer

**Parser Layer:**
- Purpose: rustdoc JSON deserialization
- Location: `src/parser/` (planned)
- Contains: JSON schema types, deserialization logic
- Depends on: External rustdoc JSON format
- Used by: Index layer

**Cache Layer:**
- Purpose: Persistent storage of parsed indices
- Location: `src/cache/` (planned), `target/doc-query/` (runtime)
- Contains: Cache metadata, bincode serialization, hash-based invalidation
- Depends on: File system, Cargo.lock hash
- Used by: Commands layer

## Data Flow

**Build Command Flow:**

1. CLI receives `cargo doc-query build`
2. Commands layer invokes `cargo rustdoc` with JSON output
3. Parser layer deserializes rustdoc JSON per crate
4. Index layer constructs `CrateIndex` with items graph
5. Cache layer serializes to `target/doc-query/crates/{crate}.bin`
6. Metadata written to `target/doc-query/metadata.json`

**Query Command Flow:**

1. CLI receives `cargo doc-query methods <TypePath>`
2. Cache layer validates metadata hash against Cargo.lock
3. If invalid: trigger incremental rebuild of changed crates
4. If valid: load cached `CrateIndex` shards
5. Index layer resolves type path to `TypeInfo`
6. Commands layer formats methods as JSON output

**State Management:**
- No runtime state; all state persisted to disk cache
- Hash-based cache invalidation using Cargo.lock + rustc version
- Sharded crate storage for selective rebuilds

## Key Abstractions

**CrateIndex:**
- Purpose: In-memory representation of a crate's public API
- Examples: `src/index/crate_index.rs` (planned)
- Pattern: Graph structure with `ItemId` keys and `Item` values

**TypeInfo:**
- Purpose: Queryable view of a type's methods and traits
- Examples: `src/index/type_info.rs` (planned)
- Pattern: Aggregate root containing methods and trait implementations

**CacheManager:**
- Purpose: Orchestrates cache validation and rebuilds
- Examples: `src/cache/manager.rs` (planned)
- Pattern: Content-addressable storage with metadata tracking

**Command Trait:**
- Purpose: Common interface for all CLI subcommands
- Examples: `src/commands/mod.rs` (planned)
- Pattern: Trait-based dispatch with shared context

## Entry Points

**Main Entry Point:**
- Location: `src/main.rs`
- Triggers: User invokes `cargo doc-query <subcommand>`
- Responsibilities: Initialize logging, parse args, dispatch to commands

**Build Command:**
- Location: `src/commands/build.rs` (planned)
- Triggers: `cargo doc-query build` or cache miss on query
- Responsibilities: Run rustdoc, parse output, populate cache

**Query Methods Command:**
- Location: `src/commands/methods.rs` (planned)
- Triggers: `cargo doc-query methods <type>`
- Responsibilities: Load cache, resolve type, output methods JSON

**Expand Command:**
- Location: `src/commands/expand.rs` (planned)
- Triggers: `cargo doc-query expand <type> --depth N`
- Responsibilities: Recursive type graph traversal

## Error Handling

**Strategy:** Structured error types with user-friendly messages

**Patterns:**
- Custom `Error` enum per module
- `thiserror` for error derivation
- Error context chaining for CLI diagnostics
- Non-zero exit codes for script integration

## Cross-Cutting Concerns

**Logging:** `tracing` crate with env_logger-style initialization
**Validation:** Cache metadata hash verification before operations
**Authentication:** None required (local tool)

---

*Architecture analysis: 2026-02-12*
