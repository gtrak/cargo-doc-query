# Project State: cargo-doc-query

**Current Phase:** 1 — Foundation (JSON Ingestion & Index)
**Phase Status:** 🔄 In progress
**Last Updated:** 2026-02-12

---

## Current Position

**Working on:** Phase 1 — Foundation
**Next milestone:** Phase 1 success criteria completion

### Phase 1 Progress

| Success Criterion | Status |
|-------------------|--------|
| User can run `cargo doc-query build` | ✅ Implemented |
| Build generates rustdoc JSON for all dependencies | ✅ Implemented |
| Format version validated (fail fast on incompatible) | ✅ Implemented |
| Graph-based index created and persisted | ✅ Implemented |

**Progress:** ████░░░░░░░ 40% (Plan 01-02 complete)

---

## Active Plan

**Plan:** 01-03
**Status:** 🔄 In progress
**Next milestone:** Complete Phase 1 foundation with cache layer

### Completed Plans

| Plan | Name | Commit | Summary |
| ---- | ---- | ------ | ------- |
| 01-01 | CLI foundation | d1e47d2 | Command trait, clap CLI, module structure |
| 01-02 | Build workflow | 21304e5 | Dependency discovery, rustdoc JSON generation, format validation, graph index |

### Completed Tasks (All Plans)

| Plan | Task | Name | Commit |
| ---- | ---- | ---- | ------ |
| 01-01 | 1 | Declare dependencies in Cargo.toml | 45b77d0 |
| 01-01 | 2 | Create CLI entry point with clap | 36b7bfb |
| 01-01 | 3 | Create Command trait architecture | 33e3410 |
| 01-02 | 1 | Implement dependency discovery | 22c23cc |
| 01-02 | 2 | Implement format version validation | 9475523 |
| 01-02 | 3 | Implement graph-based index | b97e66c |
| 01-02 | 4 | Implement BuildCommand workflow | 21304e5 |

---

## Project Reference

### Core Value

Sub-100ms deterministic structured API extraction that reduces LLM context usage compared to raw source or LSP, without requiring a long-running daemon.

### Key Constraints

- **Tech Stack:** Rust, rustdoc JSON, bincode/postcard, serde
- **Performance:** Sub-100ms queries (cached), <5s build for small projects
- **Compatibility:** Nightly Rust required for JSON output
- **Runtime:** Stateless CLI (no daemon), safe for repeated invocation

### Architecture Overview

```
CLI Layer (clap)
    ↓
Commands Layer (Command trait)
    ↓
Index Layer (petgraph::Graph)
    ↓
Parser & Cache Layer (serde_json, postcard)
```

---

## Accumulated Context

### Decisions Made

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-02-12 | Use rustdoc JSON instead of LSP | Deterministic, no daemon, machine-readable |
| 2026-02-12 | 5-phase roadmap | Balances depth vs. delivery; follows architectural dependencies |
| 2026-02-12 | Content-hash cache invalidation | Automatic rebuild on dependency changes |
| 2026-02-12 | Use clap derive macros for CLI | Type-safe argument parsing, easier maintenance |
| 2026-02-12 | Establish Command trait pattern | Enables extensible command architecture for future phases |
| 2026-02-12 | Use rustdoc-json wrapper | Better error handling than direct rustdoc invocation |
| 2026-02-12 | Fail-fast format validation | Prevents cryptic serde errors, provides clear error messages |
| 2026-02-12 | Graph key is (name, version) tuple | Handles multiple versions of same crate correctly |
| 2026-02-12 | Graceful error handling on individual crates | Continues build even if one dependency fails |

### Open Questions

- Cache key design: which inputs to hash? (Cargo.lock hash, rustc version, feature flags?)
- Graph schema details: specific Node and Edge variants needed
- Performance baseline: memory usage on large crates (aws-sdk-ec2)

### Known Blockers

None

### Technical Debt

- **Dead code warnings:** json_path field in CrateNode and DependencyEdge variants are unused (expected - will be used in phase 01-03 and 02-01)
- **Missing postcard integration:** Index structure created but not yet persisted (planned for phase 01-03)
- **No BLAKE3 hashing:** Cache key generation not yet implemented (planned for phase 01-03)

---

## Session Continuity

### Recent Context

- Project initialized with requirements and research complete
- Phase 1 roadmap created with 5 phases covering 18 v1 requirements
- 01-01: CLI foundation with clap and Command trait (complete)
- 01-02: Build workflow with dependency discovery, rustdoc JSON generation, format validation, and graph index (complete)

### Session Handoff Notes

If resuming work:
1. Current phase: Phase 1 (Foundation)
2. Active plan: 01-03 (cache layer and serialization)
3. Build workflow is complete: users can run `cargo doc-query build`
4. Graph index created in memory, needs persistence (next phase)

---

## Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Query latency (cached) | — | <100ms |
| Build time (small project) | — | <5s |
| Requirements implemented | 2/18 | 18/18 |
| Phases complete | 2/5 | 5/5 |

---

*This file is updated after each session. Check `git log .planning/STATE.md` for history.*
