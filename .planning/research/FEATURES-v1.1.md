# Feature Research: Output Refinement (v1.1)

**Domain:** Rust crate API documentation querying - output refinement features
**Researched:** 2026-02-13
**Confidence:** HIGH (based on API documentation tool patterns + LLM context management research)

## Feature Landscape for Output Refinement

### Table Stakes (Users Expect These)

Features that API documentation tools must have. Missing these = unusable for documentation exploration.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Unified item kind rendering** | Users expect consistent display regardless of item type (function, struct, enum) | MEDIUM | Uniform format for modules, functions, structs, enums, traits |
| **Visibility filtering** | Users need to filter by public vs private items | LOW | Basic visibility filtering is standard expectation |
| **Whitespace normalization** | Clean output without excessive indentation or newlines | LOW | Essential for human-readable output and piping |
| **Order preservation** | Maintain declaration order from source | LOW | Helps users follow code structure |
| **Basic name matching filtering** | Filter results by substring matching | LOW | Essential for finding items quickly |

### Differentiators (Competitive Advantage)

Features that set this tool apart from competitors (docs.rs, ripdoc, rust-docs-mcp).

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Pattern-based include/exclude filtering** | Flexible pattern matching using glob/syntax patterns (e.g., `--include "*test*" --exclude "*private*"`) | HIGH | Differentiates from simple visibility filters |
| **Token-aware doc comment truncation** | Smart truncation that preserves code blocks and signatures | HIGH | Critical for LLM context efficiency |
| **Rich field discovery (visibility, attributes, generics, const generics)** | Extract all metadata fields from rustdoc JSON that are typically missing from output | MEDIUM | Makes debugging and understanding easier |
| **Item grouping and indentation** | Smart grouping of related items (e.g., methods of a struct) | MEDIUM | Improves readability of hierarchical output |
| **Multiple output style profiles** | Predefined styles (compact, standard, verbose) for different use cases | LOW | Saves users from specifying individual flags |
| **Line number references** | Include source file and line number references | HIGH | Helps users navigate to source code |
| **Generic parameter rendering** | Full generic parameter display with bounds | MEDIUM | Essential for understanding complex types |
| **Associated items with context** | Display associated constants, types, and methods with their parent | MEDIUM | Complete trait and impl information |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem helpful but create complexity or conflicts.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Full source code extraction** | "Show me the complete source" | Massive output, token bloat, parsing complexity | Show signatures and doc comments only |
| **Type annotation in output** | "Tell me what types are involved" | Already visible in signatures, adds noise | Keep signatures clean |
| **Line-by-line rendering** | "Exactly match source code appearance" | Ignores grouping, harder to parse | Render by item groups, not source lines |
| **Per-item formatting options** | "Custom formatting per item type" | Unmaintainable configuration | Provide group-level styles |
| **Rich text formatting** | "Bold keywords, add colors" | Breaks piping, increases token count | Plain text, use shell for coloring |
| **Complex query syntax** | "SQL-like queries over documentation" | Steep learning curve, duplicates cargo check | Simple glob patterns + filters |
| **Real-time file watching** | "Update when I save" | Daemon complexity, conflicts with cache model | Explicit rebuild triggers |

## Output Refinement Features

### 1. Unified Item Kind Rendering

**What:** Consistent output format across all item types (modules, functions, structs, enums, traits, impls, constants, type aliases).

**Implementation:** Single render function that takes an `ItemKind` enum and produces uniform output.

**Complexity:** MEDIUM
- Requires type-specific formatting logic
- Need to handle trait items vs impl items specially
- Associated items need special indentation

**Dependencies:**
- Requires rustdoc_types crate for item kind definitions
- Requires Index/Graph query system for item retrieval

**Example Output:**

```
Module: my_crate::utils
─────────────────────────────────────────────────────────────
Function: parse_json(input: &str) -> Result<Value, Error>
  └─ Doc: Parses JSON string into Value
  └─ Visibility: pub
  └─ Source: src/utils.rs:12

Struct: Config
─────────────────────────────────────────────────────────────
  Field: debug: bool
  Field: version: String

  Method: new() -> Self
    └─ Doc: Creates new config with defaults

  Method: save(&self) -> Result<(), Error>
    └─ Doc: Saves config to disk

Trait: Display
─────────────────────────────────────────────────────────────
  Method: fmt(&self, f: &mut Formatter<'_>) -> Result<()>
    └─ Doc: Formats using given formatter
```

**Table Stakes:** YES — Users expect consistent formatting across item types
**Differentiators:** Partial — Most tools do basic unified rendering

---

### 2. Include/Exclude Filtering

**What:** Pattern-based filtering using Unix-style globs and regex patterns.

**Implementation:** Filter item lists before rendering based on glob patterns.

**Complexity:** HIGH
- Needs glob pattern matching library (glob, globset)
- Need to handle path component separation (module::Function vs Module::function)
- Performance considerations for large crates (needs efficient filtering)
- Support for negative matches in exclude patterns

**Dependencies:**
- `glob` or `globset` crate for pattern matching
- rustdoc_types crate for item path extraction

**Example Usage:**

```bash
# Include only items matching "test*" pattern
cargo doc-query methods my_crate --include "*test*"

# Exclude private items
cargo doc-query expand my_crate --exclude "*private*"

# Multiple patterns
cargo doc-query expand std --include "alloc*" --exclude "*test*"

# Recursive pattern matching
cargo doc-query expand rayon --include "*parallel*"
```

**Table Stakes:** NO — Basic filtering exists in most tools
**Differentiators:** YES — Flexible pattern-based filtering is rare

---

### 3. Doc Comment Truncation

**What:** Token-aware truncation of documentation text to fit within token budgets while preserving important content (code blocks, signatures, key terms).

**Implementation:** Truncation algorithm that:
1. Prioritizes content by importance (code blocks > signatures > paragraphs)
2. Trims whitespace and newlines aggressively
3. Preserves markdown syntax
4. Adds "..." indicators at truncation points

**Complexity:** HIGH
- Requires markdown parsing to identify code blocks
- Need to count tokens (not characters) for accurate budgeting
- Must handle edge cases (truncation in middle of sentence, etc.)
- Performance: token counting can be expensive on long docs

**Dependencies:**
- `markdown` crate for parsing and token counting
- `token-counting` crate or custom implementation
- Existing token budget constraint system

**Example Behavior:**

```rust
// Long doc comment
/// Creates a new vector with capacity N. This is useful for when you know the exact size
/// you need. The vector will not reallocate memory until it has grown beyond N elements.
/// 
/// # Examples
/// 
/// ```
/// let mut vec = Vec::with_capacity(100);
/// ```
///
/// # Panics
/// 
/// This function will panic if N is larger than the capacity of the heap.
///
/// # Performance
/// 
/// This function is O(1) and does not allocate heap memory unless N exceeds the current capacity.
///
/// # Thread Safety
/// 
/// This function is thread-safe.

// Output with 50-token budget:
/// Creates a new vector with capacity N. This is useful for when you know
/// the exact size you need... (324 more tokens)
```

**Table Stakes:** NO — Token budget is already implemented
**Differentiators:** YES — Token-aware intelligent truncation is unique

---

### 4. Field Discovery

**What:** Extract and display rich metadata fields from rustdoc JSON that are commonly missing from output, including:
- Visibility modifiers (pub, pub(crate), pub(super), pub(in path))
- Item attributes (#[derive(Debug)] etc.)
- Generic parameter information
- Const generic parameters
- Safety attributes (#[unsafe], #[must_use])
- Stability attributes (#[stable], #[unstable])
- Deprecated status with replacement hints
- Source file references and line numbers

**Implementation:** Map rustdoc JSON fields to display fields, create rich metadata structures.

**Complexity:** MEDIUM
- rustdoc JSON structure varies by item kind
- Need to handle attribute parsing and display
- Source location extraction
- Generic parameter resolution

**Dependencies:**
- rustdoc_types crate for JSON structure access
- `regex` crate for attribute parsing
- Existing Index/Graph query system

**Example Output:**

```rust
// Input: rustdoc JSON for a method

// Output with full field discovery:
/// Parses JSON string into Value.
///
/// # Errors
/// Returns Err if input is not valid JSON.
///
/// # Examples
///
/// ```
/// let result = parse_json("{\"a\": 1}").unwrap();
/// ```
#[must_use]
pub fn parse_json(input: &str) -> Result<Value, Error>
  └─ Source: src/utils.rs:12
  └─ Line: 12
  └─ Visibility: pub
  └─ Attributes: [must_use]
  └─ Generics: <T: Serialize, E: DeserializeOwned>
  └─ Safety: Safe
  └─ Stability: stable
```

**Table Stakes:** NO — Most tools show basic information
**Differentiators:** YES — Rich field discovery improves debugging

---

### 5. Item Grouping and Indentation

**What:** Smart grouping of related items to improve readability. Examples:
- Group all methods of a struct together
- Group trait implementations
- Display structs within their defining modules
- Hierarchical indentation based on scope

**Implementation:** Sort items by hierarchy, apply indentation based on scope, group by type.

**Complexity:** MEDIUM
- Need to maintain hierarchical relationships
- Custom sorting logic
- Indentation calculations

**Dependencies:**
- Existing Index/Graph query system for relationships
- rustdoc_types crate for hierarchy information

**Example Output:**

```
my_crate::config (Module)
├─ Config (Struct)
│  ├─ Field: debug: bool
│  ├─ Field: version: String
│  └─ Method: new() -> Self
│     └─ Doc: Creates new config
├─ ConfigError (Enum)
│  └─ Variant: InvalidPath(String)
└─ load() -> Result<Config, ConfigError>
   └─ Doc: Loads config from file
```

**Table Stakes:** NO — Basic hierarchy exists
**Differentiators:** Partial — Good grouping improves readability

---

### 6. Multiple Output Style Profiles

**What:** Predefined output styles that combine various options:
- **compact:** Minimal whitespace, no doc comments, signatures only
- **standard:** Standard formatting, some doc comments, normal whitespace
- **verbose:** All metadata, full doc comments, detailed grouping
- **json:** Structured JSON output (already implemented)

**Implementation:** Style profiles that define combinations of flags/options.

**Complexity:** LOW
- Profiles are just flag combinations
- Could be defined as TOML or code constants

**Dependencies:**
- Existing flag system (--tokens, --minimal, etc.)
- Output rendering functions

**Example Usage:**

```bash
# Use compact profile
cargo doc-query methods my_crate --style compact

# Use verbose profile
cargo doc-query methods my_crate --style verbose

# Custom profile (future)
cargo doc-query methods my_crate --style custom --tokens 500 --include "*" --no-visibility
```

**Table Stakes:** NO — Format options exist
**Differentiators:** Partial — Style profiles improve UX

---

## Feature Dependencies

```
[Unified Item Rendering]
    ├──requires──> [Index/Graph Query System] (v1.0)
    ├──requires──> [Item Kind Definitions] (rustdoc-types)
    └──enhances──> [All Output Features]

[Include/Exclude Filtering]
    ├──requires──> [Pattern Matching Library]
    ├──requires──> [Index/Graph Query System] (v1.0)
    └──requires──> [Item Path Extraction]

[Token-Aware Doc Truncation]
    ├──requires──> [Token Budget System] (v1.0)
    ├──requires──> [Markdown Parsing]
    └──requires──> [Token Counting]
        └──requires──> [Unified Item Rendering]

[Field Discovery]
    ├──requires──> [rustdoc-types JSON Structure]
    ├──requires──> [Source Location System]
    └──requires──> [Attribute Parsing]
        └──requires──> [Unified Item Rendering]

[Item Grouping and Indentation]
    ├──requires──> [Index/Graph Query System] (v1.0)
    ├──requires──> [Module Hierarchy] (rustdoc_types)
    └──requires──> [Unified Item Rendering]

[Multiple Output Style Profiles]
    └──requires──> [All Output Format Options]
        ├──requires──> [Token Budget System] (v1.0)
        ├──requires──> [Include/Exclude Filtering]
        └──requires──> [Unified Item Rendering]
```

### Dependency Notes

- **Unified Item Rendering is foundational:** All output refinement features depend on having a consistent item representation
- **Include/Exclude Filtering requires robust pattern matching:** Performance is critical for large crates (hundreds of items)
- **Token-Aware Truncation requires token counting:** This is computationally expensive, may need caching or lazy evaluation
- **Field Discovery requires parsing rustdoc JSON:** Some fields are optional or have complex nesting
- **Item Grouping requires hierarchical relationships:** Must traverse the item graph to find parent modules and related items

## MVP Definition for v1.1

### Must Have (Launch with)

- [ ] **Unified Item Rendering** — Consistent format for all item types
- [ ] **Visibility Filtering** — Filter by public/private items
- [ ] **Whitespace Normalization** — Clean, readable output
- [ ] **Basic Name Matching** — Filter by substring matching

### Should Have (Add After Validation)

- [ ] **Rich Field Discovery** — Visibility, attributes, generics, etc.
- [ ] **Item Grouping and Indentation** — Hierarchical display
- [ ] **Multiple Output Style Profiles** — Predefined styles
- [ ] **Generic Parameter Rendering** — Full generic type display

### Nice to Have (Future Consideration)

- [ ] **Pattern-based Include/Exclude** — Glob patterns for filtering
- [ ] **Token-Aware Doc Comment Truncation** — Smart token-aware truncation
- [ ] **Line Number References** — Source location information
- [ ] **Associated Item Context** — Complete trait/impl information

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Dependencies | Priority |
|---------|------------|---------------------|--------------|----------|
| Unified Item Rendering | HIGH | MEDIUM | v1.0 Index | P1 |
| Visibility Filtering | HIGH | LOW | v1.0 Index | P1 |
| Whitespace Normalization | HIGH | LOW | v1.0 Rendering | P1 |
| Basic Name Matching | HIGH | LOW | v1.0 Index | P1 |
| Rich Field Discovery | MEDIUM | MEDIUM | rustdoc_types | P2 |
| Item Grouping | MEDIUM | MEDIUM | v1.0 Index | P2 |
| Style Profiles | MEDIUM | LOW | All formatting | P2 |
| Generic Rendering | MEDIUM | LOW | rustdoc_types | P2 |
| Pattern Filtering | MEDIUM | HIGH | globset | P3 |
| Token-Aware Truncation | HIGH | HIGH | token counting | P3 |
| Line Number References | LOW | MEDIUM | rustdoc_types | P3 |

**Priority key:**
- P1: Must have for v1.1 launch
- P2: Should have for v1.1
- P3: Nice to have, add if time permits

## Competitive Analysis for Output Refinement

| Feature | docs.rs | ripdoc | rust-docs-mcp | Our Approach |
|---------|---------|--------|---------------|--------------|
| **Unified rendering** | Yes (web UI) | Yes (markdown) | Yes (JSON) | YES (CLI + JSON) |
| **Visibility filtering** | Yes (tab) | Partial (grep) | No | YES (CLI flag) |
| **Pattern filtering** | No | Partial (grep) | No | YES (glob patterns) |
| **Doc comment truncation** | No | No | No | YES (token-aware) |
| **Rich field discovery** | Limited | No | No | YES (visibility, attrs, etc.) |
| **Item grouping** | Yes (hierarchical) | Limited | No | YES (smart grouping) |
| **Style profiles** | No | No | No | YES (multiple styles) |
| **Generic parameter display** | Yes | Partial | No | YES (full display) |
| **Source line references** | Yes | No | No | YES (include if possible) |

### Key Differentiation Strategy

1. **Token-Aware Truncation:** Unlike other tools, we intelligently truncate documentation to fit token budgets while preserving important content
2. **Pattern-Based Filtering:** Flexible glob-based filtering is rare in CLI tools; most offer only visibility filtering
3. **Rich Field Discovery:** Extract and display all metadata from rustdoc JSON for complete type information
4. **Style Profiles:** Predefined output styles reduce configuration burden

## LLM Context Considerations

### Output Format Recommendations

**For LLM Agents (Rich Field Discovery):**
```json
{
  "type": "struct",
  "name": "Config",
  "visibility": "pub(crate)",
  "attributes": ["derive(Debug, Clone)"],
  "generics": "<T: Serialize>",
  "source": "src/config.rs:5",
  "line": 5,
  "doc": "Configuration settings for the application...",
  "fields": [
    {
      "name": "debug",
      "visibility": "pub",
      "type": "bool",
      "doc": "Enable debug logging"
    },
    {
      "name": "version",
      "visibility": "pub",
      "type": "String",
      "doc": "Application version"
    }
  ],
  "methods": [
    {
      "name": "new",
      "signature": "fn new() -> Self",
      "visibility": "pub",
      "doc": "Creates new config with defaults",
      "source": "src/config.rs:15",
      "line": 15
    }
  ]
}
```

**For CLI Humans (Standard Style):**
```
my_crate::Config (Struct)
  Visibility: pub(crate)
  Attributes: [derive(Debug, Clone)]
  Generics: <T: Serialize>
  Source: src/config.rs:5
  Line: 5

  Field: debug: bool
    └─ Visibility: pub
    └─ Doc: Enable debug logging

  Field: version: String
    └─ Visibility: pub
    └─ Doc: Application version

  Method: new() -> Self
    └─ Visibility: pub
    └─ Doc: Creates new config with defaults
    └─ Source: src/config.rs:15
    └─ Line: 15
```

**For Minimal Mode (Token Efficiency):**
```
struct Config { debug: bool, version: String }
fn new() -> Self (pub(crate), derives(Debug, Clone))
```

## Implementation Notes

### Rendering Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Rendering Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Item Kind  │  │   Style      │  │    Truncation    │  │
│  │   Renderer   │  │    Profile   │  │    Engine        │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
└─────────┴─────────────────┴────────────────────┴────────────┘
        ↓                     ↓                      ↓
┌─────────────────────────────────────────────────────────────┐
│                   Data Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │    Index     │  │   Metadata   │  │   Source Loc     │  │
│  │  (v1.0)      │  │   Extractor  │  │   Extractor      │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Separation of Rendering from Querying:** Keep rendering logic separate from index/graph logic for maintainability
2. **Rich Metadata as Optional:** Field discovery fields should be optional in output; not all items have all metadata
3. **Token Counting on Demand:** Only count tokens when needed (e.g., before truncating docs), not on every query
4. **Pattern Matching Performance:** Pre-compile glob patterns before filtering; reuse across multiple queries
5. **Style Profiles as Composition:** Profiles should be composable (e.g., `compact = --minimal --no-doc`)

## Sources

### API Documentation Tools Analyzed
- **docs.rs** (https://docs.rs/) — Web-based documentation search
- **ripdoc** (https://github.com/Alb-O/ripdoc) — Markdown-focused documentation tool
- **rust-docs-mcp** (https://lib.rs/crates/rust-docs-mcp) — MCP server for LLM documentation access
- **cargo-public-api** (https://github.com/cargo-public-api/cargo-public-api) — API surface diffing tool

### Rust Documentation Standards
- **rustdoc JSON format** (https://rust-lang.github.io/rfcs/2963-rustdoc-json.html) — Official JSON structure
- **rustdoc-types crate** (https://docs.rs/rustdoc-types) — TypeScript definitions mirroring JSON
- **llms.txt specification** (https://llmstxt.org/) — Emerging standard for LLM-friendly docs

### LLM Context Management
- **Token budget strategies** (https://agenta.ai/blog/top-6-techniques-to-manage-context-length-in-llms) — Best practices for token efficiency
- **Context window limits** (https://github.com/taylorwilsdon/llm-context-limits) — Current model limits

### Pattern Matching Libraries
- **glob crate** (https://docs.rs/glob) — Rust glob pattern matching
- **globset crate** (https://docs.rs/globset) — Fast glob pattern filtering with exclusion
- **ignore crate** (https://docs.rs/ignore) — Ignore file parsing with glob support

---

*Feature research for: cargo-doc-query v1.1 output refinement*
*Researched: 2026-02-13*
