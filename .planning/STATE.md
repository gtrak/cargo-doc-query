# Project State: cargo-doc-query

**Current Phase:** 1 — Foundation (JSON Ingestion & Index)  
**Phase Status:** ⏳ Pending  
**Last Updated:** 2026-02-12

---

## Current Position

**Working on:** Phase 1 — Foundation  
**Next milestone:** Phase 1 success criteria completion  

### Phase 1 Progress

| Success Criterion | Status |
|-------------------|--------|
| User can run `cargo doc-query build` | ⏳ Pending |
| Build generates rustdoc JSON for all dependencies | ⏳ Pending |
| Format version validated (fail fast on incompatible) | ⏳ Pending |
| Graph-based index created and persisted | ⏳ Pending |

**Progress:** ░░░░░░░░░░ 0%

---

## Active Plan

**Plan:** Phase 1 Implementation  
**Status:** ⏳ Not Started  

### Current Tasks

*No active tasks. Run `/gsd-plan-phase 1` to generate Phase 1 plan.*

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

### Open Questions

- Cache key design: which inputs to hash? (Cargo.lock hash, rustc version, feature flags?)
- Graph schema details: specific Node and Edge variants needed
- Performance baseline: memory usage on large crates (aws-sdk-ec2)

### Known Blockers

None

### Technical Debt

None yet

---

## Session Continuity

### Recent Context

- Project initialized with requirements and research complete
- Roadmap created with 5 phases covering 18 v1 requirements
- Ready to begin Phase 1 planning

### Session Handoff Notes

If resuming work:
1. Current phase: Phase 1 (Foundation)
2. No active plan — need to run `/gsd-plan-phase 1`
3. Focus: JSON generation, format validation, graph index creation

---

## Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Query latency (cached) | — | <100ms |
| Build time (small project) | — | <5s |
| Requirements implemented | 0/18 | 18/18 |
| Phases complete | 0/5 | 5/5 |

---

*This file is updated after each session. Check `git log .planning/STATE.md` for history.*
