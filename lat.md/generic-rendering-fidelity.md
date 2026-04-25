---
lat:
  title: Generic Rendering Fidelity
---
# Generic Rendering Fidelity

Specification for which generic parameter details appear in rendered output and under what conditions.

## Purpose

Generic parameters render with fidelity commensurate to their kind.

Type parameters carry bounds but not default values. Const parameters carry both type annotations and default values. Trait bounds preserve structural detail including higher-ranked quantifiers and lifetime annotations.

## Non-goals

Scope boundaries that define what this spec does not cover.

- Faithful rendering of type parameter defaults — they appear as a placeholder, not the actual type.
- Rendering of synthetic compiler-generated parameters.
- Displaying generic argument lists beyond the parameter declarations themselves.

## Invariants

Conditions that must hold for all generic parameter rendering.

- Type parameters render with their name and bounds; default values never appear verbatim.
- Const parameters render with name, declared type, and default value when one exists.
- Lifetime parameters render with outlives relationships.
- Trait bounds in generic contexts preserve trait path, higher-ranked quantifiers, and lifetime information.
- Where predicates preserve type-bound structure and HRTB prefixes.
- Dynamic trait types include trait paths, HRTB annotations, and explicit lifetimes.

## Constraints

Limitations on what can be rendered verbatim in generic output.

- Type parameter defaults render as an abbreviation marker rather than the actual default type.
- Synthetic parameters are excluded from rendering regardless of kind.
- Where predicate equality constraints render with the left-hand side only; right-hand side is abbreviated.
- Use bounds render as a fixed abbreviation rather than their captured item list.

## Rationale

Fidelity decisions follow from bounded expansion concern and signal value.

Type parameter defaults may reference arbitrarily complex types requiring recursive resolution — abbreviating them bounds output size without losing the information that a default exists. Const parameter defaults are string literals with bounded complexity, so rendering them verbatim does not introduce unbounded expansion.

Trait bound detail is structural metadata that aids in understanding generic contracts; omitting it would remove more signal than noise.

## Related

Concepts and source locations referenced by this spec.

- [[type-expansion#Invariants#Detail Levels]] — DetailLevel enum and check-method semantics
- [[token-budgeting#Token Budgeting]] — budget enforcement during rendering
- [[src/types/detail.rs#format_generics]]
- [[src/types/detail.rs#format_generic_param]]
- [[src/types/detail.rs#format_generic_bound]]
- [[src/types/detail.rs#format_where_predicate]]
- [[src/types/detail.rs#format_type_simple]]
