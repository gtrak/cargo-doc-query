# Global Per-Crate Index Cache

## Problem

When a project's dependencies change (version bump, addition, removal), the current system invalidates the entire project cache and rebuilds all rustdoc JSON from scratch via `cargo +nightly doc --all-features`. This is slow because unchanged deps are rebuilt needlessly. Two projects that share the same `serde v1.0.204` cannot reuse each other's cached documentation.

## Root Cause

The current cache key is a single BLAKE3 hash of the project's `Cargo.toml` + `Cargo.lock` + `rustc_version` + `target_triple` + `features` + `rustdoc_types_version` (`src/cache/key.rs:54-70`). Any change to any dependency invalidates the entire index. All rustdoc JSON is stored at project-local paths (`target/.cargo-doc-query/doc/*.json`), and the index (`target/doc-query/<hash>.idx`) contains absolute paths to those project-local files (`SerializableCrateNode.json_path`).

## Solution

Replace the project-local monolithic cache with a **global per-crate cache**. Each crate's rustdoc JSON is keyed by `(name, version, env_hash)` where `env_hash = blake3(rustc_version + target_triple + features_hash)`. When deps change, only newly-added or version-changed crates need building. Everything else is found in the global cache instantly.

A `clean` command is added for cache management. Backwards compatibility with the old format is not required.

## Global Cache Layout

```
~/.cache/cargo-doc-query/              # XDG_CACHE_HOME on Linux, ~/Library/Caches/ on macOS
  crates/
    serde/
      1.0.204/
        <envhash>/
          serde.json
    anyhow/
      1.0.89/
        <envhash>/
          anyhow.json
    ...
```

Where `<envhash>` = `blake3(rustc_version + target_triple + "all-features")` — a 64-char hex directory name that groups JSON by build environment. The `features_hash` defaults to `"all-features"` since we always build with `--all-features`. If per-feature builds are added later, this becomes a hash of the sorted feature set.

## Project Index (Simplified)

```
target/doc-query/index.idx              # Fixed filename, no hash
```

Contains `format_version: 2` with nodes referencing `(name, version, env_hash)`. No more absolute `json_path` fields — JSON location is resolved at query time via `GlobalCacheStore`.

Old format: `target/doc-query/<blake3hash>.idx` with `format_version: 1` and `json_path` fields pointing to `target/.cargo-doc-query/doc/`. Old files are simply ignored.

## Build Flow

### Current Flow

```
1. Compute project cache key (BLAKE3 of Cargo.toml + Cargo.lock + ...)
2. Check if <hash>.idx exists
3. Cache miss → cargo doc --all-features (builds ALL deps)
4. Scan target/.cargo-doc-query/doc/*.json
5. Save .idx with absolute json_path per crate
```

### New Flow

```
1. Discover all deps (direct + transitive) via cargo metadata
2. For each dep, compute CrateCacheKey(name, version)
3. Check each against GlobalCacheStore
4. Partition into cached / uncached

   Fast path (all cached):
     "Found 18/18 dependencies in global cache"
     → Skip cargo doc entirely, just build project index

   Partial miss:
     "Found 15/18 in global cache, building 3..."
     → cargo +nightly doc -p <uncached1> -p <uncached2> -p <uncached3> --all-features
     → Copy resulting JSONs to global cache

   Complete miss:
     "No dependencies in global cache, building all 18..."
     → cargo +nightly doc -p <dep1> -p <dep2> ... --all-features
     → Copy resulting JSONs to global cache

5. Build SerializableIndex { format_version: 2, nodes: crate_refs }
6. Save to target/doc-query/index.idx
```

## Key Design: CrateCacheKey

```rust
pub struct CrateCacheKey {
    pub name: String,           // "serde"
    pub version: String,        // "1.0.204"
    pub rustc_version: String,  // from `rustc --version`
    pub target_triple: String,  // from std::env::consts (full triple)
    pub features_hash: String,  // "all-features" (sentinel for now)
}

impl CrateCacheKey {
    pub fn from_crate(name: &str, version: &str) -> Result<Self> {
        // Auto-captures rustc_version and target_triple
        // Default features_hash = "all-features"
    }

    pub fn env_hash(&self) -> String {
        // blake3(rustc_version + target_triple + features_hash)
        // 64-char hex string
    }

    pub fn json_filename(&self) -> String {
        // name.replace("-", "_") + ".json"
    }
}
```

## Experimental Verification

To validate the core assumption that `cargo doc -p <crate>` produces usable JSON for individual crates, I tested:

```bash
# In a project with serde and anyhow as dependencies:
cargo +nightly doc -p serde -p anyhow --all-features \
  RUSTDOCFLAGS="-Z unstable-options --output-format json --document-private-items" \
  CARGO_TARGET_DIR="target/.cargo-doc-query"

ls target/.cargo-doc-query/doc/
# serde.json  anyhow.json  (plus transitive dep JSONs like cfg_if.json)
```

The `-p` flag works for building individual crates, and the resulting JSON files are identical to what `cargo doc --all-features` (without `-p`) would produce for those same crates. Crucially, building with `-p` also builds and caches JSON for transitive dependencies in the same invocation, which means we get extra cache entries "for free."

If `--all-features` fails for a batch (conflicting features across crates), we fall back to building crates individually without `--all-features`, using `features_hash = "default-features"` in the cache key. Both variants coexist in the global cache.

## Changes By File

### NEW: `src/cache/global.rs`

`CrateCacheKey` and `GlobalCacheStore`:

- `CrateCacheKey::from_crate(name, version)` — auto-captures rustc, target, features_hash
- `CrateCacheKey::env_hash()` — blake3 for cache directory name
- `CrateCacheKey::json_filename()` — name with hyphens→underscores + `.json`
- `GlobalCacheStore::new()` — resolve XDG cache dir, create if needed
- `GlobalCacheStore::get(key)` → `Option<PathBuf>` — does JSON exist?
- `GlobalCacheStore::put(key, src_path)` → `Result<PathBuf>` — copy JSON into global cache
- `GlobalCacheStore::resolve(key)` → `PathBuf` — expected path (whether or not file exists)
- `GlobalCacheStore::clean()` → `Result<CacheStats>` — wipe entire `crates/` directory
- `GlobalCacheStore::stats()` → `Result<CacheStats>` — count entries and total size

Uses `dirs::cache_dir()` for XDG-compliant location.

### NEW: `src/cli/clean.rs`

`CleanCommand` with flags:

- Default (no flags) — clear project cache only (`target/doc-query/`, `target/.cargo-doc-query/`)
- `--global` — clear `~/.cache/cargo-doc-query/crates/`
- `--all` — clear both

Reports stats before/after removal.

### MODIFY: `src/cache/key.rs`

**Delete entire file.** All cache key logic moves to `src/cache/global.rs` as `CrateCacheKey`. The old `CacheKeyInputs` struct is no longer needed — the project index no longer uses a project-wide hash.

### MODIFY: `src/cache/store.rs`

**`SerializableCrateNode`:** Replace `json_path: String` with `env_hash: String`.

**`SerializableIndex`:** Remove `cache_key: String` field. Set `format_version: 2`.

**`CacheStore`:**
- `new()` → use fixed path `target/doc-query/index.idx`
- `save()` → `save(&self, index: &SerializableIndex)` — no key param
- `load()` → `load(&self) -> Result<Option<SerializableIndex>>` — loads from fixed path, no key param
- Remove `load_current()` — was mtime-scanning, no longer needed
- Remove all hash-keyed `.idx` logic

### MODIFY: `src/cache/mod.rs`

```rust
pub mod global;   // NEW
pub mod store;     // EXISTS
// REMOVE: pub mod key;
```

### MODIFY: `Cargo.toml`

Add `dirs = "6"` dependency.

### MODIFY: `src/cargo/dependencies.rs`

Add `get_all_dependencies(manifest_path)` function that returns direct + transitive deps. Uses `cargo_metadata` resolve graph to walk the full dependency tree from the root package. Excludes workspace members. Returns `Vec<(String, String, Utf8PathBuf)>` (name, version, manifest_path).

The existing `get_workspace_dependencies()` (direct-only) is kept for compatibility but the build command will call the new function.

### MODIFY: `src/cli/build.rs` (Major Rewrite)

New `execute()` flow:

1. Discover all deps via `get_all_dependencies(manifest_path)`
2. Compute `CrateCacheKey` for each dep
3. Check each against `GlobalCacheStore::get()` → partition into cached/uncached
4. Show progress: "Found {cached}/{total} in global cache"
5. If uncached deps exist:
   - Batch all uncached deps into single `cargo +nightly doc -p <d1> -p <d2> ... --all-features` with RUSTDOCFLAGS + CARGO_TARGET_DIR
   - If `--all-features` fails for batch, try building individually without it as fallback
   - Scan output dir for JSON files
   - Copy each to global cache via `GlobalCacheStore::put()`
6. Build `SerializableIndex { format_version: 2, nodes: crate_refs }` where each `CrateRef` has `(name, version, env_hash)`
7. Save to `target/doc-query/index.idx`

Remove:
- `CacheKeyInputs` usage and `generate_key()` call
- Hash-based cache check
- All code producing `json_path` in index nodes
- `TARGET_DIR` / `get_output_dir()` logic for project-local JSON (replaced by global cache paths)

### MODIFY: `src/cli/expand.rs`

- Load index via `CacheStore::load()` (fixed path)
- For each node, check `GlobalCacheStore::get(key)` exists
- If entries missing → rebuild just those crates (targeted rebuild)
- If no index exists → full build

### MODIFY: `src/query/expand.rs`

`TypeExpander::load_crate()` currently reads `json_path` from `crate_node.json_path`. Change to:

```rust
let key = CrateCacheKey::from_crate(crate_name, crate_version)?;
let global_store = GlobalCacheStore::new()?;
let json_path = global_store.get(&key)
    .ok_or_else(|| anyhow!("Crate {} v{} not in global cache", ...))?;
```

Similarly update `expand_type()` and `expand_type_with_config()` top-level functions.

### MODIFY: `src/query/engine.rs`

`QueryEngine::from_cache()` and `load_crate()` — same pattern: resolve JSON via `GlobalCacheStore` instead of `crate_node.json_path`.

### MODIFY: `src/cli/mod.rs`

Add: `pub mod clean;`

### MODIFY: `src/main.rs`

- Add `Commands::Clean { global, project, all }` variant with `--global`/`--project`/`--all` flags
- Wire up `CleanCommand`
- Remove `CacheStore::new()` from `run()` — no longer needed at top level
- Update `Commands::Build` handler for new `BuildCommand` API
- Update `suggest_similar_types()` to work with new index format

### MODIFY: `src/lib.rs`

Add `pub mod cargo;` — currently only declared in `main.rs`, needs to be accessible from `cache::global` for `get_all_dependencies`.

## What Gets Removed

| Item | Location | Reason |
|------|----------|--------|
| `CacheKeyInputs` struct | `src/cache/key.rs` | Replaced by `CrateCacheKey` in `global.rs` |
| Entire `src/cache/key.rs` file | — | All logic moved to `global.rs` |
| `SerializableIndex.cache_key` field | `src/cache/store.rs` | No project-wide hash needed |
| `SerializableCrateNode.json_path` field | `src/cache/store.rs` | Replaced by `env_hash`, resolved via `GlobalCacheStore` |
| `CacheStore::load_current()` | `src/cache/store.rs` | Was mtime-scanning for latest `.idx`, replaced by fixed `index.idx` |
| Hash-based `.idx` naming | `src/cache/store.rs` | Replaced by fixed `index.idx` path |
| Project-local JSON storage | `target/.cargo-doc-query/doc/` | JSON now lives in global cache |

## Clean Command

```bash
cargo doc-query clean              # Clear project cache only (default)
cargo doc-query clean --global    # Clear global cache (~/.cache/cargo-doc-query/crates/)
cargo doc-query clean --all        # Clear both project and global cache
```

Output:
```
Cleared project cache (target/doc-query/, target/.cargo-doc-query/)
Cleared global cache: 42 entries (1.2 GB)
```

## Cargo Doc Invocation Detail

For uncached deps, we batch them into a single invocation:

```bash
cargo +nightly doc \
  -p serde -p anyhow -p clap \
  --all-features \
  RUSTDOCFLAGS="-Z unstable-options --output-format json --document-private-items" \
  CARGO_TARGET_DIR="target/.cargo-doc-query"
```

After this completes, scan `target/.cargo-doc-query/doc/*.json` for all generated files, extract `(name, version)` from each JSON, and copy to the global cache.

Transitive deps built in this invocation are also cached — they get copied to the global cache as a bonus.

## Fallback Strategy

If `--all-features` fails for the batch (conflicting features between crates):

1. Try building each uncached crate individually without `--all-features`
2. Use `features_hash = "default-features"` for these entries in the cache
3. Both `all-features` and `default-features` variants can coexist in the global cache under different `env_hash` directories

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| Feature unification: `cargo doc -p serde --all-features` may produce different JSON than when built as part of a project with specific features | Cache key includes `features_hash`. Currently always `"all-features"`. If we add per-feature control later, the hash changes. Same crate version with different features gets different cache entries. |
| `cargo doc -p` builds transitive deps too, which may already be cached | This is a bonus — after building, we scan all JSON files and cache transitive deps too. No wasted work. |
| First build on a system is slightly slower (individual crate query overhead) | We batch all uncached deps into a single `cargo doc` invocation, so it's still one cargo call. |
| Large global cache over time | `clean --global` command. Per-crate keyed structure makes size-based eviction straightforward to add later. |
| `cargo doc -p` may not work for some crates (e.g., proc-macro crates) | The fallback strategy handles this. We also get a list of successfully generated JSON files from the output directory, so partial success is handled gracefully. |

## Execution Order

Files should be modified in this order to maintain compilability:

1. `Cargo.toml` — add `dirs` dependency
2. `src/cache/global.rs` — new file: `CrateCacheKey` + `GlobalCacheStore`
3. `src/cache/key.rs` — delete entire file
4. `src/cache/store.rs` — update `SerializableCrateNode`, `SerializableIndex`, `CacheStore`
5. `src/cache/mod.rs` — add `pub mod global;`, remove `pub mod key;`
6. `src/cargo/dependencies.rs` — add `get_all_dependencies()`
7. `src/cli/build.rs` — rewrite build flow
8. `src/cli/clean.rs` — new file: `CleanCommand`
9. `src/cli/mod.rs` — add `pub mod clean;`
10. `src/cli/expand.rs` — resolve JSON via global cache
11. `src/query/expand.rs` — resolve JSON via global cache in `load_crate()`
12. `src/query/engine.rs` — resolve JSON via global cache in `load_crate()` and `from_cache()`
13. `src/main.rs` — add `Commands::Clean`, wire up changes
14. `src/lib.rs` — add `pub mod cargo;`
15. Update all tests to match new structures