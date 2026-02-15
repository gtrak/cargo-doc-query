# Roadmap: cargo-doc-query

## Milestones

- ✅ **v1.0 MVP** — Phases 1-5 (shipped 2026-02-13)
- ✅ **v1.1 Output Refinement** — Phases 6-10 (shipped 2026-02-14)
- 🚧 **v2.0 Infrastructure** — Phases 11-15 (planned)
- 📋 **v3.0** — Future

## v2.0 Infrastructure (Planned)

**Goal:** Shared cache, stdlib queries, garbage collection

### Requirements

- INFRA-01: Shared cache directory across projects in `~/.cargo/doc-query/`
- INFRA-02: Stdlib queries (Vec, String, Iterator) with rust build system integration
- INFRA-03: Garbage collection command to clean stale cache files
- INFRA-04: Cache deduplication for identical dependencies across projects

### Phase Ideas (TBD during planning)

- Phase 11: Shared cache directory implementation
- Phase 12: Stdlib query support
- Phase 13: Cache garbage collection
- Phase 14: Cache deduplication
- Phase 15: Performance verification

---

## Progress

| Phase | Milestone | Plans | Status |
|-------|-----------|-------|--------|
| 1-5 | v1.0 MVP | 19/19 | Complete |
| 6-10 | v1.1 Output Refinement | 20/20 | Complete |
| 11-15 | v2.0 Infrastructure | 0/5 | Not started |

---

*Roadmap archived: v1.0 in milestones/v1.0-ROADMAP.md, v1.1 in milestones/v1.1-ROADMAP.md*
