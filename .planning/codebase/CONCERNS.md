# Codebase Concerns

**Analysis Date:** 2026-02-12

## Tech Debt

**Empty Implementation:**
- Issue: The codebase is a skeleton with only a "Hello, world!" placeholder. The `plan.md` describes a complex 4-phase cargo subcommand tool, but no implementation exists.
- Files: `src/main.rs`
- Impact: Zero functionality delivered despite comprehensive planning. 4-6 weeks of work outlined in roadmap not started.
- Fix approach: Begin Phase 1 implementation as defined in `plan.md` sections 11.1-11.4.

**Missing Dependency Declarations:**
- Issue: `Cargo.toml` has empty `[dependencies]` section but the plan requires: clap (CLI), serde (JSON), bincode (caching), and rustdoc-json parsing.
- Files: `Cargo.toml`
- Impact: Cannot build any meaningful functionality. Build will fail once imports are added.
- Fix approach: Add required dependencies per Phase 1 requirements.

**No Module Structure:**
- Issue: Single-file `main.rs` with no modules. Plan describes complex architecture with CrateIndex, TypeInfo, caching, and CLI subcommands.
- Files: `src/main.rs`
- Impact: Architecture not reflected in code organization. No separation of concerns.
- Fix approach: Create module hierarchy matching planned architecture (cli/, cache/, index/, parser/).

## Known Bugs

**No Known Bugs:**
- The codebase has no implementation to contain bugs. All functionality is future work.

## Security Considerations

**Cache Directory Permissions:**
- Risk: The plan specifies caching in `target/doc-query/` but does not address directory permissions or cache poisoning.
- Files: `plan.md` section 7.3
- Current mitigation: None implemented
- Recommendations: Implement cache validation with checksums. Use appropriate file permissions (0o700) for cache directories containing compiled artifacts.

**Command Injection via rustdoc:**
- Risk: Plan involves running `cargo rustdoc -- -Z unstable-options`. User-provided type paths in queries could be注入攻击 if not properly escaped.
- Files: `plan.md` section 7.1
- Current mitigation: None implemented
- Recommendations: Strict input validation on all query parameters. Use std::process::Command with argument lists, not string interpolation.

## Performance Bottlenecks

**No Performance Baseline:**
- Problem: No code to measure, but plan sets aggressive targets (<100ms queries, <5s builds).
- Files: `plan.md` section 8
- Cause: Implementation not started
- Improvement path: Establish benchmarks in `benches/` directory early to validate architecture decisions.

**JSON Parsing Strategy Undefined:**
- Problem: Plan mentions parsing rustdoc JSON into internal graph model but no parsing library selected.
- Files: `plan.md` section 7.1
- Cause: Implementation gap
- Improvement path: Evaluate rustdoc-types crate vs custom parsing. rustdoc-types is the authoritative source for rustdoc JSON structure.

## Fragile Areas

**Rustdoc JSON Unstable:**
- Files: `plan.md` section 9
- Why fragile: The tool depends on nightly-only `-Z unstable-options --output-format json` flag. Rustdoc JSON format changes without notice.
- Safe modification: Pin specific nightly version. Add version detection and graceful degradation.
- Test coverage: No tests exist. Need integration tests against multiple rustc versions.

**Generic Type Expansion:**
- Files: `plan.md` sections 2.3, 7.2
- Why fragile: Recursive expansion of generic types can explode combinatorially. No depth limits implemented.
- Safe modification: Implement hard depth limits and cycle detection before any expansion logic.
- Test coverage: None. Need property-based tests for recursive type scenarios.

## Scaling Limits

**Memory Usage for Large Dependency Graphs:**
- Current capacity: N/A (no implementation)
- Limit: Plan acknowledges large dependency graphs increase memory but no mitigation designed.
- Scaling path: Implement streaming/sharded processing. Load crate indexes on-demand rather than full graph in memory.

**Cache Invalidation:**
- Current capacity: N/A
- Limit: Plan proposes content-hash based caching but no eviction strategy.
- Scaling path: Implement LRU eviction. Set maximum cache size. Provide `cargo doc-query clean` command.

## Dependencies at Risk

**Nightly Rust Required:**
- Risk: `-Z unstable-options` requires nightly compiler. If rustdoc JSON stabilizes with different format, tool breaks.
- Impact: Users must install nightly. Format changes break parsing.
- Migration plan: Track rustdoc JSON RFC stabilization. Maintain compatibility layer for format versions.

**No Dependency Pinning Strategy:**
- Risk: When dependencies are added, Cargo.lock should be committed for reproducible builds.
- Impact: Build non-determinism
- Migration plan: Add Cargo.lock to git once dependencies declared.

## Missing Critical Features

**Error Handling Framework:**
- Problem: No error types defined. Complex tool needs structured error handling for: IO failures, JSON parse errors, rustdoc failures, cache corruption.
- Blocks: Robust CLI operation

**Test Infrastructure:**
- Problem: No tests directory, no test dependencies, no CI configuration.
- Blocks: Confidence in changes. Regression prevention.

**CI/CD Pipeline:**
- Problem: No `.github/workflows/`, no automated testing, no release automation.
- Blocks: Reliable deployment. Contributor confidence.

**Documentation Generation:**
- Problem: No `cargo doc` setup. Tool intended to query docs but its own docs don't exist.
- Blocks: User adoption. Contributor onboarding.

## Test Coverage Gaps

**No Tests Exist:**
- What's not tested: Everything. Zero test coverage.
- Files: All files in `src/`
- Risk: No way to verify functionality. No safety net for refactoring.
- Priority: Critical

**Integration Test Requirements:**
- Untested area: rustdoc JSON parsing against real crate outputs
- Priority: High - Core functionality depends on external tool output

**Performance Regression Tests:**
- Untested area: Query timing, cache hit rates, memory usage
- Priority: Medium - Required to meet <100ms target

---

*Concerns audit: 2026-02-12*
