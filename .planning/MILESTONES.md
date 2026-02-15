# Project Milestones: cargo-doc-query

## v1.1 Output Refinement (Shipped: 2026-02-14)

**Delivered:** Unified rendering, robust filtering, doc comment extraction, and rich metadata for LLM-optimized API queries.

**Phases completed:** 6-10 (20 plans total)

**Key accomplishments:**

- FilterEngine with glob patterns, validation, and statistics
- CLI integration with --include, --exclude, --kind, --crate, --visibility flags
- Result type extensions: visibility, deprecation, generics, attributes, function modifiers
- Unified rendering: single dispatcher for all 24 ItemKind variants
- Doc comment extraction with smart truncation at sentence boundaries
- Token budget enforcement at rendering layer
- Integration tests (43+) and snapshot tests (12) for real crates
- Comprehensive README with all v1.1 features documented
- Code quality: dead code removal, clippy fixes (160→113 warnings)

**Stats:**

- 91 files created/modified
- ~14,000 lines of Rust
- 5 phases, 20 plans
- 2 days from start to ship (2026-02-12 → 2026-02-14)
- Git range: `a81e236` → `88be87b`

**What's next:** v2.0 — Shared cache, stdlib queries, garbage collection

---

## v1.0 MVP (Shipped: 2026-02-13)

**Delivered:** Production-ready Cargo subcommand for fast, structured API queries over Rust dependency documentation.

**Phases completed:** 1-5 (19 plans total)

**Key accomplishments:**

- Sub-100ms query performance (verified at 7ms average)
- Automatic cache invalidation with BLAKE3 content hashing
- Comprehensive error handling with typed errors and exit codes
- Progress indicators and timing output for long operations
- Token budget constraints (--tokens) and minimal mode (--minimal)
- Recursive type expansion with cycle detection
- 187 unit tests with property-based testing

**Stats:**

- 5 phases, 19 plans, ~60+ tasks
- ~50 files created/modified
- ~7,172 lines of Rust code
- 2 days from start to ship (2026-02-12 → 2026-02-13)
- Git range: Initial commit → 41a403c

**What's next:** v1.1 enhancements including shared cache directory, stdlib queries, and garbage collection.

---

*For detailed phase information see `.planning/milestones/`*
