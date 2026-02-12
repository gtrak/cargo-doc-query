# Project Roadmap: cargo-doc-query

**Project:** cargo-doc-query  
**Created:** 2026-02-12  
**Depth:** Standard  

## Overview

A 5-phase roadmap delivering a Cargo subcommand for fast, structured API queries over Rust dependency documentation. Phases follow the architectural dependency chain: Foundation → Query Core → Performance → LLM Optimizations → Polish. Each phase delivers a coherent, verifiable capability that builds on the previous.

---

## Phase 1: Foundation — JSON Ingestion & Index

**Goal:** Users can generate documentation indexes from their Rust dependencies with format validation and version checking.

**Dependencies:** None (foundational phase)

**Requirements:**
- BUILD-01: `cargo doc-query build` command
- BUILD-02: Generate rustdoc JSON for all dependencies
- BUILD-05: Format version checking for rustdoc JSON compatibility

**Success Criteria:**
1. User can run `cargo doc-query build` to generate documentation index
2. Build command successfully generates rustdoc JSON for all workspace dependencies
3. Format version is validated before processing; incompatible versions fail fast with clear error
4. Index structure (graph-based) is created and persisted to disk

**Key Risks Addressed:**
- Format Version Blindness (validated via rustdoc-types)
- Multiple Package Version Output Collision
- Rustdoc failures handled gracefully

---

## Phase 2: Core Querying — Methods & Traits

**Goal:** Users can query methods and traits for any type, receiving structured JSON output with signatures and fully qualified types.

**Dependencies:** Phase 1 (requires indexed documentation)

**Requirements:**
- QUERY-01: Query methods for any type by fully qualified path
- QUERY-02: Method output includes name, signature, and fully qualified return types
- QUERY-03: Query responses are formatted as structured JSON
- TRAIT-01: Query trait implementations for any type
- TRAIT-02: Trait output includes associated types and methods
- FMT-01: JSON output format for programmatic/LLM consumption
- FMT-03: Support for piping and shell integration

**Success Criteria:**
1. User can query methods for any type (e.g., `cargo doc-query methods std::vec::Vec`)
2. Method output includes complete signature and fully qualified return types
3. User can query trait implementations for any type
4. Trait output includes associated types and all trait methods
5. All query responses are valid, parseable JSON
6. Command output can be piped to other tools (e.g., `| jq`)

**Key Risks Addressed:**
- Cross-Crate ID Resolution Failures (handled via two-tier lookup)

---

## Phase 3: Performance — Caching & Incremental Rebuilds

**Goal:** Query responses complete in under 100ms through intelligent caching and automatic incremental rebuilds.

**Dependencies:** Phase 2 (requires query patterns to design cache keys)

**Requirements:**
- BUILD-03: Index is cached to disk for sub-100ms query performance
- BUILD-04: Index automatically rebuilds when Cargo.lock changes

**Success Criteria:**
1. First query after build completes in under 100ms (cached response)
2. Cache is persisted to disk in `target/doc-query/`
3. Modifying Cargo.lock triggers automatic rebuild on next query
4. Unchanged dependencies use cached data (no unnecessary rebuilds)

**Key Risks Addressed:**
- Cache Invalidation Complexity (content-addressable storage)

---

## Phase 4: Advanced Features — Recursive Expansion & Token Budgets

**Goal:** Users can explore type hierarchies recursively with depth limits and constrain output by token budgets for LLM efficiency.

**Dependencies:** Phase 3 (fast queries required for iterative exploration)

**Requirements:**
- QUERY-04: User can limit output with token budget constraints
- QUERY-05: User can request minimal/signature-only mode
- QUERY-06: Recursive type expansion supports depth limits
- TYPE-01: User can expand a type recursively to see its full graph
- TYPE-02: Expansion respects depth limits to prevent context overflow
- FMT-02: Minimal output format for token efficiency

**Success Criteria:**
1. User can expand a type recursively (e.g., `cargo doc-query expand std::result::Result`)
2. Expansion respects user-specified depth limits (e.g., `--depth 2`)
3. User can set token budget constraints on output size
4. Minimal mode outputs signature-only (reducing token usage by 50%+ vs full)
5. Recursive expansion prevents infinite loops on circular type references

**Key Risks Addressed:**
- Unbounded Recursive Type Expansion (depth limits + cycle detection)

---

## Phase 5: Integration & Polish

**Goal:** Tool is production-ready with comprehensive CLI ergonomics, error handling, and shell integration.

**Dependencies:** Phase 4 (all features complete, focus on polish)

**Requirements:**
- (Polish requirements derived from research best practices)

**Success Criteria:**
1. All commands have consistent, helpful error messages
2. CLI provides progress indicators for long operations (rustdoc generation)
3. Help text and documentation are complete for all commands
4. Tool handles edge cases gracefully (missing types, network issues)
5. Exit codes are appropriate for shell scripting (0=success, non-zero=failure with meaning)

---

## Progress

| Phase | Status | Requirements | Success Criteria Met |
|-------|--------|--------------|---------------------|
| 1 - Foundation | ⏳ Pending | 3/18 | 0/4 |
| 2 - Core Querying | ⏳ Pending | 7/18 | 0/6 |
| 3 - Performance | ⏳ Pending | 2/18 | 0/4 |
| 4 - Advanced Features | ⏳ Pending | 6/18 | 0/5 |
| 5 - Integration & Polish | ⏳ Pending | 0/18 | 0/5 |

**Coverage:** 18/18 v1 requirements mapped ✓  
**Completion:** 0/5 phases complete

---

## Research Flags

Phases requiring deeper research during planning:

- **Phase 1 (Foundation):** LOW — Well-documented via rustdoc-types, rustdoc-json crates
- **Phase 2 (Core Querying):** MEDIUM — Graph schema design benefits from prototyping
- **Phase 3 (Performance):** MEDIUM — Cache key design (which inputs to hash)
- **Phase 4 (Advanced Features):** LOW — Clear requirements from feature research
- **Phase 5 (Integration):** LOW — Standard CLI patterns

---

## Phase Dependencies

```
Phase 1 (Foundation)
    ↓
Phase 2 (Core Querying)
    ↓
Phase 3 (Performance)
    ↓
Phase 4 (Advanced Features)
    ↓
Phase 5 (Integration & Polish)
```

---

*Last updated: 2026-02-12*
