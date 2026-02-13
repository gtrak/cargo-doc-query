# Requirements: cargo-doc-query v1.1

**Defined:** 2026-02-13
**Core Value:** Sub-100ms deterministic structured API extraction that reduces LLM context usage compared to raw source or LSP, without requiring a long-running daemon.

---

## v1.1 Requirements — Output Refinement

### Rendering (REND)

Unified rendering across all item types and depths.

- [ ] **REND-01**: All 24 ItemKind variants render with consistent formatting
- [ ] **REND-02**: Single `format_item()` dispatcher handles all item types
- [ ] **REND-03**: Depth-aware formatting (root=full details, nested=condensed, minimal=signatures only)
- [ ] **REND-04**: Token budget integrated at rendering layer (tracks per-item overhead)

### Filtering (FILT)

Robust pattern-based filtering for precise queries.

- [ ] **FILT-01**: `--include` flag accepts glob patterns for item paths
- [ ] **FILT-02**: `--exclude` flag accepts glob patterns to filter out items
- [ ] **FILT-03**: `--kind` flag filters by item kind (struct, enum, trait, function, etc.)
- [ ] **FILT-04**: `--crate` flag restricts results to specific crate(s)
- [ ] **FILT-05**: `--visibility` flag filters by visibility level (pub, pub(crate), etc.)
- [ ] **FILT-06**: Multiple filter flags combine with AND logic
- [ ] **FILT-07**: Invalid glob patterns produce helpful error messages

### Documentation (DOCS)

Doc comment extraction with token-aware display.

- [ ] **DOCS-01**: Doc comments extracted from `Item::docs` field
- [ ] **DOCS-02**: Doc comments display in standard output mode
- [ ] **DOCS-03**: Doc comments omitted in minimal mode to save tokens
- [ ] **DOCS-04**: Smart truncation at sentence boundaries when budget exceeded
- [ ] **DOCS-05**: Code blocks preserved over prose during truncation
- [ ] **DOCS-06**: Truncated docs show "..." indicator with warning
- [ ] **DOCS-07**: Token budget enforcement includes doc comment tokens

### Additional Fields (FIELD)

Expose rich metadata from rustdoc JSON.

- [ ] **FIELD-01**: Visibility modifiers displayed (pub, pub(crate), pub(super), pub(in path))
- [ ] **FIELD-02**: Deprecation status shown with replacement hints when available
- [ ] **FIELD-03**: Generic parameters and bounds displayed for structs, enums, functions
- [ ] **FIELD-04**: Key attributes shown: #[must_use], #[non_exhaustive], #[deprecated]
- [ ] **FIELD-05**: Function modifiers displayed: const, unsafe, async, ABI info
- [ ] **FIELD-06**: New fields omitted in minimal mode for token efficiency
- [ ] **FIELD-07**: JSON output includes all new fields with backward-compatible schema

---

## v2.0 Requirements (Deferred from v1.1)

Infrastructure features deferred to v2.0.

### Infrastructure (INFRA)

- **INFRA-01**: Shared cache directory across projects in `~/.cargo/doc-query/`
- **INFRA-02**: Stdlib queries (Vec, String, Iterator) with rust build system integration
- **INFRA-03**: Garbage collection command to clean stale cache files
- **INFRA-04**: Cache deduplication for identical dependencies across projects

---

## Out of Scope

Explicitly excluded from v1.1.

| Feature | Reason |
|---------|--------|
| Full source code extraction | Massive token bloat, parsing complexity |
| Real-time file watching | Out of scope for stateless CLI tool |
| IDE integration | LSP handles this use case |
| GUI interface | CLI-first tool philosophy |
| Type checking | cargo check handles errors |
| Complex regex filtering | Glob patterns sufficient, regex adds complexity |
| Full markdown rendering | rustdoc handles rich docs; we extract plain text |
| Source location display (span) | Minimal value, high token cost |
| Generic impl tracking | Too expensive for common queries |

---

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FILT-01 | Phase 6 | Pending |
| FILT-02 | Phase 6 | Pending |
| FILT-03 | Phase 6 | Pending |
| FILT-04 | Phase 6 | Pending |
| FILT-05 | Phase 6 | Pending |
| FILT-06 | Phase 6 | Pending |
| FILT-07 | Phase 6 | Pending |
| FIELD-01 | Phase 8 | Pending |
| FIELD-02 | Phase 8 | Pending |
| FIELD-03 | Phase 8 | Pending |
| FIELD-04 | Phase 8 | Pending |
| FIELD-05 | Phase 8 | Pending |
| FIELD-06 | Phase 8 | Pending |
| FIELD-07 | Phase 8 | Pending |
| REND-01 | Phase 9 | Pending |
| REND-02 | Phase 9 | Pending |
| REND-03 | Phase 9 | Pending |
| REND-04 | Phase 9 | Pending |
| DOCS-01 | Phase 9 | Pending |
| DOCS-02 | Phase 9 | Pending |
| DOCS-03 | Phase 9 | Pending |
| DOCS-04 | Phase 9 | Pending |
| DOCS-05 | Phase 9 | Pending |
| DOCS-06 | Phase 9 | Pending |
| DOCS-07 | Phase 9 | Pending |

**Coverage:**
- v1.1 requirements: 25 total
- Mapped to phases: 25
- Unmapped: 0 ✓

**Phase Distribution:**
- Phase 6 (Foundation — FilterEngine): 7 requirements (FILT-01..07)
- Phase 7 (CLI Integration): 0 requirements (implements Phase 6 deliverables)
- Phase 8 (Result Type Extensions): 7 requirements (FIELD-01..07)
- Phase 9 (Unified Rendering + Docs): 11 requirements (REND-01..04, DOCS-01..07)
- Phase 10 (Integration + Polish): 0 requirements (validates all previous)

---

*Requirements defined: 2026-02-13*
*Last updated: 2026-02-13 after v1.1 scope definition*
