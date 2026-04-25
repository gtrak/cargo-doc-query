# Crate Loading and Deduplication

Loads rustdoc JSON for a requested crate version from global cache into memory, suppressing duplicate loads.

## Purpose

Provides a deterministic path to obtain a parsed rustdoc crate in memory. The observable contract is binary: present or absent.

## Non-goals

Things the loader intentionally does not do.

- Schema-level validation before consumption. Structural validity is enforced at deserialization time; there is no separate pre-validation step.
- Tracking whether a load was fresh or deduplicated. Both outcomes yield true.
- Enforcing stack depth, concurrency policy, or memory-lifetime guarantees on the caller.

## Invariants

Conditions that must hold for every load attempt.

1. **Binary outcome** — Every well-formed request yields exactly one of two observable results: present (true) or absent (false). No tertiary outcomes exist.
2. **Idempotent presence** — Re-requesting an already-present crate returns true without re-loading from disk.
3. **Index gate** — A crate must exist in the local index with a matching name and version before any cache lookup occurs. Missing index entries short-circuit to error, not false.
4. **Cache-path determinism** — The on-disk location for a given (name, version) pair is fully determined by crate identity plus current build environment: rustc version, target triple, feature set. Same inputs resolve to the same path.

## Constraints

Limits on what the loader may do during execution.

- **Environment binding** — The cache path encodes build-environment metadata. A change in rustc version, target triple, or feature set produces a distinct on-disk location even for identical name-version pairs. The in-memory key does not encode this distinction.
- **Index membership** — Only crates listed in the local index are eligible for loading. Requesting an unindexed crate is an error, not a soft miss.
- **Graceful absence** — If a valid index entry has no corresponding on-disk artifact, the result is false, not an error. The caller decides whether to retry or skip.

## Rationale

Design justifications for each invariant and constraint.

Binary outcome collapses two internally distinct paths (fresh load vs. dedup hit) into one observer-facing value. This avoids callers depending on which internal path executed.

Separating index membership from cache presence gives the caller a clear error boundary: "not part of my build graph" versus "artifact not cached yet." Confusing the two would force the caller to guess whether absence means misconfiguration or transient cache miss.

Encoding environment metadata in the cache path but not the in-memory key reflects an operating assumption: within a single indexing session, the build environment does not change. If it did, the in-memory map and on-disk store would diverge — a known limitation rather than an invariant to enforce at load time.

## Related

Concepts this spec references.

- [[two-tier-caching#Two-Tier Caching]] — On-disk storage keyed by crate identity plus environment hash
- [[src/cli/build.rs]] — Builds the local index that gates loading eligibility
- [[src/cache/global.rs#CrateCacheKey]]
- [[src/query/loader.rs#CrateLoader]]

</content>, 