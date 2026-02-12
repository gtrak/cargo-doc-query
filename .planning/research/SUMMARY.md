# Project Research Summary

**Project:** cargo-doc-query
**Domain:** Rust CLI tool for querying rustdoc JSON documentation
**Researched:** 2026-02-12
**Confidence:** HIGH

## Executive Summary

This project is a Cargo subcommand that parses rustdoc JSON output to enable fast, LLM-optimized queries about Rust API documentation. Based on research of existing tools like `cargo-semver-checks`, `cargo-public-api`, and `rust-docs-mcp`, the recommended approach follows a four-layer architecture: CLI (clap), Commands, Index (graph-based), and Parser/Cache layers. The core challenge is balancing fast query performance (target: <100ms cached) with the inherent slowness of rustdoc JSON generation (~5s per crate). The solution centers on a two-level cache system using binary serialization (postcard) and content-addressable storage to enable incremental rebuilds and sub-second query responses.

The primary risk is rustdoc JSON format instability — it's nightly-only and changes 3-5 times per year. Tools that ignore format version checking or hardcode assumptions about the schema will break silently. This risk is mitigated by using the official `rustdoc-types` crate (maintained by the rustdoc team per RFC 3673) and implementing version range support rather than single-version assumptions. Secondary risks include memory exhaustion on large crates (aws-sdk-ec2 produces ~500MB JSON), cross-crate ID resolution failures, and unbounded recursive type expansion — all of which have established mitigation patterns from production tools.

The competitive differentiation strategy focuses on token efficiency for LLM consumption. While existing tools like ripdoc prioritize human-readable markdown and rust-docs-mcp emit verbose JSON, cargo-doc-query targets minimal output (100B-5KB per query) with configurable depth limits and signature-only modes. This addresses the context window constraints of modern LLMs (128K-2M tokens) while maintaining sub-100ms query latency through aggressive caching.

## Key Findings

### Recommended Stack

The technology stack is well-established in the Rust ecosystem for documentation tooling. All core technologies are officially maintained or de-facto standards with production validation from tools like `cargo-semver-checks` (obi1kenobi) and `cargo-public-api` (Enselic).

**Core technologies:**
- **clap ^4.5.x** — CLI argument parsing with native cargo subcommand support via derive macros — Standard for Rust CLI, excellent ergonomics
- **rustdoc-types ^0.57.x** — Official rustdoc JSON type definitions — Maintained by rustdoc team (RFC 3673), enables format version checking
- **rustdoc-json ^0.9.x** — Programmatic rustdoc JSON generation — Wrapper around cargo, handles nightly toolchain detection
- **cargo_metadata ^0.23.x** — Cargo workspace/dependency introspection — Standard for cargo plugins, 2M+ downloads/month
- **serde + serde_json** — Serialization framework — Required for JSON parsing, rustdoc-types uses internally
- **postcard ^1.x** (or bincode ^2.x) — Binary cache serialization — 2-5x faster than JSON, 3-10x smaller output
- **blake3 ^1.8** (or xxhash-rust ^0.8) — Content hashing for cache keys — Hardware-accelerated, fast for large files
- **anyhow ^1.x** — Error handling — Simplified error propagation, used by cargo-semver-checks

**Development tools:**
- **insta** — Snapshot testing for CLI output
- **assert_cmd + predicates** — CLI integration testing
- **cargo-nextest** — Faster test runner for CI

### Expected Features

Feature research analyzed ripdoc, rust-docs-mcp, and docs.rs to identify table stakes versus differentiators. The primary insight is that existing tools ignore LLM context window constraints — this is the key competitive opportunity.

**Must have (table stakes):**
- **Method queries by type** — "What methods does Vec have?" — Core use case, parse impl blocks from JSON
- **Trait implementation discovery** — Essential for understanding type capabilities (Display, Iterator, etc.)
- **JSON output format** — Required for LLM/programmatic consumption
- **Local crate support** — Must work on user's own code
- **crates.io dependency support** — Primary use case is querying third-party crates
- **Basic caching** — Without caching, ~5s rebuilds make tool unusable
- **Search/filtering** — Basic name matching for finding items

**Should have (competitive differentiators):**
- **Token-budget constrained output** — LLMs have 128K-2M token limits; efficient packing maximizes useful context — Critical differentiator, most tools ignore this
- **Depth-limited recursive type expansion** — "Show full picture but not too much" — prevents context overflow
- **Incremental rebuilds** — Only rebuild changed crates in large workspaces
- **Content-addressable cache storage** — Automatic invalidation when dependencies change
- **Multiple output modes** — JSON for agents, markdown for humans, minimal for piping
- **Sub-100ms query latency** — Fast enough for interactive CLI usage

**Defer (v2+):**
- **Workspace-wide queries** — Query across all workspace members
- **Documentation full-text search** — Requires tantivy integration
- **Export to llms.txt format** — Format still evolving

### Architecture Approach

A four-layer architecture separates concerns and enables independent testing and evolution. The graph-based index is the core innovation — transforming hierarchical JSON into bidirectional edges enables efficient relationship queries.

**Major components:**

1. **CLI Layer** — clap derive macros define command-line interface, global flags, and help generation
2. **Commands Layer** — Each subcommand (build, methods, expand, traits) implements a `RunCommand` trait; follows Command Pattern for testability
3. **Index Layer** — Graph structure (`petgraph::Graph<Node, Edge>`) representing type relationships; enables "what implements this trait?" and "what methods does this type have?" queries
4. **Parser & Cache Layer** — JSON deserialization (serde_json), binary serialization (postcard), and two-level cache (metadata JSON + per-crate binary files)

**Key patterns:**
- **Command Pattern with Trait** — Each command implements `RunCommand::run()`, enabling filesystem-based routing
- **Graph-Based Index** — Bidirectional edges for efficient traversal queries
- **Two-Level Cache** — Metadata tracks versions/timestamps; binary files store serialized index data
- **TypePath struct** — Parse and validate type paths (e.g., `std::vec::Vec<String>`) instead of stringly-typed paths

**Build order (by dependency):**
1. Parser Layer (foundation) → 2. Cache Layer (persistence) → 3. Index Layer (core logic) → 4. Commands Layer (user interface) → 5. CLI Layer (entry point)

### Critical Pitfalls

Research of production tools and rustdoc issues identified seven critical pitfalls, with format version blindness and cross-crate ID resolution being the most commonly encountered.

1. **Format Version Blindness** — Rustdoc JSON changes 3-5x/year; tools that ignore `format_version` field will break. **Mitigation:** Use `rustdoc-types` crate, support version ranges (28-35), fail fast with clear errors on unknown versions. **Address in:** Phase 1 (JSON Ingestion)

2. **Cross-Crate ID Resolution Failures** — IDs are only valid within a single JSON blob; external items reference but don't define items. **Mitigation:** Two-tier lookup (check `index` first, fall back to `paths`), handle missing external info gracefully, use `external_crates` map. **Address in:** Phase 2 (Query Engine Core)

3. **Multiple Package Version Output Collision** — When a crate depends on multiple versions of the same package, rustdoc JSON files collide (same filename). **Mitigation:** Use `-Cmetadata` or `-Cextra-filename` flags, store outputs in versioned directories. **Address in:** Phase 1 (Build Orchestration)

4. **Unbounded Recursive Type Expansion** — Generic substitution and recursive types can cause infinite recursion or stack overflow. **Mitigation:** Hard depth limits (10-20 levels), track visited type IDs with `HashSet<Id>`, use breadth-first expansion. **Address in:** Phase 3 (Type Expansion & Resolution)

5. **Memory Exhaustion on Large Crates** — aws-sdk-ec2 produces ~500MB JSON; loading into memory can OOM. **Mitigation:** Enable `rustc-hash` feature on rustdoc-types (~3% speedup, lower memory), implement lazy loading, use streaming JSON parsing for large files. **Address in:** Phase 1 (JSON Ingestion) and Phase 6 (Performance Optimization)

6. **Cache Invalidation Complexity** — Tools either rebuild too often (slow) or use stale data (incorrect). **Mitigation:** Content-addressable storage (hash JSON output), track all inputs (source hash, rustc version, feature flags, Cargo.lock), per-crate granularity. **Address in:** Phase 4 (Caching Layer)

7. **Assuming Rustdoc Always Succeeds** — Rustdoc can fail even if `cargo build` succeeds (broken doc links, proc-macro failures). **Mitigation:** Always check exit code, capture stderr, implement fallback strategies. **Address in:** Phase 1 (Build Orchestration)

## Implications for Roadmap

Based on research, suggested phase structure follows the architectural dependency chain while grouping related features:

### Phase 1: Foundation — JSON Ingestion & Build Orchestration
**Rationale:** Parser layer is the foundation — everything else depends on it. Must handle format versions, multiple package versions, and rustdoc failures before building on top.
**Delivers:** Rustdoc JSON generation, format version checking, basic deserialization, per-crate cache files
**Addresses:** Local crate support, crates.io support, basic caching
**Avoids:** Format Version Blindness, Multiple Package Version Output Collision, Assuming Rustdoc Always Succeeds
**Research needed:** LOW — Patterns well-documented via rustdoc-types and rustdoc-json crates

### Phase 2: Index & Query Core
**Rationale:** Index layer transforms JSON into queryable graph. Must be solid before building command layer on top.
**Delivers:** Graph-based index (types, methods, traits), ID resolution (including cross-crate), basic query methods
**Uses:** petgraph for graph structure, rustdoc-types for types
**Implements:** Index Layer architecture component
**Avoids:** Cross-Crate ID Resolution Failures
**Research needed:** MEDIUM — Graph schema design benefits from prototyping

### Phase 3: Commands & Output
**Rationale:** Commands layer depends on Index; groups user-facing functionality. Starting with methods query validates core value proposition.
**Delivers:** `build`, `methods`, `traits` commands, JSON output format, multiple output modes (JSON/minimal)
**Addresses:** Method queries by type, trait implementation discovery, JSON output format, minimal output mode
**Implements:** Commands Layer architecture component
**Research needed:** LOW — Standard CLI patterns

### Phase 4: Caching & Performance
**Rationale:** Caching is required for usable performance. Content-addressable storage enables reliable invalidation.
**Delivers:** Content-addressable cache, incremental rebuilds, cache metadata tracking, <100ms query latency
**Addresses:** Incremental rebuilds, content-addressable cache storage, sub-100ms query latency
**Avoids:** Cache Invalidation Complexity
**Research needed:** MEDIUM — Cache key design (which inputs to hash)

### Phase 5: LLM Optimizations
**Rationale:** Token efficiency is the key differentiator. Building on stable foundation from previous phases.
**Delivers:** Token-budget constrained output, depth-limited recursive type expansion, signature-only mode
**Addresses:** Token-budget constrained output, depth-limited type expansion, signature-only mode
**Avoids:** Unbounded Recursive Type Expansion
**Research needed:** LOW — Clear requirements from feature research

### Phase 6: Scale & Polish
**Rationale:** Performance optimization for large crates and production hardening.
**Delivers:** Lazy loading for large crates, memory limits, parallel processing (rayon), streaming JSON parsing
**Avoids:** Memory Exhaustion on Large Crates
**Research needed:** HIGH — Performance optimization requires profiling real large crates (aws-sdk-ec2)

### Phase Ordering Rationale

- **Parser before Index:** Index consumes Parser output; one-way dependency
- **Index before Commands:** Commands query the Index; can't build UI without queryable data
- **Basic Commands before Caching:** Need to understand query patterns to design cache keys
- **Caching before LLM Optimizations:** Token budget constraints require fast queries to be usable; caching enables fast queries
- **Performance optimization last:** Premature optimization is wasteful; optimize after functionality is proven

This ordering also minimizes technical debt:
- Phase 1 addresses 3 critical pitfalls (format versions, version collisions, rustdoc failures)
- Phase 2 addresses the cross-crate resolution pitfall
- Phase 4 addresses cache invalidation complexity
- Phase 5 addresses recursive expansion limits
- Phase 6 addresses memory exhaustion

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (Index & Query Core):** Graph schema design — specific Node/Edge types and their relationships will benefit from prototyping with real rustdoc JSON
- **Phase 4 (Caching & Performance):** Cache key design — which inputs to track (Cargo.lock hash? rustc version? feature flags?)
- **Phase 6 (Scale & Polish):** Performance optimization — requires real large crate testing (aws-sdk-ec2 ~500MB JSON)

Phases with standard patterns (skip research-phase):
- **Phase 1 (Foundation):** Well-documented via rustdoc-types, rustdoc-json crates; cargo-semver-checks reference implementation
- **Phase 3 (Commands & Output):** Standard CLI patterns with clap; ripdoc and rust-docs-mcp provide reference implementations
- **Phase 5 (LLM Optimizations):** Clear requirements from feature research; no novel algorithms

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | **HIGH** | All technologies are official/de-facto standards with production validation from cargo-semver-checks and cargo-public-api |
| Features | **HIGH** | Clear differentiation strategy based on LLM context constraints; validated against existing tools (ripdoc, rust-docs-mcp, docsrs) |
| Architecture | **HIGH** | Four-layer pattern proven by cargo-public-api; graph-based indexing is standard for relationship queries |
| Pitfalls | **HIGH** | All pitfalls documented in rustdoc-types docs, rust-lang RFCs, and cargo-semver-checks lessons learned |

**Overall confidence:** HIGH

All four research areas have HIGH confidence based on official documentation, production tool validation, and community consensus. The primary uncertainty is not "what to build" but "performance characteristics at scale" — which is an implementation detail to be resolved during Phase 6.

### Gaps to Address

**During Phase 1 (Foundation):**
- **Nightly toolchain handling:** Exact mechanism for detecting and invoking nightly rustdoc — verify rustdoc-json crate behavior with various toolchain configurations

**During Phase 2 (Index & Query Core):**
- **Graph schema details:** Specific Node and Edge variants needed — prototype with 2-3 representative crates to validate schema covers query needs

**During Phase 4 (Caching & Performance):**
- **Cache invalidation inputs:** Which exact inputs to hash (Cargo.lock? features? rustc version?) — test with real dependency updates to ensure correct invalidation

**During Phase 6 (Scale & Polish):**
- **Large crate performance:** Memory usage and query latency on aws-sdk-ec2-scale crates — requires real-world testing

## Sources

### Primary (HIGH confidence)
- **docs.rs/rustdoc-types** (0.57.0) — Official API documentation, format version handling
- **RFC 3673** — rustdoc-types official maintenance by rustdoc team
- **docs.rs/rustdoc-json** (0.9.8) — JSON generation utilities
- **docs.rs/cargo_metadata** (0.23.1) — Cargo integration standard
- **docs.rs/clap** (4.5.x) — CLI derive patterns, cargo subcommand examples
- **RFC 2963** — rustdoc JSON format specification

### Secondary (MEDIUM confidence)
- **GitHub: cargo-semver-checks** (obi1kenobi) — Production usage validation, performance benchmarks, format version handling patterns
- **GitHub: cargo-public-api** (Enselic) — Architecture reference, command patterns
- **GitHub: ripdoc** (Alb-O) — Markdown output focus, CLI UX patterns
- **lib.rs: rust-docs-mcp** — MCP protocol integration, content-addressable cache pattern

### Tertiary (LOW confidence)
- **llmstxt.org** — Emerging llms.txt format standard (still evolving)
- **Context window limits research** — Model specifications change frequently; verify current limits during implementation

---
*Research completed: 2026-02-12*
*Ready for roadmap: yes*
