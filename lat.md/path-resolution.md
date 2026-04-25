# Path Resolution

Resolves items against a crate index by qualified path string or stable identifier, supporting single-crate and cross-crate queries.

## Purpose

Describes how items are resolved from paths and identifiers against the crate index.

- Return all indexed items whose qualified path matches a given path string within one crate or across multiple crates.
- Resolve an item by its stable identifier within a single crate or a specific named crate.
- Allow optional crate scoping so a query targets exactly one crate by name rather than the full set.

## Non-goals

Categorizes areas deliberately out of scope for path resolution.

- Distinguishing why an identifier lookup failed (missing crate vs. missing item).
- Validating that a path string is syntactically well-formed before attempting resolution.
- Tracking provenance of matched items beyond the crate they reside in.
- Case-insensitive or fuzzy matching of any kind — paths and crate names match exactly as provided.

## Invariants

Conditions that must hold for every resolution operation.

- Every returned item exists in the crate index at the time of lookup.
- Path matching is inclusive: an item matches if its full path equals the query or the query constitutes a terminal suffix component. Partial component overlap never matches.
- Crate name filtering is strict equality — no prefix, suffix, or case-folding logic applies.
- Identifier lookup yields nothing when either the crate or the identifier is absent; the two failure modes are indistinguishable to the caller.
- Resolution carries no mutable state between invocations.

## Constraints

Fixed parameters and operational limitations of the resolution process.

- Suffix matching inherently crosses module boundaries: querying a short name matches any path ending with that name as its final component regardless of which crate or module it belongs to.
- Queries that are empty or whitespace-only produce no matches.
- Cross-crate resolution depends on exact crate name alignment; if a user-supplied name differs from the index key, no results are returned without explanation.
- Malformed or stale identifiers produce the same outcome as never-existing ones — silent absence.
- The resolver cannot reject malformed queries before evaluation because no validation step exists.

## Rationale

Design justifications for each invariant and constraint.

- Inclusive path matching (exact or terminal-suffix) reflects how developers search: they typically know a partial name and want every candidate, not just top-level items.
- No error distinction on identifier failure keeps the interface minimal; callers already hold the identifier and can infer absence from context.
- Crate name equality avoids ambiguity in multi-crate environments where case-folding or prefix logic could silently target the wrong crate.
- Stateless resolution ensures idempotent, composable queries — no session concept or cache invalidation to reason about.

## Related

Concepts and source locations connected to path resolution.

- [[query-engine#Query Engine]] — uses path resolution to find items across crates
- [[type-expansion#Type Expansion]] — uses path resolution to locate target types before expansion
- [[src/query/lookup.rs]]
