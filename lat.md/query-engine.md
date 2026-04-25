---
lat:
  title: "Query Engine — Corrected Spec"
  status: specified
---
# Query Engine

Executes path-based queries against the cached rustdoc index and produces structured responses. Output fidelity is controlled by a detail-level mechanism; missing crate data is an error condition.

## Purpose

Provide a deterministic interface for looking up documentation by fully-qualified Rust path. A query resolves the path against the cached index, extracts structured metadata from matching items, and returns results governed by the detail level.

## Non-goals

Things the engine intentionally does not do.

- **Crate fetching**: The engine does not download or build rustdoc JSON. Crates must already reside in the global cache.
- **Incremental caching**: No background refresh of stale entries. The index and global cache are treated as authoritative at query time.
- **Cross-crate search heuristics**: Matching is path-based, not semantic.

## Invariants

Conditions that must hold for every query execution.

### Path Resolution Uniqueness

For any given path and optional crate filter, the set of matched items is deterministic across repeated executions with the same index. [[path-resolution#Invariants]]

### Crate Cache Integrity

If a query targets a path whose crate node exists in the local index but whose rustdoc JSON file is absent from the global cache, the query returns an error rather than silently skipping or returning empty results.

### Detail-Level Fidelity

Metadata fields exposed in query output exactly match the permissions granted by the active [[generic-rendering-fidelity#Generic Rendering Fidelity]]:

- **Minimal**: Signatures only. No visibility, generics, deprecation, attributes, function modifiers, or documentation at any nesting depth — including method-level docs within type and trait results.
- **Standard**: Adds visibility strings, generic parameter lists, and documentation text.
- **Detailed**: Adds deprecation flags, semantic attribute annotations, and function modifier information (const, async, unsafe, ABI).

No field appears at a lower tier than its defining level. No field is omitted at a higher tier that includes it.

### Public Status Reflects Actual Visibility

The `is_public` flag on method entries derives from the item's actual `Visibility` field in rustdoc JSON, not from a constant. A method with visibility other than `Visibility::Public` must report `is_public: false`.

### Trait Method Origination

The `is_trait_method` flag on method entries reflects the provenance of the declaring impl block. Methods originating from a trait implementation carry `is_trait_method: true`; methods from inherent impls carry `is_trait_method: false`.

### Classification Completeness

The `kind` field on every match covers the full set of rustdoc item variants. The spec enumerates all possible values rather than collapsing them into a handful of categories:

| kind | source variant |
|---|---|
| `type` | Struct, Enum, Union, TypeAlias |
| `trait` | Trait, TraitAlias |
| `function` | Function |
| `module` | Module |
| `impl` | Impl |
| `constant` | Constant |
| `static` | Static |
| `field` | StructField |
| `variant` | Variant |
| `macro` | Macro |
| `proc_macro` | ProcMacro |
| `use` | Use |
| `extern_crate` | ExternCrate |
| `primitive` | Primitive |
| `other` | any remaining variant |

This enumeration is stable: adding a new rustdoc variant requires adding a row, not collapsing into an existing category.

### Detail Level and Minimal Mode Are Distinct Mechanisms

[[generic-rendering-fidelity#Generic Rendering Fidelity]] controls what metadata is extracted from the source. A separate minimal-mode flag post-processes the response to strip optional fields for compactness. The two mechanisms overlap in effect but differ in intent:

- **Detail level**: determines extraction scope — whether visibility, generics, docs, deprecation, attributes, and modifiers are gathered at all.
- **Minimal mode**: applies after extraction — removes already-extracted metadata from the serialized response for size reduction.

A query with `DetailLevel::Standard` plus minimal mode enabled will extract visibility and generics (per Standard rules) but then strip them (per minimal-mode rules). A query with `DetailLevel::Minimal` extracts nothing beyond signatures regardless of whether minimal mode is enabled.

### Result Completeness

All items matching the queried path are included in the response. The engine does not short-circuit after encountering the first match.

## Constraints

Limits on what the engine may do during execution.

### Cache Dependency

The query engine operates exclusively on data available through the local index and global cache. No network access, disk I/O beyond reading cached JSON, or subprocess invocation occurs during query execution.

If a crate referenced by the index has no corresponding file in the global cache, the query fails.

### Selective Crate Loading

Crate data is accessed only when needed for a matching path. The engine does not preload all indexed crates.

### Output Boundedness

The response size respects the configured token budget if one is set. Expansion of nested types halts when the budget would be exceeded.

## Rationale

Design justifications for each invariant and constraint.

**Crate absence as error**: Silently skipping missing crates produces empty results that are indistinguishable from "no match found." The consumer cannot tell whether a type truly does not exist or whether the cache is incomplete. Treating missing crate data as an error forces the caller to address cache freshness.

**Full classification enumeration**: Collapsing 16+ rustdoc variants into 4 categories loses information and creates ambiguity about which concrete Rust construct produced a given result. The full enumeration ensures every kind value maps back to a specific source variant.

**DetailLevel bounds method docs**: Without gating, method documentation leaks into Minimal output, violating the "signatures only" guarantee and defeating the purpose of the detail-level mechanism at nested depth.

**is_public from actual visibility**: A constant `true` makes the field meaningless and contradicts its documented semantics. The visibility field in rustdoc JSON is authoritative.

**is_trait_method reflects provenance**: A constant `false` erases the distinction between inherent methods and trait-provided methods, which matters for API comprehension and downstream consumers.

**Separation of DetailLevel and minimal mode**: Conflating them makes it impossible to express "extract Standard metadata but produce a compact response" or "extract Minimal metadata without post-processing." Two mechanisms with distinct responsibilities allow independent composition.

## Related

Concepts this spec references.

- [[path-resolution#Invariants]] — Path resolution strategy and ordering guarantees
- [[type-expansion#Invariants#Detail Levels]] — DetailLevel enum definition and check-method semantics
- [[two-tier-caching#Two-Tier Caching]] — Cache key composition for rustdoc JSON files
- [[filter-engine#Filter Engine]] — Post-query filtering by pattern, kind, crate, visibility
- [[token-budgeting#Token Budgeting]] — Token budgeting for response size control
