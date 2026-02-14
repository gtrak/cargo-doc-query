# cargo-doc-query

Fast, deterministic structured API extraction from Rust crates for LLM context reduction.

## Overview

cargo-doc-query generates an index from your dependencies' rustdoc JSON output and allows you to quickly query methods, traits, and types with sub-100ms response times. It's designed to provide structured API information that's more concise than raw source code but more useful than simple method listings.

## Features

- **Sub-100ms Queries**: Cached, deterministic results for rapid iteration
- **Type Expansion**: Recursively expand nested types to understand complex data structures
- **Token Budgeting**: Limit output tokens for LLM context constraints
- **Rich Metadata**: Visibility, generics, deprecation, attributes, function modifiers
- **Filtering**: Glob patterns, kind filtering, visibility filtering, crate filtering
- **Multiple Output Formats**: Human-readable text and JSON output
- **Detail Levels**: Minimal (signatures only), Standard, Detailed modes

## Installation

```bash
cargo install cargo-doc-query
```

Or run directly:

```bash
cargo run --quiet -- query <path>
```

## Quick Start

```bash
# Build the documentation index (run first)
cargo doc-query build

# Query a type's methods and traits
cargo doc-query query std::vec::Vec
cargo doc-query query anyhow::Error --minimal

# Query with nested type expansion
cargo doc-query query anyhow::Error --depth 2
cargo doc-query query std::collections::HashMap --depth 1

# Query with token budget for LLM contexts
cargo doc-query query serde_json::Value --tokens 500
```

## v1.1 New Features

### Filter Flags

Filter results using glob patterns and criteria. Multiple filters combine with AND logic. Multiple values for the same flag combine with OR logic.

```bash
# Include only items from specific crate
cargo doc-query query Vec --include "std::*"

# Exclude test-related items
cargo doc-query query Error --exclude "*test*"

# Filter by item kind (struct, enum, trait, function, etc.)
cargo doc-query query Serialize --kind struct

# Combine filters
cargo doc-query query Serialize --include "serde::*" --kind fn
```

**Available Filter Flags:**

| Flag | Description |
|------|-------------|
| `--include, -i` | Include items matching glob pattern |
| `--exclude, -e` | Exclude items matching glob pattern |
| `--kind, -k` | Filter by item kind |
| `--crate-filter` | Filter by crate name |
| `--visibility` | Filter by visibility (pub, pub(crate), etc.) |
| `--only` | Include only matching items (shorthand, excludes all others) |

For detailed glob pattern syntax:

```bash
cargo doc-query query --help-filters
```

### Detail Level

Control the amount of metadata displayed in output:

```bash
# Minimal - signatures only
cargo doc-query query Vec --minimal

# Standard - default output
cargo doc-query query Vec

# Detailed - includes visibility, generics, docs, attributes
cargo doc-query query Vec --detailed
```

### Token Budget

Limit output tokens to control LLM context usage:

```bash
# Limit to approximately 500 tokens
cargo doc-query query serde_json::Value --tokens 500

# Strict limit with warning
cargo doc-query query Vec --tokens 200
```

The token budget is approximate and uses a heuristic based on field counts and text length.

### Depth Expansion

Recursively expand nested types to understand complex structures:

```bash
# No expansion - just methods and traits
cargo doc-query query Error

# Expand direct field types (depth 1)
cargo doc-query query Error --depth 1

# Recursively expand nested types
cargo doc-query query HashMap --depth 2
```

### JSON Output

For programmatic use:

```bash
# JSON output for parsing
cargo doc-query query Vec --json

# Combine with filters
cargo doc-query query Serialize --json --kind struct
```

## CLI Reference

```
cargo-doc-query is a tool for querying Rust crate documentation.

Usage: cargo-doc-query [OPTIONS] <COMMAND>

Commands:
  build  Generate documentation index from Rust dependencies
  query  Query a type's methods, traits, and optionally expand nested types
  help   Print this message or the help of the given subcommand(s)

Options:
  -m, --manifest <MANIFEST>  Path to Cargo.toml manifest (default: current directory)
      --all-features        Include all features when generating documentation
      --no-color           Disable colored output
  -q, --quiet               Suppress progress indicators and timing info
  -h, --help                Print help
  -V, --version             Print version
```

### Query Command

```
Usage: cargo-doc-query query [OPTIONS] [PATH]

Arguments:
  [PATH]  The path to query (e.g., std::vec::Vec)

Options:
      --depth <N>           Maximum recursion depth for expanding nested types
      --crate-name <CRATE>  Limit to specific crate
      --minimal             Output minimal representation
  -d, --detailed            Display detailed metadata
      --no-color            Disable colored output
      --tokens <N>          Maximum tokens in output
      --json                Output as JSON
  -q, --quiet               Suppress progress indicators
  -i, --include <PATTERN>   Include items matching glob pattern
  -e, --exclude <PATTERN>   Exclude items matching glob pattern
  -k, --kind <KIND>         Filter by item kind
      --crate-filter <CRATE> Filter by crate name
      --visibility <VIS>    Filter by visibility
      --only <PATTERN>      Include only matching items
      --help-filters        Display glob syntax help
```

## Architecture

- **Query Engine**: Matches paths against indexed items using exact matching
- **Type Expander**: Recursively expands nested types to specified depth
- **Filter Engine**: Applies glob patterns and criteria filters
- **Cache Store**: Persists index using blake3 hashing for invalidation

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | No cache found (run 'build' first) |
| 3 | Query returned no results |
| 4 | Build failed |
| 5 | Invalid query |
| 6 | Cache error |
| 7 | IO error |
| 8 | JSON parsing error |
| 9 | Configuration error |

## Performance

Typical performance metrics on modern hardware:

- **Query latency (cached)**: <10ms
- **Build time (small project)**: <5s
- **Cache size**: ~1-5MB per crate

## License

MIT
