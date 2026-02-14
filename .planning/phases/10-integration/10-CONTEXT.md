# Phase 10: Integration - Context

**Gathered:** 2026-02-14
**Status:** Ready for planning

<domain>
## Phase Boundary

End-to-end validation of all v1.1 features working together reliably. Tests filter combinations, output formats, error handling, and integration with real crates. No new features — validates existing work from Phases 6-9.

</domain>

<decisions>
## Implementation Decisions

### Test Crates Selection
- Use crates cargo-doc-query depends on directly (serde, clap, glob, petgraph, etc.)
- Include one complex workspace with nested crates and internal dependencies
- Target ~10 crates total for integration testing
- Skip Rust version compatibility research for now (deferred)

### Test Coverage Strategy
- Test both output formats: text (default) and JSON (`--json` flag)
- Test all filter combinations:
  - Filter + depth (e.g., `--include "std::*" --depth 2`)
  - Filter + token budget (e.g., `--kind "struct" --tokens 500`)
  - Filter + minimal mode (e.g., `--exclude "*Test*" --minimal`)
- Use **snapshot testing** (golden files) for output verification
- Test all error paths:
  - Invalid glob patterns
  - Empty result sets (no matches)
  - Conflicting flags (e.g., `--include` + `--only`)
  - Missing or wrong crate names

### Error Handling Validation
- All error scenarios covered in integration tests

### Claude's Discretion
- Exact test file organization and naming conventions
- Snapshot file format and storage location
- Specific assertion patterns for snapshot tests
- How to handle snapshot updates (CLI flag vs manual)
- Test execution order and parallelization

</decisions>

<specifics>
## Specific Ideas

- "test coverage should cover all features and error paths"
- Snapshot testing (golden files) for output verification
- Complex workspace for nested crate testing

</specifics>

<deferred>
## Deferred Ideas

- Rust version compatibility research — requires investigation, not critical for v1.1 ship
- Performance benchmarks — skipped for now

</deferred>

---

*Phase: 10-integration*
*Context gathered: 2026-02-14*
