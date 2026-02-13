# Phase 7: CLI Integration - Context

**Gathered:** 2026-02-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire filter configuration through CLI to query execution. Users specify filter criteria via command-line flags that are passed through to the query engine. This phase delivers the CLI interface for Phase 6's FilterEngine — making filtering accessible to users.

</domain>

<decisions>
## Implementation Decisions

### Flag naming and structure
- Short flags: `-i` (include), `-e` (exclude), `-k` (kind)
- Long flags only for less common: `--crate`, `--visibility`
- Reserve `-c` for future `--config` flag

### Multiple filter handling
- Multiple instances of same flag = OR logic within flag type
- Example: `--include "std::*" --include "alloc::*"` matches items in EITHER std OR alloc
- Different flag types = AND logic across types
- Example: `--include "std::*" --exclude "*test*"` matches std items AND NOT test items

### Conflicting flag behavior
- Fail fast on contradictions with clear error messages
- Empty result sets are OK (user's filters just match nothing)
- Overlapping include/exclude is OK (exclude wins as it's applied after include)
- Contradiction example: `--include "std::*" --exclude "std::*"` → error: "Filter patterns have no overlap — review your patterns"

### Error messages and help
- Add FILTERING section to `--help` with 3-4 real examples
- Add `--help-filters` flag for detailed glob syntax guide
- Invalid patterns show: (1) what was wrong, (2) a valid example, (3) glob syntax reference

### Flag value formats
- Patterns: shell handles quotes, receive raw strings
- `--kind`: case-insensitive (`function`, `Function`, `fn` all work)
- Match rustdoc ItemKind variants internally
- `--visibility`: accept `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`, `private` (for non-pub)

### Filter application order
- Filters branch off defaults (applied ON TOP OF base query results)
- Query runs first, filters refine the results
- This preserves the existing query behavior while adding filtering

### The `--only` flag
- Shorthand for "include this pattern and exclude everything else"
- `--only "std::*"` is equivalent to `--include "std::*"` + exclude everything not matching
- Mutually exclusive with `--include` (error if both provided)
- Useful for: "show me ONLY std items" without having to specify complex exclude patterns

### Claude's Discretion
- Exact error message wording
- Validation implementation details
- Help text formatting and examples
- Case normalization for `--kind` values

</decisions>

<specifics>
## Specific Ideas

- Error message style: "Filter patterns have no overlap — review your patterns" (clear, actionable)
- Help examples should show real patterns: `--include "std::vec::*"`, `--exclude "*test*"`, `--kind function`
- `--only` is the 80% use case — most users want to focus on one crate/module

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-cli-integration*
*Context gathered: 2026-02-13*
