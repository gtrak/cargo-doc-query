# Two-Tier Caching

Deterministic path-based caching for crate documentation artifacts keyed by metadata and build environment. Provides read-or-absent semantics through an index layer with self-healing on corruption.

## Purpose

Describes what caching guarantees the system provides and how lookups behave.

- Deterministic path resolution: identical metadata produces the same cache entry path across invocations.
- Put-get coherence by existence: after a successful write, subsequent reads return a present path for that entry.
- Index roundtrip integrity: a successfully persisted index is recoverable without data loss.
- Corruption self-heal: on deserialization failure, corrupt state is discarded and operations proceed as if no prior state existed; diagnostic output to standard error does not constitute caller-visible error signaling.
- Missing index yields absence, not failure.

## Non-goals

Categorizes areas deliberately out of scope for the caching system.

- Does not verify data fidelity of cached artifacts beyond path existence after write.
- Does not purge the index tier; only the content tier is removable.
- Does not support configurable extension filters for statistics reporting.
- Does not enforce the environment hash constraint at the type level — enforcement relies on convention, not mechanism.

## Invariants

Conditions that must hold for every cache operation.

- **Deterministic resolution**: Given identical crate metadata and environment input, the resolved cache entry path is identical across invocations.
- **Put-get coherence by existence**: After a successful write, subsequent reads return a present path. Does not guarantee data content matches what was written.
- **Index roundtrip integrity**: A successfully persisted index is recoverable on subsequent loads without data loss.
- **Missing index yields absence**: When no index exists on disk, all read operations report the requested entry as absent.
- **Corruption self-heal is silent to the caller**: On deserialization failure, the corrupt state is discarded and operations proceed as if no prior state existed.

## Constraints

Fixed parameters and operational limitations of the caching system.

- **Fixed path layout**: Directory structure and file locations are fixed; the index resides at a single predetermined path.
- **Fixed extension filter for statistics**: Statistics reporting counts entries by a hardcoded `.json` extension only; no configuration is available.
- **Feature label is fixed**: The feature set used in environment computation is not configurable.
- **Per-process rustc information caching**: Environment data resolved via external commands is cached for the lifetime of the process, making it susceptible to staleness.
- **Non-atomic purge**: Content tier removal followed by recreation is not an atomic operation; intermediate state may be observable.
- **Environment hash is structurally mutable**: The constraint that the environment hash is immutable after derivation relies on convention — the underlying data structure exposes fields that could be reassigned.

## Rationale

Design justifications for each invariant and constraint.

- Path existence as the coherence contract avoids requiring content comparison or checksums, keeping reads cheap and independent of artifact size.
- Self-healing on corruption prevents a single bad index state from permanently breaking cache operations at the cost of transient read-absence.
- Stale stderr output during self-heal preserves observability for debugging without coupling error semantics to the caller interface.
- Per-process rustc caching avoids repeated subprocess spawning while accepting the trade-off that mid-run environment changes are not detected.
- Content-only purge reflects the separation between metadata (index) and artifacts (content); the index is expected to be regenerated, not destroyed with the content.

## Related

Concepts and source locations connected to the two-tier caching system.

- [[build-pipeline#Build Pipeline]] — consumes the cache for persisting rustdoc JSON
- [[crate-loading#Crate Loading and Deduplication]] — reads from the cache to load crates
- [[src/cache/global.rs]]
- [[src/cache/store.rs]]
