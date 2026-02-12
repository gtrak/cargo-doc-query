# Feature Research: Rust Documentation Query Tools

**Domain:** Rust crate API documentation querying (CLI tools)
**Researched:** 2026-02-12
**Confidence:** HIGH (based on analysis of existing tools and LLM requirements)

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Method queries by type** | Core use case: "What can I do with this type?" | LOW | Parse rustdoc JSON `impl` blocks, filter by `self` type |
| **Trait implementation discovery** | Essential for understanding behavior (Display, Iterator, etc.) | LOW | Query `impl` items with `trait` property in rustdoc JSON |
| **JSON output format** | Required for LLM/programmatic consumption | LOW | Structured output is standard expectation (ripdoc, rust-docs-mcp) |
| **Local crate support** | Developers need to query their own code | LOW | Generate rustdoc JSON from local `Cargo.toml` |
| **crates.io dependency support** | Querying third-party crates is primary use case | MEDIUM | Fetch from docs.rs or build locally (requires network) |
| **Caching** | Rebuilding rustdoc JSON is slow (~5s per crate) | MEDIUM | All tools (ripdoc, rust-docs-mcp, docsrs) implement this |
| **Search/filtering** | Finding items in large crates | LOW | Basic name matching expected at minimum |
| **Associated items retrieval** | Methods, fields, constants belong to types | LOW | Part of basic type introspection |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Token-budget constrained output** | LLMs have context limits (128K-2M tokens). Efficient packing maximizes useful context. | MEDIUM | Critical differentiator for LLM agents. Most tools ignore this constraint. |
| **Depth-limited recursive type expansion** | "Show me the full picture, but not too much" — essential for complex generic types | MEDIUM | Prevents context overflow from deeply nested types |
| **Incremental rebuilds** | Rebuild only changed crates in large workspaces | MEDIUM | Saves significant time in multi-crate projects |
| **Content-addressable cache storage** | Automatic invalidation when dependencies change, deduplication | MEDIUM | Rust-docs-mcp uses this; enables reliable caching |
| **Multiple output modes** | Markdown for humans, JSON for agents, minimal for piping | LOW | Ripdoc supports this well |
| **Fuzzy search with typo tolerance** | "I think it's called spawn_something?" | MEDIUM | Docsrs implements this; improves discoverability |
| **Associated type resolution** | Understanding `type Item = X` in trait impls | MEDIUM | Required for complete generic type understanding |
| **Sub-100ms query latency** | Fast enough for interactive CLI usage | HIGH | Requires efficient indexing and caching strategy |
| **Signature-only output mode** | Strip docs to save tokens when types matter more than explanations | LOW | LLM-specific optimization rarely seen in tools |
| **Trait bound visualization** | `where T: Display + Debug` constraints are critical for generics | LOW | Display where clauses prominently |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **LSP/IDE integration** | "Can't you just extend rust-analyzer?" | Conflicts with stateless design, brings daemon complexity, slow reindexing | Stay CLI-focused; emit results in formats IDEs can consume |
| **Full semantic search** | "Search by what code does, not just names" | Requires AST parsing, type inference, significant complexity | Use symbol names + documentation text search; defer to ripgrep for content |
| **Type checking/validation** | "Tell me if my usage is correct" | Duplicates `cargo check`, requires full type system implementation | Stay focused on documentation extraction; integrate with existing tools |
| **Real-time file watching** | "Rebuild when I save" | Adds daemon/process management complexity | Explicit rebuild triggers; use `cargo watch` externally if needed |
| **GUI/Web interface** | "Visual browsing is easier" | Out of scope for CLI tool, significant maintenance burden | Generate markdown suitable for viewer tools; emit JSON for custom UIs |
| **Documentation editing** | "Let me fix docs from the CLI" | Scope creep into documentation generation | Read-only tool focused on consumption, not production |
| **Cross-crate type resolution** | "Follow this type through 5 crates" | Exponential blowup in generic expansion, complex dependency tracking | Depth-limited expansion within single crate; manual navigation between crates |

## Feature Dependencies

```
[Caching]
    └──requires──> [JSON Generation]
                        └──requires──> [Nightly Rust Toolchain]

[Incremental Rebuilds]
    └──requires──> [Caching]
    └──requires──> [Cargo.lock Change Detection]

[Token-Budget Output]
    └──requires──> [Multiple Output Formats]
    └──enhances──> [Depth-Limited Expansion]

[Fuzzy Search]
    └──requires──> [Search Index]
                        └──requires──> [Caching]

[Associated Type Resolution]
    └──requires──> [Trait Implementation Discovery]

[crates.io Support]
    └──requires──> [Caching]
    └──requires──> [Network Fetching]
```

### Dependency Notes

- **Caching requires JSON Generation:** Must generate rustdoc JSON before caching; JSON generation is expensive (~5s per crate)
- **Incremental Rebuilds requires Caching:** Need stored state to determine what changed; without cache, everything is a full rebuild
- **Token-Budget enhances Depth-Limited Expansion:** Budget constraints inform expansion depth; they're complementary features
- **Fuzzy Search requires Search Index:** Need indexed data structure for fast fuzzy matching; naive iteration is too slow
- **Associated Type Resolution requires Trait Discovery:** Must first identify trait impls before resolving their associated types
- **crates.io Support requires both Caching and Network:** Remote crates must be fetched and cached locally; network is unreliable

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [ ] **Method queries by type** — Core value proposition; users query "what methods on Vec?"
- [ ] **Trait implementation discovery** — Essential for understanding type capabilities
- [ ] **JSON output format** — Required for LLM consumption; differentiates from ripdoc's markdown focus
- [ ] **Local crate support** — Must work on user's own code first
- [ ] **Basic caching** — Without caching, 5s rebuilds make tool unusable
- [ ] **crates.io support** — Third-party crates are the primary use case
- [ ] **Minimal output mode** — Token-efficient output for LLMs (key differentiator)

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] **Depth-limited type expansion** — Trigger: Users need to see nested type structure
- [ ] **Incremental rebuilds** — Trigger: Large workspace with many crates
- [ ] **Fuzzy search** — Trigger: Users struggle to find items by exact name
- [ ] **Associated type resolution** — Trigger: Complex generic codebases need full type info
- [ ] **Signature-only mode** — Trigger: LLM agents hitting context limits with full docs
- [ ] **Content-addressable cache** — Trigger: Cache invalidation bugs reported

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Workspace-wide queries** — Query across all workspace members (defer: complex aggregation)
- [ ] **Documentation search** — Full-text search of doc comments (defer: tantivy integration)
- [ ] **Export to llms.txt format** — Generate standard LLM context files (defer: format still evolving)
- [ ] **Build script integration** — Hook into cargo build process (defer: unclear value)

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Method queries by type | HIGH | LOW | P1 |
| Trait implementation discovery | HIGH | LOW | P1 |
| JSON output format | HIGH | LOW | P1 |
| Local crate support | HIGH | LOW | P1 |
| Basic caching | HIGH | MEDIUM | P1 |
| crates.io support | HIGH | MEDIUM | P1 |
| Minimal output mode | HIGH | LOW | P1 |
| Depth-limited expansion | HIGH | MEDIUM | P2 |
| Incremental rebuilds | MEDIUM | MEDIUM | P2 |
| Fuzzy search | MEDIUM | MEDIUM | P2 |
| Associated type resolution | MEDIUM | LOW | P2 |
| Content-addressable cache | MEDIUM | MEDIUM | P2 |
| Token-budget constraints | HIGH | MEDIUM | P2 |
| Signature-only mode | MEDIUM | LOW | P2 |
| Multiple output formats | MEDIUM | LOW | P2 |
| Fuzzy search | MEDIUM | MEDIUM | P3 |
| Workspace-wide queries | LOW | HIGH | P3 |
| Documentation search | LOW | HIGH | P3 |
| Export to llms.txt | LOW | LOW | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | ripdoc | rust-docs-mcp | docsrs | Our Approach |
|---------|--------|---------------|--------|--------------|
| **Output formats** | Markdown (default), JSON | JSON (MCP protocol) | JSON | JSON (primary), Markdown, Minimal |
| **crates.io support** | Yes (fetch + cache) | Yes (fetch + cache) | Yes (fetch) | Yes with content-addressable cache |
| **Local crate support** | Yes | Yes | Yes | Yes |
| **Caching** | File-based | File-based | None | Content-addressable, incremental |
| **Fuzzy search** | Basic regex | No | Yes (fuzzy-matcher) | Add in v1.x |
| **Token budget control** | No | Truncation only | No | First-class budget constraints |
| **Type expansion depth** | No | No | No | Configurable depth limits |
| **Incremental rebuilds** | No | No | N/A | Per-crate incremental |
| **Output speed** | ~1-2s | ~1-2s | N/A | Target <100ms with cache |
| **LLM-optimized** | Markdown focus | MCP protocol focus | General purpose | Token-efficient first |
| **Daemon required** | No | No (stateless) | N/A | No (stateless CLI) |

### Key Differentiation Strategy

1. **Token Efficiency:** Unlike ripdoc (markdown focus) or rust-docs-mcp (full JSON), we prioritize minimal token output for LLMs
2. **Speed:** Target sub-100ms cached queries vs 1-2s competitors achieve
3. **Intelligent Expansion:** Depth-limited recursive expansion prevents context overflow
4. **Statelessness:** No daemon (unlike LSP), but with smart caching for speed

## LLM-Specific Considerations

### Context Window Constraints (2025)

| Model | Context Window | Typical API Limit | Notes |
|-------|----------------|-------------------|-------|
| GPT-4o / GPT-5 | 128K | 128K input | ~4000-16000 output |
| Claude 3.5/4 Sonnet | 200K | 200K input | ~8000-64000 output |
| Claude 3 Opus | 200K (1M beta) | 200K standard | Premium pricing above 200K |
| Gemini 2.5 Pro | 2M | 2M input | Largest available |
| DeepSeek-V3 | 128K | 128K input | Cost-effective option |

### Token Efficiency Targets

For a typical crate API query:
- **Full rustdoc JSON:** 10KB-50MB (too large)
- **ripdoc markdown output:** 1KB-500KB (variable)
- **rust-docs-mcp full response:** Unbounded (can overflow)
- **Our target minimal mode:** 100B-5KB per query

### Output Format Recommendations

**For LLM Agents (Primary):**
```json
{
  "type": "struct",
  "name": "Vec<T>",
  "methods": [
    {"name": "push", "sig": "fn push(&mut self, value: T)"},
    {"name": "pop", "sig": "fn pop(&mut self) -> Option<T>"}
  ],
  "traits": ["Clone", "Debug", "Default", "Eq", "Hash", "Ord", "PartialEq", "PartialOrd"]
}
```

**For CLI Humans:**
```
Vec<T>
  fn push(&mut self, value: T)
  fn pop(&mut self) -> Option<T>
  fn len(&self) -> usize
```

**For Piping (Minimal):**
```
push(&mut self, value: T)
pop(&mut self) -> Option<T>
len(&self) -> usize
```

## Sources

### Tools Analyzed
- **ripdoc** (https://github.com/Alb-O/ripdoc) — Fork of ruskel, markdown-focused, good CLI UX
- **rust-docs-mcp** (https://lib.rs/crates/rust-docs-mcp) — MCP server for LLMs, comprehensive but verbose
- **docsrs** (https://docs.rs/docsrs) — Fuzzy search library, type-state pattern
- **cargo-llms-txt** (https://github.com/masinc/cargo-llms-txt) — llms.txt generator from source

### Documentation Standards
- **llms.txt specification** (https://llmstxt.org/) — Emerging standard for LLM-friendly docs
- **rustdoc JSON RFC** (https://rust-lang.github.io/rfcs/2963-rustdoc-json.html) — JSON format specification
- **rustdoc-types crate** (https://docs.rs/rustdoc-types) — Official JSON type definitions

### LLM Context Research
- **Context window limits** (https://github.com/taylorwilsdon/llm-context-limits) — Up-to-date model limits
- **LLM API pricing 2025** (https://intuitionlabs.ai/articles/llm-api-pricing-comparison-2025) — Cost analysis
- **Context management techniques** (https://agenta.ai/blog/top-6-techniques-to-manage-context-length-in-llms) — Best practices

---
*Feature research for: cargo-doc-query*
*Researched: 2026-02-12*
