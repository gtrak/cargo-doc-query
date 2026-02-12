#!/bin/bash

# Context Budget Calculator
# Estimates token counts for delegation contexts

set -e

# Parse arguments
MODEL_PROFILE="${1:-balanced}"
COMPONENT="${2:-all}"
PHASE_DIR="${3:-.planning/phases}"

# Token estimates (rough: 1 token ≈ 4 chars)
CHARS_PER_TOKEN=4

# Context budgets
declare -A CAPACITIES
CAPACITIES[quality]=200000
CAPACITIES[balanced]=100000
CAPACITIES[budget]=32000
CAPACITIES[tiny]=8000

declare -A TARGET_PERCENT
TARGET_PERCENT[quality]=30
TARGET_PERCENT[balanced]=25
TARGET_PERCENT[budget]=15
TARGET_PERCENT[tiny]=8

# Get capacity
CAPACITY=${CAPACITIES[$MODEL_PROFILE]:-100000}
TARGET=$(echo "$CAPACITY * ${TARGET_PERCENT[$MODEL_PROFILE]:-25} / 100" | bc 2>/dev/null || echo "25000")

echo "=== Context Budget Calculator ==="
echo "Model Profile: $MODEL_PROFILE"
echo "Context Capacity: $CAPACITY tokens"
echo "Target Budget: $TARGET tokens (${TARGET_PERCENT[$MODEL_PROFILE]}%)"
echo ""

# Check if we're in a planning directory
if [ ! -f .planning/STATE.md ]; then
  echo "⚠ Not in a GSD project directory (no .planning/STATE.md)"
  echo "Run this from your project root"
  echo ""
fi

# File size estimator
estimate_tokens() {
  local file="$1"
  if [ -f "$file" ]; then
    local chars=$(wc -c < "$file" 2>/dev/null | tr -d ' ')
    local tokens=$(echo "$chars / $CHARS_PER_TOKEN" | bc 2>/dev/null || echo "0")
    echo "$tokens"
  else
    echo "0"
  fi
}

# Calculate executor delegation (sparse mode)
calculate_executor() {
  local phase_dir="$1"

  echo "=== Executor Delegation (Sparse Mode) ==="

  # Find PLAN.md
  local plan=$(find "$phase_dir"/*-PLAN.md 2>/dev/null | head -1)

  if [ -z "$plan" ]; then
    echo "⚠ No PLAN.md found in $phase_dir"
    echo "Example: calculate-context-budget.sh budget executor .planning/phases/03-authentication"
    echo ""
    return 1
  fi

  # Estimate frontmatter only (first 30 lines)
  local plan_front=$(head -30 "$plan" 2>/dev/null | wc -c | tr -d ' ')
  local plan_front_tokens=$(echo "$plan_front / $CHARS_PER_TOKEN" | bc 2>/dev/null || echo "0")

  # STATE frontmatter + 2 sections
  if [ -f .planning/STATE.md ]; then
    local state_front=$(head -30 .planning/STATE.md 2>/dev/null | wc -c | tr -d ' ')
    local state_sec1=$(sed -n '/### Current Position/,/^###/p' .planning/STATE.md | head -10 | wc -c | tr -d ' ' 2>/dev/null)
    local state_sec2=$(sed -n '/### Decisions Made/,/^###/p' .planning/STATE.md | head -20 | wc -c | tr -d ' ' 2>/dev/null)
    local state_tokens=$(echo "( $state_front + $state_sec1 + $state_sec2 ) / $CHARS_PER_TOKEN" | bc 2>/dev/null || echo "0")
  else
    local state_tokens=0
  fi

  # Config frontmatter
  if [ -f .planning/config.json ]; then
    local config=$(head -20 .planning/config.json | grep -E '"model_profile"|"mode"' | wc -c | tr -d ' ' 2>/dev/null)
    local config_tokens=$(echo "$config / $CHARS_PER_TOKEN" | bc 2>/dev/null || echo "0")
  else
    local config_tokens=0
  fi

  local total=$(echo "$plan_front_tokens + $state_tokens + $config_tokens" | bc 2>/dev/null || echo "$plan_front_tokens")
  local percent=$(echo "100 * $total / $TARGET" | bc 2>/dev/null || echo "0")

  echo "Plan (frontmatter only): ${plan_front_tokens} tokens"
  echo "State (frontmatter + 2 sections): ${state_tokens} tokens"
  echo "Config (frontmatter): ${config_tokens} tokens"
  echo "---"
  echo "Total: $total tokens (${percent}% of target)"

  if [ -z "$total" ] || [ "$total" -gt "$TARGET" ] 2>/dev/null; then
    remaining=$(echo "$total - $TARGET" | bc 2>/dev/null || echo "0")
    echo "❌ EXCEEDS BUDGET by ~${remaining} tokens"
    return 1
  else
    remaining=$(echo "$TARGET - $total" | bc 2>/dev/null || echo "0")
    echo "✅ Within budget (~${remaining} tokens remaining)"
    return 0
  fi
}

# Calculate planner delegation (sparse mode)
calculate_planner() {
  echo "=== Planner Delegation (Sparse Mode) ==="

  # Estimate sparse planner context
  if [ -f .planning/STATE.md ]; then
    local state_front=$(head -30 .planning/STATE.md 2>/dev/null | wc -c | tr -d ' ')
    local state_sec1=$(sed -n '/### Current Position/,/^###/p' .planning/STATE.md | head -10 | wc -c | tr -d ' ' 2>/dev/null)
    local state_sec2=$(sed -n '/### Decisions Made/,/^###/p' .planning/STATE.md | head -20 | wc -c | tr -d ' ' 2>/dev/null)
    local state_sec3=$(sed -n '/### Pending Todos/,/^###/p' .planning/STATE.md | head -10 | wc -c | tr -d ' ' 2>/dev/null)
    local state_tokens=$(echo "( $state_front + $state_sec1 + $state_sec2 + $state_sec3 ) / $CHARS_PER_TOKEN" | bc 2>/dev/null || echo "0")
  else
    local state_tokens=0
  fi

  if [ -f .planning/ROADMAP.md ]; then
    local phase_roadmap=$(grep -A10 "Phase " .planning/ROADMAP.md | head -20 | wc -c | tr -d ' ' 2>/dev/null)
    local roadmap_tokens=$(echo "$phase_roadmap / $CHARS_PER_TOKEN" | bc 2>/dev/null || echo "0")
  else
    local roadmap_tokens=0
  fi

  local total=$(echo "$state_tokens + $roadmap_tokens" | bc 2>/dev/null || echo "0")
  local percent=$(echo "100 * $total / $TARGET" | bc 2>/dev/null || echo "0")

  echo "State (frontmatter + 3 sections): ${state_tokens} tokens"
  echo "Current phase from roadmap: ${roadmap_tokens} tokens"
  echo "---"
  echo "Total: $total tokens (${percent}% of target)"

  if [ -z "$total" ] || [ "$total" -gt "$TARGET" ] 2>/dev/null; then
    remaining=$(echo "$total - $TARGET" | bc 2>/dev/null || echo "0")
    echo "❌ EXCEEDS BUDGET by ~${remaining} tokens"
    return 1
  else
    remaining=$(echo "$TARGET - $total" | bc 2>/dev/null || echo "0")
    echo "✅ Within budget (~${remaining} tokens remaining)"
    return 0
  fi
}

# Print usage
print_usage() {
  echo "Usage: $0 [quality|balanced|budget|tiny] [executor|planner|all] [phase_dir]"
  echo ""
  echo "Arguments:"
  echo "  MODEL_PROFILE: quality (200k), balanced (100k), budget (32k), tiny (8k)"
  echo "  COMPONENT: executor, planner, or all"
  echo "  PHASE_DIR: Path to phase directory (for executor calculation)"
  echo ""
  echo "Examples:"
  echo "  $0 balanced all                          # All components with default phase"
  echo "  $0 budget planner                        # Planner only with budget profile"
  echo "  $0 budget executor .planning/phases/03  # Executor for phase 03"
}

# Run calculations
case "$COMPONENT" in
  executor)
    if [ -z "$3" ]; then
      echo "Error: executor requires phase_dir argument"
      echo ""
      print_usage
      exit 1
    fi
    calculate_executor "$PHASE_DIR"
    ;;
  planner)
    calculate_planner
    ;;
  all|"")
    calculate_planner
    echo ""
    # Try to find first phase directory
    if [ -d "$PHASE_DIR" ]; then
      first_phase=$(find "$PHASE_DIR" -maxdepth 1 -type d -name "??-*" | head -1)
      if [ -n "$first_phase" ]; then
        calculate_executor "$first_phase"
      else
        echo "⚠ No phase directories found in $PHASE_DIR"
        echo "Specify phase explicitly: calculate-context-budget.sh $1 executor .planning/phases/03-authentication"
      fi
    else
      echo "⚠ Phase directory not found: $PHASE_DIR"
      echo "Specify phase explicitly: calculate-context-budget.sh $1 executor .planning/phases/03-authentication"
    fi
    ;;
  *)
    echo "Error: Invalid component '$COMPONENT'"
    echo ""
    print_usage
    exit 1
    ;;
esac

echo ""
echo "=== Summary ==="
echo "Model Profile: $MODEL_PROFILE (capacity: $CAPACITY tokens)"
echo "Budget Target: $TARGET tokens (${TARGET_PERCENT[$MODEL_PROFILE]}%)"
