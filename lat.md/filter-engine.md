# Filter Engine

Provides deterministic inclusion/exclusion decisions over query results based on path patterns, item kind, crate identity, and visibility level. The filter acts as a gate: every dimension must agree for an item to pass.

## Purpose

Defines how filtering decisions are made over query results across multiple dimensions.

- Evaluate items against path patterns using glob semantics, item kind, crate identity, and visibility level.
- AND semantics across all dimensions — any single dimension rejecting an item causes rejection.
- Exclude patterns always dominate include patterns; an item matching both is rejected.
- Validate patterns before evaluation begins; empty or syntactically invalid patterns are rejected at configuration time.
- Provide batch filtering with statistics and diagnostic data.

## Non-goals

Categorizes areas deliberately out of scope for the filter engine.

- Not a query builder — filtering operates only on already-matched items.
- Not a pattern language designer — glob semantics are inherited from the underlying library.
- Not a deduplication layer — duplicate patterns are accepted and evaluated redundantly.
- Not a strict validator — overly broad include patterns compile successfully; warnings are advisory.
- Not visibility-aware at configuration time — visibility defaults for sparse metadata are inferred, not validated.

## Invariants

Conditions that must hold for every filter evaluation.

- An empty configuration admits every item unconditionally.
- All filter dimensions evaluate with AND semantics; any single dimension rejecting an item causes rejection.
- Path pattern matching uses glob semantics, not regular expression semantics.
- Exclude patterns dominate include patterns — an item matching both is rejected, regardless of evaluation order.
- Item kind comparison is case-insensitive; crate identity and visibility level are case-sensitive.

## Constraints

Fixed parameters and operational limitations of the filter engine.

- Patterns that are empty or syntactically invalid are rejected at configuration time, not during item evaluation.
- Visibility information may be incomplete: when method-level visibility metadata is absent, the item's effective visibility defaults to `pub`.
- The same configuration cannot simultaneously include and exclude an item — exclude always wins.
- Duplicate patterns persist independently; no deduplication or merging occurs at configuration time.
- Warning signals for broad include patterns do not block compilation.

## Rationale

Design justifications for each invariant and constraint.

- AND semantics across dimensions prevent accidental over-inclusion: a filter is only permissive when every dimension agrees.
- Exclude domination ensures that explicit exclusions always override blanket includes, matching user intent in real-world usage.
- Rejecting invalid patterns before evaluation prevents silent mismatches where a malformed pattern fails to match anything without notice.
- The `pub` visibility default for sparse metadata preserves pass-through behavior when the underlying data source does not carry granular visibility information.

## Related

Concepts and source locations connected to the filter engine.

- [[query-engine#Query Engine]] — produces query results that are subject to filtering
- [[src/types/filter.rs]]
