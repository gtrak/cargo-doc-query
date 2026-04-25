# Error Handling with Exit Codes

Ensures every failure produces a human-readable message and deterministic exit code. A single boundary between internal operations and the terminal catches unstructured errors exactly once.

## Purpose

Defines the scope of error handling responsibilities at the dispatch boundary.

- Define a closed set of observable error categories; any unanticipated failure is caught by a catch-all and presented as a generic error.
- Provide a uniform exit signaling mechanism: every error category resolves to the same non-zero exit code.
- Coerce foreign errors into the local error set without loss of surface-level context.
- Catch errors exactly once at the dispatch boundary, format them for display, and terminate with the resolved exit code.
- Validate required configuration before any handler runs, preventing partial execution.

## Non-goals

Categorizes areas deliberately out of scope for error handling at the dispatch boundary.

- Distinguishing error categories by exit code — the system treats all failures as equivalent from the outside.
- Preserving full error chains through category boundaries — only the payload required for display survives coercion.
- Preventing runtime misconfiguration entirely — invalid flag combinations degrade with warnings rather than hard failure.
- Guaranteeing suggestion availability — supplementary lookup is best-effort and silently drops when unavailable.

## Invariants

Conditions that must hold for every error path through the application.

- **Error exhaustiveness** — The application defines a closed set of observable error categories; any unanticipated failure is caught by a catch-all.
- **Uniform exit signaling** — Every error category resolves to the same non-zero exit code; the boundary does not encode severity or classification.
- **Foreign error coercion** — Errors originating outside the application domain are absorbed into the local error set without loss of surface-level context.
- **Single top-level boundary** — The dispatch layer catches errors exactly once, formats them for display, and terminates with the resolved exit code.
- **Validation before execution** — Missing required configuration prevents any handler from running; the check occurs at the boundary, not inside handlers.

## Constraints

Fixed parameters and operational limitations of the error handling boundary.

- **Path mandatory for query operations** — A query cannot execute without a target path; absence is rejected before dispatch.
- **Depth floor** — Depth zero is promoted to depth one; the system does not support the documented "methods and traits only" mode.
- **Flag precedence is deterministic** — When conflicting output-level flags are present, minimal always wins and a warning is emitted if not silenced.
- **Help metadata bypasses validation** — Requesting filter help returns immediately without enforcing path or configuration requirements.
- **Suggestion lookup is non-fatal** — Failure to resolve suggestions never escalates; the original error propagates unchanged.

## Rationale

Design justifications for each invariant and constraint.

- A single exit code for all errors reduces the cognitive burden on scripts and users who only need to distinguish success from failure, and avoids committing to a differentiation scheme that would require maintenance as categories evolve.
- Error coercion at boundaries prevents type leakage into handlers and keeps the control layer focused on routing rather than error translation.
- Validating required configuration at the boundary, before dispatch, avoids partial execution and ensures consistent failure messages.
- Promoting depth zero to one eliminates a degenerate case where no meaningful output is produced, while avoiding silent confusion about whether a query succeeded.
- Best-effort suggestions improve UX without introducing additional failure paths into the primary error flow.

## Related

Concepts and source locations connected to error handling.

- [[type-suggestion#Type Suggestion Engine]] — provides best-effort suggestions on path-not-found errors
- [[src/error/errors.rs]]
- [[src/main.rs]]
