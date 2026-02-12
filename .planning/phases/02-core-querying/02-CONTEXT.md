# Phase 2: Core Querying - Context

**Gathered:** 2026-02-12
**Status:** Ready for planning

---

## Phase Boundary

Users can query methods, traits, and type information for any Rust type, receiving structured JSON output. The focus is on the core query commands — extracting API signatures, trait definitions, and type information from the indexed documentation. Not in scope: caching (Phase 3), recursive expansion (Phase 4), or interactive modes (Phase 5).

---

## Implementation Decisions

### Query command structure

- Unified interface: `cargo doc-query query <path>` (single command, not separate subcommands)
- Path syntax: Follows Rust module naming conventions (`crate::module::Type`)
- Searching with crate: Optional `--crate <crate-name>` flag limits search to specific crate

### Query semantics

- **Automatic inference:** Command infers whether the path refers to a type or trait, and what to return by default
- **Type queries:** When querying a type (e.g., `std::vec::Vec`), returns methods and trait implementations for that type
- **Trait queries:** When querying a trait (e.g., `std::iter::Iterator`), returns the trait definition only (methods, associated types) — NOT implementing types
- **Query scoping:** Optional `--kind` flag overrides default: `--kind methods|traits|types|all` (default is 'all')

### Output detail level

- **Default:** Public method signatures only (no private items)
- **Generic parameters:** Show generic declaration as written (e.g., `fn push<T>(&mut self, item: T)`)
- **Resolved generics:** Optional `--include=trait_parameterization` shows fully resolved type parameters (e.g., `fn push(&mut self, item: u8)`) — **Note:** This flag may produce significantly larger output volumes and may need additional filtering options
- **Documentation:** Optional `--include=docs` flag includes doc comments in output
- **Private APIs:** Optional `--include=private` flag includes non-public items (private, pub(crate), etc.)

### Type matching behavior

- **Multiple matches:** Show all matches, do not prompt user for selection
- **Disambiguation context:** Each match includes:
  - Crate name and version
  - Fully qualified path
  - Kind (struct/enum/trait/fn/etc.)

### Error handling strategy

- **Fail fast:** Query fails immediately on non-existent paths, ambiguous matches, or items not in index
- **No fuzzy matching:** No "did you mean?" suggestions or spelling corrections
- **Assumption:** LLM users know exact paths or will navigate via iterative queries

### JSON output format

- All query responses are valid, parseable JSON
- Command output can be piped to other tools (`| jq` handled by standard shell integration)
- Structure optimized for programmatic and LLM consumption

### Claude's Discretion

- Exact JSON schema and field naming conventions for output
- How `--include=trait_parameterization` handles complex type resolution
- Performance optimizations for large queries (within single-query scope)
- Whether trait parameterization needs additional filtering flags if output is too large

---

## Specific Ideas

- "Fail fast on errors — the LLM should know what it's looking for or can navigate there from successive calls"
- `crate::module::Type` path syntax should match Rust's module naming conventions exactly
- Trait query returning definition only prevents overwhelming output for widely-implemented traits

---

## Deferred Ideas

- fzf integration for interactive query mode → Phase 5: Integration & Polish
- Additional filtering options for `--include=trait_parameterization` if needed (to be determined based on Phase 2 performance/testing)

---

*Phase: 02-core-querying*
*Context gathered: 2026-02-12*
