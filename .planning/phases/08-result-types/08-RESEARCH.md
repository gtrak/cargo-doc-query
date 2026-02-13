# Phase 8: Result Types - Research

**Researched:** 2026-02-13
**Domain:** Rust Documentation Extraction from rustdoc JSON Schema
**Confidence:** HIGH

## Summary

This research investigates the rustdoc JSON schema to understand how to extract rich metadata for Phase 8: Result Types. The primary focus is on understanding the structure of rustdoc-types fields for visibility, deprecation, generics, attributes, and function modifiers.

The rustdoc JSON output (generated via `rustdoc --output-format json`) provides comprehensive type information through the `rustdoc-types` crate (v0.57.0). The `Crate` struct is the root containing an `index: HashMap<Id, Item>` where all items are stored.

**Primary recommendation:** Use the `rustdoc_types::Item` struct fields directly: `visibility`, `deprecation`, `attrs`, and extract generics from item-specific inner types (Struct, Enum, Function, Trait, etc.).

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rustdoc-types | 0.57 | JSON schema types for rustdoc output | Official Rust schema, format version 57 |
| serde | 1.0 | Serialization/deserialization | Standard Rust serialization |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| cargo_metadata | 0.19 | Build documentation via cargo | Invoke rustdoc programmatically |
| serde_json | 1.0 | JSON parsing | Working with raw JSON output |

### rustdoc-types Key Types
| Type | Location | Contains |
|------|----------|----------|
| `Item` | Top-level | visibility, deprecation, attrs, docs, inner |
| `Visibility` | Item.visibility | Public, Default, Crate, Restricted |
| `Deprecation` | Item.deprecation | since, note |
| `Attribute` | Item.attrs[] | MustUse, NonExhaustive, Other, etc. |
| `Generics` | Struct/Enum/Function/etc | params, where_predicates |
| `FunctionHeader` | Function.header | is_const, is_unsafe, is_async, abi |

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **--detailed flag**: Provides richer metadata per item node, orthogonal to --depth (recursion control)
- **--detailed with --minimal**: Detailed metadata still omitted when --minimal takes precedence
- **Visibility Display**: Full modifiers (pub, pub(crate), pub(super), pub(in path::to::module)), inline with item name
- **Deprecation**: Capture is_deprecated boolean and deprecation_note text, skip "since version"
- **Generic Bounds**: Full trait bounds in Rust syntax (K: Eq + Hash, V), inline with item name, include defaults
- **Attribute Selection**: Focus on #[must_use], #[non_exhaustive], #[deprecated], skip #[derive], #[repr], doc attrs
- **Function Modifiers**: Include const, unsafe, async as booleans, ABI only when non-Rust
- **JSON Structure**: Flat structure with optional fields using #[serde(skip_serializing_if = "Option::is_none")]

### Claude's Discretion
- Exact field names in struct definitions
- Error handling for missing rustdoc JSON fields
- Performance optimization for metadata extraction
- Test coverage scope
- Implementation order of the 7 FIELD requirements

### Deferred Ideas (OUT OF SCOPE)
- None identified - all ideas stayed within phase scope

## Architecture Patterns

### Recommended Item Structure
```rust
// Source: rustdoc-types 0.57.0 lib.rs
pub struct Item {
    pub id: Id,
    pub crate_id: u32,
    pub name: Option<String>,
    pub span: Option<Span>,
    pub visibility: Visibility,        // PHASE 8: Extract this
    pub docs: Option<String>,
    pub links: HashMap<String, Id>,
    pub attrs: Vec<Attribute>,         // PHASE 8: Filter for must_use, non_exhaustive
    pub deprecation: Option<Deprecation>, // PHASE 8: Extract this
    pub inner: ItemEnum,               // PHASE 8: Extract generics from here
}
```

### Visibility Extraction Pattern
**What:** Convert rustdoc `Visibility` enum to display string
**When to use:** All public API items
**Example:**
```rust
// Source: rustdoc-types 0.57.0 lib.rs
pub enum Visibility {
    Public,                    // -> "pub"
    Default,                   // -> "" (private)
    Crate,                     // -> "pub(crate)"
    Restricted { parent: Id, path: String }, // -> "pub(in path)"
}

// Real JSON example from bitflags crate:
// "visibility": { "restricted": { "parent": 2, "path": "::iter" } }
// Should display as: "pub(in ::iter)"
```

### Deprecation Extraction Pattern
**What:** Extract deprecation status and note from `Option<Deprecation>`
**When to use:** Any item that may be deprecated
**Example:**
```rust
// Source: rustdoc-types 0.57.0 lib.rs
pub struct Deprecation {
    pub since: Option<String>,  // SKIP per requirements
    pub note: Option<String>,   // KEEP: deprecation_note
}

// Usage in extraction:
fn extract_deprecation(item: &Item) -> Option<String> {
    item.deprecation.as_ref()?.note.clone()
}
```

### Generic Bounds Extraction Pattern
**What:** Format `Generics` struct as Rust syntax string
**When to use:** Structs, enums, functions, traits, type aliases
**Example:**
```rust
// Source: rustdoc-types 0.57.0 lib.rs
pub struct Generics {
    pub params: Vec<GenericParamDef>,
    pub where_predicates: Vec<WherePredicate>,
}

pub struct GenericParamDef {
    pub name: String,
    pub kind: GenericParamDefKind,
}

pub enum GenericParamDefKind {
    Lifetime { outlives: Vec<String> },
    Type { 
        bounds: Vec<GenericBound>, 
        default: Option<Type>,  // Include this!
        is_synthetic: bool 
    },
    Const { type_: Type, default: Option<String> },
}
```

**Real JSON example from bitflags:**
```json
{
  "params": [{
    "name": "B",
    "kind": {
      "type": {
        "bounds": [{"outlives": "'static"}],
        "default": null,
        "is_synthetic": false
      }
    }
  }],
  "where_predicates": []
}
// Should format as: <B: 'static>
```

### Attribute Filtering Pattern
**What:** Parse `Vec<Attribute>` to find semantic attributes
**When to use:** Items with important usage semantics
**Example:**
```rust
// Source: rustdoc-types 0.57.0 lib.rs
pub enum Attribute {
    NonExhaustive,                    // #[non_exhaustive]
    MustUse { reason: Option<String> }, // #[must_use]
    MacroExport,                      // #[macro_export]
    AutomaticallyDerived,             // #[automatically_derived]
    Repr(AttributeRepr),              // #[repr(...)] - SKIP per requirements
    NoMangle,                         // #[no_mangle]
    TargetFeature { enable: Vec<String> },
    Other(String),                    // Fallback for unknown attrs
}

// Real JSON examples from bitflags:
// { "must_use": { "reason": null } }           // #[must_use]
// ["macro_export"]                             // #[macro_export]
// { "other": "#[forbid(unsafe_code)]" }       // Other attributes
```

**Per requirements, filter for:**
- `Attribute::MustUse` -> include
- `Attribute::NonExhaustive` -> include  
- `Attribute::Other` containing "deprecated" -> include (but note: #[deprecated] is in Item.deprecation)
- Skip: Repr, AutomaticallyDerived, doc attributes

### Function Modifiers Extraction Pattern
**What:** Extract `const`, `unsafe`, `async`, ABI from `FunctionHeader`
**When to use:** Functions, methods
**Example:**
```rust
// Source: rustdoc-types 0.57.0 lib.rs
pub struct Function {
    pub sig: FunctionSignature,
    pub generics: Generics,
    pub header: FunctionHeader,  // PHASE 8: Extract this
    pub has_body: bool,
}

pub struct FunctionHeader {
    pub is_const: bool,
    pub is_unsafe: bool,
    pub is_async: bool,
    pub abi: Abi,
}

pub enum Abi {
    Rust,                          // Skip displaying (default)
    C { unwind: bool },
    Cdecl { unwind: bool },
    // ... other variants
}

// Real JSON example from bitflags:
// "header": {
//   "is_const": true,
//   "is_unsafe": false,
//   "is_async": false,
//   "abi": "Rust"
// }
// Should display as: "const fn new(...)" (abi: "Rust" is skipped)
```

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON schema parsing | Custom structs | `rustdoc-types` crate | Maintained by Rust team, versioned (format_version: 57) |
| Visibility formatting | String concatenation | Match on `Visibility` enum | Handles all 4 variants including Restricted with parent ID lookup |
| Generic bounds formatting | Manual string building | Recursive type formatter | Complex nested types, HRTBs, associated types |
| Attribute parsing | Regex on strings | Match on `Attribute` enum | Structured data, type-safe matching |
| Function signature display | Manual formatting | `FunctionSignature` fields | Handles variadics, self parameters, complex return types |

**Key insight:** The rustdoc-types crate provides complete type safety and structure. Manual parsing will miss edge cases like synthetic generics, HRTBs, and restricted visibility paths.

## Common Pitfalls

### Pitfall 1: Visibility Parent Resolution
**What goes wrong:** `Visibility::Restricted` contains a `parent: Id` that needs to be resolved to get the full path. The `path` field may be relative (e.g., "::iter").
**Why it happens:** The parent ID references the module in the crate index. Without lookup, you can't construct the full `pub(in crate::foo::bar)` path.
**How to avoid:** Use the `paths` field in `Crate` to resolve ID to full path, or use the `path` field directly if it contains the full path.
**Warning signs:** Seeing `pub(in ::relative)` instead of `pub(in crate::module)`.

### Pitfall 2: Synthetic Generics
**What goes wrong:** `impl Trait` in argument position generates synthetic type parameters with `is_synthetic: true`. Displaying these confuses users.
**Why it happens:** Compiler transforms `fn foo(x: impl Trait)` to `fn foo<impl Trait: Trait>(x: impl Trait)` behind the scenes.
**How to avoid:** Check `is_synthetic` flag on `GenericParamDefKind::Type` and skip synthetic parameters.
**Warning signs:** Seeing generic parameters named things like `impl Trait` or `_`

### Pitfall 3: Nested Attribute Format
**What goes wrong:** `Attribute::Other(String)` contains the raw attribute text, which may include internal compiler representations.
**Why it happens:** Unknown attributes are captured as strings, not parsed.
**How to avoid:** Focus on structured `Attribute` variants (MustUse, NonExhaustive) rather than parsing `Other`.
**Warning signs:** Seeing attributes like `"#[attr = CfgTrace([...])]"` in output.

### Pitfall 4: Where Clause Placement
**What goes wrong:** `where_predicates` can appear on the item (Generics) AND on trait bounds (GenericBound::TraitBound).
**Why it happens:** HRTBs (`for<'a>`) attach generic_params to the bound, not the item.
**How to avoid:** Check both `Generics.where_predicates` and `GenericBound::TraitBound.generic_params`.
**Warning signs:** Missing `for<'a>` lifetime bounds in output.

### Pitfall 5: ABI Default Handling
**What goes wrong:** `Abi::Rust` is the default but still appears in JSON. Displaying `extern "Rust"` is redundant.
**Why it happens:** rustdoc always outputs the ABI field, even for default Rust ABI.
**How to avoid:** Skip displaying ABI when it equals `Abi::Rust`.
**Warning signs:** Seeing `extern "Rust" fn` in output.

### Pitfall 6: Deprecation vs Deprecated Attribute
**What goes wrong:** `#[deprecated]` appears in `Item::deprecation`, not in `Item::attrs`.
**Why it happens:** rustdoc specially handles deprecated attributes and moves them to the deprecation field.
**How to avoid:** Check `Item::deprecation` for deprecation info, not `Item::attrs`.
**Warning signs:** Not finding "deprecated" in attrs when the item is clearly deprecated.

## Code Examples

### Visibility to String Conversion
```rust
// Source: Based on rustdoc-types 0.57.0 Visibility enum
fn visibility_to_string(vis: &rustdoc_types::Visibility, _index: &HashMap<Id, Item>) -> String {
    match vis {
        rustdoc_types::Visibility::Public => "pub".to_string(),
        rustdoc_types::Visibility::Default => "".to_string(),
        rustdoc_types::Visibility::Crate => "pub(crate)".to_string(),
        rustdoc_types::Visibility::Restricted { parent: _, path } => {
            format!("pub(in {})", path)
        }
    }
}

// Note: The parent ID can be resolved via Crate::paths if full path needed
```

### Deprecation Extraction
```rust
// Source: Based on rustdoc-types 0.57.0 Deprecation struct
fn extract_deprecation(item: &Item) -> Option<DeprecationInfo> {
    item.deprecation.as_ref().map(|d| DeprecationInfo {
        is_deprecated: true,
        note: d.note.clone(),  // Skip d.since per requirements
    })
}

struct DeprecationInfo {
    is_deprecated: bool,
    note: Option<String>,
}
```

### Generic Parameters Formatting
```rust
// Source: Based on rustdoc-types 0.57.0 Generics struct
fn format_generics(generics: &Generics) -> String {
    if generics.params.is_empty() {
        return String::new();
    }
    
    let params: Vec<String> = generics.params.iter()
        .filter(|p| !is_synthetic(p))  // Skip synthetic params
        .map(|p| format_generic_param(p))
        .collect();
    
    if params.is_empty() {
        return String::new();
    }
    
    let mut result = format!("<{}>", params.join(", "));
    
    // Add where clauses if present
    if !generics.where_predicates.is_empty() {
        let where_clauses: Vec<String> = generics.where_predicates.iter()
            .map(|wp| format_where_predicate(wp))
            .collect();
        result.push_str(&format!(" where {}", where_clauses.join(", ")));
    }
    
    result
}

fn is_synthetic(param: &GenericParamDef) -> bool {
    matches!(&param.kind, GenericParamDefKind::Type { is_synthetic: true, .. })
}
```

### Attribute Filtering for MustUse and NonExhaustive
```rust
// Source: Based on rustdoc-types 0.57.0 Attribute enum
fn extract_semantic_attrs(attrs: &[Attribute]) -> Vec<String> {
    attrs.iter()
        .filter_map(|attr| match attr {
            Attribute::MustUse { reason } => {
                let mut s = "#[must_use]".to_string();
                if let Some(r) = reason {
                    s.push_str(&format!("(\"{}\")", r));
                }
                Some(s)
            }
            Attribute::NonExhaustive => Some("#[non_exhaustive]".to_string()),
            // Note: #[deprecated] is in Item.deprecation, not attrs
            _ => None,  // Skip Repr, AutomaticallyDerived, etc.
        })
        .collect()
}
```

### Function Header Formatting
```rust
// Source: Based on rustdoc-types 0.57.0 FunctionHeader and Abi enums
fn format_function_header(header: &FunctionHeader) -> String {
    let mut modifiers = Vec::new();
    
    if header.is_const {
        modifiers.push("const");
    }
    if header.is_async {
        modifiers.push("async");
    }
    if header.is_unsafe {
        modifiers.push("unsafe");
    }
    
    // ABI - only display when non-Rust
    match &header.abi {
        rustdoc_types::Abi::C { .. } => modifiers.push("extern \"C\""),
        rustdoc_types::Abi::Cdecl { .. } => modifiers.push("extern \"cdecl\""),
        rustdoc_types::Abi::Stdcall { .. } => modifiers.push("extern \"stdcall\""),
        // ... other non-Rust ABIs
        rustdoc_types::Abi::Rust => {} // Skip default
        _ => {} // Other ABIs can be handled as needed
    }
    
    if modifiers.is_empty() {
        String::new()
    } else {
        format!("{} ", modifiers.join(" "))
    }
}
```

### Complete Item Extraction Example
```rust
// Example: Extracting all Phase 8 metadata from an Item
fn extract_detailed_metadata(item: &Item, crate_data: &Crate) -> DetailedMetadata {
    DetailedMetadata {
        visibility: visibility_to_string(&item.visibility, &crate_data.index),
        is_deprecated: item.deprecation.is_some(),
        deprecation_note: item.deprecation.as_ref().and_then(|d| d.note.clone()),
        attributes: extract_semantic_attrs(&item.attrs),
        generics: extract_generics_from_item(item),
        is_const: extract_is_const(item),
        is_unsafe: extract_is_unsafe(item),
        is_async: extract_is_async(item),
        abi: extract_abi(item),
    }
}

fn extract_generics_from_item(item: &Item) -> Option<String> {
    match &item.inner {
        ItemEnum::Struct(s) => Some(format_generics(&s.generics)),
        ItemEnum::Enum(e) => Some(format_generics(&e.generics)),
        ItemEnum::Function(f) => Some(format_generics(&f.generics)),
        ItemEnum::Trait(t) => Some(format_generics(&t.generics)),
        ItemEnum::TypeAlias(t) => Some(format_generics(&t.generics)),
        ItemEnum::Union(u) => Some(format_generics(&u.generics)),
        _ => None,
    }
}

fn extract_is_const(item: &Item) -> bool {
    matches!(&item.inner, ItemEnum::Function(f) if f.header.is_const)
}

fn extract_is_unsafe(item: &Item) -> bool {
    matches!(&item.inner, ItemEnum::Function(f) if f.header.is_unsafe)
}

fn extract_is_async(item: &Item) -> bool {
    matches!(&item.inner, ItemEnum::Function(f) if f.header.is_async)
}

fn extract_abi(item: &Item) -> Option<String> {
    match &item.inner {
        ItemEnum::Function(f) => match &f.header.abi {
            rustdoc_types::Abi::Rust => None,  // Skip default
            abi => Some(format!("{:?}", abi)),
        },
        _ => None,
    }
}
```

## State of the Art

### rustdoc-types Format Versions
| Version | Changes | Date |
|---------|---------|------|
| 57 | Add `ExternCrate::path` | Current (v0.57.0) |
| 56 | Various changes | Recent |
| ... | ... | ... |

**Current format version:** 57 (as of rustdoc-types 0.57.0)

**Key insight:** Always check `Crate::format_version` when parsing to handle future changes gracefully.

### Common Gotchas in Real JSON

From analysis of bitflags crate JSON output:

1. **Visibility encoding:**
   - `{"restricted": {"parent": 2, "path": "::iter"}}` needs parent resolution
   - Some items have `"default"` (private), others `"public"`

2. **Generic bounds:**
   - Often empty `[]` for simple types
   - Lifetimes appear as `{"outlives": "'static"}` in bounds array

3. **Attributes come in multiple formats:**
   - Structured: `{"must_use": {"reason": null}}`
   - String tags: `["macro_export"]`
   - Raw strings: `{"other": "#[forbid(unsafe_code)]"}`

4. **Function headers always present:**
   - `is_const`, `is_unsafe`, `is_async` are booleans
   - `abi` is always present (often `"Rust"`)

## Open Questions

1. **Restricted Visibility Path Resolution**
   - What we know: `Visibility::Restricted` has `parent: Id` and `path: String`
   - What's unclear: Should we resolve `parent` ID to get full absolute path?
   - Recommendation: Use `path` field directly, prepend "crate" if relative path starts with "::"

2. **Synthetic Generic Handling**
   - What we know: `impl Trait` generates synthetic params with `is_synthetic: true`
   - What's unclear: Should we completely hide these or indicate them differently?
   - Recommendation: Skip synthetic params entirely in generic display

3. **Where Predicate Formatting**
   - What we know: `WherePredicate` has `BoundPredicate`, `LifetimePredicate`, `EqPredicate` variants
   - What's unclear: How to format complex HRTBs with nested generic_params
   - Recommendation: Start with simple bounds, add HRTB support incrementally

4. **Attribute Note Truncation**
   - What we know: `MustUse` can have a reason string
   - What's unclear: How long can these notes be? Should we truncate?
   - Recommendation: Display full note, but have sensible limits in JSON output

## Sources

### Primary (HIGH confidence)
- **rustdoc-types 0.57.0 source** (`~/.cargo/registry/src/.../rustdoc-types-0.57.0/src/lib.rs`)
  - Complete schema definitions for all types
  - Doc comments explaining field semantics
  - JSON structure verified with real output

- **Real JSON output from bitflags crate** (`target/doc/bitflags.json`)
  - Verified visibility encoding patterns
  - Confirmed attribute structures
  - Validated generic parameter examples

### Secondary (MEDIUM confidence)
- **rustdoc JSON format documentation** (https://doc.rust-lang.org/rustdoc/json.html)
  - High-level overview of output format
  - Version history and breaking changes

- **cargo-doc-query existing codebase**
  - Current usage of rustdoc_types::Item
  - Pattern for visibility extraction (placeholder in doc.rs)

### Tertiary (LOW confidence)
- None - all findings verified with primary sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Using official rustdoc-types crate
- Architecture: HIGH - Verified against real JSON output
- Pitfalls: HIGH - Based on actual JSON analysis and doc comments

**Research date:** 2026-02-13
**Valid until:** 2026-05-13 (3 months) - Check for new rustdoc-types versions

**Format version compatibility:**
- Tested with: Format version 57
- Expected compatibility: 55-58 (major changes rare)
- Breaking changes: Bumped in FORMAT_VERSION constant
