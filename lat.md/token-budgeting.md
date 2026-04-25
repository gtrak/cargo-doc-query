---
lat:
  title: Token Budgeting
---
# Token Budgeting

Specification of token budget tracking and estimation for rendering-layer truncation decisions.

## Purpose

Enforce a configurable upper bound on output size during formatting. The system estimates each item's approximate cost, tracks cumulative usage, and decides inclusion or truncation based on the cap.

## Non-goals

What this specification does not cover. Details below are scoped to budget tracking only.

- Not a precise token counter — estimation is approximate by design; exact counts require running output through an LLM tokenizer.
- Not extensible for custom weights — estimation categories and per-category costs are fixed at compile time.
- Not incremental across sessions — each formatting pass begins with zero cumulative usage.

## Invariants

Conditions that hold throughout the lifetime of a budgeting session. These properties are implementation-independent.

### Budget Enforcement

How items are evaluated against the configured budget cap.

- An item is included if and only if its estimated cost plus current cumulative usage does not strictly exceed the budget cap.
- An item that would exceed the budget contributes zero to the cumulative total — partial inclusion never occurs.
- A budget of zero means every item would exceed the budget and therefore all items are truncated.

### Unbounded Case

Behavior when no budget is configured, including query return values.

- When no budget is configured, every item is included regardless of cost. Remaining capacity queries report `None` rather than an unbounded numeric value.
- No warning indicators activate in the unbounded case — proximity to a cap is meaningless when no cap exists.

### Truncation Decision Values

The two permissible outcomes for a per-item inclusion decision.

- A truncation decision yields one of two outcomes: **Include** (the item contributes to output) or **Truncate** (the item is omitted). These are the only permissible values.

### Budget State Queries

Return types and behavior of queries about current budget state.

- Remaining capacity returns an integer when a budget is configured, or `None` when unbounded. The integer saturates at zero rather than becoming negative.
- A proximity indicator yields a boolean — true when cumulative usage has crossed the warning threshold relative to the configured budget, false otherwise. In the unbounded case it always yields false.

### Estimation Categories and Weights

Enumerated categories with their per-unit costs.

- Every item receives a base cost of 20 tokens, representing path overhead.
- Each field adds 5 tokens.
- Each variant adds 5 tokens.
- Each nested module item adds 10 tokens.
- Documentation text contributes one token per four characters of content, rounded down.
- A present signature adds 10 tokens.
- A present generics annotation adds 5 tokens.
- A present visibility marker adds 3 tokens.
- Each attribute adds 3 tokens.
- A present modifiers field adds 5 tokens.

### Truncation Boundary

How per-item evaluation interacts with budget exhaustion.

- The budget cap is evaluated per-item, not in bulk. An item that would exceed the budget is rejected even if subsequent items might fit within the remaining capacity after rejection.

## Constraints

Fixed parameters and system limitations.

- The warning threshold is fixed at eighty percent of the configured budget — it cannot be reconfigured at runtime or through input parameters.
- Estimation weights are compiled in — no per-item custom weighting, no category overrides.
- Documentation token estimation uses integer arithmetic — fractional tokens do not exist as an output type.

## Rationale

Why the design choices above were made, independent of implementation strategy.

- **Approximate estimation over exact counting**: running output through a real tokenizer before deciding inclusion would require serializing the item, measuring its serialized length, then potentially discarding it — a wasted serialization pass. Approximation avoids that round-trip cost.
- **Two-value truncation outcome**: allowing partial inclusion or graded responses would require splitting items mid-stream and introducing boundary artifacts. A binary include-or-exclude decision keeps output coherent.
- **Fixed warning threshold**: the primary audience is humans reading CLI output; a single threshold at eighty percent gives timely notice without introducing configuration burden that most users will never adjust.
- **Saturated remaining capacity**: reporting negative remaining values would be misleading — there is no such thing as "negative budget." Saturating at zero communicates exhaustion without implying invalid state.
- **Per-item boundary evaluation with rejection persistence**: once an item is rejected, it does not consume budget. Subsequent items can still fit within the original cap. Allowing partial inclusion would require mid-item truncation boundaries that degrade readability.

## Related

Concepts and integration points connected to this specification.

- [[type-expansion#Invariants#Token Budgeting]] — integration point for budget enforcement during expansion traversal
- [[rendering#Rendering]] — FormattedItem structure and rendering pipeline
