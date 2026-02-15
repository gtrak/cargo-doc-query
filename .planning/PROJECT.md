# cargo-doc-query

## Current Milestone: v2.0 Infrastructure (Planned)

**Goal:** Shared cache, stdlib queries, garbage collection

**Target features:**
- Shared cache directory across projects in `~/.cargo/doc-query/`
- Stdlib queries (Vec, String, Iterator) with rust build system integration
- Garbage collection command to clean stale cache files
- Cache deduplication for identical dependencies across projects

## Previous Milestones

### v1.1 Output Refinement (Shipped: 2026-02-14)

Unified rendering, robust filtering, doc comment extraction, and rich metadata for LLM-optimized API queries.

- FilterEngine with glob patterns, validation, and statistics
- CLI integration with --include, --exclude, --kind, --crate, --visibility flags
- Result type extensions: visibility, deprecation, generics, attributes
- Unified rendering: single dispatcher for all ItemKind variants
- Doc comment extraction with smart truncation
- Token budget enforcement at rendering layer
- Integration tests (43+) and snapshot tests (12)
- Comprehensive README with all v1.1 features documented

### v1.0 MVP (Shipped: 2026-02-13)

A production-ready Cargo subcommand for fast, structured API queries over Rust dependencies.

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

### Validated (v1.1)

- ✓ Unified kind rendering (modules/functions/types/structs/enums) at any depth — v1.1
- ✓ Robust `--include`/`--exclude` filtering flags — v1.1
- ✓ Doc comment extraction and display — v1.1
- ✓ Discovery and implementation of additional rustdoc JSON fields — v1.1

### Active (v2.0) — Infrastructure

- [ ] Shared cache directory across projects
- [ ] Stdlib queries (Vec, String, Iterator)
- [ ] Garbage collection command
- [ ] Cache deduplication

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
| FilterEngine with glob | Simple pattern matching, pre-compiled | ✓ Good — performant filtering |
| DetailLevel enum | Control metadata at render time | ✓ Good — clean separation |
| ItemFormatter dispatcher | Unified rendering pipeline | ✓ Good — REND-01 achieved |

---

*Last updated: 2026-02-14 — Milestone v1.1 complete, v2.0 ready to plan*
