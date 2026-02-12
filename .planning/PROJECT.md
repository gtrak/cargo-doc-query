# cargo-doc-query

## What This Is

A Cargo subcommand that provides fast, structured, low-context queries over Rust dependency APIs using rustdoc JSON output. Designed for LLM agents and CLI users who need quick API discovery without the overhead of LSP daemons or raw source code.

## Core Value

Sub-100ms deterministic structured API extraction that reduces LLM context usage compared to raw source or LSP, without requiring a long-running daemon.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Generate rustdoc JSON for all dependencies
- [ ] Query methods for any type with signatures and fully qualified types
- [ ] JSON output optimized for LLM agents
- [ ] Recursive type expansion with depth limits
- [ ] Trait implementation discovery
- [ ] Associated type resolution
- [ ] Automatic rebuild when Cargo.lock changes
- [ ] Per-crate incremental rebuilds
- [ ] Content-addressable cache storage
- [ ] Token-budget constrained output modes
- [ ] Minimal and verbose output formats

### Out of Scope

- Type checking (cargo check handles errors)
- IDE features (go-to-definition, refactoring)
- Runtime code execution
- General semantic search across codebases
- Real-time collaboration features
- GUI interface

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
|----------|-----------|---------|
| Use rustdoc JSON instead of LSP | Deterministic, no daemon, machine-readable | — Pending |
| Sharded crate storage | Memory efficiency, selective rebuilds | — Pending |
| Content-hash cache invalidation | Automatic rebuild on dependency changes | — Pending |
| Nightly Rust required | Only way to get JSON output from rustdoc | — Accepted |

---
*Last updated: 2026-02-12 after initialization*
