# Architecture Research: Rust Documentation Query Tool

**Domain:** Rust CLI tool for querying rustdoc JSON documentation
**Researched:** 2026-02-12
**Confidence:** HIGH

## System Overview

A 4-layer architecture for parsing, indexing, and querying Rust documentation:

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                Command Line Interface                 │  │
│  │              (clap derive macros)                     │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      Commands Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │    build     │  │   methods    │  │    expand        │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
├─────────┴─────────────────┴────────────────────┴────────────┤
│                        Index Layer                           │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Documentation Graph Model               │   │
│  │    (Type → Method, Trait → Implementor edges)      │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                   Parser & Cache Layer                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Parser     │  │    Cache     │  │  Persistence │      │
│  │ (serde_json) │  │   (Index)    │  │ (postcard)   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### CLI Layer

| Component | Responsibility | Implementation |
|-----------|----------------|----------------|
| `Cli` struct | Define command-line interface | `clap::Parser` derive macro |
| `Commands` enum | Enumerate all subcommands | `clap::Subcommand` derive |
| Global flags | Options available to all commands | `#[arg(global = true)]` |
| Help generation | Auto-generated --help output | Built into clap |

### Commands Layer

| Component | Responsibility | Implementation |
|-----------|----------------|----------------|
| `build` | Generate rustdoc JSON, populate cache | Calls rustdoc, invokes parser |
| `methods` | Query methods for a given type | Uses Index graph lookups |
| `expand` | Show type hierarchy with depth limit | Graph traversal with depth |
| `traits` | Query trait implementations | Edge queries on trait graph |

### Index Layer

| Component | Responsibility | Implementation |
|-----------|----------------|----------------|
| `GraphIndex` | In-memory graph of type relationships | `petgraph::Graph<Node, Edge>` |
| `Node` | Type/method/trait representation | Custom struct with metadata |
| `Edge` | Relationship (implements, has_method, etc) | Enum with edge type |
| `Lookup` | Fast ID → node resolution | `HashMap<Id, NodeIndex>` |

### Parser & Cache Layer

| Component | Responsibility | Implementation |
|-----------|----------------|----------------|
| `Parser` | Deserialize rustdoc JSON | `serde_json` → `rustdoc_types::Crate` |
| `Cache` | Persistent storage of parsed/indexed data | `target/doc-query/` directory |
| `CacheEntry` | Per-crate indexed data | Binary format via `postcard` |
| `Metadata` | Cache version, timestamp, dependencies | `metadata.json` |

## Recommended Project Structure

```
src/
├── main.rs                 # Entry point, CLI parsing
├── cli.rs                  # Clap CLI definition
├── commands/
│   ├── mod.rs              # Command trait definition
│   ├── build.rs            # Build command implementation
│   ├── methods.rs          # Methods query command
│   ├── expand.rs           # Type expansion command
│   └── traits.rs           # Trait query command
├── index/
│   ├── mod.rs              # Index module exports
│   ├── graph.rs            # Petgraph wrapper
│   ├── node.rs             # Node types and metadata
│   └── edge.rs             # Edge types
├── parser/
│   ├── mod.rs              # Parser module exports
│   └── rustdoc.rs          # Rustdoc JSON deserialization
└── cache/
    ├── mod.rs              # Cache module exports
    ├── disk.rs             # File I/O operations
    ├── metadata.rs         # Cache metadata handling
    └── entry.rs            # Individual cache entry
```

### Structure Rationale

- **`commands/`:** Each command gets its own file implementing a `RunCommand` trait. This follows the Command Pattern and enables filesystem-based routing where command structure mirrors file structure.

- **`index/`:** Separates graph operations from the rest of the system. Graph is the core data structure for all queries.

- **`parser/`:** Isolates rustdoc JSON deserialization. Can be swapped if rustdoc format changes.

- **`cache/`:** Handles persistence separately from indexing. Enables cache invalidation strategies without affecting graph logic.

## Architectural Patterns

### Pattern 1: Command Pattern with Trait

**What:** Each subcommand implements a `RunCommand` trait that encapsulates its execution logic. The CLI parses arguments, then calls `run()` on the matched command.

**When to use:** For all CLI tools with multiple subcommands. Provides clean separation and makes commands testable independently.

**Trade-offs:** 
- **Pros:** Clean separation of concerns, easy to add new commands, testable
- **Cons:** Slight boilerplate for trait implementation

**Example:**
```rust
// src/commands/mod.rs
use anyhow::Result;

pub trait RunCommand {
    async fn run(&self) -> Result<()>;
}

// src/commands/build.rs
use super::RunCommand;

pub struct BuildCommand {
    pub manifest_path: Option<PathBuf>,
    pub toolchain: Option<String>,
}

#[async_trait::async_trait]
impl RunCommand for BuildCommand {
    async fn run(&self) -> Result<()> {
        // Build rustdoc JSON
        let json_path = rustdoc_json::Builder::default()
            .manifest_path(&self.manifest_path)
            .toolchain(&self.toolchain)
            .build()?;
        
        // Parse and index
        let krate = parser::parse_file(&json_path)?;
        let index = index::Index::from_crate(krate)?;
        
        // Cache
        cache::write(&index)?;
        
        Ok(())
    }
}
```

### Pattern 2: Graph-Based Index

**What:** Transform the hierarchical rustdoc JSON into a graph structure with bidirectional edges. Enables efficient traversal queries.

**When to use:** When you need to answer relationship queries ("what implements this trait?", "what methods does this type have?").

**Trade-offs:**
- **Pros:** Efficient for complex queries, supports graph algorithms (shortest path, topological sort)
- **Cons:** Memory overhead for edge storage, requires graph library dependency

**Example:**
```rust
// src/index/graph.rs
use petgraph::graph::{Graph, NodeIndex};
use rustdoc_types::Id;
use std::collections::HashMap;

pub struct DocGraph {
    graph: Graph<Node, Edge>,
    id_to_index: HashMap<Id, NodeIndex>,
}

pub enum Node {
    Type(TypeInfo),
    Trait(TraitInfo),
    Method(MethodInfo),
    Module(ModuleInfo),
}

pub enum Edge {
    Implements,      // Type -> Trait
    HasMethod,       // Type -> Method
    DefinedIn,       // Item -> Module
    Extends,         // Type -> Parent Type
}

impl DocGraph {
    pub fn methods_of_type(&self, type_id: &Id) -> Vec<&MethodInfo> {
        let idx = self.id_to_index.get(type_id)?;
        self.graph
            .edges(*idx)
            .filter(|e| matches!(e.weight(), Edge::HasMethod))
            .filter_map(|e| match &self.graph[e.target()] {
                Node::Method(m) => Some(m),
                _ => None,
            })
            .collect()
    }
}
```

### Pattern 3: Two-Level Cache

**What:** Cache consists of (1) a metadata JSON file tracking versions/timestamps and (2) binary data files per crate containing serialized index data.

**When to use:** When rustdoc JSON generation is slow and queries need to be fast. Enables incremental updates.

**Trade-offs:**
- **Pros:** Fast subsequent queries, supports invalidation by crate version or source hash
- **Cons:** Cache invalidation complexity, disk space usage

**Example:**
```rust
// src/cache/metadata.rs
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct CacheMetadata {
    pub version: u32,                    // Cache format version
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub entries: HashMap<String, CacheEntry>,  // crate_name -> entry
}

#[derive(Serialize, Deserialize)]
pub struct CacheEntry {
    pub crate_version: String,
    pub source_hash: String,             // Hash of Cargo.toml + src
    pub index_file: String,              // Path to .bin file
    pub indexed_at: chrono::DateTime<chrono::Utc>,
}

// src/cache/disk.rs
use postcard;

pub fn write_index(crate_name: &str, index: &Index) -> Result<()> {
    let path = format!("target/doc-query/{}.bin", crate_name);
    let encoded = postcard::to_allocvec(index)?;
    std::fs::write(&path, encoded)?;
    Ok(())
}

pub fn read_index(crate_name: &str) -> Result<Index> {
    let path = format!("target/doc-query/{}.bin", crate_name);
    let bytes = std::fs::read(&path)?;
    let index = postcard::from_bytes(&bytes)?;
    Ok(index)
}
```

## Data Flow

### Build Flow

```
User runs: cargo doc-query build
    ↓
[CLI] Parses arguments → BuildCommand
    ↓
[Commands] Invokes rustdoc JSON generation
    ↓
[Parser] Deserializes JSON → rustdoc_types::Crate
    ↓
[Index] Builds graph from Crate
    ↓
[Cache] Serializes Index → .bin file
    ↓
Updates metadata.json
```

### Query Flow

```
User runs: cargo doc-query methods std::vec::Vec
    ↓
[CLI] Parses arguments → MethodsCommand
    ↓
[Commands] Checks cache freshness
    ↓
[Cache] Loads Index from .bin (or triggers build)
    ↓
[Index] Graph lookup: find type node
    ↓
[Index] Traverse HasMethod edges
    ↓
[Commands] Format output
    ↓
Print to stdout
```

### Data Transformation

```
Rust Source Code
       ↓ (rustdoc)
JSON File (target/doc/<crate>.json)
       ↓ (serde_json)
rustdoc_types::Crate
       ↓ (graph construction)
Index (petgraph::Graph)
       ↓ (postcard)
Binary Cache (.bin file)
       ↓ (postcard)
Index (in-memory graph)
       ↓ (graph queries)
Query Results
```

## Build Order Implications

Based on component dependencies, recommended implementation order:

1. **Parser Layer** (Foundation)
   - Depends on: `rustdoc-types` crate
   - Provides: `parse_file()` function
   - Blocked by: None

2. **Cache Layer** (Persistence)
   - Depends on: Parser (for types)
   - Provides: `read()`, `write()` functions
   - Blocked by: Parser data structures

3. **Index Layer** (Core Logic)
   - Depends on: Parser types, Cache for storage
   - Provides: `Index::from_crate()`, query methods
   - Blocked by: Parser, Cache interface defined

4. **Commands Layer** (User Interface)
   - Depends on: Index for queries, Cache for storage
   - Provides: Command implementations
   - Blocked by: Index, Cache

5. **CLI Layer** (Entry Point)
   - Depends on: Commands
   - Provides: `main()` function
   - Blocked by: All other layers

### Dependency Graph

```
CLI
  ↓
Commands
  ↓
Index ←──→ Cache
  ↓          ↓
Parser ←───→ Cache
  ↓
rustdoc-types (external)
```

## Anti-Patterns

### Anti-Pattern 1: Direct JSON Querying

**What people do:** Parse JSON on every query and search through arrays/objects directly.

**Why it's wrong:** 
- O(n) lookups for every query
- No relationship navigation (can't ask "what implements this trait?")
- Repetitive JSON parsing overhead

**Do this instead:** Build an in-memory index (graph or HashMap-based) during build phase, query the index during operations.

### Anti-Pattern 2: Monolithic Cache

**What people do:** Store entire workspace index in single cache file.

**Why it's wrong:**
- Must rebuild entire cache when any crate changes
- Memory overhead loading unused crate data
- Slower cache validation

**Do this instead:** Per-crate cache files with metadata tracking. Only load crates needed for current query.

### Anti-Pattern 3: Synchronous Everything

**What people do:** Use blocking I/O for rustdoc generation and file operations.

**Why it's wrong:**
- Rustdoc generation can take seconds
- CLI feels unresponsive
- Can't show progress

**Do this instead:** Use async for I/O-bound operations (rustdoc generation, cache I/O) with `tokio` or `async-std`. Keep graph operations synchronous (CPU-bound).

### Anti-Pattern 4: Stringly-Typed Paths

**What people do:** Use `String` for all type paths without validation.

**Why it's wrong:**
- No compile-time validation
- Hard to handle generic types (`Vec<T>` vs `Vec<String>`)
- Inconsistent formatting

**Do this instead:** Define a `TypePath` struct with parsing and validation:
```rust
pub struct TypePath {
    pub crate_name: String,
    pub modules: Vec<String>,
    pub name: String,
    pub generics: Vec<TypePath>,
}

impl FromStr for TypePath {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse "std::vec::Vec<String>"
    }
}
```

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| rustdoc (nightly) | Command execution via `std::process::Command` | Requires nightly toolchain, use `rustdoc-json` crate for convenience |
| Cargo | `cargo_metadata` crate for workspace info | Already used by `rustdoc-json` |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI → Commands | Direct trait method call | Commands implement `RunCommand::run()` |
| Commands → Index | Borrow index from cache | Index is read-only after build |
| Index → Parser | Parser outputs `Crate`, Index consumes | One-way transformation |
| Cache ↔ Disk | `postcard` serialization | Binary format for speed |

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Single crate | All operations in memory, simple cache |
| Workspace (10-50 crates) | Lazy cache loading, per-crate index files |
| Large workspace (50+ crates) | Parallel rustdoc builds, incremental cache updates, query result caching |

### Performance Priorities

1. **Cache hit performance:** Index should load in <100ms for workspace queries
2. **Graph traversal:** Common queries (methods, traits) should complete in <10ms
3. **Cache miss penalty:** Rustdoc generation is unavoidable, but cache warming can help

## Sources

- [clap Subcommand Pattern](https://docs.rs/clap/latest/clap/trait.Subcommand.html) - Official docs, HIGH confidence
- [Building Well-Organized Rust CLI](https://bgenc.net/2023.12.09.building-well-organized-rust-cli-tool/) - Blog post, MEDIUM confidence
- [petgraph Documentation](https://docs.rs/petgraph/) - Official docs, HIGH confidence
- [rustdoc-types Crate](https://docs.rs/rustdoc-types/latest/rustdoc_types/) - Official types, HIGH confidence
- [rustdoc JSON RFC](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html) - Official RFC, HIGH confidence
- [postcard Binary Serialization](https://docs.rs/postcard/latest/postcard/) - Official docs, HIGH confidence
- [cargo-public-api Architecture](https://github.com/cargo-public-api/cargo-public-api) - Reference implementation, MEDIUM confidence

---
*Architecture research for: cargo-doc-query*
*Researched: 2026-02-12*
