# Technology Stack

**Analysis Date:** 2026-02-12

## Languages

**Primary:**
- Rust 1.93.0 (2024 edition) - Core application language

**Secondary:**
- None

## Runtime

**Environment:**
- Rust native (compiled to binary)
- Edition: 2024

**Package Manager:**
- Cargo 1.93.0
- Lockfile: Not present (no dependencies)

## Frameworks

**Core:**
- None (vanilla Rust)

**Testing:**
- Built-in `cargo test` (Rust's standard test framework)

**Build/Dev:**
- Cargo (built-in build system)
- Rustc 1.93.0

## Key Dependencies

**Critical:**
- None (no dependencies declared)

**Infrastructure:**
- None

**Planned Dependencies** (per `plan.md`):
- `bincode` - For serialized cache storage
- `serde` - For JSON serialization
- `clap` - For CLI argument parsing
- Custom rustdoc JSON parser

## Configuration

**Environment:**
- No `.env` file present
- No environment configuration required currently

**Build:**
- `Cargo.toml` - Standard Rust package manifest
- `.gitignore` - Excludes `/target` build directory
- Edition: 2024 (latest Rust edition)

**Planned Configuration:**
- Cache directory: `target/doc-query/`
- Metadata file: `target/doc-query/metadata.json`

## Platform Requirements

**Development:**
- Rust 1.93.0 or later
- Cargo
- Nightly Rust required for `rustdoc --output-format json`

**Production:**
- Binary deployment as Cargo subcommand
- Compatible with all Rust-supported platforms

## Notes

This is a brand new project ("Hello, world!" stub) with no dependencies yet.
The `plan.md` describes a Cargo subcommand for querying Rust documentation.

---

*Stack analysis: 2026-02-12*
