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
| User can run `cargo doc-query build` | ✅ Verified (01-04) |
| Build generates rustdoc JSON for all dependencies | ✅ Verified (01-04) |
| Format version validated (fail fast on incompatible) | ✅ Implemented |
| Graph-based index created and persisted | ✅ Implemented |
| Cache persistence with automatic invalidation | ✅ Implemented |

**Progress:** ██████████░ 90% (Plans 01-04 complete)

---

## Active Plan

**Plan:** 01-05 (next planned in foundation phase)
**Status:** Ready to plan
**Last activity:** 2026-02-12 - Completed 01-04-SUMMARY.md

### Completed Plans

| Plan | Name | Commit | Summary |
| ---- | ---- | ------ | ------- |
| 01-01 | CLI foundation | d1e47d2 | Command trait, clap CLI, module structure |
| 01-02 | Build workflow | 21304e5 | Dependency discovery, rustdoc JSON generation, format validation, graph index |
| 01-03 | Cache persistence | cd35fbe | BLAKE3 key generation, postcard serialization, cache store integration |
| 01-04 | Gap closure - manifest resolution | e13ab03 | Fixed rustdoc-json manifest resolution, filtered external dependencies, graceful error handling |

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
| 01-03 | 1 | Create cache module structure | 3e2b99d |
| 01-03 | 2 | Implement cache key generation | ed40e09 |
| 01-03 | 3 | Implement cache store with postcard | a4e82c4 |
| 01-03 | 4 | Integrate cache into BuildCommand | cd35fbe |
| 01-04 | 1 | Filter external dependencies only | 0583c81 |
| 01-04 | 2 | Fix rustdoc-json manifest path handling | e13ab03 |
| 01-04 | 3 | Verify cache contains actual data | (skipped - gitignored) |

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
| 2026-02-12 | BLAKE3 cache key from Cargo.lock + rustc version + target | Ensures cache invalidation on dependency changes |
| 2026-02-12 | Postcard binary serialization for cache storage | Smaller files, faster I/O, sub-100ms reads |

### Open Questions

- Graph schema details: specific Node and Edge variants needed (planned for Phase 02)
- Performance baseline: memory usage on large crates (aws-sdk-ec2)
- Cache file size optimization for extremely large dependencies (future consideration)

### Known Blockers

None

### Technical Debt

- **Dead code warnings:** json_path field in CrateNode and DependencyEdge variants are unused (expected - will be used in phase 02-01)
- **Empty edges in SerializableIndex:** Edges are not populated (planned for Phase 02 graph relationships)

---

## Session Continuity

### Recent Context

- Project initialized with requirements and research complete
- Phase 1 roadmap created with 5 phases covering 18 v1 requirements
- 01-01: CLI foundation with clap and Command trait (complete)
- 01-02: Build workflow with dependency discovery, rustdoc JSON generation, format validation, and graph index (complete)
- 01-03: Cache persistence with BLAKE3 keys and postcard serialization (complete)
- 01-04: Gap closure - fixed rustdoc-json manifest resolution and verified cache (complete)

### Session Handoff Notes

If resuming work:
1. Current phase: Phase 1 (Foundation)
2. Active plan: Plans 01-01 through 01-04 complete, ready for next phase (01-05 or Phase 2)
3. Cache is working: users can run `cargo doc-query build` with caching for 80+ external dependencies
4. Graph relationships not yet populated (Phase 02 work)
5. BUILD-01 and BUILD-02 requirements verified (cargo doc-query build succeeds, no virtual manifest errors)

---

## Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Query latency (cached) | — | <100ms |
| Build time (small project) | <5s | <5s |
| Requirements implemented | 5/18 | 18/18 |
| Phases complete | 1/5 | 5/5 |

---

*This file is updated after each session. Check `git log .planning/STATE.md` for history.*
