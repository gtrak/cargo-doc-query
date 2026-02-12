---
phase: 04-advanced-features
plan: 01
date: 2026-02-12
status: complete
---

# Plan 04-01 Summary: Recursive Type Expansion

## Deliverables

### Types (src/types/expand.rs)
- `ExpansionResult` - Top-level result containing type graph and cycle info
- `TypeGraph` - Container for all discovered types with metadata
- `TypeNode` - Individual type with fields, variants, and generics
- `FieldInfo` - Struct/enum field with type information
- `VariantInfo` - Enum variant with discriminant and fields

### Query Engine (src/query/expand.rs)
- `TypeExpander` - Core expansion engine with:
  - Cycle detection via HashSet<Id>
  - Depth limiting
  - Multi-crate support
  - Field/variant extraction for structs and enums
  - Type formatting for complex types (slices, arrays, tuples, etc.)
- `expand_type()` - Convenience function for expansion

### CLI (src/cli/expand.rs)
- `ExpandCommand` with:
  - `--depth` flag for recursion limit (default: 3)
  - `--crate-name` flag for crate filtering
  - Manifest change detection (rebuilds on changes)
  - JSON output with timing

## Usage

```bash
# Expand a type to depth 1
cargo doc-query expand anyhow::Error --depth 1

# Expand with crate filter
cargo doc-query expand std::vec::Vec --depth 2 --crate-name std
```

## Verification

- ✅ `cargo doc-query expand --help` shows usage
- ✅ `cargo doc-query expand anyhow::Error --depth 1` returns valid JSON
- ✅ Cycle detection prevents infinite loops (via visited HashSet)
- ✅ Depth limit respected (--depth 1 stops at root level)

## Technical Notes

- Uses existing QueryEngine patterns from Phase 2
- Handles all rustdoc_types::Type variants for complete type info
- Borrow checker workarounds for HashMap access during iteration
- Field extraction handles plain, tuple, and unit structs
- Enum variant extraction handles all variant kinds

## Commits

- `7f2c3c8` feat(04-01): recursive type expansion with depth limits and cycle detection
