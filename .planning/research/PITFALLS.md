# Domain Pitfalls: Rustdoc JSON Parsing & API Query Tools

**Domain:** Tools that parse rustdoc JSON output and query Rust APIs
**Researched:** 2026-02-12
**Confidence:** HIGH (based on official rustdoc docs, cargo-semver-checks lessons, and rust-lang RFCs)

## Critical Pitfalls

### Pitfall 1: Format Version Blindness

**What goes wrong:**
Tool crashes or produces incorrect results when rustdoc JSON format changes. The `format_version` field exists but tools ignore it, assuming the schema is stable.

**Why it happens:**
- Rustdoc JSON is explicitly unstable (nightly-only)
- Format changes 3-5 times per year on average
- Developers test on their current Rust version only
- No compile-time enforcement of format compatibility

**How to avoid:**
1. **Always check `format_version`** before parsing
2. **Support a range of format versions** (e.g., 28-35) rather than one version
3. **Use the `rustdoc-types` crate** — it provides versioned types
4. **Fail fast with clear errors** when encountering unknown format versions
5. **Update format version support proactively** with new Rust releases

**Warning signs:**
- Tool works on developer's machine but fails in CI
- Parsing errors after `rustup update`
- Fields that used to exist are now `null` or missing
- Enum variants have different structures

**Phase to address:**
Phase 1 (JSON Ingestion & Schema Handling)

---

### Pitfall 2: Cross-Crate ID Resolution Failures

**What goes wrong:**
Tools fail to resolve type/method references that cross crate boundaries. IDs point to items in dependency crates that aren't in the current JSON blob's `index`.

**Why it happens:**
- IDs are only valid within a single JSON blob
- External items are referenced but not fully defined in the local crate's JSON
- The `paths` field provides minimal info (just name/path) for external items
- Multiple versions of the same crate create ambiguity

**How to avoid:**
1. **Understand the two-tier lookup:**
   - First check `index` for local items and external trait definitions
   - Fall back to `paths` for external items (gives name only)
2. **Handle missing external crate info gracefully**
3. **Use `external_crates` map** to understand crate dependencies
4. **Don't assume you can resolve all IDs**
5. **For deep analysis, generate JSON for all dependencies**

**Warning signs:**
- "Missing item" errors for types clearly present in dependencies
- Methods returning external types showing as `Unknown`
- Panics on ID lookup in the index
- Broken links to external crate documentation

**Phase to address:**
Phase 2 (Query Engine Core)

---

### Pitfall 3: Multiple Package Version Output Collision

**What goes wrong:**
When a crate depends on multiple versions of the same package, rustdoc JSON output files collide. The second version overwrites the first.

**Why it happens:**
- Rustdoc uses crate name as filename base: `target/doc/{crate_name}.json`
- Cargo allows multiple versions via dependency renaming
- No version disambiguation in output filenames by default

**How to avoid:**
1. **Use `-Cmetadata` or `-Cextra-filename`** flags to disambiguate output
2. **Generate JSON for each version separately** with explicit package specs
3. **Store outputs in versioned directories**
4. **Use content-hash based storage** to deduplicate identical outputs
5. **Check for file existence before writing**

**Warning signs:**
- Missing types from "old" version of a dependency
- Intermittent test failures depending on build order
- File timestamps don't match expected build times
- Different query results on clean vs. incremental builds

**Phase to address:**
Phase 1 (Build Orchestration)

---

### Pitfall 4: Unbounded Recursive Type Expansion

**What goes wrong:**
When expanding generic types or following trait hierarchies, tools hit infinite recursion or stack overflow.

**Why it happens:**
- Rust allows recursive type definitions: `struct Node { children: Vec<Node> }`
- Generic substitution can expand exponentially
- Trait bounds create cycles
- No natural termination point for naive recursive traversal

**How to avoid:**
1. **Always implement depth limits** — hard cap at 10-20 levels
2. **Track visited type IDs** — use a `HashSet<Id>` to detect cycles
3. **Implement breadth-first expansion** instead of depth-first
4. **Use "opaque" type representation** when depth exceeded
5. **Test with pathological crates** like `aws-sdk-ec2`, `windows-rs`

**Warning signs:**
- Stack overflow errors on certain crate queries
- Tool hangs when querying types with many generic parameters
- Exponential slowdown on trait-heavy code
- Memory usage ballooning during type expansion

**Phase to address:**
Phase 3 (Type Expansion & Resolution)

---

### Pitfall 5: Memory Exhaustion on Large Crates

**What goes wrong:**
Tools crash with OOM or become unusably slow when processing large crates. `aws-sdk-ec2` produces ~500MB of JSON.

**Why it happens:**
- Some crates have massive APIs (AWS SDK, Windows API bindings)
- Loading entire JSON into memory doubles RAM usage
- No streaming or pagination support
- Serde's default HashMap has high memory overhead

**How to avoid:**
1. **Use streaming JSON parsing** for large files when possible
2. **Enable the `rustc-hash` feature** on `rustdoc-types` — 3% faster, lower memory
3. **Implement lazy loading** — parse only what's needed for the query
4. **Add memory limits** — fail gracefully instead of OOM
5. **Consider binary formats** — docs.rs notes space inefficiency of JSON

**Warning signs:**
- OOM kills on CI runners
- Tool is fast on small crates, unusable on large ones
- Memory usage growing linearly with crate size
- Swap thrashing during JSON parsing

**Phase to address:**
Phase 1 (JSON Ingestion) and Phase 6 (Performance Optimization)

**Sources:**
- [rustdoc-types docs](https://docs.rs/rustdoc-types/latest/rustdoc_types/): "cargo-semver-checks saw a -3% improvement when benchmarking using the aws_sdk_ec2 JSON output (~500MB of JSON)"

---

### Pitfall 6: Cache Invalidation Complexity

**What goes wrong:**
Tools either rebuild too often (slow) or use stale data (incorrect). Getting cache invalidation right is notoriously hard.

**Why it happens:**
- Rustdoc JSON depends on source code AND compiler version
- Feature flags change the public API
- Transitive dependencies affect output
- Build profiles affect available items

**How to avoid:**
1. **Use content-addressable storage** — hash the JSON output itself
2. **Track all inputs:** source hash, rustc version, feature flags, Cargo.lock
3. **Implement per-crate granularity** — only rebuild changed crates
4. **Use file modification times** as a fast-path check
5. **Store metadata alongside cached data** for validation

**Warning signs:**
- Queries returning data from old crate versions
- Rebuilding when nothing changed
- Not rebuilding when dependencies updated
- Inconsistent results across machines

**Phase to address:**
Phase 4 (Caching Layer)

---

### Pitfall 7: Assuming rustdoc Always Succeeds

**What goes wrong:**
Tools assume rustdoc JSON generation always works if `cargo build` succeeds. But rustdoc can fail independently.

**Why it happens:**
- Doc comments can have broken intra-doc links
- `#[doc(hidden)]` items affect what's available
- Nightly compiler bugs (it's unstable!)
- proc-macros can fail during doc generation

**How to avoid:**
1. **Always check rustdoc exit code**
2. **Capture and parse stderr** for meaningful error messages
3. **Implement fallback strategies** — e.g., skip docs on failure
4. **Handle missing JSON files gracefully**
5. **Test with broken doc links** in your test suite

**Warning signs:**
- "File not found" errors for expected JSON files
- Empty query results on valid crates
- Silent failures where tool appears to work but has no data

**Phase to address:**
Phase 1 (Build Orchestration)

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hardcode format version | Simpler code | Breaks every Rust update | Never — use version ranges |
| Load all JSON into memory | Easier parsing | OOM on large crates | MVP only, fix before prod |
| Ignore external crate IDs | Simpler queries | Missing cross-crate info | MVP only |
| Skip cycle detection | Faster implementation | Stack overflow crashes | Never — always detect cycles |
| Use string IDs instead of typed | Less code | Bugs from ID confusion | Never — use rustdoc-types |
| Assume single crate version | Simpler file handling | Data loss on version conflicts | Never — handle collisions |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| **rustdoc nightly** | Assume stable API | Pin to known-good nightly, test before updating |
| **Cargo workspace** | Build each crate separately | Build workspace together to resolve dependencies |
| **Feature flags** | Ignore them | Track enabled features as cache key inputs |
| **Proc macros** | Assume they expand correctly | Handle expansion failures gracefully |
| **Cross-compilation** | Use host target only | Respect `--target` flag for correct JSON |
| **RUSTFLAGS** | Ignore them | Include RUSTFLAGS in cache invalidation |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Naive HashMap | High memory, slow lookups | Use FxHashMap from rustc-hash | >10MB JSON files |
| Loading entire crate | OOM, slow startup | Lazy/partial loading | >100MB JSON files |
| No query caching | Repeated work | Cache query results | Repeated queries |
| String copying everywhere | High memory churn | Use references/Arc<str> | High query volume |
| Synchronous everything | Slow responses | Async I/O for multiple crates | Workspace queries |
| No depth limits | Stack overflow | Cap recursion at 10-20 | Deep generic hierarchies |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Deserializing untrusted JSON | RCE via malicious JSON | Use serde's deny_unknown_fields, validate input |
| No output size limits | DoS via huge JSON | Set max file size limits |
| Path traversal in cache | Write files anywhere | Validate cache keys, sandbox paths |
| Executing rustdoc on untrusted code | Arbitrary code execution | Sandbox, don't auto-build untrusted |

---

## "Looks Done But Isn't" Checklist

- [ ] **Format version handling:** Actually checks version, doesn't just parse
- [ ] **Cross-crate IDs:** Handles external references gracefully
- [ ] **Version collisions:** Works with multiple versions of same crate
- [ ] **Recursion limits:** Has hard caps on all recursive operations
- [ ] **Memory limits:** Fails gracefully before OOM
- [ ] **Cache invalidation:** Rebuilds when any input changes
- [ ] **Error handling:** Handles rustdoc failures, missing files
- [ ] **Large crate support:** Tested on aws-sdk-ec2 or similar
- [ ] **Feature flag awareness:** Different queries respect features
- [ ] **Workspace support:** Handles multi-crate projects

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Format version mismatch | LOW | Update rustdoc-types, rebuild |
| Cross-crate resolution failure | MEDIUM | Generate JSON for dependencies, rebuild index |
| Version collision | LOW | Use versioned output directories |
| Stack overflow | MEDIUM | Add cycle detection, limit depth |
| OOM on large crate | HIGH | Implement streaming, add memory limits |
| Cache corruption | LOW | Clear cache, rebuild |
| Rustdoc failure | LOW | Check error, fix source or skip crate |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Format Version Blindness | Phase 1 | Test with multiple Rust versions |
| Cross-Crate ID Failures | Phase 2 | Query types from external crates |
| Version Output Collision | Phase 1 | Test with serde 1.x and 2.x in same project |
| Recursive Expansion | Phase 3 | Test with recursive generic types |
| Memory Exhaustion | Phase 1, 6 | Test on aws-sdk-ec2 JSON |
| Cache Invalidation | Phase 4 | Modify dependency, verify rebuild |
| Rustdoc Failures | Phase 1 | Test with broken doc links |

---

## Sources

- [rustdoc-types crate documentation](https://docs.rs/rustdoc-types/latest/rustdoc_types/)
- [RFC 2963: rustdoc JSON](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html)
- [Rustdoc JSON 2023 Review](https://alona.page/posts/rustdoc-json-2023/)
- [rust-lang/rust#142370: Multiple package versions clobber output](https://github.com/rust-lang/rust/issues/142370)
- [cargo#16291: rustdoc JSON rebuild issues](https://github.com/rust-lang/cargo/issues/16291)
- [cargo-semver-checks blog posts](https://predr.ag/blog/)
- [Trustfall rustdoc adapter](https://github.com/obi1kenobi/trustfall-rustdoc-adapter)

---
*Pitfalls research for: cargo-doc-query*
*Researched: 2026-02-12*
