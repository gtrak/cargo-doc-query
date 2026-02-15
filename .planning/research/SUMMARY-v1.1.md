# Project Research Summary v1.1

**Project:** cargo-doc-query
**Domain:** Rust CLI tool for querying rustdoc JSON documentation
**Researched:** 2026-02-12 (v1.0), 2026-02-13 (v1.1)
**Confidence:** HIGH (v1.0), HIGH (v1.1)

## Executive Summary

This project is a Cargo subcommand that parses rustdoc JSON output to enable fast, LLM-optimized queries about Rust API documentation. v1.1 builds on v1.0 by adding output refinement capabilities: unified rendering across all item types, robust pattern-based filtering, doc comment extraction with token-aware truncation, and rich metadata field discovery from rustdoc JSON. The architecture extends the existing four-layer system with a new Filter Engine component, while maintaining backward compatibility through optional fields and versioned output formats.

The competitive differentiation remains focused on token efficiency for LLM consumption — v1.1 enables flexible output refinement to maximize useful context within LLM context windows (128K-2M tokens). All 24 ItemKind variants render consistently, glob-based filters (include/exclude, crate, kind) enable precise queries, and token budget enforcement prevents context overflow while preserving important content (code blocks, signatures, key terms).

Key v1.1 enhancements:
- **Unified Kind Rendering:** Single render function for all ItemKind variants (modules, structs, enums, traits, functions, etc.)
- **Robust Filtering:** Glob pattern matching with include/exclude, crate filtering, kind filtering
- **Doc Comment Extraction:** Full markdown parsing with token-aware truncation (sentence boundaries, smart ellipsis)
- **Field Discovery:** Visibility modifiers, deprecation status, generic parameters, attributes, source locations
- **Filter Engine:** New `src/types/filter.rs` module with compiled glob patterns for efficient matching

Critical v1.1 pitfalls to avoid:
- Inconsistent depth-based formatting (single render function required)
- Generic truncation mid-parameters (track separately, bounds before parameters)
- Naive name-based filtering only (support path, attribute, and visibility filters)
- Doc truncation mid-sentence (truncate at boundaries, add "..." indicator)
- Performance degradation from new features (<5% overhead targeted)
- Memory explosion from text processing (streaming JSON, lazy loading)

## Key Findings by Area

### Unified Rendering Capabilities

**What's New:**
- All 24 ItemKind variants render with consistent formatting:
  - Core types: Module, Struct, Enum, Union, TypeAlias, Trait, TraitAlias, Impl
  - Members: StructField, Variant, AssocType, AssocConst, Constant, Static
  - Other: Function, Macro, ProcDerive, ProcAttribute, Use, ExternCrate, ExternType, Primitive, Keyword, Attribute

**Implementation:**
- Single `format_item()` dispatcher in `src/format/text.rs`
- Depth-aware formatting: root (0) shows details, nested (1+) shows condensed summary
- Minimal mode omits implementation details, shows type signatures only
- Follows existing format module structure (text.rs, json.rs)

**Key Design:**
- Use rustdoc-types `ItemKind` for type discrimination
- No custom markup parsers needed (rustdoc's markdown already formatted)
- Token budget tracking integrated (base overhead: ~30 tokens per item)

### Filtering Architecture

**What's New:**
- Pattern-based include/exclude filtering with glob patterns
- Crate and kind filtering (restrict to specific crates or item types)
- Support for `--include`, `--exclude`, `--crate`, `--kind` CLI flags

**Implementation:**
- New `FilterEngine` in `src/types/filter.rs`
- Uses `glob` crate (0.3.1) for pattern matching (already in dependencies)
- Compiled patterns reused across queries for efficiency
- Filter application after content extraction (doesn't affect graph traversal)

**Pattern Types Supported:**
- `*` — Match any characters (except /)
- `**` — Match any path (including /)
- `?` — Match any single character
- `[seq]` — Match any character in sequence
- `!` — Negate pattern

**Key Design:**
- FilterConfig struct stores include/exclude patterns, crate filter, kind filter
- Performance: O(n×p) where n=items, p=patterns (optimized with pre-compiled patterns)
- Target: <5% overhead on total query time

### Doc Comment Handling

**What's New:**
- Full markdown doc comment extraction from `Item::docs` field
- Token-aware truncation with sentence boundary detection
- Smart ellipsis ("...") when truncated
- Intra-doc link rendering (extracted from `Item::links` field)

**Implementation:**
- Doc extraction integrated with existing TokenConfig from `src/types/expand.rs`
- Token budget: base doc overhead ~30 tokens, typical docs 100-500 tokens, long docs 1000-5000 tokens
- Truncation modes: full (all), condensed (first 3 lines or 80 chars), none (minimal mode)
- Warning shown when doc budget exceeded

**Key Design:**
- Don't add custom markdown parser (rustdoc's markdown already formatted)
- Use `Item::links` for intra-doc link mappings (no cross-reference resolution needed)
- Example blocks preserved in truncation (code blocks > signatures > paragraphs)

### Additional Field Opportunities

**New Fields to Surface:**
- **Visibility:** `Item::visibility` enum (pub, pub(crate), pub(super), pub(in path))
- **Deprecation:** `Item::deprecation: Option<Deprecation>` with replacement hints
- **Generics:** Struct::generics, Enum::generics, Function::generics with bounds
- **Attributes:** `Item::attrs` containing `#[must_use]`, `#[non_exhaustive]`, etc.
- **Source Location:** `Item::span: Option<Span>` (include if `--show-source` flag added later)
- **Target and ABI:** Function::header with const, unsafe, async, ABI info

**Display Strategy:**
- Visibility/Deprecation: always included in query results
- Generics/ABI: include only in non-minimal mode to reduce token count
- Attributes: show `must_use` and `deprecated` in standard rendering, rest in minimal
- Implementation tracking (`Struct::impls`, `Enum::impls`): skip in non-minimal mode (too expensive)

**Field Map Summary:**

| Field | Source | v1.1 Exposure | Minimal Mode | Default |
|-------|--------|---------------|--------------|---------|
| `docs` | `Item::docs` | ✅ Yes | ✅ Yes | Full |
| `visibility` | `Item::visibility` | ✅ Yes | ❌ No | Public |
| `deprecation` | `Item::deprecation` | ✅ Yes | ❌ No | None |
| `generics` | Struct::generics, etc. | ✅ Yes | ❌ No | None |
| `attrs` | `Item::attrs` | ⚠️ Selective | ⚠️ Selective | `[must_use]` |
| `span` | `Item::span` | ❌ No | ❌ No | None |
| `header` | Function::header | ✅ Yes | ❌ No | None |

## Implementation Implications

### Roadmap Impact

v1.1 extends v1.0 architecture while maintaining backward compatibility:

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                Command Line Interface                 │  │
│  │  • Add filter flags (--include, --exclude, --kind)    │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      Commands Layer                          │
│  ┌──────────────┐  ┌──────────────────────────────────────┐ │
│  │    build     │  │      query/expand (unified)          │ │
│  └──────┬───────┘  │  • Filter engine integration          │ │
├─────────┴──────────┤  • Doc comment rendering              │ │
│                        • Token-aware output                │ │
│                   ┌──────────────────────────────────────┐ │
│                   │       Filter Engine (NEW)            │ │
│                   │  • Include/exclude pattern matching  │ │
│                   │  • Kind and crate filtering          │ │
│                   └──────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                        Index Layer                           │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Documentation Graph Model               │   │
│  │    (Existing query engine extended with filtering)  │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                   Parser & Cache Layer                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Parser     │  │    Cache     │  │  Persistence │      │
│  │ (rustdoc JSON)│  │   (Index)    │  │ (postcard)   │      │
│  │  + metadata  │  │ (no changes) │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### New Components

**Filter Engine** (`src/types/filter.rs`):
- `FilterConfig` struct with include/exclude patterns, crate filter, kind filter
- `FilterEngine` for matching items against patterns
- Uses `glob` crate (0.3.1) for compiled pattern matching

**Enhanced Query Types** (`src/types/query.rs`):
- Add optional fields to result types: `visibility`, `deprecation`, `attrs`, `generics`
- Backward compatibility via `#[serde(skip_serializing_if = "Option::is_none")]`
- `QueryMatch` union variant extended to support generic `Item` content

### Build Order Recommendations

**Phase 1: Foundation (2-3 hours)**
1. Add `src/types/filter.rs` with glob pattern matching
2. Create `FilterConfig` and `FilterEngine` structs
3. Add basic unit tests for filter patterns

**Phase 2: CLI Integration (1-2 hours)**
1. Add filter flags to `Cli::Query` struct
2. Update `ExpandCommand::from_args()` to accept filters
3. Wire filter config through to QueryEngine

**Phase 3: Result Type Extensions (3-4 hours)**
1. Add `visibility`, `deprecation`, `attrs`, `generics` fields to result types
2. Update `to_minimal()` to handle new optional fields
3. Modify extraction functions to populate new fields

**Phase 4: Doc Comment Rendering (2-3 hours)**
1. Update `format/text.rs` to display doc comments (first line or first 3 lines)
2. Integrate token budget with doc comment display
3. Add doc truncation logic and warning for budget exceeded

**Phase 5: Polish and Testing (3-4 hours)**
1. End-to-end testing with real crates (serde, tokio)
2. Test filter performance with large indexes
3. Test token budget exhaustion scenarios
4. Verify minimal mode produces token-efficient output

**Total Estimated Time:** ~11-16 hours

### Performance Considerations

**Query Latency Impact:**
- Path resolution: O(n) — unchanged
- Content extraction: O(m) — unchanged
- Filter application: O(m×p) — m=items, p=patterns (target <5% overhead)
- Doc rendering: O(d) — unchanged (d=doc length)

**Memory Usage:**
- Index structure: ~100KB/crate — unchanged
- Compiled patterns: ~100 bytes — negligible
- Result structures: ~250 bytes/match — +25%

**Token Budget:**
- Base overhead: 30 tokens per item
- Doc comment: ~10-20 tokens per line
- Field list: ~5 tokens per field
- **Recommendation:** Set default token budget to 2000 tokens

### Success Metrics

**Functional Requirements:**
- [ ] All 24 ItemKind variants render correctly
- [ ] Glob patterns match as expected (glob library tests)
- [ ] Doc comments display in all rendering modes
- [ ] Token budget enforcement works for docs
- [ ] Visibility and deprecation fields exposed
- [ ] Minimal mode omits new fields (backward compatible)

**Performance Requirements:**
- [ ] Filtering overhead < 5% of total query time
- [ ] Doc comment display < 20% of token budget
- [ ] Memory usage < 50MB for serde crate (no increase)
- [ ] Query latency unchanged (<100ms with cache)

**UX Requirements:**
- [ ] Unified rendering across depths
- [ ] Clear indication of truncated docs ("...")
- [ ] Helpful error messages for invalid patterns
- [ ] Minimal mode produces < 500 tokens for complex types
- [ ] Filter flags intuitive and well-documented

## Confidence Assessment

| Area | v1.0 Confidence | v1.1 Confidence | Notes |
|------|-----------------|-----------------|-------|
| Stack | **HIGH** | **HIGH** | All technologies are official/de-facto standards |
| Features | **HIGH** | **HIGH** | Clear differentiation strategy based on LLM constraints |
| Architecture | **HIGH** | **HIGH** | Four-layer pattern proven; Filter Engine follows established patterns |
| Pitfalls | **HIGH** | **HIGH** | All pitfalls documented in rustdoc-types docs, RFCs, and production tools |

**Overall confidence:** HIGH (both v1.0 and v1.1)

v1.1 builds on v1.0's solid foundation. The primary uncertainty is not "what to build" but "performance characteristics at scale" — which is an implementation detail to be resolved during Phase 5-6. All new features (filtering, unified rendering, doc extraction) leverage existing infrastructure (token budget, graph index) and established patterns (glob matching, markdown parsing).

### Key Design Decisions

**Why glob instead of regex?**
- Cargo uses glob patterns (e.g., `**/*.rs`)
- Already in Cargo.toml (glob crate 0.3.1)
- Sufficient for item path matching
- Easier to extend if needed later

**Why not full source code extraction?**
- Massive output, token bloat, parsing complexity
- Show signatures and doc comments only
- Addresses token budget constraints

**Why optional fields for rich metadata?**
- Not all items have all metadata
- Maintain backward compatibility
- Minimal mode can omit unused fields

**Why single render function for all ItemKind?**
- Consistent output across depths
- Avoids Pitfall 1: Inconsistent Depth-Based Formatting
- Simplifies maintenance and testing

## Gaps to Address

**During Phase 1 (Foundation):**
- **Nightly toolchain handling:** Exact mechanism for detecting and invoking nightly rustdoc — verify rustdoc-json crate behavior with various toolchain configurations

**During Phase 2 (Index & Query Core):**
- **Graph schema details:** Specific Node and Edge variants needed — prototype with 2-3 representative crates to validate schema covers query needs

**During Phase 4 (Filter Integration):**
- **Filter performance with large crates:** Optimize pattern matching on crates with thousands of items — benchmark before/after implementation

**During Phase 5 (Output Refinement):**
- **Token budget edge cases:** Complex generic types with nested doc comments — test truncation behavior on deeply nested structures
- **Filter pattern validation:** User-friendly error messages for invalid glob patterns — add clear feedback on pattern syntax errors

**During Phase 6 (Performance Optimization):**
- **Large crate performance:** Memory usage and query latency on aws-sdk-ec2-scale crates — requires real-world testing (>500MB JSON)
- **Filter optimization:** Pattern compilation and reuse across queries — ensure minimal performance impact

### Quick Reference: Key v1.1 Features

| Feature | Complexity | Dependencies | v1.0 Base | Value |
|---------|------------|--------------|-----------|-------|
| Unified Rendering | MEDIUM | rustdoc-types | ❌ | Consistent output across all item types |
| Filtering | HIGH | glob crate | ❌ | Precise queries with glob patterns |
| Doc Comments | HIGH | existing TokenConfig | ❌ | Full markdown extraction and truncation |
| Field Discovery | MEDIUM | rustdoc-types | ❌ | Rich metadata (visibility, attrs, generics) |
| Filter Engine | LOW | glob crate | ❌ | Efficient pattern matching component |

### Competitive Differentiation v1.1

| Feature | docs.rs | ripdoc | rust-docs-mcp | Our Approach |
|---------|---------|--------|---------------|--------------|
| **Unified rendering** | Yes (web UI) | Yes (markdown) | Yes (JSON) | YES (CLI + JSON) |
| **Pattern filtering** | No | Partial (grep) | No | YES (glob patterns) |
| **Token-aware truncation** | No | No | No | YES (smart truncation) |
| **Rich field discovery** | Limited | No | No | YES (visibility, attrs, etc.) |
| **Generic parameter display** | Yes | Partial | No | YES (full display) |
| **Minimal mode** | No | No | No | YES (token-efficient) |

**Key Differentiators:**
1. **Token-Aware Truncation:** Unlike other tools, we intelligently truncate documentation to fit token budgets while preserving important content
2. **Pattern-Based Filtering:** Flexible glob-based filtering is rare in CLI tools; most offer only visibility filtering
3. **Rich Field Discovery:** Extract and display all metadata from rustdoc JSON for complete type information

## Sources

### Primary (v1.1 HIGH confidence)
- **docs.rs/rustdoc-types** (0.57.x) — Official API documentation, all 24 ItemKind variants
- **docs.rs/glob** (0.3.1) — Glob pattern matching for filters
- **RFC 2963** — rustdoc JSON format specification
- **RFC 3673** — rustdoc-types official maintenance by rustdoc team
- **GitHub: ripdoc** (Alb-O) — Markdown output focus, CLI UX patterns, output refinement inspiration
- **GitHub: rust-docs-mcp** — Comprehensive JSON output, field discovery patterns
- **GitHub: cargo-public-api** (Enselic) — Architecture reference, command patterns

### Secondary (v1.1 MEDIUM confidence)
- **JSON Schema Draft 7** — Output schema definition (Appendix B in STACK-v1.1)
- **Markdown specification** — Doc comment rendering standards (Appendix C in STACK-v1.1)
- **llmstxt.org** — Emerging llms.txt format standard
- **Context window limits research** — Model specifications change frequently; verify current limits during implementation

### Tertiary
- **Token management in LLMs** — Context window constraints, truncation strategies
- **Documentation formatting** — Rust documentation standards

---

*Research completed: 2026-02-12 (v1.0), 2026-02-13 (v1.1)*
*Ready for roadmap: yes*
