# Type Suggestion Engine

Provides fuzzy-text suggestions from a known identifier set given a user query, producing an ordered list of candidate matches ranked by relevance.

## Purpose

Defines how fuzzy suggestions are computed from a known identifier set.

- Produce an ordered list of candidate crate-name matches ranked by similarity score.
- Tolerate case variation — letter casing never affects ranking or inclusion.
- Reward cases where the query begins with a crate name above general similarity scoring.
- Exclude candidates below a quality floor.
- Eliminate duplicates and limit results to a configurable maximum.

## Non-goals

Categorizes areas deliberately out of scope for the suggestion engine.

- Not a general-purpose search engine; only single-token identifier matching is in scope.
- Not a version-aware lookup; semantic properties of identifiers beyond their name are excluded.
- Not an incremental or streaming interface; the query is resolved as a complete unit before results are returned.

## Invariants

Conditions that must hold for every suggestion computation.

- Every similarity score lies in the closed interval [0.0, 1.0]; no scoring branch may violate these bounds under any input pair.
- Letter casing of query or candidate never affects scores or ranking — both are treated identically regardless of case.
- The output list contains no duplicate identifiers; when a crate qualifies through multiple paths, only one instance survives and it carries the highest score attained.
- The output is deterministically ordered descending by score; for equal scores, relative order is stable across runs on the same index state.

## Constraints

Fixed parameters and operational limitations of the suggestion engine.

- Only identifiers scoring strictly above 0.3 qualify; a score of exactly 0.3 or below is excluded without exception.
- When the query string starts with a crate name, that crate receives elevated priority above general similarity scoring.
- The result list is truncated to a configurable maximum length after deduplication, never before.
- Empty query or empty identifier yields no qualifying score and is excluded by the threshold.

## Rationale

Design justifications for each invariant and constraint.

- The > 0.3 threshold eliminates noise while allowing partial matches through; 0.3 was chosen as the minimum recognizable similarity floor.
- Prefix bonus rewards intent-signalling (user typed the beginning of an identifier) without outranking exact or containment matches that may already exist.
- Case insensitivity reflects user behavior: crate names are conventionally case-stable but queries are not.
- Deduplication after scoring preserves the highest-evidence instance of any identifier rather than arbitrarily discarding early candidates.
- Deterministic ordering on equal scores follows from the fixed alphabetical iteration order over the index — introducing a secondary sort key would add ordering assumptions not grounded in user intent.

## Related

Concepts and source locations connected to the type suggestion engine.

- [[error-handling#Error Handling with Exit Codes]] — triggers suggestion lookup on path-not-found errors
- [[src/query/suggest.rs]]
