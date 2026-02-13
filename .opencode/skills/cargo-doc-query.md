# Cargo Doc Query Skill

Tool for querying Rust documentation using cargo-doc-query.

## Usage

This skill allows the agent to query Rust crate documentation to understand APIs, types, and modules during development.

**Important:** This is the development version. Use the temporary entrypoint:
```
cargo-doc-query <command> [args]
```

## Commands

### Query
Query methods, traits, or modules for a type:
```
cargo-doc-query query <path> [--crate-name <name>] [--minimal] [--tokens <n>]
```

Examples:
- `cargo-doc-query query anyhow::Error`
- `cargo-doc-query query anyhow --crate-name anyhow`
- `cargo-doc-query query std::vec::Vec --minimal`

### Expand
Expand a type or module recursively to see its hierarchy:
```
cargo-doc-query expand <path> --depth <n> [--minimal] [--tokens <n>]
```

Examples:
- `cargo-doc-query expand anyhow::Error --depth 1`
- `cargo-doc-query expand anyhow --depth 2 --crate-name anyhow`

### Build
Build the documentation index (run first):
```
cargo-doc-query build
```

## Flags

- `--crate-name <name>`: Limit to specific crate
- `--minimal`: Output minimal representation (signatures only)
- `--tokens <n>`: Limit output to approximately n tokens
- `--depth <n>`: Maximum recursion depth for expand

## When to Use

Use this skill when:
- Understanding external crate APIs
- Exploring type hierarchies
- Preparing context for LLM prompts about Rust code
- Checking method signatures and trait implementations
- Exploring module structures

## Workflow

1. **First time**: Run `cargo-doc-query build` to generate the index
2. **Query types**: Use `query` for specific types or modules
3. **Explore hierarchies**: Use `expand` for recursive exploration
4. **Use filters**: Add `--minimal` or `--tokens` to control output size

## Output Format

All commands output JSON for easy parsing. Use `jq` for filtering:
```bash
cargo-doc-query query anyhow::Error | jq '.matches[0].content'
```

## Common Patterns

**Get all methods of a type:**
```
cargo-doc-query query <Type> | jq '.matches[].content.methods'
```

**List module contents:**
```
cargo-doc-query query <module_path> --crate-name <crate>
```

**Quick type info (minimal):**
```
cargo-doc-query query <Type> --minimal
```
