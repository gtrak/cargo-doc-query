---
phase: 04-advanced-features
plan: 02
date: 2026-02-12
status: complete
---

# Plan 04-02 Summary: Token Budgets & Minimal Mode

## Deliverables

### Types (src/types/expand.rs)
- `TokenConfig` - Configuration struct for:
  - `budget: Option<usize>` - Maximum token limit
  - `minimal_mode: bool` - Enable minimal output
  - `warning_threshold: f32` - Warning at % of budget (default 0.8)
- `ExpansionResult` extended with:
  - `token_count: usize` - Estimated tokens used
  - `budget_exceeded: bool` - Flag when budget hit
  - `truncated_paths: Vec<String>` - Types not fully expanded
- `TypeGraph.estimate_tokens()` - JSON string length / 4 approximation
- `TypeGraph.to_minimal()` - Convert to minimal representation
- `TypeNode.to_minimal()` - Remove field/variant details, keep counts

### Query Engine (src/query/expand.rs)
- `TypeExpander.with_config()` - Constructor with TokenConfig
- Budget-aware expansion:
  - Check budget before adding each node
  - Track cumulative token count
  - Record truncated types when budget exceeded
- `expand_type_with_config()` - Entry point with config

### CLI (src/cli/expand.rs)
- `--tokens <N>` flag for token budget (minimum 100)
- `--minimal` flag for signature-only output
- Validation: rejects budgets < 100 tokens
- Warnings printed to stderr when budget exceeded
- Token count printed with timing info

## Usage

```bash
# Minimal mode (counts only, no field details)
cargo doc-query expand anyhow::Error --minimal

# Token budget (stops expansion at ~500 tokens)
cargo doc-query expand anyhow::Error --tokens 500

# Combined for maximum efficiency
cargo doc-query expand anyhow::Error --minimal --tokens 500
```

## Verification

- ✅ `--minimal` flag works and reduces output size
- ✅ `--tokens` flag validates minimum (100) and enforces budget
- ✅ Token count printed to stderr (e.g., "57ms (24 tokens)")
- ✅ Budget warnings shown when truncated
- ✅ JSON validity maintained with all flags

## Performance

Minimal mode reduces output by ~60-80%:
- Full mode: ~100+ tokens for typical types
- Minimal mode: ~30-40 tokens (counts only)
- Budget enforcement: stops expansion gracefully

## Commits

- `0884afb` feat(04-02): token budget constraints and minimal mode for expand
