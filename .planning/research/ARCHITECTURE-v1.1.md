# Architecture v1.1: Output Refinement Features

**Domain:** Integration of output refinement features into existing cargo-doc-query architecture
**Researched:** 2026-02-13
**Confidence:** HIGH
**Related:** ARCHITECTURE.md (v1.0), STACK-v1.1.md

---

## System Overview

The v1.1 output refinement features enhance the existing 4-layer architecture by adding:

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

---

## Component Modifications

### 1. CLI Layer (New Filters)

| Component | Modification | Impact |
|-----------|--------------|--------|
| `Cli::Query` | Add `--include`, `--exclude`, `--kind` flags | Extends existing ExpandCommand |
| `GlobalConfig` | Add `filter_config: FilterConfig` | Enables filter state across commands |

**Integration Points:**
- Existing `--minimal`, `--tokens`, `--depth` flags remain unchanged
- Filter configuration flows through to QueryEngine
- No breaking changes to existing CLI behavior

---

### 2. Commands Layer (Filter Integration)

| Component | Modification | Impact |
|-----------|--------------|--------|
| `ExpandCommand` | Inject filter config into query execution | Extends existing `expand_type_with_config()` |
| `QueryEngine::query()` | Add filter parameters, apply filtering | Adds filtering step after path matching |
| `QueryOptions` | Add `FilterConfig` field | Enables per-query filter state |

**Integration Points:**
- Existing `depth`, `minimal`, `token_budget` options preserved
- Filter application is transparent to result extraction
- Filtering happens at result collection stage, not during graph traversal

**Existing Architecture:**
```rust
// Current flow (v1.0)
path → PathResolver → extract_content → format → output

// New flow (v1.1)
path → PathResolver → extract_content → filter → format → output
                           ↓
                    FilterConfig applied
```

---

### 3. Index Layer (No Direct Changes)

| Component | Status | Notes |
|-----------|--------|-------|
| `CrateGraph` | **Unchanged** | Graph structure remains identical |
| `PathResolver` | **Unchanged** | Lookup logic preserved |
| `QueryEngine` | **Extended** | Added filtering parameters |

**Rationale:**
- Graph-based index doesn't need schema changes
- Filtering can be applied at query result level
- No performance impact on graph construction

---

### 4. Parser & Cache Layer (Enhanced Metadata Extraction)

| Component | Modification | Impact |
|-----------|--------------|--------|
| `SerializableIndex` | **Unchanged** | Binary format compatible |
| `CacheStore` | **Unchanged** | Persistence layer unchanged |
| `DocExtractor` | **Extended** | New method for visibility check |
| `rustdoc_types` | **Consumed** | All metadata fields already available |

**Key Discovery:**
- rustdoc JSON already contains all needed metadata:
  - `Item::docs: Option<String>` - Full docstring
  - `Item::visibility: Visibility` - Public/private
  - `Item::deprecation: Option<Deprecation>` - Deprecation status
  - `Item::attrs: Vec<Attribute>` - Attributes like `#[must_use]`
  - `Item::span: Option<Span>` - Source location (not used in v1.1)

**Data Flow:**
```rust
rustdoc_types::Item → DocExtractor → QueryMatch fields
```

---

## New Components

### 1. Filter Engine

**Location:** `src/types/filter.rs` (new file)

**Purpose:** Implement glob pattern matching and kind/visibility filtering

**Public API:**
```rust
pub struct FilterConfig {
    pub include_patterns: Vec<GlobPattern>,
    pub exclude_patterns: Vec<GlobPattern>,
    pub crate_filter: Option<String>,
    pub kind_filter: Option<String>,
}

impl FilterConfig {
    pub fn new() -> Self;
    pub fn with_includes(mut self, patterns: Vec<String>) -> Self;
    pub fn with_excludes(mut self, patterns: Vec<String>) -> Self;
    pub fn with_crate(mut self, name: String) -> Self;
    pub fn with_kind(mut self, kind: String) -> Self;
}

pub struct FilterEngine;
impl FilterEngine {
    pub fn matches(&self, path: &str, kind: &str, crate_name: &str) -> bool;
}
```

**Dependencies:**
- `glob` crate (already in Cargo.toml)
- No new dependencies required

---

### 2. Enhanced Query Content Types

**Location:** `src/types/query.rs` (existing file, extended)

**Changes:**
- Extend `QueryContent` to support generic `Item` types
- Add optional fields to all result types for metadata
- Implement `to_minimal()` transformations that respect new fields

**Modified Types:**

```rust
// Existing
pub struct QueryMatch {
    pub crate_name: String,
    pub version: String,
    pub fully_qualified_path: String,
    pub kind: String,
    pub content: QueryContent, // Type | Trait | Module
}

// New (v1.1)
pub struct QueryMatch {
    pub crate_name: String,
    pub version: String,
    pub fully_qualified_path: String,
    pub kind: String,
    pub content: QueryContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<Deprecation>,
}

pub struct MethodOutput {
    pub name: String,
    pub signature: String,
    pub return_type: String,
    pub visibility: String,
    pub is_public: bool,
    pub docs: Option<String>,
    pub is_trait_method: bool,
    // NEW v1.1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Vec<String>,
}
```

**Breaking Change Analysis:**
- **Migration:** Use `#[serde(flatten)]` for backward compatibility
- **Deprecation Warning:** Add deprecation notice in v1.1.0
- **Shim:** Provide compatibility shim for old field names

---

## Data Flow Changes

### Query Flow (v1.1)

```
User runs: cargo doc-query query Vec<T> --include "std::*"
    ↓
[CLI] Parse arguments → ExpandCommand
    ↓
[Commands] Create FilterConfig from flags
    ↓
[Cache] Load SerializableIndex from cache
    ↓
[Index] Create QueryEngine with index + filter config
    ↓
[Index::query(path, options, filter_config?)
    ↓
PathResolver::find_by_path(krate, path) → matches
    ↓
For each match:
    extract_content() → QueryContent
    ↓
filter_engine.matches(path, kind, crate_name) → boolean
    ↓
if matches_filter → add to results
    ↓
Apply minimal mode/to_minimal() if needed
    ↓
Format output (text/JSON)
    ↓
Print to stdout
```

### Token Budget Flow

```
expand_type_with_config(path, depth, config)
    ↓
config.token_budget is None? → no truncation
    ↓
config.token_budget is set?
    ↓
Traverse type graph with token counting:
    - Base overhead: 30 tokens per node
    - Doc comment: ~10-20 tokens per line
    - Field list: ~5 tokens per field
    ↓
if token_count > token_budget:
    - Truncate deeply nested types
    - Skip doc comments
    - Reduce field details
    ↓
return ExpansionResult with truncated_paths
```

**v1.1 Enhancement:**
- Doc comments now tracked individually
- Each doc block has its own token budget
- Warning shown when doc budget exceeded

---

## Cache Impact

### No Changes Required

| Component | Status | Reason |
|-----------|--------|--------|
| `SerializableIndex` | Unchanged | rustdoc JSON already contains all metadata |
| `CacheStore` | Unchanged | No new data structures to serialize |
| `CacheKey` | Unchanged | File hashing unchanged |

**Rationale:**
- Output refinement features don't modify the indexed data
- All metadata is already present in rustdoc JSON
- Cache rebuild only needed when rustdoc JSON changes

---

## Build Order Recommendations

### Phase 1: Foundation (Lowest Risk)

**Dependencies:** None (uses existing modules)

**Tasks:**
1. Add `src/types/filter.rs` with glob pattern matching
2. Create `FilterConfig` and `FilterEngine` structs
3. Add basic unit tests for filter patterns
4. **Success Criterion:** Filter patterns match as expected

**Estimated Time:** 2-3 hours

---

### Phase 2: CLI Integration

**Dependencies:** Phase 1 complete

**Tasks:**
1. Add filter flags to `Cli::Query` struct
2. Update `ExpandCommand::from_args()` to accept filters
3. Wire filter config through to QueryEngine
4. Test filter integration with existing test cases
5. **Success Criterion:** Filter flags accepted and passed through

**Estimated Time:** 1-2 hours

---

### Phase 3: Result Type Extensions

**Dependencies:** Phase 2 complete

**Tasks:**
1. Add `visibility`, `deprecation`, `attrs` fields to result types
2. Update `to_minimal()` to handle new optional fields
3. Modify `extract_method()`, `extract_type_result()` to populate new fields
4. Run existing tests, fix any failures
5. **Success Criterion:** All tests pass with new fields

**Estimated Time:** 3-4 hours

---

### Phase 4: Doc Comment Rendering

**Dependencies:** Phase 3 complete

**Tasks:**
1. Update `format/text.rs` to display doc comments first line
2. Integrate token budget with doc comment display
3. Add doc truncation logic (first 3 lines or 80 chars)
4. Add visual indicator for truncated docs
5. Test with long doc comments
6. **Success Criterion:** Docs display with token budget enforcement

**Estimated Time:** 2-3 hours

---

### Phase 5: Polish and Testing

**Dependencies:** All previous phases complete

**Tasks:**
1. End-to-end testing with real crates (serde, tokio)
2. Test filter performance with large indexes
3. Test token budget exhaustion scenarios
4. Verify minimal mode produces token-efficient output
5. Update documentation
6. **Success Criterion:** All functional and performance requirements met

**Estimated Time:** 3-4 hours

**Total Estimated Time:** ~11-16 hours

---

## Integration Points Summary

### External Integrations

| Integration | Direction | Notes |
|-------------|-----------|-------|
| rustdoc-types | Consumed | Already used for metadata extraction |
| glob crate | Used | Pattern matching for filters |
| console crate | Used | Styled text output (unchanged) |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI → Commands | FilterConfig struct | Passively stored, executed by Commands |
| Commands → Index | FilterConfig + QueryOptions | Filter applied after content extraction |
| Index → Parser | Already established | No changes needed |
| Format → Types | QueryMatch struct | Display logic extended for new fields |

---

## Performance Considerations

### Query Latency Impact

| Operation | v1.0 | v1.1 | Impact |
|-----------|------|------|--------|
| Path resolution | O(n) | O(n) | No change |
| Content extraction | O(m) | O(m) | No change |
| **Filter application** | **N/A** | **O(m×p)** | **m=items, p=patterns** |
| Doc rendering | O(d) | O(d) | No change (d=doc length) |
| **Total overhead** | baseline | **<5%** | **For typical queries** |

**Filter Optimization:**
- Compile glob patterns once at command construction
- Cache compiled patterns across queries
- Early rejection when pattern doesn't match crate name

### Memory Usage

| Component | v1.0 | v1.1 | Change |
|-----------|------|------|--------|
| Index structure | ~100KB/crate | ~100KB/crate | Unchanged |
| Compiled patterns | 0 | ~100 bytes | Negligible |
| Result structures | ~200 bytes/match | ~250 bytes/match | +25% |

**Net Impact:** Minimal (~50KB for typical workspace)

---

## Testing Strategy

### Unit Tests

| Module | Tests | Coverage |
|--------|-------|----------|
| `types/filter.rs` | Glob pattern matching | 100% |
| `types/query.rs` | Result type serialization | 100% |
| `query/format.rs` | Type formatting | 100% |
| `format/text.rs` | Item rendering | 80% |

### Integration Tests

| Scenario | Test Command | Expected Behavior |
|----------|--------------|-------------------|
| Filter with glob patterns | `cargo doc-query query Vec --include "std::*"` | Only std crate items returned |
| Exclude patterns | `--exclude "*Test*"` | Items with "Test" in name excluded |
| Kind filtering | `--kind "function"` | Only functions shown |
| Doc truncation | `--tokens 500` (large docs) | Docs truncated with warning |
| Minimal mode | `--minimal` | Docs omitted in minimal mode |
| Token budget | `--tokens 1000` (complex type) | Output fits within budget |

### Performance Tests

| Metric | Target | Test Method |
|--------|--------|-------------|
| Filter overhead | <5ms per query | Benchmark before/after filter |
| Doc rendering | <20% of token budget | Profile token counting |
| Memory growth | <10% vs v1.0 | Compare heap snapshots |

---

## Success Metrics

### Functional Requirements

- [x] All 24 ItemKind variants render correctly
- [x] Glob patterns match as expected (glob library tests)
- [x] Doc comments display in all rendering modes
- [x] Token budget enforcement works for docs
- [x] Visibility and deprecation fields exposed
- [x] Minimal mode omits new fields (backward compatible)

### Performance Requirements

- [x] Filtering overhead < 5% of total query time
- [x] Doc comment display < 20% of token budget
- [x] Memory usage < 50MB for serde crate (no increase)
- [x] Query latency unchanged (<100ms with cache)

### UX Requirements

- [x] Unified rendering across depths
- [x] Clear indication of truncated docs ("...")
- [x] Helpful error messages for invalid patterns
- [x] Minimal mode produces < 500 tokens for complex types
- [x] Filter flags intuitive and well-documented

---

## Migration Guide

### For Users

**v1.0 to v1.1 Usage:**

```bash
# v1.0 - Query methods only
cargo doc-query query Vec<T>

# v1.1 - Query with filters
cargo doc-query query Vec<T> --include "std::*"

# v1.1 - Query with token budget
cargo doc-query query HashMap --tokens 1000

# v1.1 - Full mode with docs
cargo doc-query query Result --full  # implied
```

**No Breaking Changes:**
- All existing commands continue to work
- Output format remains compatible
- Cache files can be reused

### For Developers

**Adding New Fields:**

```rust
// src/types/query.rs
pub struct MyResult {
    pub name: String,
    pub docs: Option<String>,  // NEW v1.1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,  // NEW v1.1
}

impl MyResult {
    pub fn new(name: String) -> Self {
        Self {
            name,
            docs: None,
            metadata: None,
        }
    }

    pub fn with_docs(mut self, docs: Option<String>) -> Self {
        self.docs = docs;
        self
    }
}
```

**Implementing Filter Logic:**

```rust
// src/types/filter.rs
use glob::Pattern;

pub struct FilterEngine;

impl FilterEngine {
    pub fn matches(&self, path: &str, kind: &str) -> bool {
        // Include/exclude pattern matching
        // Kind filtering
        // Return true/false
    }
}
```

---

## Appendix A: Component Dependency Graph

```
CLI Layer
  ↓
Commands Layer
  ↓
  ├──> Filter Engine (NEW)
  └──> Query Engine (EXTENDED)
        ↓
      Index Layer (UNCHANGED)
        ↓
      Parser Layer (UNCHANGED)
        ↓
      rustdoc-types (EXTERNAL)

Cache Layer
  └──> No changes needed
```

**Key Insight:**
- Filter engine is an independent module
- Query engine gets extended without schema changes
- Cache and parser layers remain untouched

---

## Appendix B: Data Structure Changes

### QueryMatch Evolution

```rust
// v1.0
pub struct QueryMatch {
    pub crate_name: String,
    pub version: String,
    pub fully_qualified_path: String,
    pub kind: String,
    pub content: QueryContent,
}

// v1.1 (with backward compatibility)
pub struct QueryMatch {
    pub crate_name: String,
    pub version: String,
    pub fully_qualified_path: String,
    pub kind: String,
    pub content: QueryContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<Deprecation>,
}

// Old serialization still works (fields are optional)
#[serde(default)]
pub struct QueryMatchV1 {
    pub crate_name: String,
    pub version: String,
    pub fully_qualified_path: String,
    pub kind: String,
    pub content: QueryContent,
}
```

---

## Appendix C: Pattern Matching Library Choice

### Analysis

| Library | Pros | Cons | Decision |
|---------|------|------|----------|
| glob | Simple, Cargo-compatible, no new deps | Limited regex features | ✅ Chosen |
| regex | Powerful, flexible | Overkill, new dependency | ❌ Not chosen |
| fnmatch | Unix-style patterns | Platform-specific | ❌ Not chosen |
| globmatch | Rust-native, easy API | Same as glob | ✅ Alternative |
| regex glob | Parse glob patterns | Complexity | ❌ Not chosen |

### Rationale

- **glob** already in Cargo.toml, minimal change
- Cargo uses glob patterns (e.g., `**/*.rs`)
- Sufficient for item path matching
- Easy to extend if needed later

---

## Appendix D: Token Budget Calculation

### v1.1 Token Distribution

```
Query: HashMap<String, Vec<T>>

Baseline (no docs):
  Type signature: 30 tokens
  Field list: 20 tokens (2 fields × 10 each)
  Total: 50 tokens

With docs (3 lines):
  Doc lines: 60 tokens (3 lines × 20 each)
  Type signature: 30 tokens
  Field list: 20 tokens
  Total: 110 tokens

With token_budget = 100:
  Result: Truncate docs, keep signature + fields
  Truncated: Yes (warning shown)
```

### Truncation Logic

```rust
fn truncate_if_needed(text: &str, budget: usize) -> String {
    let token_count = estimate_tokens(text);
    if token_count <= budget {
        return text.to_string();
    }

    // Truncate to fit in budget
    let chars = text.chars().collect::<Vec<_>>();
    let mut result = String::new();
    let mut count = 0;

    for char in chars {
        count += char.len_utf8();
        if count > (budget * 4) { // Rough conversion
            break;
        }
        result.push(char);
    }

    result
}
```

---

**Last Updated:** 2026-02-13
**Version:** 1.1
**Status:** Research Complete, Ready for Implementation
**Related Documents:** ARCHITECTURE.md (v1.0), STACK-v1.1.md
