# Model Context Profiles

## Overview

Different models have vastly different context capacities. GSD adapts its delegation strategy based on the model profile to ensure efficient operation across cloud and local LLMs.

## Profiles

| Profile | Context | Budget | Strategy        | Use Case                     |
|---------|---------|--------|-----------------|------------------------------|
| quality  | 200k    | 60k    | Full context    | Claude Opus, cloud LLMs      |
| balanced | 100k    | 25k    | Sparse state    | Claude Sonnet, mid-tier cloud |
| budget   | 32k     | 5k     | Frontmatter ref | Llama 2 70B, local high-end  |
| tiny     | 8k      | 1k     | Minimal only    | Llama 2 7B, local low-end    |

---

## Profile: quality

**Context Capacity:** 200k tokens
**Budget:** 60k tokens (30%)
**Strategy:** Full context inline (legacy behavior)

### Characteristics
- Full content inlined in delegation prompts
- No need for sparse context protocol
- Maximum flexibility for complex phases
- Best for: Claude Opus, GPT-4, other premium models

### Executor Delegation

| Component | Size      | Notes                           |
|-----------|-----------|---------------------------------|
| Plan      | ~20k      | Full PLAN.md content            |
| State     | ~15k      | Full STATE.md content           |
| Config    | ~2k       | Full config.json if exists      |
| **Total** | **~37k**  | Within budget                   |

### Planner Delegation

| Component     | Size      | Notes                           |
|---------------|-----------|---------------------------------|
| State         | ~15k      | Full STATE.md                   |
| Roadmap       | ~20k      | Full ROADMAP.md                 |
| Requirements  | ~8k       | Full REQUIREMENTS.md if exists  |
| Context       | ~10k      | Full phase CONTEXT.md           |
| Research      | ~12k      | Full phase RESEARCH.md          |
| **Total**     | **~55k**  | Within budget                   |

### Plan Creation Rules
- **Max task size:** No specific limit (trust planner judgment)
- **Max tasks per plan:** 3-6 (standard/comprehensive depth)
- **Depth defaults to:** User's choice
- **TDD enabled:** Yes (full context available)

### Reference Loading
- **Executor:** Read full plan file upfront
- **Planner:** Read all referenced files upfront
- **No lazy loading needed**

---

## Profile: balanced

**Context Capacity:** 100k tokens
**Budget:** 25k tokens (25%)
**Strategy:** Sparse state, full plans

### Characteristics
- Frontmatter-only for large files (STATE, ROADMAP)
- Full content for plan-specific files
- Good balance for most use cases
- Best for: Claude Sonnet, GPT-4 Turbo

### Executor Delegation

| Component | Size      | Notes                           |
|-----------|-----------|---------------------------------|
| Plan      | ~10k      | Full plan (not just frontmatter)|
| State     | ~5k       | Frontmatter + 3 sections        |
| Config    | ~1k       | Frontmatter only                |
| **Total** | **~16k**  | Within budget                   |

### Planner Delegation

| Component     | Size      | Notes                           |
|---------------|-----------|---------------------------------|
| State         | ~4k       | Frontmatter + 3 sections        |
| Roadmap       | ~3k       | Phase + adjacent phases         |
| Requirements  | ~2k       | Phase-specific only             |
| Context       | ~1.5k     | Frontmatter only                |
| Research      | ~1.5k     | Frontmatter only                |
| **Total**     | **~10k**  | Within budget                   |

### Plan Creation Rules
- **Max task size:** 15% of context (~3.75k tokens)
- **Max tasks per plan:** 3 (standard), 4 (comprehensive)
- **Depth defaults to:** Standard
- **TDD enabled:** Yes

### Reference Loading
- **Executor:** Read full plan, lazy-load STATE sections
- **Planner:** Read phase from ROADMAP, lazy-load other files
---

## Profile: budget

**Context Capacity:** 32k tokens
**Budget:** 5k tokens (15%)
**Strategy:** Frontmatter + references only

### Characteristics
- Aggressive sparsification
- Frontmatter-only for all files
- References to full files for when needed
- Best for: Llama 2 70B, Claude Haiku, other 32k context models

### Executor Delegation

| Component | Size      | Notes                           |
|-----------|-----------|---------------------------------|
| Plan      | ~2k       | Frontmatter only                |
| State     | ~1.5k     | Frontmatter + 2 sections        |
| Config    | ~0.5k     | Frontmatter only                |
| **Total** | **~4k**   | Within budget                   |

**Context reduction:** 90% from quality profile

### Planner Delegation

| Component     | Size      | Notes                           |
|---------------|-----------|---------------------------------|
| State         | ~1.8k     | Frontmatter + 3 sections        |
| Roadmap       | ~0.5k     | Single phase only               |
| Requirements  | ~0.2k     | Phase-specific filtering        |
| Context       | ~0.2k     | Frontmatter only                |
| Research      | ~0.3k     | Frontmatter only                |
| **Total**     | **~2.8k** | Within budget                   |

**Context reduction:** 95% from quality profile

### Plan Creation Rules
- **Max task size:** 8% of context (~2.56k tokens)
- **Max tasks per plan:** 2 (budget), 3 (balanced)
- **Depth defaults to:** Quick (aggressive combination)
- **TDD disabled:** Yes (too expensive, 3× overhead)

### Reference Loading Protocol

**Executor:**
- Read plan file: Full (required for execution)
- Read STATE: Lazy - only when accessing decisions/position
- Read config: Never (use runtime defaults)

**Planner:**
- Read STATE: Frontmatter + specific sections only
- Read ROADMAP: Single phase only
- Read CONTEXT: Frontmatter only
- Read RESEARCH: Frontmatter only

**Never:**
- Read multi-file sequences without breaks
- Read entire STATE.md upfront
- Combine multiple full files in single context

### Lazy Loading Example

```xml
<!-- Task 1: Create model -->
<task type="auto">
  <name>Create User model</name>
</task>

<!-- Executor: -->
<!-- No STATE read needed for this task -->

<!-- Task 2: Create API using model -->
<task type="auto">
  <name>Create user API endpoints</name>
</task>

<!-- Executor with lazy loading: -->
<!-- Read: src/models/user.ts only (150 tokens) -->
<!-- Import User type in new file -->
<!-- STATE.md not read -->
```

---

## Profile: tiny

**Context Capacity:** 8k tokens
**Budget:** 1k tokens (8%)
**Strategy:** Minimal only

### Characteristics
- Extreme constraints
- Minimal context only
- Single-task plans required
- Best for: Llama 2 7B, Mistral 7B, other 8k context models
- **Warning:** Very limited functionality, use budget profile instead when possible

### Executor Delegation

| Component | Strategy | Size      |
|-----------|----------|-----------|
| Plan      | Task actions only | ~2k      |
| State     | None (generate summary) | 0     |
| Config    | None (hardcoded defaults) | 0     |
| **Total** | | **~2k** |

**Context reduction:** 95% from quality profile (exceeds budget but this profile is extreme)

### Planner Delegation

| Component | Strategy | Size      |
|-----------|----------|-----------|
| STATE     | Current phase entry only | ~0.3k    |
| ROADMAP   | Goal line only | ~0.1k               |
| CONTEXT   | Goal statement only | ~0.1k          |
| **Total** | | **~0.5k** |

**Context reduction:** 99% from quality profile

### Extreme Constraints

- **Max task size:** 5% of context (~0.4k tokens)
- **Max tasks per plan:** 1 (always auto-split if >1)
- **Depth forces:** Quick (no option to override)
- **TDD disabled:** Never use (3× overhead is fatal)
- **Reference loading:** Single-file reads only, no sections
- **No multi-task plans:** Every task is its own plan

### Ultra-Reference Protocol

**Executor:**
Before starting:
1. Read task title (10 chars)
2. Read task action (500 chars max)
3. Execute immediately

After each task:
1. Write minimal summary to continue-here.md
2. Next task reads summary, not all file history

**Planner:**
1. Read phase goal (single line)
2. Create 1-task plan
3. Stop

---

## Profile Detection

### Auto-Detection (default)

If `context.mode=auto` in config.json:

```bash
DETECTED_MODEL=$(claude --model-info 2>/dev/null || echo "unknown")

case "$DETECTED_MODEL" in
  *"Opus"*|*"200k"*|*"claude-3-opus"*)
    MODEL_PROFILE="quality"
    ;;
  *"Sonnet"*|*"100k"*|*"claude-3-sonnet"*)
    MODEL_PROFILE="balanced"
    ;;
  *"Haiku"*|*"Llama"*|*"70B"*|*"32k"*)
    MODEL_PROFILE="budget"
    ;;
  *"7B"*|*"8B"*|*"8k"*|*"mistral-7b"*)
    MODEL_PROFILE="tiny"
    ;;
  *)
    MODEL_PROFILE="balanced"  # Default fallback
    ;;
esac
```

### Manual Override

Set in `.planning/config.json`:
```json
{
  "model_profile": "budget",
  "context": {
    "mode": "manual"
  }
}
```

---

## Context Budget Enforcement

### Calculating Budget

```bash
CAPACITY={context_capacity}
TARGET_PERCENT={profile_target_percent}
BUDGET=$(( CAPACITY * TARGET_PERCENT / 100 ))
```

### Validation

Before delegation:

```bash
ESTIMATED=$(calculate_context_size_estimate)
if [ "$ESTIMATED" -gt "$BUDGET" ]; then
  echo "Context exceeds budget: $ESTIMATED > $BUDGET"
  echo "Reduce by sparsifying or splitting tasks"
  exit 1
fi
```

### Budget Allocation

| Profile | Executor | Planner | Checker | Total Budget |
|---------|----------|---------|---------|--------------|
| quality  | 60k      | 60k     | 40k     | 60k (30%)    |
| balanced | 25k      | 25k     | 15k     | 25k (25%)    |
| budget   | 5k       | 5k      | 2k      | 5k (15%)     |
| tiny     | 2k       | 2k      | 0.5k    | 1k (8%)      |

---

## Migration Guide

### From quality (full context) to budget

**What changes:**
1. Executor receives plan frontmatter instead of full plan
2. Planner receives key sections instead of full files
3. Behavior: Must lazy-load files when needed

**What stays the same:**
1. Task logic (still do same work)
2. PLAN.md structure (unchanged)
3. git commit patterns (unchanged)

**Testing:**
```bash
# Set budget mode
cat > .planning/config.json <<EOF
{
  "model_profile": "budget",
  "context": {
    "mode": "manual"
  }
}
EOF

# Validate context sizes
./scripts/calculate-context-budget.sh budget all

# Run plan-phase
/gsd-plan-phase 03
```

### From budget to tiny

**Additional constraints:**
1. Only 1 task per plan
2. No TDD cycles
3. Ultra-minimal delegation

**Warning:** Tiny profile is extremely restrictive. Use budget profile unless model truly has 8k context.

---

## Tips for Local LLM Users

1. **Use budget profile** for 32k models (most realistic workable option)
2. **Validate context** before delegation: use calculate-context-budget.sh
3. **Split aggressively**: If uncertain, split (better to have 2 small tasks than 1 large one)
4. **Disable TDD** for budget/tiny profiles (save 3× overhead)
5. **Use depth=quick** for faster phase planning
6. **Parallel execution** works great with sparse contexts (executor stays small)

---

## Troubleshooting

### Issue: "Context exceeds budget"

**Solutions:**
1. Check model_profile setting (maybe set too aggressive)
2. Run validate-task-sizes.sh and split oversize tasks
3. Reduce depth setting from comprehensive to quick
4. Enable lazy loading (check executor is reading files on-demand)

### Issue: "Agent returns 'I don't have enough context'"

**Solutions:**
1. Verify sparse context protocol is being followed
2. Check agent is reading referenced files, not ignoring them
3. Increase budget slightly (or switch to balanced profile)
4. Add more frontmatter to delegation (trade: size vs completeness)

### Issue: "Tiny profile too restrictive"

**Solution:**
Switch to budget profile (32k context instead of 8k). Tiny is only for genuine 8k context models.

---

**Document Version:** 1.0
**Last Updated:** 2025-02-06
**Related Docs:**
- `protocols/sparse-context.md` - Sparse context implementation
- `scripts/calculate-context-budget.sh` - Budget calculator
- `templates/planner-subagent-prompt.md` - Planner delegation template
