# Project State: cargo-doc-query

**Current Phase:** 4 — Advanced Features (Recursive Expansion & Token Budgets)
**Phase Status:** ✅ Complete
**Last Updated:** 2026-02-12

---

## Current Position

**Completed:** Phase 4 — Advanced Features ✅
**Working on:** Phase 5 — Integration & Polish (ready to start)
**Next milestone:** Production-ready CLI with comprehensive error handling and shell integration

### Phase 1 Progress

| Success Criterion | Status |
|-------------------|--------|
| User can run `cargo doc-query build` | ✅ Verified (01-04) |
| Build generates rustdoc JSON for all dependencies | ✅ Verified (01-04) |
| Format version validated (fail fast on incompatible) | ✅ Implemented |
| Graph-based index created and persisted | ✅ Implemented |
| Cache persistence with automatic invalidation | ✅ Implemented |

**Progress:** ██████████░ 90% (Plans 01-04 complete)

**Status:** ✅ Complete (2026-02-12)

### Phase 2 Progress

| Success Criterion | Status |
|-------------------|--------|
| User can query methods for external dependency types | ✅ Verified |
| Method output includes signature and return types | ✅ Verified |
| User can query trait implementations for external types | ✅ Verified |
| Trait output includes associated types and methods | ✅ Verified |
| Query responses are valid, parseable JSON | ✅ Verified |
| Command output can be piped to other tools | ✅ Verified |

**Progress:** ████████████ 100% (All plans complete)

**Delivered:**
- 80 external dependency crates indexed via cached JSON
- Working queries: anyhow::Error, semver::Version, petgraph::Graph, etc.
- Complete JSON output with methods, signatures, documentation
- CLI with --crate, --kind, --include flags

**Deferred to v1.1:** Stdlib queries (Vec, String, Iterator) - requires rust build system integration

---

## Phase 3 Preview

**Goal:** Query responses complete in under 100ms through intelligent caching and automatic incremental rebuilds.

**Requirements:**
- BUILD-03: Index is cached to disk for sub-100ms query performance ✅
- BUILD-04: Index automatically rebuilds when Cargo.lock changes ✅

**Plans:**
- **03-01:** ✅ Automatic cache invalidation — extends cache key to include Cargo.toml, adds manifest change detection to query command, triggers transparent rebuilds
- **03-02:** [TODO] Parallel query execution

**Current Status:**
- ✅ Plan 01 complete — cache key includes Cargo.toml, automatic rebuild on manifest changes, sub-100ms query verified (7ms)
- ✅ Cache invalidation via Cargo.lock hash extended to include Cargo.toml
- ✅ Dependency filtering excludes transitive dependencies (7 direct deps indexed)

---

## Phase 4 Progress

**Goal:** Users can explore type hierarchies recursively with depth limits and constrain output by token budgets for LLM efficiency.

| Success Criterion | Status |
|-------------------|--------|
| User can expand a type recursively | ✅ Implemented (04-01) |
| Expansion respects depth limits | ✅ Implemented (04-01) |
| User can set token budget constraints | ✅ Implemented (04-02) |
| Minimal mode outputs signature-only | ✅ Implemented (04-02) |
| Query command supports --minimal and --tokens | ✅ Implemented (04-03) |

**Progress:** ████████████ 100% (All plans complete)

**Delivered:**
- `cargo doc-query expand <path> --depth N` command
- Type hierarchy exploration with cycle detection
- `--tokens N` flag for token budget control
- `--minimal` flag for signature-only output
- Consistent flags across query and expand commands
- Token estimation: JSON length / 4
- Budget exceeded warnings with truncated paths

---

## Active Plan

**Plan:** 03-02 - [TODO] Parallel query execution
**Status:** Planned, ready for execution
**Last activity:** 2026-02-12 - Completed 03-01-SUMMARY.md

### Completed Plans

| Plan | Name | Commit | Summary |
| ---- | ---- | ------ | ------- |
| 01-01 | CLI foundation | d1e47d2 | Command trait, clap CLI, module structure |
| 01-02 | Build workflow | 21304e5 | Dependency discovery, rustdoc JSON generation, format validation, graph index |
| 01-03 | Cache persistence | cd35fbe | BLAKE3 key generation, postcard serialization, cache store integration |
| 01-04 | Gap closure - manifest resolution | e13ab03 | Fixed rustdoc-json manifest resolution, filtered external dependencies, graceful error handling |
| 03-01 | Automatic cache invalidation | 5a94b94 | Cache key includes Cargo.toml, automatic rebuild on manifest changes, sub-100ms query verified (7ms) |
| 04-01 | Recursive type expansion | 7f2c3c8 | Expand command with --depth, cycle detection, field extraction |
| 04-02 | Token budgets & minimal mode | 0884afb | --tokens and --minimal flags for expand command |
| 04-03 | Query integration | a816fcd | --tokens and --minimal flags for query command |

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
| 03-01 | 1 | Extend CacheKeyInputs to include Cargo.toml | 5a94b94 |
| 03-01 | 2 | Add manifest change detection to query command | 4d2f372 |
| 03-01 | 3 | Fix dependency discovery to exclude transitive deps | 6d79aed |
| 03-01 | 4 | Add benchmark timing to verify performance | 4d2f372 |

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
| 2026-02-12 | Include Cargo.toml in cache key | Ensures cache invalidates on dependency/features changes; complements Cargo.lock version pinning |
| 2026-02-12 | Query auto-rebuild on manifest change | Transparent user experience; eliminates manual rebuild after `cargo update` |
| 2026-02-12 | Use metadata.resolve for direct deps only | Reduces index size; prevents duplicate indexing; excludes implementation details |
| 2026-02-12 | Print timing to stderr | Separates metadata from JSON output; clean stdout for scripts |

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
- Phase 1 complete: CLI foundation, build workflow, cache persistence, manifest resolution (80+ external deps indexed)
- Phase 2 complete: Core querying with methods, traits, JSON output (verified with 80+ external crate queries)
- Phase 3: Automatic cache invalidation with sub-100ms queries (deferred parallel execution to v1.1)
- Phase 4 complete: Recursive type expansion, token budgets, minimal mode for both query and expand commands

### Session Handoff Notes

If resuming work:
1. Current phase: Phase 5 (Integration & Polish)
2. Phase 4 is complete: Recursive expansion, token budgets, minimal mode all implemented
3. Both query and expand commands support --minimal and --tokens flags
4. Ready to start Phase 5: Error handling, progress indicators, documentation polish

---

## Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Query latency (cached) | 7ms (verified) | <100ms |
| Build time (small project) | <5s | <5s |
| Requirements implemented | 13/18 | 18/18 |
| Phases complete | 4/5 | 5/5 |

---

*This file is updated after each session. Check `git log .planning/STATE.md` for history.*
