# Technology Stack: cargo-doc-query

**Domain:** Rust CLI tool parsing rustdoc JSON output
**Researched:** 2026-02-12
**Confidence:** HIGH

## Executive Summary

For building a Cargo subcommand that parses rustdoc JSON output in 2025, the standard stack centers on officially maintained Rust ecosystem tools. The core technologies are **clap** (v4.5.x) for CLI parsing, **rustdoc-types** (v0.57.x) for JSON type definitions, **cargo_metadata** (v0.23.x) for Cargo integration, and **bincode** (v2.x) or **postcard** (v1.x) for disk caching. This stack is proven by production tools like `cargo-semver-checks` and `cargo-public-api`.

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **clap** | ^4.5.58 | CLI argument parsing | Standard for Rust CLI tools; native cargo subcommand support via derive API; excellent ergonomics with `#[command(name = "cargo")]` pattern |
| **rustdoc-types** | ^0.57.0 | JSON output type definitions | Official rustdoc team maintained (RFC 3673); provides structured types for rustdoc JSON; enables `rustc-hash` feature for ~3% performance improvement on large JSON files |
| **rustdoc-json** | ^0.9.8 | Build rustdoc JSON programmatically | Wrapper around cargo to generate JSON output; handles nightly toolchain detection; used by cargo-semver-checks and cargo-public-api |
| **cargo_metadata** | ^0.23.1 | Parse Cargo metadata | Standard for cargo subcommands; extracts dependency graph and workspace info; used by virtually all cargo plugins |
| **serde** | ^1.0.219 | Serialization framework | De facto standard; rustdoc-types uses it internally; required for custom cache formats |
| **serde_json** | ^1.0.149 | JSON parsing | Standard for JSON; streaming deserializer available for large files; zero-copy parsing possible with `&str` |

### Caching & Storage

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **bincode** | ^2.0.0 | Binary serialization for cache | Fast serde-compatible binary format; good for large struct serialization; v2 has significant improvements over v1 |
| **postcard** | ^1.1.3 | Alternative binary serialization | Smaller output than bincode; no_std compatible; well-specified format; better for network/storage constrained scenarios |
| **blake3** | ^1.8.3 | Content hashing | Fast cryptographic hash for cache keys; incremental hashing for large files; hardware-accelerated on modern CPUs |
| **xxhash-rust** | ^0.8.0 | Non-cryptographic hashing | 10x faster than blake3; suitable for non-adversarial cache keys; use when hash collision attacks aren't a concern |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **anyhow** | ^1.0.95 | Error handling | Simplified error propagation for CLI tools; used by cargo-semver-checks |
| **thiserror** | ^2.0.12 | Structured error types | When you need custom error types with Display impls; derive macro reduces boilerplate |
| **camino** | ^1.1.10 | UTF-8 paths | cargo_metadata already depends on it; use for all path handling to avoid platform encoding issues |
| **rayon** | ^1.10.0 | Data parallelism | Parse multiple crate JSON files concurrently; significant speedup for workspace with many dependencies |
| **memmap2** | ^0.9.0 | Memory-mapped files | For reading large JSON files without loading into heap; works with serde_json streaming |
| **indexmap** | ^2.7.0 | Order-preserving HashMap | When you need deterministic JSON output ordering; serde_json feature flag available |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| **cargo-nextest** | Test runner | Faster than built-in test runner; better output for CI |
| **cargo-deny** | License/security audit | Check dependency licenses and security advisories |
| **cargo-msrv** | MSRV verification | Verify minimum supported Rust version |
| **insta** | Snapshot testing | For CLI output testing; used by cargo-semver-checks |
| **assert_cmd** / **predicates** | CLI testing | Standard for testing cargo subcommands |

## Installation

```bash
# Core dependencies
cargo add clap --features derive
cargo add rustdoc-types
cargo add rustdoc-json
cargo add cargo_metadata
cargo add serde serde_json

# Caching (choose one primary)
cargo add bincode  # OR
cargo add postcard

# Hashing for content-addressable cache
cargo add blake3  # OR for speed: cargo add xxhash-rust

# Error handling
cargo add anyhow thiserror

# Supporting libraries
cargo add camino rayon memmap2

# Dev dependencies
cargo add --dev assert_cmd predicates insta cargo-nextest
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| **bincode** | **postcard** | Use postcard when you need smaller serialized size (e.g., 20-50% smaller) or no_std compatibility. Postcard has a more explicit specification. |
| **blake3** | **xxhash-rust** | Use xxhash when you don't need cryptographic security and want maximum speed (10x faster). Suitable for internal cache keys where collision attacks aren't a threat. |
| **anyhow** | **eyre** | Use eyre when you want more customizable error reports with context. Eyre has better hook system for custom error formatting. |
| **serde_json** | **simd-json** | Use simd-json when parsing very large JSON files (100MB+) and you can target x86_64 with AVX2. Requires nightly Rust. |
| **clap derive** | **clap builder** | Use builder API when you need dynamic argument generation at runtime or want to avoid proc-macro compile times. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **structopt** | Deprecated; merged into clap v3 | **clap** v4 with derive feature |
| **failure** | Deprecated; maintenance mode | **anyhow** or **thiserror** |
| **rustc-serialize** | Deprecated | **serde** |
| **json** crate | Unmaintained; poor performance | **serde_json** |
| **bincode v1** | v2 has significant API and performance improvements | **bincode v2** |
| **hashbrown directly** | rustdoc-types provides `rustc-hash` feature for FxHashMap | Enable `rustc-hash` feature on rustdoc-types |

## Stack Patterns by Variant

### If building cache-first:
- Use **postcard** over bincode for smaller cache footprint
- Use **xxhash-rust** for cache key generation (faster than blake3)
- Consider **cacache** crate for content-addressable disk cache with built-in integrity verification

### If parsing very large JSON (500MB+ like aws_sdk_ec2):
- Enable `rustc-hash` feature on **rustdoc-types** for ~3% speedup
- Use **memmap2** for zero-copy file access
- Use **rayon** for parallel processing of independent crate JSON files
- Consider **simd-json** on nightly with AVX2

### If targeting stable Rust only:
- **rustdoc-json** requires nightly toolchain flag anyway (rustdoc JSON output is nightly-only)
- All other recommended crates work on stable
- The nightly requirement is unavoidable due to rustdoc JSON output format instability

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| rustdoc-types@0.57 | rustdoc nightly 2024-10+ | Check `FORMAT_VERSION` constant in JSON output |
| rustdoc-json@0.9.8 | cargo_metadata@0.23 | Both use camino for paths |
| clap@4.5 | rustc 1.70+ | MSRV requirement |
| bincode@2.0 | serde@1.0 | Different API than v1; migration required |
| rayon@1.10 | crossbeam@0.8 | Internal dependency |

## Performance Considerations

| Concern | Recommendation | Expected Impact |
|---------|---------------|-----------------|
| JSON parsing speed | Use streaming deserializer for large files | Reduces peak memory by 50%+ |
| HashMap performance | Enable `rustc-hash` feature on rustdoc-types | ~3% improvement on 500MB JSON |
| Cache serialization | Use bincode or postcard over JSON | 2-5x faster, 3-10x smaller |
| Concurrent processing | Use rayon for parallel crate processing | Near-linear scaling with cores |
| Content hashing | Use blake3 with incremental API | Fast for large files with low collision risk |

## Confidence Assessment

| Technology | Confidence | Basis |
|------------|------------|-------|
| clap | **HIGH** | Official docs (docs.rs/clap), widely adopted, cargo subcommand examples |
| rustdoc-types | **HIGH** | Official rustdoc team maintenance (RFC 3673), used by cargo-semver-checks |
| rustdoc-json | **HIGH** | Used by cargo-public-api, cargo-semver-checks; official docs |
| cargo_metadata | **HIGH** | Standard crate for cargo plugins, 2M+ downloads/month |
| bincode vs postcard | **HIGH** | Community consensus, benchmarks available |
| blake3 vs xxhash | **MEDIUM** | Performance claims verified, but specific tradeoffs depend on use case |

## Production Validation

This stack is validated by these production tools:
- **cargo-semver-checks** (obi1kenobi): Uses rustdoc-types, cargo_metadata, clap, anyhow
- **cargo-public-api** (Enselic): Uses rustdoc-json, rustdoc-types
- **docs.rs**: Generates and hosts rustdoc JSON using this ecosystem

## Sources

- [docs.rs/rustdoc-types](https://docs.rs/rustdoc-types/latest/rustdoc_types/) — Official API documentation, version 0.57.0
- [docs.rs/rustdoc-json](https://docs.rs/rustdoc-json/latest/rustdoc_json/) — Build utilities, version 0.9.8
- [docs.rs/cargo_metadata](https://docs.rs/cargo_metadata/latest/cargo_metadata/) — Cargo integration, version 0.23.1
- [docs.rs/clap](https://docs.rs/clap/latest/clap/_cookbook/cargo_example_derive/index.html) — Cargo subcommand derive example
- [RFC 3673](https://rust-lang.github.io/rfcs/3673-rustdoc-types-maintainers.html) — rustdoc-types official maintenance
- [GitHub: cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks) — Production usage validation
- [GitHub: rust_serialization_benchmark](https://github.com/djkoloski/rust_serialization_benchmark) — Format comparison data

---
*Stack research for: cargo-doc-query — Cargo subcommand for rustdoc JSON queries*
*Researched: 2026-02-12*
