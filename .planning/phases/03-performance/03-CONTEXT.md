# Phase 3: Performance - Context

**Gathered:** 2026-02-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Optimize query performance to achieve sub-100ms responses through efficient index caching and automatic rebuilds when dependencies change. No new query capabilities - just making existing queries fast.

</domain>

<decisions>
## Implementation Decisions

### Cache Strategy
- **Cache what:** The graph index itself (not query results)
- **Rationale:** Querying the index should be fast enough on its own; no separate query result cache needed
- **Storage location:** `target/doc-query/` (existing pattern from Phase 1)
- **Format:** Binary serialization via postcard (existing pattern)

### Cache Invalidation
- **Trigger:** Rebuild index when either `Cargo.toml` or `Cargo.lock` changes
- **Detection:** Hash comparison of both files (not just Cargo.lock)
- **Behavior:** Automatic rebuild on next query command (transparent to user)
- **No manual cache management:** No explicit "clean" or "refresh" commands needed

### Claude's Discretion
- Exact hash algorithm (BLAKE3 already established, continue using)
- Cache file organization and naming
- Whether to show rebuild progress to user
- Error handling when rebuild fails mid-query

</decisions>

<specifics>
## Specific Ideas

- Keep it simple: one cache (the index), invalidate on manifest changes
- Query performance should come from fast graph traversal, not result caching
- Transparent to users - they shouldn't need to think about the cache

</specifics>

<deferred>
## Deferred Ideas

- Query result caching (explicitly out of scope per user decision)
- Shared cache across projects (v1.1 enhancement noted in ROADMAP.md)
- Manual cache management commands
- Cache size limits or garbage collection

</deferred>

---

*Phase: 03-performance*
*Context gathered: 2026-02-12*
