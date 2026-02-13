# cargo-doc-query

## Current Milestone: v1.1 Output Refinement

**Goal:** Unify item rendering across all depths and provide robust filtering with documentation support

**Target features:**
- Consistent kind rendering (modules/functions/types/structs/enums) at any tree depth
- Robust `--include`/`--exclude` filtering flags
- Doc comment extraction and display
- Discovery and implementation of additional rustdoc JSON fields

## Previous Milestone

**Shipped:** v1.0 MVP (2026-02-13)

A production-ready Cargo subcommand for fast, structured API queries over Rust dependencies. Currently used for LLM agent contexts and CLI exploration.

## What This Is

A Cargo subcommand that provides fast, structured, low-context queries over Rust dependency APIs using rustdoc JSON output. Designed for LLM agents and CLI users who need quick API discovery without the overhead of LSP daemons or raw source code.

## Core Value

Sub-100ms deterministic structured API extraction that reduces LLM context usage compared to raw source or LSP, without requiring a long-running daemon.

## Requirements

### Validated (v1.0)

- ✓ Generate rustdoc JSON for all dependencies — v1.0
- ✓ Query methods for any type with signatures and fully qualified types — v1.0
- ✓ JSON output optimized for LLM agents — v1.0
- ✓ Recursive type expansion with depth limits — v1.0
- ✓ Trait implementation discovery — v1.0
- ✓ Associated type resolution — v1.0
- ✓ Automatic rebuild when Cargo.lock changes — v1.0
- ✓ Per-crate incremental rebuilds — v1.0
- ✓ Content-addressable cache storage — v1.0
- ✓ Token-budget constrained output modes — v1.0
- ✓ Minimal and verbose output formats — v1.0

### Active (v1.1) — Output Refinement

- [ ] Unified kind rendering (modules/functions/types/structs/enums) at any depth
- [ ] Robust `--include`/`--exclude` filtering flags
- [ ] Doc comment extraction and display
- [ ] Discovery and implementation of additional rustdoc JSON fields

### Deferred to v2.0

- Stdlib queries (Vec, String, Iterator) — deferred for infrastructure focus
- Shared cache directory across projects — deferred for infrastructure focus  
- Garbage collection command — deferred for infrastructure focus

### Out of Scope

- Type checking (cargo check handles errors)
- IDE features (go-to-definition, refactoring)
- Runtime code execution
- General semantic search across codebases
- Real-time collaboration features
- GUI interface

## Next Milestone Goals (v2.0)

1. **Shared Cache** — Deduplicate dependency JSON across projects in `~/.cargo/doc-query/`
2. **Stdlib Support** — Query standard library types (requires rust build system integration)
3. **Garbage Collection** — Clean up stale cache files with `cargo doc-query gc`

## v1.1 Goals (In Progress)

1. **Unified Rendering** — Consistent display of item kinds regardless of tree depth
2. **Robust Filtering** — Comprehensive `--include`/`--exclude` flag support
3. **Documentation** — Extract and display doc comments with token-aware truncation
4. **Field Discovery** — Audit rustdoc JSON schema for missing display fields

## Context

### Problem Being Solved

LLM agents struggle with unfamiliar third-party libraries:
- Reliably discovering method sets and trait implementations
- Operating within limited context windows
- Waiting for slow LSP reindexing

LSP (rust-analyzer) limitations:
- Requires a daemon
- Slow to reindex
- Produces high-overhead responses

### Target Users

1. **LLM Agents** — Need structured API info with minimal tokens
2. **CLI Developers** — Want quick lookup of method sets

### Technical Environment

- Rust 1.93.0+ with nightly for rustdoc JSON output
- Cargo subcommand pattern
- Stateless CLI (no daemon)
- Disk-based caching in `target/doc-query/`

### Known Risks

1. rustdoc JSON instability (requires nightly flag)
2. Large dependency graphs increase memory usage
3. Generic-heavy crates increase expansion size

## Constraints

- **Tech Stack**: Rust, rustdoc JSON, bincode, serde
- **Performance**: Sub-100ms queries, <5s build for small projects
- **Compatibility**: Nightly Rust required for JSON output
- **Runtime**: Stateless, no daemon, safe for repeated invocation

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|----------|
| Use rustdoc JSON instead of LSP | Deterministic, no daemon, machine-readable | ✓ Good — sub-100ms queries achieved |
| Sharded crate storage | Memory efficiency, selective rebuilds | ✓ Good — 83 crates indexed efficiently |
| Content-hash cache invalidation | Automatic rebuild on dependency changes | ✓ Good — BLAKE3 works well |
| Nightly Rust required | Only way to get JSON output from rustdoc | ✓ Accepted — documented requirement |
| Typed errors (ExpandError) | Replace fragile string matching | ✓ Good — cleaner error handling |
| Unified query/expand command | Simplify CLI, single code path | ✓ Good — removed duplication |

---
*Last updated: 2026-02-13 — Milestone v1.1 started (output refinement focus)*
