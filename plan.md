# cargo doc-query

## Product Requirements Document (PRD)

Version: 0.1\
Author: Internal\
Status: Draft

------------------------------------------------------------------------

# 1. Overview

`cargo doc-query` is a Cargo subcommand that provides fast, structured,
low-context queries over Rust dependency APIs using `rustdoc` JSON
output.

It enables LLM agents and CLI users to:

-   Query available methods on types
-   Inspect return types with full qualification
-   Traverse type graphs
-   Reduce context usage compared to raw source or LSP
-   Rebuild documentation artifacts incrementally when dependencies
    change

The tool is designed to: - Avoid long-running daemons - Operate as a
stateless CLI - Maintain an on-disk cache - Rebuild selectively when
needed

------------------------------------------------------------------------

# 2. Problem Statement

LLM agents struggle to: - Use unfamiliar third-party libraries -
Reliably discover method sets and trait implementations - Operate within
limited context windows

LSP (e.g., rust-analyzer): - Is slow to reindex - Requires a daemon -
Produces high-overhead responses

We need: - Deterministic structured API extraction - Fast incremental
refresh - CLI-only interface - Machine-readable output optimized for
agents

------------------------------------------------------------------------

# 3. Goals

1.  Provide sub-100ms queries once indexed.
2.  Support dependency-wide exploration.
3.  Provide fully qualified type paths.
4.  Allow traversal of output types recursively.
5.  Rebuild automatically when Cargo.lock changes.
6.  Be safe for repeated invocation by LLM agents.

------------------------------------------------------------------------

# 4. Non-Goals

-   Not a type checker (cargo check handles errors).
-   Not an IDE replacement.
-   Not a runtime code executor.
-   Not a general semantic search engine.

------------------------------------------------------------------------

# 5. User Personas

### 5.1 LLM Agent

Needs structured API info with minimal tokens.

### 5.2 CLI Developer

Wants quick lookup of method sets.

------------------------------------------------------------------------

# 6. User Stories by Phase

------------------------------------------------------------------------

# Phase 1 --- Minimal Viable Index

### Story 1.1

As a user, I can generate rustdoc JSON for all dependencies.

### Story 1.2

As a user, I can query methods for a type.

### Story 1.3

As an LLM agent, I can receive JSON output describing: - Methods -
Signatures - Fully qualified types

Deliverable: - `cargo doc-query build` -
`cargo doc-query methods <TypePath>`

------------------------------------------------------------------------

# Phase 2 --- Recursive Type Expansion

### Story 2.1

As an LLM agent, I can expand the return type graph.

### Story 2.2

I can see trait implementations for a type.

### Story 2.3

I can resolve associated types.

Deliverable: - `cargo doc-query expand <TypePath> --depth N`

------------------------------------------------------------------------

# Phase 3 --- Incremental Rebuild & Caching

### Story 3.1

If Cargo.lock changes, the index rebuilds automatically.

### Story 3.2

If only one crate changes, only that crate rebuilds.

### Story 3.3

Query time remains \<100ms.

Deliverable: - Content-hash based cache directory: `.cargo/doc-query/`

------------------------------------------------------------------------

# Phase 4 --- LLM Agent Optimization Layer

### Story 4.1

LLM can request minimal output mode.

### Story 4.2

LLM can request verbose type hints.

### Story 4.3

LLM can request token-budget constrained output.

Deliverable: - `--format json|minimal|verbose` - `--token-budget N`

------------------------------------------------------------------------

# 7. Technical Architecture

## 7.1 Indexing Strategy

1.  Run:

        cargo rustdoc -- -Z unstable-options --output-format json

2.  Collect JSON per crate.

3.  Parse into internal graph model.

4.  Store as:

    -   Serialized bincode graph
    -   Separate crate shards

------------------------------------------------------------------------

## 7.2 Internal Data Model

Core structures:

``` rust
struct CrateIndex {
    crate_name: String,
    items: HashMap<ItemId, Item>,
}

struct TypeInfo {
    path: String,
    methods: Vec<MethodInfo>,
    traits: Vec<TraitInfo>,
}

struct MethodInfo {
    name: String,
    signature: String,
    fully_qualified_signature: String,
}
```

------------------------------------------------------------------------

## 7.3 Cache Layout

    target/doc-query/
        metadata.json
        crates/
            serde.bin
            tokio.bin

Metadata includes: - Cargo.lock hash - rustc version - target triple

------------------------------------------------------------------------

## 7.4 Rebuild Logic

Algorithm:

1.  Compute hash of:
    -   Cargo.lock
    -   rustc --version
2.  Compare to metadata.json
3.  If mismatch:
    -   Rebuild affected crates
4.  Otherwise:
    -   Load cached graph

------------------------------------------------------------------------

## 7.5 CLI Interface

### Build

    cargo doc-query build

### Query methods

    cargo doc-query methods std::fs::File

### Expand return type graph

    cargo doc-query expand std::fs::File --depth 2

### Traits

    cargo doc-query traits tokio::net::TcpStream

------------------------------------------------------------------------

# 8. Performance Requirements

  Operation             Target
  --------------------- ---------
  Build small project   \<5s
  Build large project   \<30s
  Query                 \<100ms
  Expand depth=2        \<150ms

------------------------------------------------------------------------

# 9. Risks

1.  rustdoc JSON instability (nightly flag).
2.  Large dependency graphs increase memory.
3.  Generic-heavy crates increase expansion size.

Mitigation: - Pin rustdoc version. - Sharded crate storage. - Depth
limits.

------------------------------------------------------------------------

# 10. Future Extensions

-   Semantic embedding index.
-   Cross-crate trait resolution acceleration.
-   WASM build mode.
-   Prebuilt index registry.

------------------------------------------------------------------------

# 11. Implementation Roadmap

## Phase 1 (1--2 weeks)

-   CLI scaffolding
-   rustdoc JSON parsing
-   Basic type + method index
-   JSON output

## Phase 2 (1--2 weeks)

-   Recursive expansion
-   Trait resolution
-   Associated types

## Phase 3 (1 week)

-   Hash-based rebuild detection
-   Sharded cache

## Phase 4 (1 week)

-   Token-budget formatting
-   LLM-optimized minimal mode

------------------------------------------------------------------------

# 12. Acceptance Criteria

-   Works without daemon.
-   Automatically rebuilds on dependency change.
-   Queries complete in \<100ms after build.
-   Output machine-readable and deterministic.
-   Reduces LLM context usage compared to raw source.

------------------------------------------------------------------------

# End of Document
