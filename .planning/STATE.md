# Project State: cargo-doc-query

**Milestone:** v1.1 Output Refinement — In progress
**Current Phase:** 9 (5 phases planned)
**Status:** Requirements defined, roadmap created
**Last Updated:** 2026-02-13
**Roadmap:** `.planning/ROADMAP-v1.1.md`

---

## Current Position

**Phase:** 9 — Unified Rendering
**Plan:** 3 of N in current phase
**Status:** In progress
**Last activity:** 2026-02-13 — Completed 09-03-SUMMARY.md

Progress: ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 10% (Phase 9) | 31/33 (94%)

---

## Milestone Summary

### v1.0 MVP (Shipped 2026-02-13)

| Phase | Status | Plans | Key Deliverable |
|-------|--------|-------|-----------------|
| 1. Foundation | ✅ Complete | 4 | Build command, JSON generation, caching |
| 2. Core Querying | ✅ Complete | 6 | Query engine, methods, traits, JSON output |
| 3. Performance | ✅ Complete | 1 | Sub-100ms queries, auto-rebuild |
| 4. Advanced Features | ✅ Complete | 3 | Recursive expansion, token budgets |
| 5. Integration & Polish | ✅ Complete | 5 | Error handling, progress bars, suggestions |

**Stats:**
- 21 plans completed
- 212 tests passing
- ~7,250 lines of Rust code
- 2 days from start to ship

### v1.1 Output Refinement (In progress — 5 phases)

**Focus:** Unified rendering, robust filtering, documentation support

| Phase | Status | Requirements | Key Deliverable |
|-------|--------|--------------|-----------------|
| 6. Foundation | ✅ Complete | 2/2 | FilterEngine with glob matching, validation, stats |
| 7. CLI Integration | ✅ Complete | 3/3 | FilterEngine wired to CLI with validation and help |
| 8. Result Types | ○ Pending | 7 | Rich metadata (visibility, generics) |
| 9. Unified Rendering | ○ Pending | 11 | Doc comments + consistent display |
| 10. Integration | ○ Pending | 0 | End-to-end validation |

**Requirements:** 25 total | **Coverage:** 100% mapped ✓

---

## Project Reference

### Core Value

Sub-100ms deterministic structured API extraction that reduces LLM context usage compared to raw source or LSP, without requiring a long-running daemon.

### Current Focus

v1.1: Output refinement and UX improvements
- Unified kind rendering at all tree depths
- Robust `--include`/`--exclude` filtering
- Doc comment extraction and display
- Discovery of additional display fields

---

## Session Continuity

**Last session:** 2026-02-13 21:57 UTC
**Stopped at:** Completed 09-03-SUMMARY.md
**Resume file:** .planning/phases/09-unified-rendering/09-03-SUMMARY.md

### Recent Context

Completed plan 09-03 for token budget integration:
- Created src/format/budget.rs with BudgetTracker struct
- track_item() for per-item token accounting
- would_exceed(), remaining(), is_warning_needed() methods
- TruncationAction enum: Include/Truncate
- estimate_item_tokens() for formatted item cost estimation
- 8 comprehensive tests for all functionality
- Integrates token budget at rendering layer per REND-04
- Updated format/mod.rs to export budget module
- Integrated ItemFormatter and BudgetTracker into format/text.rs
- Added format_with_item_formatter() and format_expand_result_with_formatter()
- Displays truncation warning when budget threshold reached
- All 308 library tests pass

Completed plan 09-02 for doc comment handler:
- Created src/format/doc.rs with DocHandler struct
- extract_docs() gets docs from Item::docs field
- format_docs() respects DetailLevel (Minimal omits docs)
- truncate_docs() handles sentence boundaries, code block preservation
- 19 tests passing
- All DOCS-01 through DOCS-06 requirements implemented

Completed plan 09-01 for unified rendering dispatcher:
- Created src/format/item.rs with ItemFormatter and FormattedItem structs
- format_item() dispatcher handles all ItemKind variants with DetailLevel control
- Supports Minimal/Standard/Detailed modes for visibility, generics, docs, attributes
- Added 3 tests verifying struct formatting, detail level behavior, and token budget
- All 281 tests pass

Completed plan 08-04 for --detailed CLI flag:
- Added --detailed/-d flag to CLI Args struct (args.rs)
- Integrated DetailLevel into Commands enum (commands.rs)
- Fixed bug: main.rs Commands enum was missing --detailed flag
- Wired DetailLevel through to ExpandCommand using from_args_with_detail()
- Added warning when both --minimal and --detailed specified
- --detailed now shows in CLI help output
- All 302 tests passing

Completed plan 08-03 for expand types metadata extensions:
- Extended TypeNode with is_deprecated, deprecation_note, attributes fields (FIELD-02, FIELD-04)
- Extended TypeNode with function modifier fields (is_const, is_async, is_unsafe, abi) (FIELD-05)
- Extended ModuleItemInfo with visibility, generics, function modifier fields (FIELD-01, FIELD-03, FIELD-05)
- Added builder methods: with_deprecation, with_attributes, with_visibility, with_generics, with_function_modifiers
- Updated to_minimal() to clear all new fields and generic_params in TypeNode
- Added 8 comprehensive JSON backward compatibility tests
- All 276 library tests passing (up from 268)

Completed plan 08-02 for Result Type Extensions:
- Extended MethodOutput with is_const, is_async, is_unsafe, abi optional fields (FIELD-05)
- Extended TypeResult with generic_params optional field (FIELD-03)
- Extended TraitResult with generic_params optional field (FIELD-03)
- Added builder methods for all new fields (with_is_const, with_is_async, with_is_unsafe, with_abi, with_generic_params)
- Updated to_minimal() to clear all new optional fields (FIELD-06)
- Added 12 comprehensive JSON backward compatibility tests (FIELD-07)
- All 268 library tests passing

Completed plan 08-01 for Result Types foundation:
- Created DetailLevel enum with Minimal, Standard, Detailed variants
- Implemented from_flags() for CLI --detailed/--minimal flag handling
- Added extraction helpers: visibility_to_string(), format_generics(), extract_deprecation_info(), extract_semantic_attrs(), extract_function_modifiers()
- Synthetic generics correctly filtered (is_synthetic: true)
- Non-Rust ABIs identified correctly (C, stdcall, cdecl, etc.)
- 42 comprehensive tests, all passing

Completed plan 06-01 for FilterEngine foundation:
- Created FilterConfig struct with all filter fields
- Implemented FilterEngine with pre-compiled pattern matching
- Added comprehensive test suite (11 tests, all passing)
- Successfully supports FILT-01 (--include), FILT-02 (--exclude), and FILT-07 (error messages)

Completed plan 06-02 for FilterEngine enhancement:
- Added advanced glob pattern validation and help documentation
- Implemented Filterable trait for extensibility
- Added FilterStats struct with pass/rejection tracking
- Integrated QueryMatch filtering with statistics collection
- All filter tests passing (19 tests)

Completed plan 07-01 for CLI integration:
- Added filter flags to Commands::Query variant
- Integrated FilterConfig into ExpandCommand struct
- Created filter_config() method with --only precedence logic
- All CLI filter flags ready for use in query execution

Completed plan 07-02 for expand command filter integration:
- Added crate_name and visibility fields to TypeNode
- Implemented Filterable trait for TypeNode
- Integrated FilterEngine into ExpandCommand::execute()
- Applied filters to expansion results with statistics display
- Verified zero overhead when no filters configured
- All tests passing (212 tests)

Completed plan 07-03 for filter validation and help text:
- Added comprehensive validation for conflicting filter flags
- Implemented --include + --only mutual exclusivity detection
- Added visibility validation for pub, pub(crate), pub(super), pub(in path), private
- Enhanced FILTERING section in --help with 5 examples
- Implemented --help-filters flag (works without PATH argument)
- Added comprehensive glob syntax documentation with examples
- Enhanced error messages for invalid patterns (4-part format)
- Case-insensitive kind matching confirmed working
- All validation scenarios tested and passing

v1.1 start with focus shift:
- Deferred infrastructure goals (shared cache, stdlib, GC) to v2.0
- Prioritizing rendering consistency and output refinement
- Researching rustdoc JSON schema for missing fields

### Accumulated Context

**Decisions:**
- Phase 08-01: DetailLevel enum with Minimal/Standard/Detailed variants
- Phase 08-01: Synthetic generics filtered out (is_synthetic: true)
- Phase 08-01: Non-Rust ABIs return Some(abi_name), Rust returns None
- Phase 08-01: Deprecation "since" field skipped per requirements
- Phase 08-01: Attributes filtered to must_use and non_exhaustive only
- Phase 08-02: MethodOutput.visibility kept as String (required), only modifiers are optional
- Phase 08-02: All new fields use Option<T> with skip_serializing_if for clean JSON
- Phase 08-02: Old struct definitions in tests verify backward compatibility
- Phase 08-03: TypeNode had existing visibility/generic_params, preserved and added new Option fields
- Phase 08-03: to_minimal() clears generic_params Vec (minimal omits generics)
- Phase 08-03: ModuleItemInfo uses Option<T> for all new fields for backward compatibility
- Phase 08-04: --minimal takes precedence over --detailed when both specified
- Phase 08-04: DetailLevel wired through CLI -> Commands -> ExpandCommand execution
- Phase 08-05: DetailLevel passed through QueryOptions to extraction functions
- Phase 08-05: TypeExpander stores DetailLevel for use during recursive expansion
- Phase 08-05: Metadata extraction respects Minimal/Standard/Detailed modes
- Phase 09-01: Single format_item() handles all ItemKind variants
- Phase 09-01: DetailLevel controls visibility/generics/docs/attributes at render time
- Phase 09-01: Uses { .. } pattern for struct variants (Constant, AssocConst, AssocType)
- Phase 09-02: DocHandler returns None for docs in Minimal mode (DOCS-03)
- Phase 09-02: truncate_docs returns (String, bool) to indicate truncation
- Phase 09-02: Code blocks (```) preserved over prose during truncation
- Phase 09-03: BudgetTracker integrates at rendering layer per REND-04
- Phase 09-03: TruncationAction enum: Include/Truncate for budget decisions
- Phase 09-03: Warning threshold default 0.8 (80%)
- Phase 09-03: estimate_item_tokens() uses rough calculation: 20 base + 5/field + 5/variant + 10/nested + docs/4
- v1.1 focus: Output quality over infrastructure
- v2.0 will address shared cache, stdlib, and GC
- FilterEngine uses glob@0.3.3 for pattern matching
- Pre-compile patterns for performance
- AND logic combining all filters
- CLI filter flags use Vec<String> for multiple values
- --only takes precedence over --include for include patterns
- --kind values are normalized to lowercase (case-insensitive matching)
- crate_filter field name avoids Rust keyword conflict
- PATH argument made optional to support --help-filters
- Error messages follow 4-part format: what, why, example, reference
- pub(in path) visibility format supported alongside standard options

**Blockers:** None

---

## Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Query latency (cached) | 7ms (verified) | <100ms |
| Build time (small project) | <5s | <5s |
| Requirements implemented | 18/18 v1.0 | v1.1 TBD |
| Milestones complete | 1/1 | 1 in progress |
| Plans completed | 25/26 (96%) | 26 total |
| Tests passing | 278 | - |
| Tests passing | 276 | - |

---

*This file is updated after each session. Check `git log .planning/STATE.md` for history.*
