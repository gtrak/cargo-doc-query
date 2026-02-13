# Phase 9: Unified Rendering - Context

**Gathered:** 2026-02-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Unified rendering of all 24 ItemKind variants and doc comment extraction with token-aware display. Users see consistent formatting at any depth, with doc comments controlled by minimal mode and token budgets.

</domain>

<decisions>
## Implementation Decisions

### Rendering Consistency
- **Unified formatting regardless of position** — Root-level items and nested items follow the same formatting rules
- All 24 ItemKind variants use the same dispatcher (`format_item()`)
- Depth affects detail level, but not format consistency

### Doc Comments
- Extracted from rustdoc JSON `Item::docs` field
- Displayed in standard mode, omitted in minimal mode
- Smart truncation at sentence boundaries when budget exceeded
- Code blocks preserved over prose during truncation

### Token Budget
- Integrated at rendering layer
- Tracks per-item overhead including doc comments
- Truncation triggers with "..." indicator

### Claude's Discretion
- Exact formatting for each ItemKind variant
- Specific truncation algorithm details
- Sentence boundary detection approach
- Per-item overhead calculation method

</decisions>

<specifics>
## Specific Ideas

- "Consistency wherever the node lands" — core principle: same rendering rules apply regardless of depth/position

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 09-unified-rendering*
*Context gathered: 2026-02-13*
