---
lat:
  title: Type Expansion
---
# Type Expansion

Specification of type expansion behavior — resolving a qualified path into a tree of types, fields, variants, and module items with cycle detection, depth limiting, and token budgeting.

## Purpose

Resolve a qualified Rust path into an expandable tree of types and relationships, with depth limiting, cycle detection, detail-level metadata filtering, and token budgeting.

## Non-goals

Things this feature does not attempt to solve or guarantee.

- Not a full Rust type-checker — cross-crate type references are opaque paths, not resolved types.
- Not an exhaustive documentation browser — only the first crate containing the target path is loaded; remaining crates are skipped even if they also contain matches.
- Not incremental — each expansion request loads from cache fresh; no warm state is preserved between calls.

## Invariants

Guarantees that hold across all expansion executions regardless of implementation changes.

### Crate Loading

Behavior of missing, cached, and loaded crates during the search phase.

- Missing crate does not abort. If a crate listed in the index has no cached rustdoc JSON, it is skipped and search proceeds to the next crate. Only an I/O or parsing error terminates early.
- Each crate is loaded at most once per expansion session — duplicate load attempts are no-ops.
- Crates that do not contain the target path are released after lookup.

### Cycle Detection

Scoping and guarantees of the visited-set mechanism during recursive traversal.

- The cycle detection set is scoped to each top-level match. If a path resolves to multiple items within the same crate, each item begins with its own empty visited set — an item reachable through two different roots may be expanded in both contexts.
- Within a single expansion root, each item ID is expanded at most once. Re-entry at any depth yields no output for that branch.

### Expansion Scope

Which item kinds receive deep recursion and how module contents are enumerated.

- The first crate containing the target path determines all output. Even if multiple crates contain matching items, only the winning crate contributes nodes.
- Only these item kinds are recursively expanded into deep type nodes: struct, enum, union, type alias. Trait is listed as an expandable kind but receives no deeper traversal — it produces a shallow node containing metadata only.
- Module items are enumerated by kind: struct, enum, union, type, trait, function, module, const, static, macro. Re-exports (use items) are listed as "re-export" entries with their source path preserved.

### Depth Limiting

The depth limit constrains recursive expansion across all paths, including a floor that prevents zero-depth queries.

- The depth limit applies uniformly across all expansion paths. An item at the limit yields no children regardless of kind.
- Submodules within a module node are only expanded if depth plus one is strictly less than the limit, ensuring they do not exceed the boundary.
- The depth argument has an effective floor of 1. A user-supplied depth of 0 is not honored as "methods and traits only" — it is treated equivalently to depth 1. The help text description ("depth 0: show methods and traits only") does not match runtime behavior.

### Re-export Name Resolution

Display name derivation for use items from local name and source path.

- For use items with a non-empty local name, the local name is used as the display name.
- For use items with an empty local name, the display name is derived from the last segment of the source path (the portion after the final `::`).

### Token Budgeting

Output size estimation, capping, and warning behavior.

- Token estimation produces an approximate count. The estimate is computed by serializing the graph to JSON and dividing length by four.
- A configured budget cap halts expansion once the accumulated count reaches or exceeds it, recording the truncated paths.
- No budget means unlimited expansion (subject to depth limit only).
- A warning threshold indicates proximity to the budget without halting.

### Detail Levels

Metadata included at each of the three granularity tiers.

- Three levels exist: Minimal, Standard, Detailed. Standard is the default.
- **Minimal**: signatures and structural shape only. Visibility, generics, deprecation, attributes, and function modifiers are all excluded. Generic parameter *data* — the raw `<T>` strings — is preserved in minimal output to retain structural identity, even though DetailLevel::Minimal does not include them via `includes_generics()`. The distinction is: Metadata display of generics (whether to show them) is gated by `includes_generics()`; data retention of generic parameters (carrying them through the minimal transform) is unconditional.
- **Standard**: adds visibility and generic parameter display on top of Minimal.
- **Detailed**: adds deprecation flags, semantic attributes (`#[must_use]`, `#[non_exhaustive]`), and function modifiers (`const`, `async`, `unsafe`, ABI) on top of Standard.

### Exit Codes

The documented exit code contract and its current implementation gap.

- The help text documents differentiated exit codes (0 = success, 1 = general error, 2 = no cache, 3 = not found, 4 = build failed, 5 = invalid query, 6 = cache error, 7 = IO error, 8 = JSON parsing error, 9 = configuration error). The implementation returns exit code 1 for all error types. Differentiated codes are a design debt — the contract described to users is not yet enforced at runtime.

### Error Conditions

When expansion returns an error rather than a result.

- Empty cache yields a "no cached index" error before any crate lookup occurs.
- Path resolution failure after all crates have been checked yields a "not found" error.
- Internal load or parsing errors propagate immediately and terminate the expansion.

## Constraints

External limitations imposed by the underlying data model or API.

- Crate IDs are not globally unique — item lookup always targets the specific crate that contributed to the match, never a merged index across loaded crates.
- Generic parameters are filtered of synthetic (compiler-generated `impl Trait`) entries before display.
- Deprecation `since` fields are collected but not exposed; only the deprecation note is surfaced.
- Documentation comments are trimmed on ingestion.

## Rationale

Why each invariant and constraint exists, rooted in observed behavior or user need.

- **Graceful skip for missing crates**: build-time caching may be partial — a user might index a subset of dependencies. Aborting on a missing crate would make expansion brittle and force users to maintain perfect cache state.
- **Per-root cycle detection**: if two top-level matches share a subgraph, expanding it twice preserves the caller's expectation that each root yields independent results. Merging across roots would require cross-root deduplication logic and complicate ownership of which root "claims" a shared node.
- **First-crate-wins**: loading all matching crates would be expensive and produce ambiguous output when multiple crates export the same path (e.g., re-exports). The sorted crate order makes this deterministic.
- **Trait shallow nodes**: full trait expansion — enumerating associated items, methods, bounds — is not yet implemented. Until then, traits appear in module listings but do not recurse deeper than a metadata node. This avoids presenting partial expansion as complete.
- **Generic data preserved in minimal mode**: the structural shape of a type ("HashMap<K, V>" vs "HashMap") matters even when metadata display is suppressed. Carrying generic parameters through the minimal transform allows downstream consumers to distinguish parameterized from non-parameterized types without exposing the full generics rendering pipeline.
- **Depth floor of 1**: depth 0 would produce output with no substructure, which is not useful. The floor ensures at least one level of expansion. The mismatch between help text and runtime is a documentation debt, not a design intent to honor depth 0.
- **Exit code gap**: differentiated codes would improve scriptability (e.g., distinguishing "no cache" from "not found" in CI). They are documented as a contract but not yet wired into the error-to-exit-code mapping.

## Related

Concepts that inform or depend on this specification.

- [[crate-loading#Crate Loading and Deduplication]] — mechanics of loading rustdoc JSON from the global cache
- [[type-expansion#Invariants#Detail Levels]] — field-gating logic per granularity tier
- [[token-budgeting#Token Budgeting]] — budget enforcement strategy
- [[path-resolution#Path Resolution]] — matching semantics for qualified name resolution
