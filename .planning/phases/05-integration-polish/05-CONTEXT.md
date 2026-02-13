# Phase 05: Integration & Polish - Context

**Gathered:** 2026-02-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Production-ready CLI polish including comprehensive error handling, progress indicators for long operations, complete help text/documentation, graceful edge case handling, and appropriate exit codes for shell scripting.

</domain>

<decisions>
## Implementation Decisions

### Error message style
- Use colored output by default (red for errors, yellow for warnings, green for success)
- Include `--no-color` flag to disable colors for piping/CI environments
- Error format: `[ERROR] <message>` with optional context/suggestions
- Warnings go to stderr, never stdout (preserve clean JSON output)
- Include actionable suggestions where possible (e.g., "Run `cargo doc-query build` first")

### Progress indicators
- Show spinner during rustdoc JSON generation (takes 2-5s for typical projects)
- Display stage labels: "Generating rustdoc JSON...", "Building index...", "Caching..."
- Silent mode (`--quiet` or `-q`) suppresses all progress output
- Progress goes to stderr, never stdout
- Include timing info on completion (e.g., "Build completed in 3.2s")

### Help text approach
- Comprehensive `--help` with examples for each command
- Include common usage patterns in help text
- Short descriptions for all flags
- Man page generation not required for v1.0 (can add later)

### Edge case handling
- Missing cache: Auto-trigger build with clear message, don't fail
- Missing type in query: Clear "type not found" error with suggestions for similar types
- Corrupt cache: Detect and auto-rebuild with warning
- Network issues (if any external calls): Retry once, then fail with helpful message
- Invalid Cargo.toml: Pass through cargo's error with context
- No dependencies in project: Friendly message explaining the tool needs dependencies

### Exit codes
- `0` — Success
- `1` — General error (build failed, query error, etc.)
- `2` — Cache miss (auto-rebuild attempted but failed)
- `3` — Invalid arguments or usage error
- `130` — User interrupted (Ctrl+C)

### Claude's Discretion
- Exact color scheme and ANSI codes
- Spinner animation style
- Specific wording of error messages
- Exact timing format (seconds vs milliseconds)
- Internal error handling patterns
- Specific exit code assignment for edge cases

</decisions>

<specifics>
## Specific Ideas

- Follow Rust CLI conventions (similar to `cargo`, `rustc`, `ripgrep`)
- Error messages should feel like cargo's error style (colored, helpful, actionable)
- Progress indicators should be subtle but informative
- All output should be pipe-friendly (JSON to stdout, everything else to stderr)

</specifics>

<deferred>
## Deferred Ideas

- Man page generation — can be added post-v1.0
- Shell completions (bash/zsh/fish) — nice to have for v1.1
- Configuration file support — future enhancement
- Verbose logging levels (`-v`, `-vv`, `-vvv`) — add if requested

</deferred>

---

*Phase: 05-integration-polish*
*Context gathered: 2026-02-12*
