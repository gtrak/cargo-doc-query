# Output Refinement Pitfalls: API Documentation Tools

**Domain:** Adding unified rendering, filtering, and doc comment extraction to API documentation tools
**Researched:** 2026-02-13
**Confidence:** HIGH (based on analysis of ripdoc, rust-docs-mcp, and docs.rs implementations)
**Related to:** Existing PITFALLS.md (domain-specific, not generic)

## Unified Rendering Pitfalls

### Pitfall 1: Inconsistent Depth-Based Formatting

**What goes wrong:**
The same type is rendered differently at different nesting levels, confusing the user.

**Example:**
```
# Root level:
Vec<T> — methods: push, pop

# Nested level:
Vec<String>
  # Empty rendering, just shows name
  methods: into_vec

# Why it happens:**
Renders differently at different depths due to separate code paths for root vs nested items.

**How to avoid:**
1. **Single render function for all depths** — parameterize on `depth` or `indent_level`
2. **Define consistent output schema** — all items follow same structure regardless of position
3. **Use configuration for depth sensitivity** — let users control what changes at each depth
4. **Test with deeply nested types** — recursive structs, generic parameters, trait implementations
5. **Document rendering rules explicitly** — what's shown at depth 0, 1, 2...

**Warning signs:**
- Root-level output differs from nested-level
- Same item shown with different data at different depths
- Inconsistent indentation or whitespace
- User comments like "why is Vec<T> different from Vec<String>?"

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

### Pitfall 2: Generic Type Parameter Truncation

**What goes wrong:**
Generic types are truncated mid-parameters when hitting output size limits, losing essential type information.

**Example:**
```
# Should show:
std::collections::HashMap<String, Vec<u8>>

# Shows instead:
std::collections::HashMap<
```

**Why it happens:**
Token budget applies to full output but generic parameters aren't handled specially. Each `<` and `>` consumes tokens but provides no visible benefit until fully rendered.

**How to avoid:**
1. **Track generic parameters separately** — count them for budgeting
2. **Render generic bounds before parameters** — show `Vec<T: Clone + Debug>` even if `T` not expanded
3. **Provide truncated representation** — `[<T1, T2, ...>]` for many parameters
4. **Let users configure depth per generic** — fully expand first N parameters
5. **Add `--generic-depth` flag** — control how many levels to expand

**Warning signs:**
- Generic types cut off mid-definition
- Methods signatures show generic params as `T`, `U` but return types don't
- "Unknown" or `<unknown>` in type signatures
- Users asking for more generic type details

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

### Pitfall 3: Trait Bound Omission

**What goes wrong:**
Trait bounds are dropped or hidden, making generic code harder to understand.

**Example:**
```
# Should show:
fn map<U, F>(self, f: F) -> Map<Self, U>
where
    F: FnMut(T) -> U,
    Self: Sized,

# Shows instead:
fn map<U, F>(self, f: F) -> Map<Self, U>
```

**Why it happens:**
Boring boilerplate is often excluded by default to save tokens, but it's essential for understanding constraints.

**How to avoid:**
1. **Always include where clauses** — except when explicitly opted out
2. **Add `--include-traits` flag** — control whether to show trait bounds
3. **Display bounds prominently** — not just in a separate "where" clause
4. **Test with complex generics** — multiple bounds, nested generics
5. **Document bound visibility** — which ones appear by default

**Warning signs:**
- Generic functions missing where clauses
- Type errors when copying signatures
- Users confused about required traits
- "Why does this require `Sized` but yours doesn't?"

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

### Pitfall 4: Enum Variant vs Struct Field Inconsistency

**What goes wrong:**
Enums and structs render differently even though they both have named fields.

**Example:**
```
# Enum shows:
MyEnum::Variant {
    x: i32,
    y: i32,
}

# Struct shows:
MyStruct { x: i32, y: i32 }
```

**Why it happens:**
Separate rendering paths for enums and structs with different default formatting. User expects consistency.

**How to avoid:**
1. **Use unified field rendering** — same formatting for both
2. **Add `--variant-style` and `--struct-style` flags** — let users choose
3. **Document default differences** — why they exist, how to change
4. **Test both enums and structs** — ensure parity
5. **Check all field-containing types** — unions, tuples, etc.

**Warning signs:**
- Enum fields use braces, structs use parentheses
- Field visibility differs (e.g., `pub x` shown for structs only)
- Comments on fields only appear for structs
- User confusion about why enums look different

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

## Filtering Pitfalls

### Pitfall 5: Naive Name-Based Filtering Only

**What goes wrong:**
Only supports filtering by name, missing broader use cases like module-based filtering, attribute filtering, etc.

**Example:**
```
# Works:
cargo doc-query methods std::vec --filter "push"

# Doesn't work:
cargo doc-query methods std --filter "::vec::"
cargo doc-query methods std --filter "#[deprecated]"
```

**Why it happens:**
Start with simple pattern matching on item names, then realize users need more sophisticated filtering. Existing code is hard to extend.

**How to avoid:**
1. **Design filter system early** — not as an afterthought
2. **Support multiple filter types** — name, path, attribute, visibility
3. **Use regex or glob patterns** — not just exact matching
4. **Add filter documentation** — examples of all available options
5. **Plan for extensible filters** — plugins or custom matchers

**Warning signs:**
- Users asking for "filter by module"
- "Why can't I filter deprecated items?"
- Filter flags not matching intent
- Users working around limitations with grep

**Phase to address:**
Phase 4 (Query Engine & Filtering)

---

### Pitfall 6: Cross-Crate Name Ambiguity

**What goes wrong:**
Filtering doesn't disambiguate between identically-named items in different modules or crates.

**Example:**
```
# crate A::mod1::foo exists
# crate A::mod2::foo exists
# Filtering shows both with no distinction

# crate std::vec::Vec exists
# crate std::collections::Vec exists
# Filter by "Vec" gives mixed results
```

**Why it happens:**
Item paths are only considered in later stages. Filtering is done on name alone without considering crate/module context.

**How to avoid:**
1. **Filter with full path** — not just item name
2. **Add `--crate` and `--module` flags** — restrict scope
3. **Disambiguate ambiguous matches** — show all matches with paths
4. **Document path-based filtering** — how to use full paths
5. **Test with ambiguous names** — ensure clear output

**Warning signs:**
- Ambiguous matches not flagged
- User asking "Which Vec is this?"
- Filter results include items from wrong crates
- No way to disambiguate without full path

**Phase to address:**
Phase 4 (Query Engine & Filtering)

---

### Pitfall 7: No Attribute-Based Filtering

**What goes wrong:**
Cannot filter by documentation attributes like `#[deprecated]`, `#[doc(hidden)]`, `#[inline]`, etc.

**Example:**
```
# Can't filter:
cargo doc-query methods --include-deprecated
cargo doc-query methods --exclude-private
cargo doc-query methods --include-inline
```

**Why it happens:**
Filtering logic is hard-coded to names only. Adding attribute support requires understanding rustdoc JSON attribute structure.

**How to avoid:**
1. **Parse all item attributes** — collect them during indexing
2. **Add filtering options** — `--include-deprecated`, `--exclude-private`, etc.
3. **Document all filterable attributes** — which ones exist in rustdoc JSON
4. **Test with attribute-containing code** — verify filtering works correctly
5. **Plan for future attributes** — extensible attribute handling

**Warning signs:**
- Users asking for deprecated filtering
- "Why can't I filter hidden items?"
- Filter options don't cover common cases
- No attribute data in output

**Phase to address:**
Phase 4 (Query Engine & Filtering)

---

### Pitfall 8: Regex vs Glob Pattern Confusion

**What goes wrong:**
Users don't know whether to use regex syntax or glob syntax for filtering, leading to unexpected results.

**Example:**
```
# User expects:
--filter "*push*"    # glob style

# But implementation uses regex:
--filter ".*push.*"  # regex style
```

**Why it happens:**
Implementation defaults to one pattern type without documentation or cross-checking user expectations.

**How to avoid:**
1. **Choose one pattern type** — document and stick with it
2. **Add clear documentation** — examples of expected syntax
3. **Provide pattern validation** — show "pattern would match X, Y, Z"
4. **Support both if needed** — with explicit flags (`--pattern-type regex|glob`)
5. **Test with user-friendly examples** — match common expectations

**Warning signs:**
- Users confused about pattern syntax
- Questions on GitHub/issue tracker about pattern matching
- Users submitting bugs due to regex escaping issues
- No examples in help text

**Phase to address:**
Phase 4 (Query Engine & Filtering)

---

## Doc Comment Pitfalls

### Pitfall 9: Doc Comments Not Extracted at All

**What goes wrong:**
Documentation is completely missing from output, leaving users without explanations.

**Example:**
```
# Should show:
/// Adds an item to the collection.
/// Returns the previous value if present.
fn push(&mut self, item: T) -> Option<T>

# Shows instead:
fn push(&mut self, item: T) -> Option<T>
```

**Why it happens:**
Doc comments are overlooked during type expansion or query processing. Output schema doesn't include them.

**How to avoid:**
1. **Always include doc comments** — in all output formats
2. **Parse markdown** — handle code blocks, lists, etc.
3. **Add `--no-docs` flag** — allow opting out if needed
4. **Test with rich doc comments** — ensure all formatting is preserved
5. **Check rustdoc JSON fields** — docs are present in JSON, just not consumed

**Warning signs:**
- Empty documentation sections
- Missing doc comments on well-documented types
- Users asking "where's the documentation?"
- `--help` doesn't mention docs flag

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

### Pitfall 10: Markdown Parsing Errors

**What goes wrong:**
Doc comments with markdown fail to render properly, showing raw markdown or breaking output.

**Example:**
```rust
/// This is a list:
/// 1. First item
/// 2. Second item
fn foo() {}
```

**Why it happens:**
Simple newline-to-space conversion doesn't handle proper markdown formatting. Lists, code blocks, and formatting get mangled.

**How to avoid:**
1. **Use markdown parser** — pulldown-cmark for full markdown support
2. **Test with complex docs** — lists, code blocks, links
3. **Handle errors gracefully** — fallback on parse failure
4. **Add `--format` flag** — raw vs markdown rendering
5. **Document supported markdown** — which features are supported

**Warning signs:**
- Docs show as "1. First item" without bullets
- Code blocks show as raw text
- Links are broken or shown as `[text](url)`
- Users submitting formatting bugs

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

### Pitfall 11: Intra-Doc Links Broken

**What goes wrong:**
Links to other items (`/// See [`MyStruct`]` or `/// See [`SomeTrait::method`]`) don't work in output.

**Example:**
```
/// See [`Vec::new`]
fn foo() {}
```

**Why it happens:**
Intra-doc links are extracted but not rendered as hyperlinks or resolved to actual items.

**How to avoid:**
1. **Extract intra-doc links** from `links` field in rustdoc JSON
2. **Render as links** — markdown format or inline format
3. **Resolve link targets** — map to actual items when possible
4. **Handle broken links** — show warnings for unresolved links
5. **Test with nested links** — `Trait::Method`, `crate::Module::Type`

**Warning signs:**
- Inline links not clickable
- Links show as raw text
- "See [`...`]` text in output
- Users asking why links don't work

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

### Pitfall 12: Doc Comment Truncation Without Context

**What goes wrong:**
Long doc comments are truncated mid-sentence, losing meaning.

**Example:**
```
# Should show long doc:
/// This function does very important work. It takes an input and
/// processes it through a complex pipeline. The output is then
/// written to disk, ensuring that data integrity is maintained.
/// This is critical for production systems.

# Shows instead:
/// This function does very important work. It takes an input
#```

**Why it happens:**
Token budget is applied without considering doc comment completeness. Short truncation makes text confusing.

**How to avoid:**
1. **Track doc comment tokens** — include in budget calculations
2. **Truncate at sentence boundaries** — not mid-sentence
3. **Add ellipsis** — "..." when truncated
4. **Let users control truncation** — `--doc-length` flag
5. **Prefer truncation over incomplete text** — better than cutting mid-word

**Warning signs:**
- Doc comments cut off mid-sentence
- Incomplete explanations that don't make sense
- "..." at the end of long docs
- Users asking for more complete docs

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

### Pitfall 13: Missing `#[doc(hidden)]` Items

**What goes wrong:**
Items marked `#[doc(hidden)]` appear in output, or vice versa, creating confusion.

**Example:**
```
# Should show:
/// This is private to the module
#[doc(hidden)]
fn _private_helper() {}

# Shows instead:
fn _private_helper() {}  # without special note
```

**Why it happens:**
`doc(hidden)` flag is not checked or not documented. Users don't know why some items appear differently.

**How to avoid:**
1. **Check `has_doc_hidden` field** in rustdoc JSON
2. **Exclude hidden items by default** — keep CLI clean
3. **Add `--show-hidden` flag** — optional inclusion
4. **Document hidden behavior** — how it affects output
5. **Test with hidden items** — verify exclusion/inclusion

**Warning signs:**
- `#[doc(hidden)]` items visible in output
- No explanation for why items are hidden
- Users asking "why is this hidden?"
- Flag not documented in help

**Phase to address:**
Phase 4 (Query Engine & Filtering)

---

### Pitfall 14: Doc Comments from Dependencies Not Visible

**What goes wrong:**
Doc comments from transitive dependencies are missing, limiting understanding of code.

**Example:**
```
# User calls:
cargo doc-query types serde::Serializer

# Missing serde's doc comments
```

**Why it happens:**
Doc comments are only extracted for the primary crate. Dependencies are not parsed.

**How to avoid:**
1. **Generate JSON for dependencies** — use rustdoc with `-Z crate-depth=N`
2. **Include dependency docs in output** — make them accessible
3. **Add `--include-deps` flag** — control dependency documentation
4. **Handle version conflicts** — multiple versions of same crate
5. **Test with dependency crates** — ensure docs are present

**Warning signs:**
- Missing docs on third-party types
- Users asking for dependency documentation
- Doc comments only on local types
- Flag not available for enabling dependency docs

**Phase to address:**
Phase 1 (JSON Ingestion & Schema Handling)

---

## Field Discovery Pitfalls

### Pitfall 15: Assuming Limited JSON Field Coverage

**What goes wrong:**
Not exploring rustdoc JSON beyond basic fields, missing useful information.

**Example:**
```
# Missing fields:
- `has_where_clause` — whether generic has bounds
- `is_doc_hidden` — documentation visibility
- `links` — intra-doc link references
- `source` — source file location
- `impls` — implemented traits
```

**Why it happens:**
Start with basic functionality (methods, types) and don't explore rustdoc JSON fields that could be useful.

**How to avoid:**
1. **Explore rustdoc JSON thoroughly** — list all fields
2. **Document available fields** — in ARCHITECTURE.md or similar
3. **Add fields incrementally** — prioritize by user demand
4. **Create field discovery tool** — dump JSON for learning
5. **Track field usage** — which fields are actually consumed

**Warning signs:**
- Users asking "can I get source locations?"
- "Where is the trait implementation info?"
- Unknown rustdoc JSON fields that could be useful
- No documentation of JSON schema

**Phase to address:**
Phase 1 (JSON Ingestion & Schema Handling)

---

### Pitfall 16: Not Handling Deprecated Fields

**What goes wrong:**
Fields that were renamed or removed in newer rustdoc versions cause crashes or errors.

**Example:**
```
# Old rustdoc (v30):
"fields": [
  {"name": "x", "visibility": "pub"}

# New rustdoc (v32):
"fields": [
  {"name": "x", "visibility": 1}  // changed to enum
]
```

**Why it happens:**
Code assumes field types won't change. New rustdoc format introduces breaking changes.

**How to avoid:**
1. **Check rustdoc format version** — adapt to each version
2. **Handle deprecated fields** — map old to new
3. **Test with multiple format versions** — v28, v30, v32, etc.
4. **Use rustdoc-types crate** — handles version differences
5. **Fail gracefully** on unknown fields

**Warning signs:**
- Crashes after rustup update
- `serde_json::from_str` failures
- Fields returning `null` or unexpected types
- Breaking changes in CI builds

**Phase to address:**
Phase 1 (JSON Ingestion & Schema Handling)

---

### Pitfall 17: Missing Useful Metadata Fields

**What goes wrong:**
Not using rustdoc JSON metadata fields that could improve output quality.

**Example:**
```
# Missing useful metadata:
- `source` — source file location (useful for debugging)
- `proc_macro` — whether item is from proc-macro
- `deprecation_note` — deprecation information
- `stability_index` — item stability level
```

**Why it happens:**
Focus on functionality over metadata. Don't realize these fields could be valuable.

**How to avoid:**
1. **Create field utility catalog** — list all useful fields and their purposes
2. **Add metadata flags** — `--with-source`, `--with-deprecation`
3. **Prioritize by demand** — add fields users actually need
4. **Document metadata usage** — show examples
5. **Test with metadata-rich crates** — verify usefulness

**Warning signs:**
- Users asking "can I get source locations?"
- "Where is the deprecation note?"
- No source file information in output
- Missing useful metadata in queries

**Phase to address:**
Phase 1 (JSON Ingestion & Schema Handling)

---

## Integration Pitfalls

### Pitfall 18: Adding Features Without Breaking Existing Functionality

**What goes wrong:**
New output features change the output format or behavior, breaking existing tooling or user expectations.

**Example:**
```
# Old output:
{"name": "push", "params": ["&mut self", "T"]}

# New output:
{"name": "push", "signature": "fn push(&mut self, T)", "doc": "Adds item to collection"}
```

**Why it happens:**
Features added incrementally with minimal coordination, changing output schema without backward compatibility.

**How to avoid:**
1. **Maintain backward compatibility** — version output format
2. **Add `--output-format` flag** — control schema (current, minimal, verbose)
3. **Document output format changes** — migration guide for existing users
4. **Test with existing integrations** — scripts that parse output
5. **Fail on incompatible flags** — don't silently change behavior

**Warning signs:**
- Scripts break after update
- CI tests failing due to output format change
- Users confused about output differences
- No output format documentation

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

### Pitfall 19: Performance Degradation from New Features

**What goes wrong:**
Adding doc comments, filtering, and rendering makes queries significantly slower.

**Example:**
```
# Old query: 2ms
cargo doc-query methods std::vec::Vec

# New query: 500ms (with docs enabled)
cargo doc-query methods std::vec::Vec --include-docs
```

**Why it happens:**
New features add text processing, filtering, and rendering overhead. Not optimized for speed.

**How to avoid:**
1. **Profile before optimizing** — identify bottlenecks
2. **Use caching for doc comments** — parse once, reuse
3. **Lazy load expensive features** — only when needed
4. **Add performance flags** — `--fast` mode for speed
5. **Benchmark before/after** — track performance impact

**Warning signs:**
- Queries taking 10x longer after adding features
- CPU usage spiking on simple queries
- No performance degradation testing
- Users complaining about speed

**Phase to address:**
Phase 6 (Performance Optimization)

---

### Pitfall 20: Memory Usage Explosion from Text Processing

**What goes wrong:**
Loading all doc comments into memory causes OOM errors on large crates.

**Example:**
```
# aws-sdk-ec2 JSON has 500MB total
# Adding docs increases memory to >2GB
# Process killed by OOM
```

**Why it happens:**
Doc comments are text-heavy. Loading entire crates with all docs consumes significant memory.

**How to avoid:**
1. **Lazy load doc comments** — parse on-demand, not all at once
2. **Use streaming JSON parsing** — don't load entire crate at once
3. **Add memory limits** — fail gracefully when approaching limits
4. **Track memory usage** — profile memory consumption
5. **Test with large crates** — aws-sdk-ec2, windows-rs

**Warning signs:**
- OOM errors on large crates
- Memory usage growing linearly with crate size
- Slow startup on large workspaces
- Tool unusable on large dependencies

**Phase to address:**
Phase 1 (JSON Ingestion & Schema Handling) and Phase 6 (Performance Optimization)

---

### Pitfall 21: CLI Flag Conflicts Between Features

**What goes wrong:**
New features add flags that conflict with existing flags or create ambiguous behavior.

**Example:**
```
# Old flags:
--format=json
--compact

# New flags:
--with-docs
--show-inline

# Conflict: --compact vs --with-docs
```

**Why it happens:**
Flags added incrementally without considering conflicts. Users unsure which flags to use together.

**How to avoid:**
1. **Design CLI together** — flags planned with existing ones
2. **Document flag interactions** — which flags work together
3. **Add validation** — prevent invalid flag combinations
4. **Add help examples** — show correct flag usage
5. **Test all combinations** — ensure no conflicts

**Warning signs:**
- Users confused about flag combinations
- Help text doesn't mention conflicts
- Invalid combinations not caught
- Flag overlap not documented

**Phase to address:**
Phase 2 (CLI & Command Layer)

---

### Pitfall 22: Output Schema Not Versioned

**What goes wrong:**
Output format evolves without versioning, making it impossible to maintain compatibility.

**Example:**
```
# v1.0 output:
{"name": "push", "params": ["&mut self", "T"]}

# v1.1 output adds new fields:
{"name": "push", "params": ["&mut self", "T"], "doc": "Adds item"}

# Old parsers break
```

**Why it happens:**
Output format assumed to be stable. New features add fields without versioning.

**How to avoid:**
1. **Version output format** — `format_version` field
2. **Add `--output-version` flag** — control output format
3. **Maintain multiple schemas** — current, backward compatible
4. **Document format changes** — migration guide
5. **Fail on unknown fields** — don't silently drop data

**Warning signs:**
- Old parsers break on new output
- No way to request old format
- Users asking for stable format
- Format evolution not documented

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

## Output Format Pitfalls

### Pitfall 23: Missing JSON Schema Definition

**What goes wrong:**
JSON output format evolves without schema definition, making it hard for users to parse reliably.

**Example:**
```
# Output:
{"methods": [{"name": "push"}]}

# Which fields? Which structure?
```

**Why it happens:**
JSON is generated manually without schema. Users can't trust structure without documentation.

**How to avoid:**
1. **Define JSON schema** — JSON Schema Draft 7 or 2020-12
2. **Validate output against schema** — catch errors early
3. **Generate schema from code** — keep in sync
4. **Publish schema** — in docs repository
5. **Update schema with each format change** — versioned schemas

**Warning signs:**
- Output structure unclear
- Users building parsers from scratch
- No JSON Schema in documentation
- Format changes break existing parsers

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

### Pitfall 24: Minimal Mode Too Minimal

**What goes wrong:**
Token-efficient output mode is so minimal it's useless for most use cases.

**Example:**
```
# Token budget: 1KB
# Minimal output:
push(&mut self, T)
pop() -> Option<T>
```

**User needs:** "I want to see method signatures AND documentation to understand what to do."

**Why it happens:**
Minimal mode is prioritized for extreme token efficiency, not usability.

**How to avoid:**
1. **Define multiple output modes** — minimal, standard, verbose
2. **Make minimal mode configurable** — `--minimal` with options
3. **Balance token efficiency vs usefulness** — don't go too far
4. **Test with real use cases** — LLM context windows
5. **Document trade-offs** — what you get with each mode

**Warning signs:**
- Minimal mode too bare to be useful
- Users confused about when to use it
- No in-between options
- Minimal mode not matching user needs

**Phase to address:**
Phase 5 (Output Formatting & Display)

---

## Testing Pitfalls

### Pitfall 25: Inadequate Testing of Output Refinement Features

**What goes wrong:**
Output features pass basic tests but fail in real-world scenarios.

**Example:**
```
# Tests:
- Doc comments on simple types
- Methods with 2 parameters

# Fail in real usage:
- Nested generic types
- Doc comments with complex markdown
- Filters on module paths
```

**Why it happens:**
Test cases are too simple, not covering edge cases or real-world complexity.

**How to avoid:**
1. **Create comprehensive test suite** — unit and integration tests
2. **Test with large crates** — aws-sdk-ec2, windows-rs
3. **Test edge cases** — empty docs, missing fields, complex generics
4. **Add benchmark tests** — performance regression detection
5. **Test with real user scenarios** — common query patterns

**Warning signs:**
- Tests pass but real usage fails
- Users reporting bugs not caught by tests
- No large crate testing
- Edge cases not covered

**Phase to address:**
Phase 6 (Testing & Quality Assurance)

---

### Pitfall 26: No Format Version Testing

**What goes wrong:**
Features don't work with multiple rustdoc format versions.

**Example:**
```
# Test passes on rustdoc v30
# Fails on rustdoc v32
```

**Why it happens:**
Tests run on developer's local rust version only. Format changes not tested.

**How to avoid:**
1. **Test with multiple rust versions** — 1.70+, 1.75+, 1.80+
2. **Mock different format versions** — test without actually running rustdoc
3. **Add format version tests** — verify compatibility
4. **Use rustdoc-types crate** — handles version differences
5. **Update tests on rustup** — automatic format change detection

**Warning signs:**
- Tests fail after rustup update
- Format version not checked
- No version testing strategy
- Unknown format version handling

**Phase to address:**
Phase 1 (JSON Ingestion & Schema Handling)

---

## Summary of Output Refinement Pitfalls

### By Category

| Category | Count | Priority |
|----------|-------|----------|
| Unified Rendering | 4 | HIGH |
| Filtering | 4 | HIGH |
| Doc Comments | 6 | MEDIUM |
| Field Discovery | 3 | MEDIUM |
| Integration | 7 | HIGH |
| Output Format | 2 | MEDIUM |
| Testing | 2 | HIGH |

### By Phase

| Phase | Pitfalls | Key Focus |
|-------|----------|-----------|
| Phase 1 (JSON Ingestion) | 2, 15, 16, 17, 26 | Field discovery, format version handling |
| Phase 2 (CLI) | 21 | Flag conflicts, validation |
| Phase 4 (Query Engine) | 5, 6, 7, 13 | Filtering logic, scope handling |
| Phase 5 (Output Formatting) | 1, 2, 3, 4, 9, 10, 11, 12, 18, 23, 24 | Rendering, doc extraction, format |
| Phase 6 (Performance) | 19, 20, 25 | Memory, speed, testing |

### Quick Reference

**High Priority (Fix First):**
1. Pitfall 18 — Breaking existing functionality
2. Pitfall 19 — Performance degradation
3. Pitfall 5 — Naive filtering only
4. Pitfall 9 — Doc comments not extracted
5. Pitfall 1 — Inconsistent depth-based formatting

**Medium Priority (Fix Soon):**
6. Pitfall 11 — Intra-doc links broken
7. Pitfall 10 — Markdown parsing errors
8. Pitfall 6 — Cross-crate name ambiguity
9. Pitfall 7 — No attribute filtering
10. Pitfall 8 — Regex vs glob confusion

**Low Priority (Fix If Needed):**
11. Pitfall 2 — Generic truncation (nice to have)
12. Pitfall 3 — Trait bound omission (nice to have)
13. Pitfall 4 — Enum vs struct inconsistency (nice to have)

---

## Prevention Strategies

### Early Detection

**Warning Signs Checklist:**

| Category | Signs | Action |
|----------|-------|--------|
| Unified Rendering | Different output at different depths | Single render function |
| Filtering | Users asking for path-based filtering | Multi-type filter system |
| Doc Comments | Missing docs in output | Parse from rustdoc JSON |
| Field Discovery | Unknown rustdoc fields | Create field discovery tool |

**Before Implementation:**
- [ ] Review existing pitfall list (PITFALLS.md)
- [ ] Identify integration risks with current system
- [ ] Design output format together with users
- [ ] Plan for backward compatibility
- [ ] Create testing strategy for new features

### During Implementation

**Checklist by Phase:**

**Phase 1 (JSON Ingestion):**
- [ ] Support multiple rustdoc format versions
- [ ] Explore all useful JSON fields
- [ ] Handle deprecated fields gracefully

**Phase 2 (CLI):**
- [ ] Design flags that don't conflict
- [ ] Add flag validation
- [ ] Document flag interactions

**Phase 4 (Query Engine):**
- [ ] Support multiple filter types
- [ ] Handle cross-crate names
- [ ] Check attributes for filtering

**Phase 5 (Output Formatting):**
- [ ] Single render function for all depths
- [ ] Always include doc comments
- [ ] Version output format
- [ ] Define JSON schema

**Phase 6 (Performance):**
- [ ] Profile new features
- [ ] Add memory limits
- [ ] Test with large crates

### After Implementation

**Verification Steps:**
1. **Test with large crates** — aws-sdk-ec2 (~500MB JSON)
2. **Test with multiple rustdoc versions** — v28, v30, v32+
3. **Test edge cases** — missing docs, empty types, complex generics
4. **Benchmark performance** — before/after measurements
5. **User testing** — real-world queries and feedback

---

## "Looks Done But Isn't" Checklist

Output Refinement Specific:

- [ ] **Depth-based rendering:** Uses single function for all depths, not separate paths
- [ ] **Generic handling:** Shows generic bounds, handles parameter count
- [ ] **Trait bounds:** Always visible unless opted out
- [ ] **Enum/Struct parity:** Same field rendering for both
- [ ] **Filtering depth:** Supports path-based, attribute-based, and name-based filtering
- [ ] **Cross-crate disambiguation:** Shows paths when items are ambiguous
- [ ] **Attribute filtering:** Supports filtering by deprecated, hidden, etc.
- [ ] **Pattern type consistency:** Regex or glob, documented, validated
- [ ] **Doc comment extraction:** Always extracted, markdown parsed correctly
- [ ] **Intra-doc links:** Rendered as links, targets resolved
- [ ] **Doc truncation:** At sentence boundaries, with ellipsis
- [ ] **Hidden items:** Handled consistently, can be excluded
- [ ] **Dependency docs:** Available with flag, not just local crate
- [ ] **JSON fields:** All useful fields explored and documented
- [ ] **Format version handling:** Works across rustdoc versions
- [ ] **Backward compatibility:** Output versioned, old formats supported
- [ ] **Performance:** Not significantly slower than before
- [ ] **Memory usage:** Doesn't cause OOM on large crates
- [ ] **Flag conflicts:** No ambiguous flag combinations
- [ ] **Output schema:** JSON Schema defined and validated
- [ ] **Minimal mode:** Not too minimal to be useful
- [ ] **Comprehensive tests:** Tests pass with large crates and edge cases
- [ ] **Format version testing:** Works with multiple rust versions

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Inconsistent depth formatting | LOW | Consolidate render functions |
| Naive filtering only | MEDIUM | Add multi-type filter system |
| Missing doc extraction | LOW | Add doc parsing, test |
| Generic truncation | MEDIUM | Redesign budget calculations |
| Markdown parsing errors | MEDIUM | Replace with markdown parser |
| Broken intra-doc links | LOW | Add link rendering |
| Missing hidden items handling | LOW | Add `--show-hidden` flag |
| Cross-crate name ambiguity | MEDIUM | Add path filtering |
| Format version breakage | LOW | Update to support new version |
| Performance degradation | HIGH | Profile, optimize, cache |
| Memory explosion | HIGH | Add streaming, lazy loading |
| Flag conflicts | LOW | Redesign flags, validate |

---

## Pitfall-to-Phase Mapping for v1.1

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Inconsistent depth formatting | Phase 5 | Compare output at different depths |
| Naive filtering only | Phase 4 | Test path-based filtering |
| Missing doc extraction | Phase 5 | Verify docs appear in output |
| Generic truncation | Phase 5 | Test with complex generics |
| Markdown parsing errors | Phase 5 | Test with markdown docs |
| Broken intra-doc links | Phase 5 | Verify links are rendered |
| No attribute filtering | Phase 4 | Filter by deprecated items |
| Cross-crate name ambiguity | Phase 4 | Test ambiguous name queries |
| Field discovery | Phase 1 | Document all useful fields |
| Format version handling | Phase 1 | Test multiple rust versions |
| Performance degradation | Phase 6 | Benchmark before/after |
| Memory explosion | Phase 1, 6 | Test with large crates |
| Flag conflicts | Phase 2 | Validate all flag combinations |

---

## Sources

### Tools Analyzed
- **ripdoc** (https://github.com/Alb-O/ripdoc) — Markdown-focused, good CLI UX
- **rust-docs-mcp** (https://lib.rs/crates/rust-docs-mcp) — Comprehensive, verbose output
- **docs.rs** (https://docs.rs/docsrs) — Docs search, fuzzy matching
- **rustdoc-json** (https://github.com/rust-lang/rust-analyzer/tree/master/crates/rustdoc-json) — JSON generation

### Documentation Standards
- **JSON Schema Draft 7** (https://json-schema.org/understanding-json-schema/) — Output schema definition
- **Markdown specification** (https://spec.commonmark.org/) — Doc comment rendering
- **llms.txt specification** (https://llmstxt.org/) — Emerging standard for LLM-friendly docs
- **rustdoc JSON RFC** (https://rust-lang.github.io/rfcs/2963-rustdoc-json.html) — JSON format specification

### Research on Output Formatting
- **Token management in LLMs** (https://agenta.ai/blog/top-6-techniques-to-manage-context-length-in-llms) — Context window constraints
- **Documentation formatting** (https://doc.rust-lang.org/rustdoc/format.html) — Rust documentation standards
- **Context limits** (https://github.com/taylorwilsdon/llm-context-limits) — Model-specific limits

---

*Output refinement pitfalls research for: cargo-doc-query v1.1*
*Researched: 2026-02-13*
