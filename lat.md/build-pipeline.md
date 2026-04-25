# Build Pipeline

Produces a queryable index of all direct runtime dependencies for a Rust crate by building rustdoc JSON and persisting it to the global cache.

## Purpose

Describes what the build pipeline produces and the guarantees it makes about indexing coverage.

- Only direct (non-transitive) dependencies are discovered and indexed.
- Pre-cached crates are skipped; only missing entries trigger builds.
- Failed individual builds do not prevent the rest from succeeding.
- The index includes all discovered dependencies regardless of build outcomes, so that unbuildable crates still appear as nodes without queryable content.

## Non-goals

Categorizes areas the build pipeline deliberately does not address, setting expectations for indexing scope.

- Transitive dependencies are not discovered or indexed.
- Dev-dependencies are excluded only on the primary discovery path; the Cargo.lock fallback may include them.
- Local path dependencies are never included, regardless of discovery path.
- The pipeline does not guarantee every dependency will have queryable documentation — build failures leave entries in the index without content.
- Workspace members are never indexed as dependencies.

## Invariants

Conditions that must hold for every pipeline execution, ensuring consistent index construction.

- Every discovered dependency appears as a node in the saved index, even if its rustdoc build failed or was skipped due to caching.
- Cache keys are always derived from metadata versions, not from versions extracted from built JSON files.
- Crate names are normalized consistently between metadata and on-disk artifacts so that lookups match regardless of hyphen/underscore representation.
- Each index entry carries a deterministic fingerprint derived from the crate name and version; equal inputs produce equal fingerprints across runs.
- The index is written only after discovery completes, never during incremental build stages.

## Constraints

Limits on what the pipeline may do during execution, derived from design choices about failure tolerance.

- An empty dependency list after discovery is a terminal error — no index or cache state is persisted.
- If all parallel builds fail to produce any JSON output, the pipeline errors without saving an index.
- Individual build failures are non-terminal: the package is logged and omitted from results while remaining builds continue.
- The Cargo.lock fallback omits manifest paths and uses a placeholder; it may also include dev-dependencies that the primary path excludes.
- Missing or corrupted JSON version metadata falls back to a sentinel version for display purposes only — cache keys remain unaffected.

## Rationale

Design justifications for each invariant and constraint, rooted in observed behavior or user need.

- Indexing all dependencies including failed builds gives downstream consumers visibility into what was attempted, rather than silently dropping unbuildable crates from view.
- Using metadata versions for cache keys prevents mismatches between what cargo resolved and what a particular build emitted, which could diverge in incremental or corrupted builds.
- Name normalization bridges the gap between cargo's hyphenated crate identifiers and filesystem-safe underscored names, without requiring callers to know the transformation rules.
- Non-terminal per-package failures balance correctness with resilience — a single doc build failure should not block progress on dozens of other dependencies.
- Excluding path dependencies avoids indexing local code that the consumer already has access to and does not expect in the global cache.

## Related

Concepts and source locations connected to the build pipeline.

- [[two-tier-caching#Two-Tier Caching]] — on-disk storage for cached rustdoc JSON
- [[crate-loading#Crate Loading and Deduplication]] — consumes the index produced by this pipeline
- [[src/cli/build.rs]]
- [[src/cargo/dependencies.rs]]
