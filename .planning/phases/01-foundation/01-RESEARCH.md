# Phase 1: Foundation — JSON Ingestion & Index - Research

**Researched:** 2026-02-12  
**Domain:** Rust documentation tooling, rustdoc JSON, Cargo workspace integration  
**Confidence:** HIGH (primary sources: Context7, official docs, RFCs)

## Summary

This research covers the technical foundations for implementing a cargo subcommand that generates documentation indexes from rustdoc JSON output. The key challenges involve:

1. **rustdoc JSON generation** - Requires nightly Rust compiler with unstable flags
2. **Format validation** - The rustdoc-types crate provides official type definitions and version checking
3. **Graph indexing** - petgraph provides efficient directed graph structures for crate relationships
4. **Cargo integration** - cargo_metadata crate enables workspace dependency discovery
5. **Caching** - Content-addressable storage using BLAKE3 hashes with postcard binary serialization

**Primary recommendation:** Use the rustdoc-json helper crate to manage JSON generation, validate format versions via rustdoc-types::FORMAT_VERSION, and implement a directed graph index using petgraph::Graph with NodeIndex-based lookups.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rustdoc-types | 0.35+ | JSON format types | Official rust-lang types, tracks FORMAT_VERSION |
| rustdoc-json | 0.9+ | JSON generation helper | Wrapper around cargo rustdoc with proper error handling |
| cargo_metadata | 0.19+ | Workspace introspection | Official cargo integration, resolves dependencies |
| petgraph | 0.7+ | Graph data structure | Industry standard, used by rustc, O(V+E) space |
| postcard | 1.1+ | Binary serialization | Compact, fast, stable wire format since 1.0 |
| blake3 | 1.6+ | Content hashing | Fast, secure, used by iroh/bao for verified streaming |
| serde_json | 1.0+ | JSON parsing | Standard for rustdoc-types integration |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| camino | 1.1+ | UTF-8 paths | Required by cargo_metadata, prevents encoding issues |
| tempfile | 3.14+ | Test isolation | For integration tests with temporary cargo projects |
| anyhow | 1.0+ | Error handling | For ergonomic error propagation |
| thiserror | 2.0+ | Custom errors | For structured error types |

### Installation
```bash
cargo add rustdoc-types rustdoc-json cargo_metadata petgraph postcard blake3 serde_json anyhow thiserror
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs              # CLI entry point with clap
├── commands/
│   ├── mod.rs           # Command trait definition
│   └── build.rs         # BUILD-01: cargo doc-query build implementation
├── index/
│   ├── mod.rs           # Graph index public API
│   ├── graph.rs         # petgraph::Graph wrapper
│   ├── node.rs          # Node types (Module, Struct, Function, etc.)
│   └── edge.rs          # Edge types (Uses, Implements, etc.)
├── parser/
│   ├── mod.rs           # rustdoc JSON parsing
│   ├── validate.rs      # BUILD-05: Format version validation
│   └── loader.rs        # Multi-crate loading with collision handling
├── cache/
│   ├── mod.rs           # Cache public API
│   ├── key.rs           # BLAKE3-based cache key generation
│   └── store.rs         # Postcard-based serialization
└── cargo/
    ├── mod.rs           # Cargo integration
    └── dependencies.rs  # BUILD-02: Dependency discovery
```

### Pattern 1: rustdoc JSON Generation
**What:** Generate JSON documentation for workspace dependencies using rustdoc-json crate  
**When to use:** Building the documentation index  
**Example:**
```rust
// Source: https://docs.rs/rustdoc-json/latest/rustdoc_json/
use rustdoc_json::Builder;

let json_path = Builder::default()
    .toolchain("nightly")
    .manifest_path("Cargo.toml")
    .all_features(true)  // Include all features for complete docs
    .build()?;

// json_path is PathBuf to target/doc/<crate>.json
```

### Pattern 2: Format Version Validation
**What:** Check rustdoc-types::FORMAT_VERSION before parsing  
**When to use:** Immediately after JSON generation, fail fast on incompatibility  
**Example:**
```rust
// Source: https://docs.rs/rustdoc-types/latest/rustdoc_types/
use rustdoc_types::FORMAT_VERSION;
use serde_json::Value;

fn validate_format(json_str: &str) -> Result<(), String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    let version = value.get("format_version")
        .and_then(|v| v.as_u64())
        .ok_or("Missing format_version field")?;
    
    if version != FORMAT_VERSION as u64 {
        return Err(format!(
            "Format version mismatch: expected {}, got {}. \
             Please update rustdoc-types crate.",
            FORMAT_VERSION, version
        ));
    }
    
    Ok(())
}
```

### Pattern 3: Graph Index Construction
**What:** Build directed graph with crates as nodes and dependencies as edges  
**When to use:** After parsing all crate JSON files  
**Example:**
```rust
// Source: https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html
use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct CrateNode {
    name: String,
    version: String,
    json_path: std::path::PathBuf,
    rustdoc: Option<rustdoc_types::Crate>,
}

#[derive(Debug, Clone)]
enum DependencyEdge {
    Normal,
    Dev,
    Build,
}

struct CrateGraph {
    graph: Graph<CrateNode, DependencyEdge>,
    name_index: HashMap<(String, String), NodeIndex>,
}

impl CrateGraph {
    fn new() -> Self {
        Self {
            graph: Graph::new(),
            name_index: HashMap::new(),
        }
    }
    
    fn add_crate(&mut self, node: CrateNode) -> NodeIndex {
        let idx = self.graph.add_node(node.clone());
        self.name_index.insert(
            (node.name.clone(), node.version.clone()), 
            idx
        );
        idx
    }
    
    fn add_dependency(
        &mut self, 
        from: NodeIndex, 
        to: NodeIndex,
        kind: DependencyEdge
    ) {
        self.graph.add_edge(from, to, kind);
    }
}
```

### Pattern 4: Cargo Workspace Dependency Discovery
**What:** Use cargo_metadata to discover all workspace dependencies  
**When to use:** Before generating rustdoc JSON, to know what to document  
**Example:**
```rust
// Source: https://docs.rs/cargo_metadata/latest/cargo_metadata/
use cargo_metadata::{MetadataCommand, CargoOpt};

fn get_workspace_dependencies(manifest_path: &str) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .features(CargoOpt::AllFeatures)
        .exec()?;
    
    let mut deps = Vec::new();
    
    // Get all packages in the workspace
    for package in &metadata.packages {
        // Skip workspace members, only get external dependencies
        if !metadata.workspace_members.contains(&package.id) {
            deps.push((package.name.clone(), package.version.to_string()));
        }
    }
    
    // Remove duplicates (same crate, different versions)
    deps.sort();
    deps.dedup();
    
    Ok(deps)
}
```

### Pattern 5: Cache Key Generation
**What:** Create deterministic cache keys from inputs that affect the output  
**When to use:** Before build, to check if index needs regeneration  
**Example:**
```rust
// Source: https://docs.rs/blake3/latest/blake3/
use blake3::Hasher;
use std::collections::BTreeMap;  // BTree for deterministic ordering

struct CacheKeyInputs {
    cargo_lock_hash: String,      // Hash of Cargo.lock
    rustc_version: String,        // rustc --version
    target_triple: String,        // Target platform
    features: BTreeMap<String, bool>,  // Sorted features
}

fn generate_cache_key(inputs: &CacheKeyInputs) -> String {
    let mut hasher = Hasher::new();
    
    // Hash all inputs deterministically
    hasher.update(inputs.cargo_lock_hash.as_bytes());
    hasher.update(inputs.rustc_version.as_bytes());
    hasher.update(inputs.target_triple.as_bytes());
    
    // Features must be sorted for deterministic hashing
    for (feature, enabled) in &inputs.features {
        hasher.update(feature.as_bytes());
        hasher.update(&[*enabled as u8]);
    }
    
    hasher.finalize().to_hex().to_string()
}
```

### Pattern 6: Postcard Serialization
**What:** Binary serialization of graph index for fast loading  
**When to use:** Persisting built index to disk  
**Example:**
```rust
// Source: https://docs.rs/postcard/latest/postcard/
use postcard::{to_allocvec, from_bytes};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct SerializableIndex {
    format_version: u32,
    cache_key: String,
    nodes: Vec<CrateNode>,
    edges: Vec<(usize, usize, DependencyEdge)>,
}

fn serialize_index(index: &SerializableIndex) -> Result<Vec<u8>, postcard::Error> {
    to_allocvec(index)
}

fn deserialize_index(bytes: &[u8]) -> Result<SerializableIndex, postcard::Error> {
    from_bytes(bytes)
}
```

### Anti-Patterns to Avoid

1. **Don't parse rustdoc JSON without version checking**  
   The format changes between Rust versions. Always check `format_version` against `rustdoc_types::FORMAT_VERSION` before deserializing.

2. **Don't use crate name alone as identifier**  
   Multiple versions of the same crate can exist. Use `(name, version)` tuple or package ID.

3. **Don't assume JSON output path**  
   Output collision happens when multiple crate versions have the same name. Use `--output-format json` with explicit `--out-dir` and handle collisions by including version/hash in filename.

4. **Don't use JSON for cache storage**  
   JSON is verbose and slow. Use postcard (binary, compact) or similar binary format.

5. **Don't ignore Cargo features**  
   Different feature sets produce different documentation. Include feature flags in cache key.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON generation | Custom rustdoc invocation | `rustdoc-json` crate | Handles nightly toolchain, feature flags, and error cases |
| Type definitions | Hand-written structs | `rustdoc-types` crate | Official types, always in sync with rustdoc |
| Graph structure | Custom adjacency list | `petgraph::Graph` | Battle-tested, algorithms included, memory-efficient |
| Cargo parsing | Regex on Cargo.toml | `cargo_metadata` crate | Official API, handles all edge cases |
| Binary serialization | Custom byte format | `postcard` | Stable format, compact, fast, serde-compatible |
| Content hashing | std::collections::hash | `blake3` | Cryptographically secure, extremely fast |
| Path handling | String manipulation | `camino::Utf8PathBuf` | Pre-validated UTF-8, works with cargo_metadata |

**Key insight:** The rustdoc JSON ecosystem has mature solutions for all components. Building custom alternatives introduces format incompatibility risks and maintenance burden.

## Common Pitfalls

### Pitfall 1: Format Version Blindness
**What goes wrong:** Tool fails cryptically when rustdoc JSON format changes between Rust versions  
**Why it happens:** rustdoc JSON is unstable; format_version field changes with compiler releases  
**How to avoid:** 
- Always validate `format_version` against `rustdoc_types::FORMAT_VERSION` before parsing
- Fail fast with clear error message indicating version mismatch
- Document which rustdoc-types version is required  
**Warning signs:** "missing field" errors from serde, panics in production after Rust update

### Pitfall 2: Multiple Package Version Output Collision
**What goes wrong:** When workspace depends on `crate@1.0.0` and `crate@2.0.0`, only one JSON file is produced  
**Why it happens:** rustdoc uses `target/doc/<name>.json`, clobbering output for same-named crates  
**How to avoid:**
- Generate JSON in separate directories per crate version
- Use `--package <name>@<version>` with explicit output paths
- Implement collision detection: check if multiple crates share name before building
**Warning signs:** Missing items in index, "file not found" for expected crate version

### Pitfall 3: Cargo Cache Invalidation Issues
**What goes wrong:** `cargo rustdoc` skips rebuild when it shouldn't, producing stale JSON  
**Why it happens:** Cargo's fingerprinting doesn't always detect rustdoc-specific changes  
**How to avoid:**
- Always include feature flags in cache key
- Hash Cargo.lock content, not just timestamp
- Include rustc version in cache key (different compilers = different output)
- Force rebuild with `--force` flag when cache key mismatch detected
**Warning signs:** Index contains outdated function signatures, missing newly-added items

### Pitfall 4: Nightly Version Mismatch
**What goes wrong:** rustdoc-types version doesn't match installed nightly compiler  
**Why it happens:** rustdoc-types is published separately from Rust releases  
**How to avoid:**
- Pin rustdoc-types version in Cargo.toml
- CI should test against specific nightly version
- Provide clear error message with upgrade instructions
**Warning signs:** Format version errors despite using latest rustdoc-types

### Pitfall 5: Large Crate Memory Usage
**What goes wrong:** aws-sdk-ec2 JSON is ~500MB, causes OOM when loading  
**Why it happens:** Loading entire JSON into memory, then into graph  
**How to avoid:**
- Enable `rustc-hash` feature on rustdoc-types for 3% performance improvement
- Stream JSON parsing for very large crates
- Consider lazy loading: store paths, load JSON on demand
- Use `petgraph::Graph::with_capacity()` to pre-allocate
**Warning signs:** Slow builds, high memory usage, system thrashing

## Code Examples

### Complete Build Command Implementation

```rust
// BUILD-01: cargo doc-query build command
use anyhow::{Context, Result};
use rustdoc_json::Builder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct BuildCommand {
    manifest_path: PathBuf,
    target_dir: Option<PathBuf>,
    all_features: bool,
}

impl BuildCommand {
    pub fn execute(&self) -> Result<Index> {
        // 1. Discover dependencies (BUILD-02)
        let deps = self.discover_dependencies()?;
        
        // 2. Check cache validity
        let cache_key = self.generate_cache_key(&deps)?;
        if let Some(index) = self.try_load_cache(&cache_key)? {
            println!("Using cached index (key: {})", &cache_key[..16]);
            return Ok(index);
        }
        
        // 3. Generate rustdoc JSON for all dependencies
        let json_paths = self.generate_rustdoc_json(&deps)?;
        
        // 4. Parse and validate (BUILD-05)
        let mut index = Index::new();
        for (pkg_id, json_path) in json_paths {
            let json_str = std::fs::read_to_string(&json_path)
                .with_context(|| format!("Failed to read {}", json_path.display()))?;
            
            // Format version validation - fail fast!
            validate_format_version(&json_str)?;
            
            let krate: rustdoc_types::Crate = serde_json::from_str(&json_str)
                .with_context(|| format!("Failed to parse {}", json_path.display()))?;
            
            index.add_crate(pkg_id, krate);
        }
        
        // 5. Build dependency graph
        index.build_dependency_graph();
        
        // 6. Save to cache
        self.save_cache(&cache_key, &index)?;
        
        Ok(index)
    }
    
    fn generate_rustdoc_json(&self, deps: &[(PackageId, String)]) -> Result<HashMap<PackageId, PathBuf>> {
        let mut paths = HashMap::new();
        
        for (pkg_id, name) in deps {
            let builder = Builder::default()
                .toolchain("nightly")
                .manifest_path(&self.manifest_path)
                .package(&format!("{}@{}", name, pkg_id.version()));
            
            let builder = if self.all_features {
                builder.all_features(true)
            } else {
                builder
            };
            
            match builder.build() {
                Ok(path) => { paths.insert(pkg_id.clone(), path); }
                Err(e) => {
                    eprintln!("Warning: Failed to document {}: {}", name, e);
                    // Continue with other crates - don't fail entire build
                }
            }
        }
        
        Ok(paths)
    }
}

fn validate_format_version(json_str: &str) -> Result<()> {
    use rustdoc_types::FORMAT_VERSION;
    
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    let version = value.get("format_version")
        .and_then(|v| v.as_u64())
        .context("Missing format_version in rustdoc JSON")?;
    
    if version != FORMAT_VERSION as u64 {
        anyhow::bail!(
            "Format version mismatch: expected {}, got {}.\n\
             This usually means your rustdoc-types crate version doesn't match your Rust compiler.\n\
             Try: cargo update -p rustdoc-types",
            FORMAT_VERSION, version
        );
    }
    
    Ok(())
}
```

### Cache Key Design

```rust
// Recommended inputs for cache key
#[derive(Debug)]
struct CacheInputs {
    cargo_lock_content: String,   // Hash of Cargo.lock file
    rustc_version: String,        // Output of rustc --version
    target_triple: String,        // Target platform
    features_hash: String,        // Hash of enabled features
    rustdoc_types_version: String, // rustdoc-types crate version
}

impl CacheInputs {
    fn compute_cache_key(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        
        // All inputs that affect rustdoc JSON output
        hasher.update(self.cargo_lock_content.as_bytes());
        hasher.update(self.rustc_version.as_bytes());
        hasher.update(self.target_triple.as_bytes());
        hasher.update(self.features_hash.as_bytes());
        hasher.update(self.rustdoc_types_version.as_bytes());
        
        hasher.finalize().to_hex().to_string()
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual rustdoc invocation | rustdoc-json crate | 2023+ | Better error handling, toolchain management |
| Hand-rolled types | rustdoc-types crate | 2021+ | Always in sync with compiler |
| JSON cache storage | Postcard binary | 2024+ | ~70% smaller, faster deserialization |
| MD5/SHA256 hashing | BLAKE3 | 2024+ | 3-5x faster, modern standard |
| cargo_toml parsing | cargo_metadata crate | 2023+ | Official API, handles edge cases |

**Deprecated/outdated:**
- Direct `rustdoc` CLI invocation without wrapper: Use `rustdoc-json` crate instead
- save-analysis data: Deprecated in favor of rustdoc JSON
- syn-based parsing: Overkill when rustdoc JSON is available

## Open Questions

1. **Cross-crate ID resolution**
   - What we know: rustdoc JSON IDs are only valid within a single crate
   - What's unclear: Best approach for resolving references across crates
   - Recommendation: Use `paths` map from each crate's JSON, build global lookup table

2. **Incremental indexing for large workspaces**
   - What we know: aws-sdk-ec2 is ~500MB JSON, takes significant memory
   - What's unclear: Whether to support incremental updates vs full rebuilds
   - Recommendation: Start with full rebuilds, profile before optimizing

3. **Feature flag handling**
   - What we know: Different features produce different documentation
   - What's unclear: Whether to index all features or only enabled ones
   - Recommendation: Default to all features for completeness, allow opt-out

## Sources

### Primary (HIGH confidence)
- [rustdoc-types 0.35.0](https://docs.rs/rustdoc-types/0.35.0/rustdoc_types/) - Official type definitions, FORMAT_VERSION constant
- [rustdoc-json crate](https://docs.rs/rustdoc-json/latest/rustdoc_json/) - JSON generation wrapper
- [RFC 2963: rustdoc JSON](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html) - Official format specification
- [petgraph 0.7 docs](https://docs.rs/petgraph/0.7.0/petgraph/) - Graph algorithms and types
- [cargo_metadata](https://docs.rs/cargo_metadata/latest/cargo_metadata/) - Cargo workspace API
- [postcard wire format](https://postcard.jamesmunns.com/wire-format) - Stable binary format spec

### Secondary (MEDIUM confidence)
- [BLAKE3 docs](https://docs.rs/blake3/latest/blake3/) - Hashing API and performance characteristics
- [GitHub rust-lang/rust#142370](https://github.com/rust-lang/rust/issues/142370) - Multiple package version collision issue
- [GitHub rust-lang/cargo#16291](https://github.com/rust-lang/cargo/issues/16291) - Cargo rebuild detection bug

### Tertiary (LOW confidence)
- Community blog posts and discussions about rustdoc JSON tooling

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All crates are official/widely-used with stable APIs
- Architecture: HIGH - Based on rustdoc-types and petgraph official docs
- Pitfalls: MEDIUM-HIGH - Some edge cases from GitHub issues, verified with multiple sources

**Research date:** 2026-02-12  
**Valid until:** 2026-03-12 (30 days) - rustdoc-types updates frequently with Rust nightly

**Estimated Phase 1 Complexity:**
- BUILD-01 (build command): Medium - Well-understood, rustdoc-json handles complexity
- BUILD-02 (dependency discovery): Low - cargo_metadata is straightforward
- BUILD-05 (format validation): Low - Single version check against constant
- Graph indexing: Medium - Design decisions on node/edge types
- Caching: Medium - Key design is critical for correctness

**Success Criteria Verification:**
1. ✓ `cargo doc-query build` command: Use clap + rustdoc-json Builder
2. ✓ Generate rustdoc JSON for all dependencies: cargo_metadata → rustdoc-json per crate
3. ✓ Format version validation: Check against rustdoc_types::FORMAT_VERSION
4. ✓ Graph-based index: petgraph::Graph with CrateNode/DependencyEdge
5. ✓ Persist to disk: Postcard serialization with BLAKE3 cache key
