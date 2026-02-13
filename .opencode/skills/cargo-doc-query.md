# Cargo Doc Query Skill

Tool for querying Rust documentation using cargo-doc-query.

## Usage

This skill allows the agent to query Rust crate documentation to understand APIs, types, and modules during development.

**Important:** This is the development version. Use the temporary entrypoint:
```
cargo run --release -- <command> [args]
```

After installation with `cargo install`, use:
```
cargo doc-query <command> [args]
```

## Commands

### Query
Query methods, traits, or modules for a type:
```
cargo run --release -- query <path> [--crate-name <name>] [--minimal] [--tokens <n>]
```

Examples:
- `cargo run --release -- query anyhow::Error`
- `cargo run --release -- query anyhow --crate-name anyhow`
- `cargo run --release -- query std::vec::Vec --minimal`

### Expand
Expand a type or module recursively to see its hierarchy:
```
cargo run --release -- expand <path> --depth <n> [--minimal] [--tokens <n>]
```

Examples:
- `cargo run --release -- expand anyhow::Error --depth 1`
- `cargo run --release -- expand anyhow --depth 2 --crate-name anyhow`

### Build
Build the documentation index (run first):
```
cargo run --release -- build
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

1. **First time**: Run `cargo run --release -- build` to generate the index
2. **Query types**: Use `query` for specific types or modules
3. **Explore hierarchies**: Use `expand` for recursive exploration
4. **Use filters**: Add `--minimal` or `--tokens` to control output size

## Output Format

All commands output JSON for easy parsing. Use `jq` for filtering:
```bash
cargo run --release -- query anyhow::Error | jq '.matches[0].content'
```

## Common Patterns

**Get all methods of a type:**
```
cargo run --release -- query <Type> | jq '.matches[].content.methods'
```

**List module contents:**
```
cargo run --release -- query <module_path> --crate-name <crate>
```

**Quick type info (minimal):**
```
cargo run --release -- query <Type> --minimal
```
