# Roadmap: cargo-doc-query v1.1

**Milestone:** v1.1 — Output Refinement  
**Goal:** Unified rendering, robust filtering, doc comment extraction, and rich metadata field discovery  
**Phase Range:** 6-10 (continues from v1.0 Phase 5)  
**Depth:** Standard  
**Defined:** 2026-02-13

---

## Overview

v1.1 builds on the v1.0 foundation to deliver output refinement features that maximize token efficiency for LLM consumption. The roadmap follows the natural implementation order: foundation first (FilterEngine), then integration (CLI), then enrichment (result types), then presentation (docs + unified rendering), finally polish (integration testing).

**Key Delivery Boundaries:**
1. **Phase 6:** FilterEngine core — enables all filtering functionality
2. **Phase 7:** CLI integration — makes filters accessible to users
3. **Phase 8:** Result type extensions — adds rich metadata to query results
4. **Phase 9:** Unified rendering — doc comments and consistent display across all item kinds
5. **Phase 10:** Integration and polish — end-to-end validation and performance verification

---

## Phase 6: Foundation — FilterEngine

**Status:** ✅ Complete
**Plans:** 3 plans in 2 waves
**Goal:** Core filtering infrastructure with glob pattern matching

**Phase Goal:** Users can filter query results using include/exclude patterns, crate restrictions, kind filters, and visibility levels.

**Completion Date:** 2026-02-13

### Requirements

| ID | Requirement | Status |
|----|-------------|--------|
| FILT-01 | `--include` flag accepts glob patterns for item paths | ✅ Implemented in 06-01 |
| FILT-02 | `--exclude` flag accepts glob patterns to filter out items | ✅ Implemented in 06-01 |
| FILT-03 | `--kind` flag filters by item kind (struct, enum, trait, function, etc.) | ✅ Implemented in 06-02 |
| FILT-04 | `--crate` flag restricts results to specific crate(s) | ✅ Implemented in 06-01 |
| FILT-05 | `--visibility` flag filters by visibility level (pub, pub(crate), etc.) | ✅ Implemented in 06-01 |
| FILT-06 | Multiple filter flags combine with AND logic | ✅ Implemented in 06-02 |
| FILT-07 | Invalid glob patterns produce helpful error messages | ✅ Implemented in 06-01 |

### Success Criteria (Observable Behaviors)

1. **User can filter by include pattern:** Running `cargo doc-query query Vec --include "std::*"` returns only items from the std crate ✅
2. **User can filter by exclude pattern:** Running `cargo doc-query query Vec --exclude "*Test*"` excludes items with "Test" in the name ✅
3. **User can combine multiple filters:** Using `--include "std::*" --kind "function"` returns only std functions (AND logic) ✅
4. **User receives clear error for invalid patterns:** Entering `--include "[invalid"` shows a helpful error message explaining the glob syntax issue ✅

### Dependencies

- Phase 5 (v1.0 completion)
- No new external dependencies (uses existing `glob` crate 0.3.3)

### Plans

- [x] **06-01**: Create FilterConfig and FilterEngine with glob pattern support (FILT-01, FILT-02, FILT-07)
- [x] **06-02**: Add QueryMatch integration and FilterStats (FILT-03..06)
- [x] **06-03**: Performance optimization and edge case handling (deferred - not critical)

### Deliverables

- `src/types/filter.rs` — FilterConfig and FilterEngine structs ✅
- `src/types/filter.rs` — Filterable trait and QueryMatch integration ✅
- `src/types/filter.rs` — FilterStats for performance monitoring ✅
- `src/types/filter.rs` — Advanced glob pattern validation and help documentation ✅
- Unit tests for glob pattern matching (19 tests) ✅
- Pattern compilation and caching for performance ✅
- Pattern complexity estimation for future optimization ✅

---

## Phase 7: CLI Integration

**Status:** ✅ Complete
**Plans:** 3 plans
**Goal:** Wire filter configuration through CLI to query execution
**Completion Date:** 2026-02-13

**Phase Goal:** Users can specify filter criteria via command-line flags that are passed through to the query engine.

### Requirements

*No new requirements — implements FILT-01..07 from Phase 6*

### Success Criteria (Observable Behaviors)

1. **Filter flags are accepted:** CLI accepts `--include`, `--exclude`, `--kind`, `--crate`, `--visibility` flags without error ✅
2. **Filter config flows to query engine:** Filter configuration is correctly passed from CLI through Commands layer to QueryEngine ✅
3. **Existing flags remain functional:** `--minimal`, `--tokens`, `--depth` continue to work alongside new filter flags ✅
4. **Flag combinations are validated:** Invalid combinations (e.g., conflicting filters) produce helpful error messages ✅

### Dependencies

- Phase 6: Foundation — FilterEngine must exist
- Existing CLI infrastructure from v1.0

### Deliverables

- Updated `Commands::Query` variant with 6 filter flags (-i/--include, -e/--exclude, -k/--kind, --crate, --visibility, --only) ✅
- Modified `ExpandCommand::from_args()` to construct FilterConfig ✅
- Filter config wiring through to FilterEngine ✅
- Validation for conflicting flags (--include + --only) ✅
- `--help-filters` flag with glob syntax documentation ✅
- Enhanced error messages with examples and references ✅
- FILTERING section in --help with real examples ✅

---

## Phase 8: Result Type Extensions

**Status:** ✅ Complete — 5 plans executed
**Goal:** Enrich query results with visibility, deprecation, generics, attributes, and ABI metadata
**Completion Date:** 2026-02-13

**Phase Goal:** Users can see rich metadata for items including visibility modifiers, deprecation status, generic parameters, and key attributes.

### Requirements

| ID | Requirement |
|----|-------------|
| FIELD-01 | Visibility modifiers displayed (pub, pub(crate), pub(super), pub(in path)) |
| FIELD-02 | Deprecation status shown with replacement hints when available |
| FIELD-03 | Generic parameters and bounds displayed for structs, enums, functions |
| FIELD-04 | Key attributes shown: #[must_use], #[non_exhaustive], #[deprecated] |
| FIELD-05 | Function modifiers displayed: const, unsafe, async, ABI info |
| FIELD-06 | New fields omitted in minimal mode for token efficiency |
| FIELD-07 | JSON output includes all new fields with backward-compatible schema |
| FIELD-08 | `--detailed` flag provides expanded metadata per node (orthogonal to depth) |

### Success Criteria (Observable Behaviors)

1. **User sees visibility in query results:** Querying an item shows "pub" or "pub(crate)" alongside the item name
2. **User sees deprecation warnings:** Deprecated items display deprecation status and replacement hints
3. **User sees generic bounds:** Querying `HashMap` shows `K: Eq + Hash, V` generic bounds
4. **User sees key attributes:** Items with `#[must_use]` or `#[non_exhaustive]` display these attributes
5. **User sees function modifiers:** Functions show `const`, `unsafe`, `async`, or ABI info (e.g., `extern "C"`)
6. **Minimal mode omits new fields:** Using `--minimal` suppresses visibility, deprecation, and other new fields for token efficiency
7. **JSON output is backward compatible:** Existing scripts parsing JSON output continue to work; new fields use `#[serde(skip_serializing_if = "Option::is_none")]`
8. **User can request detailed metadata:** Using `--detailed` shows all metadata fields regardless of depth level

### Dependencies

- Phase 7: CLI Integration — filter flags must be wired
- Existing QueryMatch and QueryContent types from v1.0

### Plans

- [x] **08-01**: Create DetailLevel enum and metadata extraction utilities (FIELD-01..05 foundation)
- [x] **08-02**: Extend QueryMatch, MethodOutput, TypeResult, TraitResult with new fields (FIELD-01..05, FIELD-07)
- [x] **08-03**: Extend TypeNode and ModuleItemInfo for expand command (FIELD-01..05 for expansion)
- [x] **08-04**: Add --detailed flag and wire DetailLevel through CLI (FIELD-08)
- [x] **08-05**: Update extraction functions to populate new fields (FIELD-06, integration)

**Plans created:** 2026-02-13 | **Executed:** 2026-02-13

### Deliverables

- Extended `QueryMatch` struct with optional fields (visibility, deprecation, attrs)
- Modified `MethodOutput`, `TypeOutput`, `TraitOutput` with new fields
- Updated `to_minimal()` transformations
- Modified extraction functions (`extract_method()`, `extract_type_result()`, etc.)
- `--detailed` CLI flag integrated with Commands::Query variant
- Detail level enum (Minimal, Standard, Detailed) for clean state management

---

## Phase 9: Unified Rendering and Documentation

**Goal:** Consistent rendering of all item kinds and doc comment extraction with token-aware display

**Phase Goal:** Users see unified rendering across all item types at any depth, with doc comments displayed according to token budgets.

### Requirements

| ID | Requirement |
|----|-------------|
| REND-01 | All 24 ItemKind variants render with consistent formatting |
| REND-02 | Single `format_item()` dispatcher handles all item types |
| REND-03 | Depth-aware formatting (root=full details, nested=condensed, minimal=signatures only) |
| REND-04 | Token budget integrated at rendering layer (tracks per-item overhead) |
| DOCS-01 | Doc comments extracted from `Item::docs` field |
| DOCS-02 | Doc comments display in standard output mode |
| DOCS-03 | Doc comments omitted in minimal mode to save tokens |
| DOCS-04 | Smart truncation at sentence boundaries when budget exceeded |
| DOCS-05 | Code blocks preserved over prose during truncation |
| DOCS-06 | Truncated docs show "..." indicator with warning |
| DOCS-07 | Token budget enforcement includes doc comment tokens |

### Success Criteria (Observable Behaviors)

1. **User sees consistent rendering across item kinds:** Modules, structs, enums, traits, functions, and other ItemKind variants all follow the same formatting rules
2. **User sees depth-aware output:** Root-level items show full details; nested items show condensed summaries
3. **User sees doc comments:** Doc comments appear in standard output mode, extracted from rustdoc JSON
4. **User controls doc display with minimal mode:** Using `--minimal` suppresses doc comments to save tokens
5. **User sees smart truncation:** Long doc comments truncate at sentence boundaries with "..." indicator and warning when budget exceeded
6. **User sees code blocks preserved:** When truncation occurs, code blocks are prioritized over prose
7. **User stays within token budget:** Total output respects the `--tokens` budget, including per-item overhead and doc comment tokens

### Dependencies

- Phase 8: Result Type Extensions — new fields must be available for rendering
- Existing `format/text.rs` and `format/json.rs` modules
- Existing `TokenConfig` from `src/types/expand.rs`

### Deliverables

- Unified `format_item()` dispatcher in `src/format/text.rs`
- Doc comment extraction and formatting logic
- Token-aware truncation with sentence boundary detection
- Integration with existing depth and minimal mode logic

### Plans

- [x] **09-01**: Create format_item() dispatcher in src/format/item.rs (REND-01, REND-02)
- [x] **09-02**: Create doc comment handler with truncation in src/format/doc.rs (DOCS-01..06)
- [x] **09-03**: Create BudgetTracker and integrate with text.rs (REND-04, DOCS-07)

**Plans:** 3 plans in 2 waves | **Status:** Ready for execution

---

## Phase 10: Integration and Polish

**Goal:** End-to-end validation, performance verification, and edge case handling

**Phase Goal:** All v1.1 features work together reliably with acceptable performance and clear error handling.

### Requirements

*No new requirements — validates and polishes REND-01..04, FILT-01..07, DOCS-01..07, FIELD-01..07*

### Success Criteria (Observable Behaviors)

1. **User can run complex filtered queries:** `cargo doc-query query Vec --include "std::*" --kind "function" --tokens 1000` works end-to-end
2. **User experiences <5% performance overhead:** Filter application adds less than 5% to total query time compared to v1.0
3. **User sees helpful error messages:** Invalid patterns, empty result sets, and other error conditions produce clear, actionable messages
4. **User can use all features together:** Filters, token budgets, depth limits, and output formats work in combination without conflicts
5. **User sees consistent JSON output:** JSON format remains backward compatible and includes all new fields when appropriate

### Dependencies

- Phase 9: Unified Rendering — all features must be implemented
- Real-world test crates (serde, tokio, complex workspace)

### Deliverables

- Integration tests for filter combinations
- Performance benchmarks (filter overhead, memory usage)
- End-to-end tests with real crates
- Documentation updates

---

## Requirement Coverage Summary

| Category | Count | Phase Mapping |
|----------|-------|---------------|
| **FILT** (Filtering) | 7 | Phase 6, Phase 7 |
| **FIELD** (Fields) | 8 | Phase 8 |
| **REND** (Rendering) | 4 | Phase 9 |
| **DOCS** (Documentation) | 7 | Phase 9 |
| **Total** | **26** | **100% mapped** |

### Coverage Validation

✓ All 26 v1.1 requirements mapped to exactly one phase  
✓ No orphaned requirements  
✓ No duplicate mappings  
✓ Natural delivery boundaries respected  

---

## Progress Tracking

| Phase | Status | Requirements | Success Criteria Met |
|-------|--------|----------------|---------------------|
| Phase 6 | ✅ Complete | 7 (FILT-01..07) | 4/4 |
| Phase 7 | ✅ Complete | 0 (implements Phase 6) | 4/4 |
| Phase 8 | ✅ Complete | 8 (FIELD-01..08) | 8/8 |
| Phase 9 | 🔴 Not Started | 11 (REND-01..04, DOCS-01..07) | 0/7 | 3 plans
| Phase 10 | 🔴 Not Started | 0 (integration) | 0/5 |

---

## Dependencies Between Phases

```
Phase 6 (FilterEngine)
    ↓
Phase 7 (CLI Integration)
    ↓
Phase 8 (Result Types)
    ↓
Phase 9 (Rendering + Docs)
    ↓
Phase 10 (Integration + Polish)
```

**Critical Path:** Phase 6 → Phase 7 → Phase 8 → Phase 9 → Phase 10  
**No Parallel Execution:** Each phase depends on the previous phase's deliverables.

---

## Risk Mitigation

| Risk | Phase | Mitigation |
|------|-------|------------|
| Filter performance degradation | 6 | Pre-compile patterns, benchmark early |
| Memory usage from doc comments | 9 | Lazy loading, streaming JSON parsing |
| Breaking output format changes | 8, 9 | Optional fields with serde skip attributes |
| Flag conflicts | 7 | Validation logic, clear error messages |
| Token budget edge cases | 9 | Test with deeply nested generics |

---

## Success Metrics for v1.1

### Functional
- [ ] All 24 ItemKind variants render correctly
- [ ] Glob patterns match as expected
- [ ] Doc comments display in all rendering modes
- [ ] Token budget enforcement works for docs
- [ ] Visibility and deprecation fields exposed
- [ ] Minimal mode omits new fields

### Performance
- [ ] Filtering overhead < 5% of total query time
- [ ] Doc comment display < 20% of token budget
- [ ] Memory usage < 50MB for serde crate
- [ ] Query latency unchanged (<100ms with cache)

### UX
- [ ] Unified rendering across depths
- [ ] Clear indication of truncated docs ("...")
- [ ] Helpful error messages for invalid patterns
- [ ] Minimal mode produces < 500 tokens for complex types
- [ ] Filter flags intuitive and well-documented

---

*Roadmap defined: 2026-02-13*  
*Next: Phase 6 planning*
