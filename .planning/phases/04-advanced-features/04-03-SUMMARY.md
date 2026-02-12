---
phase: 04-advanced-features
plan: 03
date: 2026-02-12
status: complete
---

# Plan 04-03 Summary: Query Command Integration

## Deliverables

### Types (src/types/query.rs)
- `to_minimal()` methods added to all query output types:
  - `QueryResponse::to_minimal()` - Converts entire response
  - `QueryMatch::to_minimal()` - Preserves metadata, minimalizes content
  - `TypeResult::to_minimal()` - Minimalizes methods and trait impls
  - `TraitResult::to_minimal()` - Minimalizes methods, omits provided_methods
  - `MethodOutput::to_minimal()` - Removes docs, keeps signature
  - `TraitImplOutput::to_minimal()` - Minimalizes methods, omits provided_methods/generic_args
- `estimate_tokens()` - JSON serialization length / 4 approximation

### Query Engine (src/query/engine.rs)
- `QueryOptions` extended with:
  - `minimal_mode: bool` - Enable minimal output
  - `token_budget: Option<usize>` - Token limit
- Builder methods:
  - `with_minimal(bool)` - Set minimal mode
  - `with_token_budget(Option<usize>)` - Set token budget
- `query()` method applies minimal conversion when `minimal_mode` is true

### CLI (src/cli/query.rs)
- `--minimal` flag for signature-only output
- `--tokens <N>` flag for token budget (minimum 100)
- Token count printed to stderr (e.g., "Query completed in 45ms (123 tokens)")
- Budget exceeded warning printed when limit hit

### Main (src/main.rs)
- Added `--minimal` and `--tokens` to Commands::Query variant
- Arguments passed through to QueryCommand::new()

## Usage

```bash
# Query with minimal output
cargo doc-query query anyhow::Error --minimal

# Query with token budget
cargo doc-query query anyhow::Error --tokens 500

# Combined
cargo doc-query query anyhow::Error --minimal --tokens 300
```

## Verification

- ✅ `--minimal` flag accepted and processed
- ✅ `--tokens` flag validates minimum (100)
- ✅ Token count displayed in stderr output
- ✅ Existing queries without flags work unchanged
- ✅ Consistent with expand command behavior

## Consistency Achieved

| Command | --minimal | --tokens | Token Display |
|---------|-----------|----------|---------------|
| expand  | ✅        | ✅       | stderr        |
| query   | ✅        | ✅       | stderr        |

Both commands now share:
- Same flag names and behavior
- Same token estimation (JSON len / 4)
- Same minimum validation (100 tokens)
- Same warning format for budget exceeded

## Commits

- `a816fcd` feat(04-03): integrate --tokens and --minimal into query command

## Notes

- Backward compatibility maintained: all new flags are optional
- Query path resolution issues exist but are separate from this feature
- Minimal mode reduces output by removing docs and detail fields
- Token budget provides approximate control over LLM context usage
