#!/bin/bash
# fusion-pretool.sh - Attention Injection Engine
#
# PreToolUse hook: runs BEFORE every Write/Edit/Bash/Read/Glob/Grep call.
# Outputs a compact context summary to keep Claude focused on the current task.
#
# Design constraints:
#   - Must execute in < 50ms (no jq, pure grep/awk)
#   - Non-invasive: silent exit if no active Fusion workflow
#   - Fault-tolerant: all operations || true, never blocks Claude
#
# Output goes to stdout and appears in Claude's context window,
# implementing the Manus "attention manipulation through recitation" pattern.

FUSION_DIR=".fusion"

# Fast exit: no fusion directory → not in a workflow
[ -d "$FUSION_DIR" ] || exit 0

# Fast exit: no sessions.json → not initialized
[ -f "$FUSION_DIR/sessions.json" ] || exit 0

# --- Runtime v2.1 adapter ---
# If runtime is enabled, delegate to Python compat_v2 module.
if [ -f "$FUSION_DIR/config.yaml" ] && grep -q 'enabled: *true' "$FUSION_DIR/config.yaml" 2>/dev/null; then
    if python3 -m runtime.compat_v2 pretool "$FUSION_DIR" 2>/dev/null; then
        exit 0
    fi
    # Python failed - fall through to Shell logic
fi

# --- Cross-platform Unicode detection ---
# Use ASCII fallback for terminals that don't support Unicode well (Windows CMD)
# Priority: WT_SESSION (Windows Terminal) > TERM_PROGRAM (VSCode/iTerm) > valid TERM
USE_UNICODE=false
if [ -n "${WT_SESSION:-}" ]; then
    USE_UNICODE=true
elif [ "${TERM_PROGRAM:-}" = "vscode" ] || [ "${TERM_PROGRAM:-}" = "iTerm.app" ]; then
    USE_UNICODE=true
elif [ -n "${TERM:-}" ] && [ "${TERM:-}" != "dumb" ]; then
    USE_UNICODE=true
fi

if [ "$USE_UNICODE" = true ]; then
    CHAR_FILLED="█"
    CHAR_EMPTY="░"
else
    # Fallback for basic terminals
    CHAR_FILLED="#"
    CHAR_EMPTY="-"
fi

# --- JSON parsing helper ---
# Use jq if available (more robust), fallback to grep (faster but fragile)
json_get() {
    local file="$1" key="$2"
    if command -v jq &>/dev/null; then
        jq -r ".$key // empty" "$file" 2>/dev/null || echo ""
    else
        grep -o "\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$file" 2>/dev/null | head -1 | cut -d'"' -f4 || echo ""
    fi
}

# Read status
STATUS=$(json_get "$FUSION_DIR/sessions.json" "status")

# Fast exit: not in_progress → workflow inactive
[ "$STATUS" = "in_progress" ] || exit 0

# --- Active workflow: build context summary ---

# Read goal (truncate to 60 chars, strip control chars for safe display)
GOAL=$(json_get "$FUSION_DIR/sessions.json" "goal")
GOAL=$(printf '%.60s' "$GOAL" | tr -d '"\\\t\n\r')

# Read current phase
PHASE=$(json_get "$FUSION_DIR/sessions.json" "current_phase")
PHASE="${PHASE:-EXECUTE}"

# Phase number mapping
case "$PHASE" in
    INITIALIZE) PHASE_NUM="1/8" ;;
    ANALYZE)    PHASE_NUM="2/8" ;;
    DECOMPOSE)  PHASE_NUM="3/8" ;;
    EXECUTE)    PHASE_NUM="4/8" ;;
    VERIFY)     PHASE_NUM="5/8" ;;
    REVIEW)     PHASE_NUM="6/8" ;;
    COMMIT)     PHASE_NUM="7/8" ;;
    DELIVER)    PHASE_NUM="8/8" ;;
    *)          PHASE_NUM="?/8" ;;
esac

# Count tasks from task_plan.md (if exists)
COMPLETED=0
PENDING=0
IN_PROGRESS=0
FAILED=0
TOTAL=0
CURRENT_TASK=""
CURRENT_TASK_TYPE=""

if [ -f "$FUSION_DIR/task_plan.md" ]; then
    COMPLETED=$(grep -c '\[COMPLETED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || COMPLETED=0
    PENDING=$(grep -c '\[PENDING\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || PENDING=0
    IN_PROGRESS=$(grep -c '\[IN_PROGRESS\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || IN_PROGRESS=0
    FAILED=$(grep -c '\[FAILED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || FAILED=0
    TOTAL=$((COMPLETED + PENDING + IN_PROGRESS + FAILED))

    # Find current/next task (first IN_PROGRESS, then first PENDING)
    if [ "$IN_PROGRESS" -gt 0 ]; then
        CURRENT_TASK=$(grep '\[IN_PROGRESS\]' "$FUSION_DIR/task_plan.md" 2>/dev/null | head -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//')
    elif [ "$PENDING" -gt 0 ]; then
        CURRENT_TASK=$(grep '\[PENDING\]' "$FUSION_DIR/task_plan.md" 2>/dev/null | head -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//')
    fi

    # Get task type (look for "Type:" line after current task header)
    # Use -F for fixed string matching to avoid regex injection from task names
    if [ -n "$CURRENT_TASK" ]; then
        CURRENT_TASK_TYPE=$(grep -F -A5 "$CURRENT_TASK" "$FUSION_DIR/task_plan.md" 2>/dev/null | grep -o 'Type: *[a-z]*' | head -1 | sed 's/Type: *//')
    fi
fi

# Build progress bar (10 chars width)
# Cross-platform: use bash loop instead of seq (not available on Windows Git Bash)
PROGRESS_BAR=""
if [ "$TOTAL" -gt 0 ]; then
    FILLED=$(( COMPLETED * 10 / TOTAL ))
    EMPTY=$(( 10 - FILLED ))
    for ((i=0; i<FILLED; i++)); do PROGRESS_BAR+="$CHAR_FILLED"; done
    for ((i=0; i<EMPTY; i++)); do PROGRESS_BAR+="$CHAR_EMPTY"; done
    PERCENT=$(( COMPLETED * 100 / TOTAL ))
else
    for ((i=0; i<10; i++)); do PROGRESS_BAR+="$CHAR_EMPTY"; done
    PERCENT=0
fi

# Execution guidance based on task type
GUIDANCE=""
case "$CURRENT_TASK_TYPE" in
    implementation|verification)
        GUIDANCE="TDD flow: RED → GREEN → REFACTOR"
        ;;
    design|documentation|configuration|research)
        GUIDANCE="Direct execution"
        ;;
    *)
        if [ "$PHASE" = "EXECUTE" ] && [ -n "$CURRENT_TASK" ]; then
            GUIDANCE="Check task type in task_plan.md"
        fi
        ;;
esac

# Guardian status (quick check from loop_context.json if exists, pure grep)
GUARDIAN_STATUS="OK"
if [ -f "$FUSION_DIR/loop_context.json" ]; then
    NO_PROGRESS=$(grep -o '"no_progress_rounds"[[:space:]]*:[[:space:]]*[0-9]*' "$FUSION_DIR/loop_context.json" 2>/dev/null | grep -o '[0-9]*$') || true
    SAME_ACTION=$(grep -o '"same_action_count"[[:space:]]*:[[:space:]]*[0-9]*' "$FUSION_DIR/loop_context.json" 2>/dev/null | grep -o '[0-9]*$') || true

    if [ "${NO_PROGRESS:-0}" -ge 4 ] || [ "${SAME_ACTION:-0}" -ge 2 ]; then
        GUARDIAN_STATUS="⚠ BACKOFF"
    elif [ "${NO_PROGRESS:-0}" -ge 2 ]; then
        GUARDIAN_STATUS="~"
    fi
fi

# --- Output compact summary ---

echo "[fusion] Goal: ${GOAL:-?} | Phase: $PHASE ($PHASE_NUM)"

if [ "$TOTAL" -gt 0 ] && [ -n "$CURRENT_TASK" ]; then
    TASK_STATUS="PENDING"
    [ "$IN_PROGRESS" -gt 0 ] && TASK_STATUS="IN_PROGRESS"
    TASK_INDEX=$((COMPLETED + 1))
    TYPE_DISPLAY=""
    [ -n "$CURRENT_TASK_TYPE" ] && TYPE_DISPLAY=" (type: $CURRENT_TASK_TYPE)"
    echo "[fusion] Task ${TASK_INDEX}/${TOTAL}: ${CURRENT_TASK} [${TASK_STATUS}]${TYPE_DISPLAY}"
    echo "[fusion] Progress: ${PROGRESS_BAR} ${PERCENT}% | Guardian: ${GUARDIAN_STATUS}"
fi

if [ -n "$GUIDANCE" ]; then
    echo "[fusion] → $GUIDANCE"
fi

exit 0
