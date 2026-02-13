# Phase 8: Result Types - Context

**Gathered:** 2026-02-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Enrich query results with rich metadata: visibility modifiers, deprecation status, generic parameters, key attributes, and function modifiers. Adds `--detailed` flag to display expanded metadata per node without affecting recursive depth. Does NOT handle rendering format or doc comment extraction (Phase 9).

</domain>

<decisions>
## Implementation Decisions

### New Flag: --detailed
- `--detailed` flag provides richer metadata per item node
- Orthogonal to `--depth` (recursion control) - can combine any depth with any detail level
- Works alongside existing `--minimal` and `--tokens` flags
- When used with `--minimal`, detailed metadata is still omitted (minimal takes precedence)

### Visibility Display
- Full visibility modifiers as they appear in source: `pub`, `pub(crate)`, `pub(super)`, `pub(in path::to::module)`
- Display inline with item name (not separate field)
- Include full path for `pub(in ...)` visibility (don't abbreviate)

### Deprecation Information
- Capture: `is_deprecated` boolean flag
- Capture: `deprecation_note` text when available
- Skip "since version" (rarely useful for LLM context)
- Display deprecation status prominently (inline or prefix)

### Generic Bounds
- Full trait bounds in standard Rust syntax: `K: Eq + Hash, V`
- Display inline with item name (part of signature)
- Include default values if present: `T = String`
- Show where bounds when present: `where T: Display`

### Attribute Selection
- Focus on semantic attributes that affect API usage:
  - `#[must_use]`
  - `#[non_exhaustive]`
  - `#[deprecated]`
- Skip `#[derive]` (too verbose, rarely needed for API understanding)
- Skip `#[repr]` (implementation detail)
- Skip documentation attributes (handled separately in Phase 9)

### Function Modifiers
- Include: `const`, `unsafe`, `async` as boolean flags
- Include ABI only when explicitly non-Rust: `extern "C"`
- Display inline with signature
- Skip `extern "Rust"` (default, redundant)

### Minimal vs Detailed Mode
Three levels of detail:
- **`--minimal`**: Signatures only, no metadata (no visibility, no generics, no attributes)
- **Default**: Signatures + visibility + generics (balanced)
- **`--detailed`**: Full metadata including attributes, deprecation, function modifiers

### JSON Structure
- Flat structure with optional fields
- Use `#[serde(skip_serializing_if = "Option::is_none")]` for all new optional fields
- No version field needed (additive changes only, backward compatible)
- New fields: `visibility`, `generics`, `deprecation`, `attributes`, `is_const`, `is_unsafe`, `is_async`, `abi`

### Claude's Discretion
- Exact field names in struct definitions
- Error handling for missing rustdoc JSON fields
- Performance optimization for metadata extraction
- Test coverage scope
- Implementation order of the 7 FIELD requirements

</decisions>

<specifics>
## Specific Ideas

- `--detailed` flag inspired by `git log --stat` vs `git log --oneline` - same content, different verbosity
- Deprecation notes should be concise but preserve the "Use X instead" pattern
- Generic bounds should look like valid Rust code that could be copy-pasted
- Visibility formatting should match what you'd write in source code

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. The `--detailed` flag is a natural extension of existing detail control and fits within the "rich metadata" theme of Phase 8.

</deferred>

---

*Phase: 08-result-types*
*Context gathered: 2026-02-13*
