# STACK-v1.1: Output Refinement Capabilities

## Overview

This document defines the specific capabilities required for v1.1's output refinement features, focusing on unified kind rendering, robust filtering, doc comment extraction, and additional rustdoc JSON field discovery.

---

## 1. Unified Kind Rendering

### 1.1 Required Capabilities

**Rationale:** Current implementation has inconsistent rendering across query depths. v1.1 must provide a uniform presentation of all rustdoc item types (modules, functions, types, structs, enums, traits) at any depth.

### 1.2 Technical Requirements

#### ItemKind Enumeration
- Support all 24 ItemKind variants:
  - **Core types:** Module, Struct, Enum, Union, TypeAlias, Trait, TraitAlias, Impl
  - **Members:** StructField, Variant, AssocType, AssocConst, Constant, Static
  - **Other:** Function, Macro, ProcDerive, ProcAttribute, Use, ExternCrate, ExternType, Primitive, Keyword, Attribute

#### Rendering Strategy
- Use rustdoc-types ItemKind for type discrimination
- Implement depth-aware formatting:
  - **Root depth (0):** Show complete item details
  - **Nested depth (1+):** Show condensed summary with depth indicator
  - **Minimal mode:** Omit implementation details, show type signatures only

#### Integration Points
- **Modify `src/types/query.rs`:** Update `QueryMatch` to support generic `Item` content
- **Modify `src/format/text.rs`:** Add `format_item()` function that dispatches to kind-specific formatters
- **Reuse existing patterns:** Follow format modules structure (text.rs, json.rs)

**NOT to add:**
- New rendering libraries (console already provides styled text output)
- Custom markup parsers (rustdoc provides markdown in `Item::docs`)

---

## 2. Robust Filtering

### 2.1 Required Capabilities

**Rationale:** Existing `PathResolver::path_matches()` is too simplistic. v1.1 needs comprehensive filtering supporting both inclusion and exclusion with wildcards and regex.

### 2.2 Technical Requirements

#### Filter Specification
- **Include patterns:** `--include` flag accepts glob patterns
- **Exclude patterns:** `--exclude` flag accepts glob patterns
- **Crate filtering:** `--crate <name>` to restrict to specific crates
- **Kind filtering:** `--kind <module|struct|fn|...>` to restrict to specific item types

#### Implementation Strategy
```rust
pub struct FilterConfig {
    include_patterns: Vec<glob::Pattern>,
    exclude_patterns: Vec<glob::Pattern>,
    crate_filter: Option<String>,
    kind_filter: Option<ItemKind>,
    visibility_filter: Option<Visibility>, // pub/private
}
```

#### Integration Points
- **Add to `src/types/filter.rs`:** New module for filtering logic
- **Modify `src/cli/query.rs`:** Add filter flags to CLI
- **Modify `src/query/lookup.rs`:** Add filtering step between path matching and result collection

#### Pattern Matching Libraries
- **`glob` crate (0.3.1):** Already in dependencies, suitable for simple wildcards
- **Alternatives considered:** regex (requires new dependency), fnmatch (limited)
- **Decision:** Use `glob` for consistency with Cargo patterns

**NOT to add:**
- Custom pattern language (glob is sufficient for filesystem-like patterns)
- Exact string matching (use `PathResolver::path_matches()` for this)

---

## 3. Doc Comment Extraction and Display

### 3.1 Required Capabilities

**Rationale:** Documentation is the primary value proposition of cargo-doc-query. v1.1 must surface doc comments consistently across all item types and depths.

### 3.2 Technical Requirements

#### Doc Extraction from rustdoc JSON
- **Source:** `Item::docs` field (Option<String>) contains full markdown docstring
- **Metadata:** `Item::links` field contains intra-doc link mappings
- **Attributes:** `Item::attrs` field contains metadata like `#[deprecated]`, `#[must_use]`

#### Display Strategy
- **Token budget tracking:** Integrate with existing `TokenConfig` from `src/types/expand.rs`
- **Rendering modes:**
  - **Full:** Display entire docstring
  - **Condensed:** Show first 3 lines or 80 characters
  - **None:** Skip docs in minimal mode
- **Smart truncation:** Detect long docs and add "..." indicator

#### Integration Points
- **Modify `src/types/query.rs`:** Add `docs: Option<String>` to all result types
- **Modify `src/format/text.rs`:** Update `format_method()` to show docs first line
- **Reuse existing:** Token budget tracking from `expand.rs` is already implemented

**Token budget implications:**
- Base doc comment overhead: ~30 tokens per item
- Typical docs: 100-500 tokens
- Long docs (examples, examples): 1000-5000 tokens
- **Recommendation:** Add per-doc comment limit (e.g., 200 tokens) with warning when exceeded

**NOT to add:**
- Custom markdown parser (rustdoc's markdown is already formatted)
- Cross-reference resolution (use `Item::links` for intra-doc links)
- Example extraction (docs already contain example blocks)

---

## 4. Additional rustdoc JSON Fields Discovery

### 4.1 Required Capabilities

**Rationale:** rustdoc JSON provides rich metadata beyond basic type information. v1.1 should surface this metadata where it's most useful.

### 4.2 Additional Fields to Surface

#### Visibility
- **Field:** `Item::visibility: Visibility` enum
- **Use cases:** Filter public/private items, indicate API surface
- **Decision:** Always include in query results, add CLI flag for filtering

#### Deprecation
- **Field:** `Item::deprecation: Option<Deprecation>`
- **Use cases:** Identify deprecated items, show deprecation notice
- **Decision:** Include in minimal mode, add visual indicator

#### Generics
- **Field:** `Struct::generics`, `Enum::generics`, `Function::generics`
- **Use cases:** Show generic parameters and where clauses
- **Decision:** Include only in non-minimal mode to reduce token count

#### Source Location
- **Field:** `Item::span: Option<Span>`
- **Use cases:** Display source location, enable IDE integration
- **Decision:** Skip in minimal mode (add separate `--show-source` flag if needed)

#### Attributes
- **Field:** `Item::attrs: Vec<Attribute>`
- **Use cases:** Show `#[must_use]`, `#[non_exhaustive]`, etc.
- **Decision:** Include `must_use` and `deprecated` in standard rendering, rest in minimal

#### Target and ABI
- **Field:** `Function::header: FunctionHeader`
- **Use cases:** Show const, unsafe, async, ABI information
- **Decision:** Include in non-minimal mode, skip in minimal

#### Implementation Tracking
- **Field:** `Struct::impls: Vec<Id>`, `Enum::impls: Vec<Id>`
- **Use cases:** Discover trait implementations for types
- **Decision:** Only include in non-minimal mode (too expensive)

### 4.3 Integration Points
- **Modify `src/types/query.rs`:** Add optional fields to result types
- **Modify `src/format/text.rs`:** Display additional fields based on mode
- **Reuse existing:** Token budget tracking for field inclusion decisions

**NOT to add:**
- Source code extraction (requires significant new infrastructure)
- Cross-crate resolution (rustdoc JSON already has all needed links)
- Type inference (rustdoc JSON already has complete type information)

---

## 5. Implementation Order

### Phase 1: Foundation (Lowest Risk)
1. Add `ItemKind` to `QueryMatch` union variant
2. Implement `format_item()` dispatcher in text.rs
3. Add basic `Item` -> `QueryMatch` conversion

### Phase 2: Filtering
1. Create `src/types/filter.rs` module
2. Implement glob pattern matching
3. Add filter configuration to CLI

### Phase 3: Documentation
1. Update all result types to include `docs: Option<String>`
2. Implement doc display in text formatter
3. Add token budget integration for docs

### Phase 4: Additional Metadata
1. Add visibility field to results
2. Add deprecation field to results
3. Add generic parameter field to result types
4. Add `must_use` attribute detection

### Phase 5: Refinement
1. Polish unified rendering across depths
2. Add filtering logic integration
3. Test token budget behavior

---

## 6. Testing Strategy

### 6.1 Unit Tests
- Filter pattern matching with glob library
- Doc comment token estimation
- Item kind routing to correct formatter

### 6.2 Integration Tests
- End-to-end query with filters
- Token budget limits with doc comments
- Minimal vs full mode comparison

### 6.3 Manual Testing
- Large crates (serde, tokio) with various filters
- Deep type graphs with doc comment display
- Token budget exhaustion scenarios

---

## 7. Performance Considerations

### 7.1 Token Budget Impact
- **Doc comments:** Primary driver of token usage
- **Metadata fields:** ~5-10 tokens per field
- **Recommendation:** Set default token budget to 2000 tokens

### 7.2 Filtering Performance
- **Glob matching:** O(n*m) where n=items, m=patterns
- **Optimization:** Compile patterns once, reuse across queries
- **Crate filtering:** O(k) where k=crate count (negligible)

### 7.3 Memory Usage
- **Item storage:** ~100 bytes per item
- **Doc strings:** Variable based on comment length
- **Recommendation:** Stream JSON parsing, avoid in-memory storage of all docs

---

## 8. Dependencies Analysis

### 8.1 Current Dependencies (No Changes)
- `serde_json = "1.0"`: For JSON parsing
- `console = "0.15"`: For styled output
- `glob = "0.3.1"`: For pattern matching (already has glob!)

### 8.2 New Dependencies (Not Recommended)
- `regex`: Would be overkill for glob patterns
- `pulldown-cmark`: Custom markdown parser not needed
- `rustc-hash`: Already have hash map, no performance gain

---

## 9. Breaking Changes

### 9.1 Current API
- `QueryMatch::content` currently is an enum: Type, Trait, Module
- Field names may shift to support generic items

### 9.2 Migration Path
- Keep backward compatibility via `#[serde(flatten)]`
- Add deprecation warnings for old field names
- Provide compatibility shim in v1.1.0

---

## 10. Success Metrics

### 10.1 Functional Requirements
- [ ] All 24 ItemKind variants render correctly
- [ ] Glob patterns match as expected (glob library tests)
- [ ] Doc comments display in all rendering modes
- [ ] Token budget enforcement works for docs

### 10.2 Performance Requirements
- [ ] Filtering overhead < 5% of total query time
- [ ] Doc comment display < 20% of token budget
- [ ] Memory usage < 50MB for serde crate (no increase)

### 10.3 UX Requirements
- [ ] Unified rendering across depths
- [ ] Clear indication of truncated docs
- [ ] Helpful error messages for invalid patterns
- [ ] Minimal mode produces < 500 tokens for complex types

---

## Appendix A: rustdoc JSON Field Map

| Field | Source | v1.1 Exposure | Minimal Mode | Default |
|-------|--------|---------------|--------------|---------|
| `docs` | `Item::docs` | ✅ Yes | ✅ Yes | Full |
| `visibility` | `Item::visibility` | ✅ Yes | ❌ No | Public |
| `deprecation` | `Item::deprecation` | ✅ Yes | ❌ No | None |
| `generics` | `Struct::generics`, etc. | ✅ Yes | ❌ No | None |
| `attrs` | `Item::attrs` | ⚠️ Selective | ⚠️ Selective | `[must_use]` |
| `span` | `Item::span` | ❌ No | ❌ No | None |
| `header` | `Function::header` | ✅ Yes | ❌ No | None |
| `impls` | `Struct::impls`, etc. | ❌ No | ❌ No | None |

---

## Appendix B: ItemKind Rendering Priority

| Priority | Kind | Description | Default Display |
|----------|------|-------------|-----------------|
| 1 | Module | Container of items | List with depth |
| 2 | Struct/Enum/Union/Type | Types | Fields/Variants |
| 3 | Trait | Interface | Methods + associated types |
| 4 | Function/Method | Code | Signature + first line of docs |
| 5 | AssocType/AssocConst | Associated members | Name + bounds/default |
| 6 | Variant | Enum member | Name + fields |
| 7 | Field | Struct field | Name + type |
| 8 | Constant/Static | Global values | Name + type |
| 9 | Macro/ProcDerive | Procedural macros | Name + signature |
| 10 | Other | Use, ExternCrate, etc. | Name only |

---

## Appendix C: Pattern Syntax Examples

| Pattern | Meaning | Example |
|---------|---------|---------|
| `*` | Match any characters (except /) | `*Test` matches `MyTest` |
| `**` | Match any path (including /) | `**/*Handler` matches `handlers/*Handler` |
| `?` | Match any single character | `File?` matches `File1`, `File2` |
| `[seq]` | Match any character in sequence | `[abc]Test` matches `aTest`, `bTest`, `cTest` |
| `!` | Negate pattern | `!private/*` excludes private items |

---

**Last Updated:** 2026-02-13
**Version:** 1.1
**Status:** Research Complete, Ready for Implementation
