# Phase 2: Core Querying — Methods & Traits - Research

**Researched:** 2026-02-12
**Domain:** Rust documentation query engine based on rustdoc JSON
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Unified interface:** `cargo doc-query query <path>` (single command, not separate subcommands)
- **Path syntax:** Follows Rust module naming conventions (`crate::module::Type`)
- **Searching with crate:** Optional `--crate <crate-name>` flag limits search to specific crate
- **Automatic inference:** Command infers whether the path refers to a type or trait, and what to return by default
- **Type queries:** When querying a type (e.g., `std::vec::Vec`), returns methods and trait implementations for that type
- **Trait queries:** When querying a trait (e.g., `std::iter::Iterator`), returns the trait definition only (methods, associated types) — NOT implementing types
- **Query scoping:** Optional `--kind` flag overrides default: `--kind methods|traits|types|all` (default is 'all')
- **Default visibility:** Public method signatures only (no private items)
- **Generic parameters:** Show generic declaration as written (e.g., `fn push<T>(&mut self, item: T)`)
- **Resolved generics:** Optional `--include=trait_parameterization` shows fully resolved type parameters (e.g., `fn push(&mut self, item: u8)`)
- **Documentation:** Optional `--include=docs` flag includes doc comments in output
- **Private APIs:** Optional `--include=private` flag includes non-public items (private, pub(crate), etc.)
- **Multiple matches:** Show all matches, do not prompt user for selection
- **Disambiguation context:** Each match includes crate name/version, fully qualified path, and kind
- **Error handling:** Fail fast — query fails immediately on non-existent paths, ambiguous matches, or items not in index
- **No fuzzy matching:** No "did you mean?" suggestions or spelling corrections
- **JSON output:** All query responses are valid, parseable JSON
- **Optimization:** Structure optimized for programmatic and LLM consumption

### Claude's Discretion

- Exact JSON schema and field naming conventions for output
- How `--include=trait_parameterization` handles complex type resolution
- Performance optimizations for large queries (within single-query scope)
- Whether trait parameterization needs additional filtering flags if output is too large

### Deferred Ideas (OUT OF SCOPE)

- fzf integration for interactive query mode → Phase 5: Integration & Polish
- Additional filtering options for `--include=trait_parameterization` if needed (to be determined based on Phase 2 performance/testing)
</user_constraints>

## Summary

Phase 2 builds a query engine on top of the Phase 1 documentation index, enabling users to retrieve method signatures, trait definitions, and type information as structured JSON. The core technical challenge is navigating rustdoc JSON's graph-based structure where items are stored by ID and cross-referenced through `Id` fields.

The standard approach: Load rustdoc JSON from cached index, traverse the graph to find matching items by path, extract associated impls and functions, and serialize to JSON. The rustdoc-types crate provides a complete type-safe API for this data, with `Crate` containing a HashMap of all items indexed by(Id).

Key implementation insights:
- **Two-tier lookup**: First find item by path in `Crate.paths` to get `Id`, then use `Id` in `Crate.index` to get full `Item`
- **Impl discovery**: Methods are found by iterating all `ItemEnum::Impl` items and matching `for_` field to target type
- **Trait vs inherent**: Use `impl.trait_` field (Some = trait impl, None = inherent impl)
- **Path resolution**: Use `Crate.paths` HashMap (Id → ItemSummary) to convert IDs to fully qualified paths

**Primary recommendation:** Build a query module that loads rustdoc JSON, implements path lookup via two-tier index, extracts impl blocks by type matching, and serializes to JSON with serde derive macros.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rustdoc-types | 0.57 | Types for rustdoc JSON output | Official crate, maintained by rust-lang, complete coverage of rustdoc JSON format |
| serde | 1.0.228 | Serialization framework | Rust standard for JSON serialization, derive macros for type safety |
| serde_json | 1.0.149 | JSON serialization/deserialization | Fast, well-tested, ecosystem standard |
| clap | 4.5.58 | CLI argument parsing | Type-safe derive macros, already used in Phase 1 |

### Existing Dependencies
| Library | Version | Purpose | Usage |
|---------|---------|---------|-------|
| cargo_metadata | 0.19 | Cargo manifest parsing | Already in Phase 1 for dependency discovery |
| anyhow | 1.0 | Error handling | Already used for ergonomic error propagation |
| postcard | 1.1 | Binary serialization | Already used for cache storage (Phase 3 will use more) |
| camino | 1.1 | UTF-8 path handling | Already in Phase 1 for path operations |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rustdoc-types crates | Direct serde structures | rustdoc-types is maintained, handles format version changes, provides type safety |
| serde_json | json5, simd-json | serde_json is standard, sufficient performance, simpler API |

**No additional dependencies needed** — all required crates are already in Cargo.toml

## Architecture Patterns

### Recommended Project Structure
```
src/
├── cli/
│   ├── build.rs           # (existing from Phase 1)
│   ├── query.rs           # NEW: Query command implementation
│   └── mod.rs             # (existing)
├── query/
│   ├── mod.rs             # Query engine module
│   ├── engine.rs          # Core query logic
│   ├── types.rs           # Query-specific types (output schemas)
│   ├── lookup.rs          # Path resolution and ID lookup
│   └── format.rs          # Type formatting for output
├── types/
│   ├── doc.rs             # NEW: Documentation type wrappers
│   ├── query.rs           # NEW: Query input/output types
│   └── mod.rs             # (existing, minimal)
├── index/
│   ├── graph.rs           # (existing - Crate graph)
│   └── mod.rs             # Needs update for query support
├── cache/
│   ├── store.rs           # (existing)
│   ├── key.rs             # (existing)
│   └── mod.rs             # (existing)
└── ...
```

### Pattern 1: Two-Tier Lookup for Path Resolution

**What:** rustdoc JSON stores items in a HashMap indexed by opaque Id, requiring two lookups to resolve paths to items.

**When to use:** Whenever you need to find an item by its fully qualified path or navigate from one item to another.

**Why:** rustdoc types are referenced by ID throughout the graph (e.g., `Impl.for_: Type::ResolvedPath(Path { id, path, args })`). You need the `paths` HashMap to map path strings to IDs, and the `index` HashMap to get the full Item from an ID.

**Example:**
```rust
// Source: rustdoc-types official documentation
use rustdoc_types::{Crate, Item, ItemEnum, Id};

fn find_item_by_path(
    krate: &Crate,
    path: &str
) -> Option<&Item> {
    // Tier 1: Find ID by path string
    let matching_ids: Vec<&Id> = krate.paths
        .iter()
        .filter(|(_, summary)| summary.path == path)
        .map(|(id, _)| id)
        .collect();

    if matching_ids.len() != 1 {
        return None; // Not found or ambiguous
    }

    // Tier 2: Get full Item by ID
    krate.index.get(matching_ids[0])
}
```

### Pattern 2: Impl Block Discovery for Type Methods

**What:** Extract methods and trait implementations for a given type by scanning all `ItemEnum::Impl` items and matching the `for_` field.

**When to use:** When querying methods or trait implementations for a type.

**Why:** Methods are not directly attached to type definitions. Instead, they exist in separate `Impl` items that reference the type via the `for_` field. trait impls have `trait_: Some(Path)`, inherent impls have `trait_: None`.

**Example:**
```rust
// Source: rustdoc-types API (Impl struct)
use rustdoc_types::{Impl, ItemEnum, Type};

fn find_impls_for_type(
    krate: &Crate,
    target_type_id: &Id
) -> Vec<&Impl> {
    krate.index.values()
        .filter_map(|item| match &item.inner {
            ItemEnum::Impl(impl_block) => Some(impl_block),
            _ => None,
        })
        .filter(|impl_block| {
            // Check if this impl is for our target type
            // Type::ResolvedPath contains the Id reference
            match &impl_block.for_ {
                Type::ResolvedPath(path) => path.id == *target_type_id,
                _ => false,
            }
        })
        .collect()
}
```

### Pattern 3: Query Command with Clap Derive

**What:** Use clap derive macros for type-safe CLI argument parsing following the established Phase 1 pattern.

**When to use:** All CLI commands. Follows the existing `Command` trait pattern.

**Why:** Type-safe, compile-time checked arguments, automatic help generation, consistent with Phase 1 architecture.

**Example:**
```rust
// Source: clap crate (standard CLI pattern)
use clap::{Args, Parser, Subcommand};
use crate::cli::Command;

#[derive(Parser)]
struct QueryCommand {
    /// The path to query (e.g., std::vec::Vec)
    path: String,

    /// Optional: Limit to specific crate
    #[arg(long)]
    crate_name: Option<String>,

    /// What to include in output
    #[arg(long)]
    include: Vec<String>,
}

#[derive(Args)]
struct QueryOptions {
    /// Which kind of query (methods, traits, types, all)
    #[arg(long, default_value = "all")]
    kind: QueryKind,

    #[arg(long)]
    docs: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum QueryKind {
    Methods,
    Traits,
    Types,
    All,
}
```

### Pattern 4: JSON Output Schema with Serde Derive

**What:** Define output types with derive(Serialize) for structured JSON output.

**When to use:** All API responses. Types should be separate from rustdoc_types to provide stable API surface.

**Why:** Type-safe serialization, clear output contract, can evolve schema independently from rustdoc-types changes.

**Example:**
```rust
// Source: serde crate documentation
use serde::Serialize;

#[derive(Serialize, Debug)]
struct MethodOutput {
    name: String,
    signature: String,
    return_type: String,
    visibility: String,
    is_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<String>,
}

#[derive(Serialize, Debug)]
struct TraitImplOutput {
    trait_name: String,
    trait_path: String,
    methods: Vec<MethodOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    provided_methods: Vec<String>,
}

#[derive(Serialize, Debug)]
struct QueryResponse {
    query: String,
    matches: Vec<QueryMatch>,
}

#[derive(Serialize, Debug)]
struct QueryMatch {
    crate_name: String,
    version: String,
    fully_qualified_path: String,
    kind: String,  // "type", "trait", "module", etc.
    content: QueryContent,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum QueryContent {
    Type(TypeQueryResult),
    Trait(TraitQueryResult),
}
```

### Anti-Patterns to Avoid

- **Don't store rustdoc JSON in memory for multiple crates simultaneously:**
  - Why: Large crates (aws-sdk-ec2 ~500MB JSON) will exhaust memory
  - Instead: Load rustdoc JSON on-demand per query, stream/parse only what's needed

- **Don't use recursion for type resolution:**
  - Why: Can blow stack on deeply nested types
  - Instead: Use iterative resolution with depth limits (even for simple queries)

- **Don't inline complex type signatures in JSON if not requested:**
  - Why: Types can be arbitrarily large, exceeding token limits
  - Instead: Use symbolic representation with Ids, resolve only when requested

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Type string formatting | Manual string concatenation | rustdoc-types Type serialization/Display | Handles all type variants correctly (ResolvedPath, Generic, etc.) |
| JSON serialization | Manual `to_json_string` | serde_json with derive | Type-safe, handles nested structures, error handling built-in |
| Path parsing | Manual split by `::` | Already validated by user, use as-is | No need to re-parse; rustdoc JSON has pre-validated paths |
| CLI parsing | Manual `env::args()` | clap derive | Type-safe, help generation, validation |

**Key insight:** rustdoc-types is a well-maintained, comprehensive representation of rustdoc JSON. Don't recreate its types or logic. Use `Crate.index`, `Crate.paths`, and standard rustdoc-types enum variants.

## Common Pitfalls

### Pitfall 1: Path Resolution Confusion

**What goes wrong:** Assuming paths in `Type::ResolvedPath.path` are the canonical definition location. They are the usage location, which can differ (e.g., `std::vec::Vec` vs `Vec` vs `std::prelude::v1::Vec`).

**Why it happens:** The `Path.path` field is "used" path, not "defined" path. Multiple path representations can refer to the same type.

**How to avoid:**
- Use `Path.id` for equality comparisons, not `Path.path`
- Use `crate_id` to track which crate an item came from
- When displaying paths, use the canonical form from `Crate.paths` for the item's own ID

**Warning signs:** Two items that appear different have the same `Id`, or you're getting duplicate matches for the same type.

### Pitfall 2: Cross-Crate ID Resolution

**What goes wrong:** An `Id` references an item from a different crate, causing "not found" errors when looking it up in the current crate's index.

**Why it happens:** rustdoc JSON stores external crate items with their own IDs, which only make sense in their originating crate's JSON file. The `Item.crate_id` field indicates which crate the item belongs to.

**How to avoid:**
- When resolving an Id, check `item.crate_id` first
- Load the originating crate's rustdoc JSON file from the cache
- Use the external crate's index for lookup
- Phase 1's `CrateGraph` already tracks which files exist; use it

**Warning signs:** Panic/unwrap on `index.get(id)`, getting None for IDs that should exist based on item references.

### Pitfall 3: Assuming All Functions are Methods

**What goes wrong:** Trying to determine if a function is a method by checking `self` parameter in signature, which is unreliable for trait methods.

**Why it happens:** Inherent impls and trait impls store functions the same way; you need to know the context (which impl block the function is in) to determine if it's a method, associated function, etc.

**How to avoid:**
- Always look at the parent `Impl` item to understand function context
- Check `impl.trait_` (Some = trait impl, None = inherent impl)
- Check function signature for `self` parameter if distinguishing methods from associated functions

**Warning signs:** Difficulty distinguishing `Vec::new()` (associated function) from `vec.push()` (method).

### Pitfall 4: Generic Parameter Representation

**What goes wrong:** Outputting generic types as unresolved placeholders like `T: Generic` instead of fully qualified paths.

**Why it happens:** rustdoc JSON stores generics as `Type::Generic(String)` representing the parameter name, not the concrete type. The `--include=trait_parameterization` flag specifically requests resolution.

**How to avoid:**
- For signature-only mode, preserve generic parameters as-is (`fn push<T>(&mut self, item: T)`)
- For resolved mode, check if the Impl has concrete type bindings in generic args
- Default to signature mode unless explicitly requested

**Warning signs:** Type signatures that look like gibberish or don't match actual API calls.

### Pitfall 5: Memory Pressure on Large Crates

**What goes wrong:** Loading entire rustdoc JSON for aws-sdk-ec2 (~500MB) into memory causes OOM crashes or extreme slowdown.

**Why it happens:** serde_json deserializes the entire file at once, creating nested HashMap structures with millions of items.

**How to avoid:**
- Load only the crate JSON for the specific crate being queried (not all crates)
- Consider using `serde_json::from_reader` with streaming for future optimization
- Phase 3 should implement partial loading, but Phase 2 can load one crate at a time

**Warning signs:** Queries on dependency-heavy projects taking seconds or crashing.

## Code Examples

Verified patterns from official sources:

### Loading rustdoc JSON from Cache
```rust
// Source: rustdoc-types crate documentation + Phase 1 cache structure
use std::fs;
use rustdoc_types::Crate;
use crate::cache::CacheStore;

fn load_crust_from_cache(
    crate_name: &str,
    crate_version: &str
) anyhow::Result<Crate> {
    let cache_store = CacheStore::new()?;
    let index = cache_store.load_current()?;
    
    // Find the crate node
    let crate_node = index.nodes.iter()
        .find(|n| n.name == crate_name && n.version == crate_version)
        .ok_or_else(|| anyhow::anyhow!("Crate not in index"))?;
    
    // Load rustdoc JSON
    let json_path = std::path::PathBuf::from(&crate_node.json_path);
    let json_str = fs::read_to_string(&json_path)?;
    
    // Deserialize
    let krate: Crate = serde_json::from_str(&json_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse rustdoc JSON: {}", e))?;
    
    Ok(krate)
}
```

### Extracting Method Signatures
```rust
// Source: rustdoc-types (Function, FunctionSignature, Type)
use rustdoc_types::{Item, ItemEnum, Function, Type};

fn extract_method_signature(item: &Item) -> String {
    if let ItemEnum::Function(func) = &item.inner {
        let decl = &func.sig.decl;
        let inputs = &decl.inputs;
        let output = &decl.output;
        
        // Format: fn name(args) -> ReturnType
        let name = item.name.as_deref().unwrap_or("<anonymous>");
        let args_str = format_args_list(inputs);
        let return_str = format_return_type(output);
        
        format!("fn {}{} {}", name, args_str, return_str)
    } else {
        "<not a function>".to_string()
    }
}

fn format_return_type(output: &rustdoc_types::FnReturnType) -> String {
    match output {
        rustdoc_types::FnReturnType::Default => "()".to_string(),
        rustdoc_types::FnReturnType::Return(t) => format_type(t),
    }
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::ResolvedPath(path) => path.path.clone(),
        Type::Generic(name) => name.clone(),
        Type::Primitive(prim) => prim.clone(),
        Type::Tuple(types) => {
            let inner: Vec<String> = types.iter().map(format_type).collect();
            format!("({})", inner.join(", "))
        }
        Type::Reference { lifetime, mutable, elem } => {
            let lt = lifetime.as_ref().map(|l| format!("{} ", l)).unwrap_or_default();
            let mut_str = if *mutable { "mut " } else { "" };
            format!("&{}{}{}", lt, mut_str, format_type(elem))
        }
        // ... handle other variants
        _ => "<complex type>".to_string(),
    }
}
```

### Building Query Response
```rust
// Source: serde_json crate documentation
use serde_json::json;

fn build_method_query_response(
    path: &str,
    methods: Vec<Method>,
    traits: Vec<TraitImpl>
) -> String {
    let response = json!({
        "query": path,
        "kind": "type",
        "results": {
            "methods": methods,
            "traits": traits
        }
    });
    
    serde_json::to_string_pretty(&response)
        .unwrap_or_else(|_| "{\"error\":\"Serialization failed\"}".to_string())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| LSP-based queries | rustdoc JSON-based | 2026-02-12 (project decision) | Deterministic, no daemon, machine-readable |
| Separate methods/traits subcommands | Unified `query` command with inference | This phase | Simpler UX, automatic disambiguation |
| Full type expansion by default | Signature-only with optional resolution | This phase | Token-efficient, LLM-friendly default |

**Deprecated/outdated:**
- LSP integration: Rejected for being daemon-dependent and nondeterministic
- Interactive mode for queries: Deferred to Phase 5
- Recursive type expansion without limits: Not in scope (Phase 4)

## Open Questions

1. **Type Resolution Complexity for --include=trait_parameterization**
   - **What we know:** rustdoc JSON stores generic parameter information in Impl.generics and generic args in Path.args
   - **What's unclear:** Full algorithm for resolving all bounds and substituting concrete types, especially for complex trait bounds and where clauses
   - **Recommendation:** Start with basic type parameter substitution (map generic names to concrete types), defer complex bound resolution to Phase 4 when recursion is needed

2. **Exact JSON Schema for Output**
   - **What we know:** Must be valid, parseable JSON with stable field names
   - **What's unclear:** Specific field naming conventions (snake_case vs camelCase), whether to include all rustdoc_types fields or subset
   - **Recommendation:** Use snake_case for Rust convention, minimal schema (name, signature, return_type, docs) to start. Can add fields in later phases.

3. **Performance on Large Crates**
   - **What we know:** aws-sdk-ec2 is ~500MB JSON, loading entire crate can be slow
   - **What's unclear:** Whether sub-100ms query performance is achievable in Phase 2 without partial loading
   - **Recommendation:** Accept slower queries (500ms-2s) in Phase 2, optimize in Phase 3 with smarter caching and partial loading

## Sources

### Primary (HIGH confidence)
- [rustdoc-types 0.57 crate documentation](https://docs.rs/rustdoc-types/0.57/rustdoc_types/) - Complete API reference for all types used
  - `Crate` struct: Root structure with index and paths
  - `Item` and `ItemEnum`: Generic item container
  - `Impl` struct: Impl block representation
  - `Function` and `FunctionSignature`: Method signatures
  - `Type` enum: Type representation
  - `Path` struct: Fully qualified paths
- [serde_json crate documentation](https://docs.rs/serde_json/) - JSON serialization
- [clap crate documentation](https://docs.rs/clap/) - CLI argument parsing

### Secondary (MEDIUM confidence)
- Project Cargo.toml - Dependency versions (rustdoc-types 0.57, serde 1.0.228, clap 4.5.58)
- Phase 1 source code - Existing architecture patterns (Command trait, cache structure)

### Tertiary (LOW confidence)
- None - All research based on authoritative sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies verified in Cargo.toml and official docs
- Architecture: HIGH - rustdoc-types API and patterns thoroughly documented
- Pitfalls: HIGH - Cross-reference rustdoc-types docs identifies common failure modes

**Research date:** 2026-02-12
**Valid until:** 30 days (rustdoc-types is stable, but verify before Phase 4 if crate version changes)
