# Rendering

Determines what metadata and structural information appears in output for each item kind, and how documentation text is truncated under budget constraints.

## Purpose

Guarantees that rendered output faithfully represents item structure and documentation content within token-budget bounds. Output is deterministic: the same input always produces the same rendering order and truncation result.

## Non-goals

Boundaries of this concept — areas deliberately out of scope.

- Performance optimization of rendering — rendering correctness takes priority over speed.
- Fidelity of function signature arguments — argument lists are replaced with `[...]` placeholders; parameter names, types, and return types are not rendered for method lines.
- Completeness of generic bounds — one formatting path renders full trait paths and HRTB syntax; another produces the placeholder `"<trait>"`. The dual-behavior is an open design debt, not a correctness target.
- Version-aware ABI naming — the native Rust ABI is the default and does not render; foreign ABIs (C, stdcall, etc.) are named. Rendering `"unknown"` for the default ABI is a bug, not a design decision.
- Truncated output preserves semantic context — truncation is purely syntactic (sentence boundaries or character cutoffs), not semantic.

## Invariants

Conditions that always hold regardless of rendering path or budget state.

- **Fixed kind ordering** — module items render in a single canonical sequence: `struct, enum, trait, function, type, const, static, macro, re-export, module, other`. Every rendering path that groups items by kind must use this exact order.
- **Truncated prose always ends with `"..."`** — when doc text is truncated due to budget, the final character of the rendered output is `"..."`, regardless of whether code blocks are present. If a code block is preserved after truncation, the `"..."` suffix appears after the code block's closing fence.
- **Method first-line truncation at 77 characters** — when a method doc's first line exceeds 80 characters, it is truncated to exactly 77 characters and displayed on its own line before `"..."` on the next line.
- **Minimal detail level omits docs entirely** — no doc text renders in minimal mode, regardless of budget availability.
- **Function signatures render populated placeholders** — method lines include `"fn name([...]"` where the function name is present but arguments are elided.

## Constraints

Limitations imposed by the rendering environment and data model.

- Token budget operates on a rough heuristic: 1 token ≈ 4 characters. Truncation decisions are best-effort, not exact.
- Sentence-boundary truncation only recognizes `.`, `!`, `?` followed by whitespace or end-of-string as valid boundaries.
- When code blocks are present in doc text and budget is exceeded, the first code block is always preserved; prose before it is truncated. Prose after the code block is dropped.
- Synthetic generic parameters (compiler-generated `impl Trait`) are filtered before rendering.

## Rationale

A single kind ordering prevents user-facing inconsistency across rendering paths. The `"..."`-always-at-end rule gives readers a reliable truncation signal. Truncation at 77 characters accounts for indentation overhead.

## Related

Cross-references to connected concepts and source locations.

- [[generic-rendering-fidelity#Generic Rendering Fidelity]]
- [[token-budgeting#Token Budgeting]]
- [[src/format/doc.rs#DocHandler]]
- [[src/format/item.rs#ItemFormatter]]
- [[src/query/format.rs#TypeFormatter]]
- [[src/types/detail.rs#DetailLevel]]

