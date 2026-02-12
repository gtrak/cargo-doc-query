# Sparse Context Protocol

## Purpose

Reduce delegation context sizes for local LLMs (8k-32k context) while maintaining full functionality for cloud LLMs (200k context). Enables GSD to work efficiently across different model capabilities.

## Core Principles

1. **Reference over Content**: Pass file paths, not file contents
2. **Frontmatter-First**: Machine-readable data inline, detailed content lazy-loaded
3. **Section-Selective**: Read only needed sections, not entire files
4. **Context Budgeting**: Calculate token budget, adapt based on model profile

## Delegation Packet Structure

### Executor Delegation

**Current (problematic):**
```xml
<context>
Plan:
{FULL_PLAN_CONTENT - 20k tokens}

Project state:
{FULL_STATE_CONTENT - 15k tokens}

Config:
{FULL_CONFIG_CONTENT - 2k tokens}
</context>
```

**New (sparse):**
```xml
<context_summary>
{PLAN_FRONTMATTER_YAML - 2k tokens}
</context_summary>

<context_references>
Plan: .planning/phases/03-authentication/03-01-PLAN.md
State: .planning/STATE.md (read sections: Current Position, Decisions Made)
Config: .planning/config.json (if exists)
</context_references>

<success_criteria>
- [ ] Read referenced files when needed
- [ ] Execute all tasks
- [ ] Create SUMMARY.md
- [ ] Update STATE.md
</success_criteria>
```

**Context reduction:**
- Plan: Full (20k) → Frontmatter only (2k) = **90% reduction**
- State: Full (15k) → Frontmatter + 2 sections (1.5k) = **90% reduction**
- Total: **~37k → ~3.5k tokens (90% reduction)**

### Planner Delegation

**Current (problematic):**
```xml
<planning_context>
Project State:
{FULL_STATE - 15k tokens}

Roadmap:
{FULL_ROADMAP - 20k tokens}

Requirements:
{FULL_REQUIREMENTS - 8k tokens}

Phase Context:
{FULL_CONTEXT - 10k tokens}

Research:
{FULL_RESEARCH - 12k tokens}
</planning_context>
```

**New (sparse):**
```xml
<planning_context_summary>
Phase: {phase_number}
Mode: {standard | gap_closure}

**State Frontmatter:**
{STATE_FRONTMATTER - 0.3k tokens}

**Current Position:**
{STATE_CURRENT_POSITION_SECTION - 0.8k tokens}

**Decisions Made:**
{STATE_DECISIONS_SECTION - 0.7k tokens}

**Pending Items:**
{STATE_PENDING_ITEMS_SECTION - 0.3k tokens}

**Phase Goal:**
{PHASE_ROADMAP_ENTRY - 0.5k tokens}

**Phase Requirements:**
{PHASE_REQUIREMENTS_ONLY - 0.5k tokens}

</planning_context_summary>

<context_references>
Full files available for reference when needed:
- State: @.planning/STATE.md
- Roadmap: @.planning/ROADMAP.md
- Requirements: @.planning/REQUIREMENTS.md
- Phase Context: {phase_dir}/{phase}-CONTEXT.md
- Research: {phase_dir}/{phase}-RESEARCH.md
</context_references>
```

**Context reduction:**
- State: Full (15k) → Frontmatter + 3 sections (1.8k) = **88% reduction**
- Roadmap: Full (20k) → Current phase only (0.5k) = **97% reduction**
- Total: **~55k → ~3k tokens (95% reduction)**

## Context Budget Profiles

| Profile | Capacity | Budget  | Strategy          | Delegation Size |
|---------|----------|---------|-------------------|-----------------|
| quality  | 200k     | 60k     | Full inline (legacy) | 50-60k |
| balanced | 100k     | 25k     | Sparse state, full plans | 8-12k |
| budget   | 32k      | 5k      | Frontmatter + references | 3-5k |
| tiny     | 8k       | 1k      | Minimal only       | 1-2k |

## Implementation Guidelines

### Reading Files Sparingly

**DO:**
```bash
# Frontmatter only (first 30 lines)
head -30 file.md | sed '/^---$/,/^---$/!d'

# Specific section (current position only)
sed -n '/### Current Position/,/^###/p' STATE.md | head -10

# Phase-specific content (single phase entry)
grep -A10 "Phase 03:" ROADMAP.md

# Frontmatter + decisions
head -30 STATE.md | sed '/^---$/,/^---$/!d'
sed -n '/### Decisions Made/,/^###/p' STATE.md | head -20
```

**DON'T:**
```bash
# Full file reads (expensive!)
cat file.md

# Regex when section extraction possible
grep ".*" file.md

# Loading multiple full files at once
cat file1.md file2.md file3.md
```

### Lazy Loading Protocol

**Executor workflow:**
1. Read plan frontmatter only (2k tokens)
2. Execute task 1
3. Before task 2: Read specific section from STATE if needed (0.5k tokens)
4. Continue executing tasks
5. Never re-read entire file for one value

**Planner workflow:**
1. Read phase goal from roadmap (0.5k tokens)
2. Read STATE frontmatter + position sections (1.5k tokens)
3. Create initial tasks
4. If decision needed: Read STATE decisions section (0.7k tokens)
5. Never STATE + ROADMAP + REQUIREMENTS fully (would be 43k tokens!)

### Example: Executor Delegation

**Command (execute-phase.md):**
```bash
# Read plan frontmatter only
PLAN_FRONTMATTER=$(head -30 "{plan_path}" | sed '/^---$/,/^---$/!d')

# Read STATE frontmatter + key sections only
STATE_FRONTMATTER=$(head -30 .planning/STATE.md | sed '/^---$/,/^---$/!d')
STATE_POSITION=$(sed -n '/### Current Position/,/^###/p' .planning/STATE.md | head -10)
STATE_DECISIONS=$(sed -n '/### Decisions Made/,/^###/p' .planning/STATE.md | head -20)

# Config frontmatter only (optional)
CONFIG_FRONTMATTER=$(head -20 .planning/config.json | grep -E '"model_profile"|"mode"' 2>/dev/null || echo "")
```

**Delegation prompt:**
```xml
<objective>
Execute plan {plan_number} of phase {phase_number}-{phase_name}

Commit each task atomically. Create SUMMARY.md. Update STATE.md.
</objective>

<context_summary>
{plan_frontmatter}
</context_summary>

<context_references>
Plan: {plan_path} (read full when executing tasks)
State: .planning/STATE.md
  - Read sections: "Current Position" (for phase tracking)
  - Read sections: "Decisions Made" (for context constraints)
Config: .planning/config.json (if exists)
</context_references>

<success_criteria>
- [ ] Read referenced files when needed
- [ ] Execute all tasks
- [ ] Create SUMMARY.md
- [ ] Update STATE.md sections
</success_criteria>
```

### Example: Planner Delegation

**Command (plan-phase.md):**
```bash
# Read STATE frontmatter + priority sections only
STATE_FRONTMATTER=$(head -30 .planning/STATE.md | sed '/^---$/,/^---$/!d')
STATE_POSITION=$(sed -n '/### Current Position/,/^###/p' .planning/STATE.md | head -10)
STATE_DECISIONS=$(sed -n '/### Decisions Made/,/^###/p' .planning/STATE.md | head -20)
STATE_PENDING=$(sed -n '/### Pending Todos/,/^###/p' .planning/STATE.md | head -10)

# Read current phase from roadmap only
PHASE_ROADMAP=$(grep -A10 "Phase ${PHASE}:" .planning/ROADMAP.md)

# Read phase-specific requirements if exist
if [ -f .planning/REQUIREMENTS.md ]; then
  PHASE_REQUIREMENTS=$(grep -A20 "[A-Z]*-${PHASE}" .planning/REQUIREMENTS.md 2>/dev/null || echo "")
fi

# Context frontmatter only
if [ -f "${PHASE_DIR}"/*-CONTEXT.md ]; then
  CONTEXT_FRONTMATTER=$(head -30 "${PHASE_DIR}"/*-CONTEXT.md | sed '/^---$/,/^---$/!d')
fi

# Research frontmatter only
if [ -f "${PHASE_DIR}"/*-RESEARCH.md ]; then
  RESEARCH_FRONTMATTER=$(head -30 "${PHASE_DIR}"/*-RESEARCH.md | sed '/^---$/,/^---$/!d')
fi
```

**Delegation prompt:**
```xml
<planning_context_summary>

**Phase:** {phase_number}
**Mode:** {standard | gap_closure}

**State Frontmatter:**
{STATE_FRONTMATTER}

**Current Position:**
{STATE_POSITION}

**Decisions Made:**
{STATE_DECISIONS}

**Pending Todos:**
{STATE_PENDING}

**Phase Goal:**
{PHASE_ROADMAP}

**Phase Requirements:**
{PHASE_REQUIREMENTS}

**Phase Context Frontmatter:**
{CONTEXT_FRONTMATTER}

**Research Frontmatter:**
{RESEARCH_FRONTMATTER}

</planning_context_summary>

<context_references>

**Full files available for reference when needed:**
- State: @.planning/STATE.md
- Roadmap: @.planning/ROADMAP.md
- Requirements: @.planning/REQUIREMENTS.md (if exists)
- Phase Context: {phase_dir}/{phase}-CONTEXT.md (if exists)
- Research: {phase_dir}/{phase}-RESEARCH.md (if exists)

**Reference reading guidelines:**
- Read full file when you need complete details
- Use section-specific reads when specific information needed
- Don't read all referenced files at once
</context_references>
```

## Verification

**Test script:** `scripts/calculate-context-budget.sh`

Usage:
```bash
./scripts/calculate-context-budget.sh --model-profile budget --component planner
# Output: "Estimated: 5.2k tokens (budget: 5k). ⚠ Slightly over budget."
```

Metrics:
- **Executor delegation:** < 8k (budget), < 12k (balanced)
- **Planner delegation:** < 6k (budget), < 10k (balanced)
- **Overall reduction:** > 80%

## Backwards Compatibility

**No breaking changes:**

If context mode is not configured, the system defaults to "quality" profile (full context). This maintains backward compatibility with existing projects.

**Opt-in via config:**
```json
{
  "model_profile": "budget",
  "context": {
    "mode": "manual"
  }
}
```

## Migration Guide

### From Full Context to Sparse

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

## Tips for Implementation

1. **Test incrementally:** Start with one phase, verify reduction
2. **Measure before/after:** Use budget calculator to verify improvements
3. **Lazy loading is key:** Read files only when accessing specific data
4. **Frontmatter is sufficient:** Contains most critical structured information
5. **Section extraction works:** Use `sed -n` for specific sections
6. **Reference patterns work:** Tell agent where to find data, don't inline

## Troubleshooting

### Issue: "Agent says 'not enough context'"

**Solutions:**
1. Verify sparse context protocol is being followed
2. Check agent is reading referenced files, not ignoring them
3. Increase budget slightly (or switch from budget to balanced profile)
4. Add more frontmatter to delegation (trade: size vs completeness)

### Issue: "Context still too large"

**Solutions:**
1. Check budget calculator for actual token counts
2. Reduce included sections (e.g., remove Pending from STATE reads)
3. Switch to more aggressive profile (budget → tiny)
4. Split work into more granular tasks

### Issue: "Missing information I need"

**Solutions:**
1. Agent should read referenced files when needed
2. Verify file references are correct and accessible
3. Add additional sections to priority reads if consistently needed
4. Consider that legacy full-context mode may be needed for complex cases

---

**Protocol Version:** 1.0
**Last Updated:** 2025-02-06
**Related Docs:**
- `references/model-context-profiles.md` - Profile definitions
- `scripts/calculate-context-budget.sh` - Budget calculator
- `workflows/execute-phase.md` - Executor workflow
- `workflows/execute-plan.md` - Plan execution workflow
