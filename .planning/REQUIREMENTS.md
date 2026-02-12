# Requirements: cargo-doc-query

**Defined:** 2026-02-12
**Core Value:** Sub-100ms deterministic structured API extraction that reduces LLM context usage

## v1 Requirements

### Core Querying

- [ ] **QUERY-01**: User can query methods for any type by fully qualified path (e.g., `std::vec::Vec`)
- [ ] **QUERY-02**: Method output includes name, signature, and fully qualified return types
- [ ] **QUERY-03**: Query responses are formatted as structured JSON
- [ ] **QUERY-04**: User can limit output with token budget constraints
- [ ] **QUERY-05**: User can request minimal/signature-only mode to reduce context usage
- [ ] **QUERY-06**: Recursive type expansion supports depth limits (e.g., `--depth 2`)

### Build & Index

- [ ] **BUILD-01**: User can run `cargo doc-query build` to generate documentation index
- [ ] **BUILD-02**: Build command generates rustdoc JSON for all dependencies
- [ ] **BUILD-03**: Index is cached to disk for sub-100ms query performance
- [ ] **BUILD-04**: Index automatically rebuilds when Cargo.lock changes
- [ ] **BUILD-05**: Build handles format version checking for rustdoc JSON compatibility

### Traits & Types

- [ ] **TRAIT-01**: User can query trait implementations for any type
- [ ] **TRAIT-02**: Trait output includes associated types and methods
- [ ] **TYPE-01**: User can expand a type recursively to see its full graph
- [ ] **TYPE-02**: Expansion respects depth limits to prevent context overflow

### Output Formats

- [ ] **FMT-01**: JSON output format for programmatic/LLM consumption
- [ ] **FMT-02**: Minimal output format for token efficiency
- [ ] **FMT-03**: Support for piping and shell integration

## v2 Requirements

### Performance

- **PERF-01**: Incremental per-crate rebuilds (only rebuild changed crates)
- **PERF-02**: Content-addressable cache storage with hash-based keys
- **PERF-03**: Parallel processing for large workspaces
- **PERF-04**: Lazy loading for memory-efficient handling of large crates

### Advanced Querying

- **ADV-01**: Fuzzy search for type names
- **ADV-02**: Full-text search across documentation
- **ADV-03**: Cross-crate trait resolution
- **ADV-04**: Workspace-wide queries across all members

### Integration

- **INT-01**: Export to llms.txt format
- **INT-02**: Prebuilt index registry for common crates
- **INT-03**: MCP (Model Context Protocol) integration

## Out of Scope

| Feature | Reason |
|---------|--------|
| Type checking | cargo check handles this; out of scope for documentation tool |
| IDE features | No go-to-definition, refactoring, or real-time analysis |
| Runtime code execution | Tool is for static analysis only |
| General semantic search | Focused on structured API queries, not fuzzy code search |
| Real-time collaboration | Stateless CLI tool, no multi-user features |
| GUI interface | CLI-only by design |
| LSP protocol support | Avoids daemon complexity; different architecture |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| QUERY-01 | Phase 2 | Pending |
| QUERY-02 | Phase 2 | Pending |
| QUERY-03 | Phase 2 | Pending |
| QUERY-04 | Phase 4 | Pending |
| QUERY-05 | Phase 4 | Pending |
| QUERY-06 | Phase 4 | Pending |
| BUILD-01 | Phase 1 | Pending |
| BUILD-02 | Phase 1 | Pending |
| BUILD-03 | Phase 3 | Pending |
| BUILD-04 | Phase 3 | Pending |
| BUILD-05 | Phase 1 | Pending |
| TRAIT-01 | Phase 2 | Pending |
| TRAIT-02 | Phase 2 | Pending |
| TYPE-01 | Phase 4 | Pending |
| TYPE-02 | Phase 4 | Pending |
| FMT-01 | Phase 2 | Pending |
| FMT-02 | Phase 4 | Pending |
| FMT-03 | Phase 2 | Pending |

**Coverage:**
- v1 requirements: 18 total
- Mapped to phases: 18
- Unmapped: 0 ✓

### Phase Summary

| Phase | Requirement Count | Requirements |
|-------|-------------------|--------------|
| Phase 1 - Foundation | 3 | BUILD-01, BUILD-02, BUILD-05 |
| Phase 2 - Core Querying | 7 | QUERY-01, QUERY-02, QUERY-03, TRAIT-01, TRAIT-02, FMT-01, FMT-03 |
| Phase 3 - Performance | 2 | BUILD-03, BUILD-04 |
| Phase 4 - Advanced Features | 6 | QUERY-04, QUERY-05, QUERY-06, TYPE-01, TYPE-02, FMT-02 |
| Phase 5 - Integration & Polish | 0 | (polish requirements) |

---
*Requirements defined: 2026-02-12*
*Last updated: 2026-02-12 after initial definition*
