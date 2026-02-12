# External Integrations

**Analysis Date:** 2026-02-12

## APIs & External Services

**None Currently**

**Planned Integrations** (per `plan.md`):
- `cargo rustdoc` - For generating JSON documentation output
  - Requires nightly Rust compiler
  - Uses unstable flag: `--output-format json`

## Data Storage

**Databases:**
- None

**File Storage:**
- Local filesystem only
- Planned cache location: `target/doc-query/`
  - Sharded crate binaries (`.bin` files)
  - Metadata JSON

**Caching:**
- Planned: Content-hash based disk cache
- Cache key derived from: `Cargo.lock` hash + `rustc --version`

## Authentication & Identity

**Auth Provider:**
- None (not applicable for CLI tool)

## Monitoring & Observability

**Error Tracking:**
- None

**Logs:**
- Standard output via `println!`
- No structured logging currently implemented

## CI/CD & Deployment

**Hosting:**
- None

**CI Pipeline:**
- None (no `.github/workflows` directory)

**Distribution:**
- Planned: crates.io publication as `cargo-doc-query`
- Installation via `cargo install cargo-doc-query`

## Environment Configuration

**Required env vars:**
- None currently

**Secrets location:**
- Not applicable

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Rust Toolchain Dependencies

**Required:**
- Rust nightly toolchain (for rustdoc JSON output)
- Cargo (standard)

**Command:**
```bash
cargo rustdoc -- -Z unstable-options --output-format json
```

---

*Integration audit: 2026-02-12*
