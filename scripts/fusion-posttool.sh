#!/bin/bash
# fusion-posttool.sh - Progress Monitor
#
# PostToolUse hook: runs AFTER every Write/Edit call.
# Detects progress changes and provides structured status updates.
#
# Design constraints:
#   - Must execute quickly (no jq, pure grep)
#   - Non-invasive: silent exit if no active Fusion workflow
#   - Fault-tolerant: all operations || true, never blocks Claude
#   - Maintains a snapshot file (.fusion/.progress_snapshot) for diff detection

FUSION_DIR=".fusion"
SNAPSHOT_FILE="$FUSION_DIR/.progress_snapshot"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Fast exit: no fusion directory
[ -d "$FUSION_DIR" ] || exit 0
[ -f "$FUSION_DIR/sessions.json" ] || exit 0

# --- Runtime v2.1 adapter ---
# If runtime is enabled, delegate to Python compat_v2 module.
# Also trigger state machine events for task transitions.
if [ -f "$FUSION_DIR/config.yaml" ] && grep -q 'enabled: *true' "$FUSION_DIR/config.yaml" 2>/dev/null; then
    # Get posttool output
    if OUTPUT=$(PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" python3 -m runtime.compat_v2 posttool "$FUSION_DIR" 2>/dev/null); then
        echo "$OUTPUT"
    fi
    # Trigger state machine events based on task state changes
    # This is done regardless of posttool output success
    PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" python3 << 'PYEOF' 2>/dev/null || true
import sys
from pathlib import Path
from runtime.kernel import create_kernel
from runtime.state_machine import Event, State

fusion_dir = ".fusion"
task_plan = Path(fusion_dir) / "task_plan.md"

if not task_plan.exists():
    sys.exit(0)

content = task_plan.read_text()
completed = content.count("[COMPLETED]")
pending = content.count("[PENDING]")
in_progress = content.count("[IN_PROGRESS]")
failed = content.count("[FAILED]")

total_remaining = pending + in_progress + failed

kernel = create_kernel(fusion_dir)
kernel.load_state()

# If in EXECUTE phase and all tasks done, transition to VERIFY
if kernel.current_state == State.EXECUTE and total_remaining == 0 and completed > 0:
    kernel.dispatch(Event.ALL_TASKS_DONE)
# If task in progress, dispatch TASK_DONE to stay in EXECUTE (for logging)
elif kernel.current_state == State.EXECUTE and completed > 0:
    # Check if a task was just completed by comparing to snapshot
    snap_file = Path(fusion_dir) / ".progress_snapshot"
    if snap_file.exists():
        prev = snap_file.read_text().strip().split(":")
        prev_completed = int(prev[0]) if prev and prev[0].isdigit() else 0
        if completed > prev_completed:
            # Dispatch TASK_DONE (stays in EXECUTE if tasks remaining)
            kernel.context.pending_tasks = total_remaining
            kernel.context.completed_tasks = completed
            kernel.dispatch(Event.TASK_DONE)
PYEOF
    exit 0
fi
# Python failed or not enabled - fall through to Shell logic

# --- JSON parsing helper ---
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
[ "$STATUS" = "in_progress" ] || exit 0

# --- Active workflow: detect progress changes ---

# Current counts from task_plan.md
COMPLETED=0
PENDING=0
IN_PROGRESS=0
FAILED=0

if [ -f "$FUSION_DIR/task_plan.md" ]; then
    COMPLETED=$(grep -c '\[COMPLETED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || COMPLETED=0
    PENDING=$(grep -c '\[PENDING\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || PENDING=0
    IN_PROGRESS=$(grep -c '\[IN_PROGRESS\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || IN_PROGRESS=0
    FAILED=$(grep -c '\[FAILED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || FAILED=0
fi

TOTAL=$((COMPLETED + PENDING + IN_PROGRESS + FAILED))
CURRENT_SNAPSHOT="${COMPLETED}:${PENDING}:${IN_PROGRESS}:${FAILED}"

# Read previous snapshot
PREV_SNAPSHOT=""
if [ -f "$SNAPSHOT_FILE" ]; then
    PREV_SNAPSHOT=$(cat "$SNAPSHOT_FILE" 2>/dev/null) || true
fi

# Save current snapshot
echo "$CURRENT_SNAPSHOT" > "$SNAPSHOT_FILE" 2>/dev/null || true

# Compare: did anything change?
if [ "$CURRENT_SNAPSHOT" = "$PREV_SNAPSHOT" ]; then
    # No task status change — check if code files were changed
    # but task_plan.md wasn't updated (common oversight)
    # Hook input from stdin contains tool_name and file_path
    # We use a heuristic: if snapshot hasn't changed in a while, remind
    STALE_FILE="$FUSION_DIR/.snapshot_unchanged_count"
    UNCHANGED=0
    if [ -f "$STALE_FILE" ]; then
        UNCHANGED=$(cat "$STALE_FILE" 2>/dev/null) || UNCHANGED=0
        if ! [[ "$UNCHANGED" =~ ^[0-9]+$ ]]; then
            UNCHANGED=0
        fi
    fi
    UNCHANGED=$((UNCHANGED + 1))
    echo "$UNCHANGED" > "$STALE_FILE" 2>/dev/null || true

    # After 5 consecutive Write/Edit calls without progress update → remind
    if [ "$UNCHANGED" -ge 5 ] && [ "$TOTAL" -gt 0 ]; then
        CURRENT_TASK=$(grep '\[IN_PROGRESS\]' "$FUSION_DIR/task_plan.md" 2>/dev/null | head -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//')
        echo "[fusion] Info: ${UNCHANGED} file edits since last task status change."
        if [ -n "$CURRENT_TASK" ]; then
            echo "[fusion] Current: ${CURRENT_TASK} [IN_PROGRESS] | When done, mark [COMPLETED] in task_plan.md"
        fi
    fi
    exit 0
fi

# --- Progress changed! Parse what happened ---

# Reset unchanged counter
rm -f "$FUSION_DIR/.snapshot_unchanged_count" 2>/dev/null || true

# Parse previous values
PREV_COMPLETED=$(echo "$PREV_SNAPSHOT" | cut -d: -f1) || PREV_COMPLETED=0
PREV_PENDING=$(echo "$PREV_SNAPSHOT" | cut -d: -f2) || PREV_PENDING=0
PREV_IN_PROGRESS=$(echo "$PREV_SNAPSHOT" | cut -d: -f3) || PREV_IN_PROGRESS=0
PREV_FAILED=$(echo "$PREV_SNAPSHOT" | cut -d: -f4) || PREV_FAILED=0

# Ensure numeric
[[ "$PREV_COMPLETED" =~ ^[0-9]+$ ]] || PREV_COMPLETED=0
[[ "$PREV_PENDING" =~ ^[0-9]+$ ]] || PREV_PENDING=0
[[ "$PREV_IN_PROGRESS" =~ ^[0-9]+$ ]] || PREV_IN_PROGRESS=0
[[ "$PREV_FAILED" =~ ^[0-9]+$ ]] || PREV_FAILED=0

# Detect specific transitions
COMPLETED_DELTA=$((COMPLETED - PREV_COMPLETED))
FAILED_DELTA=$((FAILED - PREV_FAILED))

if [ "$COMPLETED_DELTA" -gt 0 ]; then
    # A task was completed
    # Find the most recently completed task (last COMPLETED entry)
    JUST_COMPLETED=$(grep '\[COMPLETED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null | tail -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//')
    echo "[fusion] Task ${JUST_COMPLETED:-?} → COMPLETED (${COMPLETED}/${TOTAL} done)"

    # Show next task if available
    NEXT_TASK=""
    NEXT_TYPE=""
    if [ "$IN_PROGRESS" -gt 0 ]; then
        NEXT_TASK=$(grep '\[IN_PROGRESS\]' "$FUSION_DIR/task_plan.md" 2>/dev/null | head -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//')
    elif [ "$PENDING" -gt 0 ]; then
        NEXT_TASK=$(grep '\[PENDING\]' "$FUSION_DIR/task_plan.md" 2>/dev/null | head -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//')
    fi

    if [ -n "$NEXT_TASK" ]; then
        NEXT_TYPE=$(grep -F -A5 "$NEXT_TASK" "$FUSION_DIR/task_plan.md" 2>/dev/null | grep -o 'Type: *[a-z]*' | head -1 | sed 's/Type: *//')
        GUIDANCE=""
        case "$NEXT_TYPE" in
            implementation|verification) GUIDANCE="TDD" ;;
            *) GUIDANCE="Direct" ;;
        esac
        echo "[fusion] Next: ${NEXT_TASK} → ${GUIDANCE} execution"
    elif [ "$PENDING" -eq 0 ] && [ "$IN_PROGRESS" -eq 0 ]; then
        echo "[fusion] All tasks completed! Proceed to VERIFY phase."
    fi
fi

if [ "$FAILED_DELTA" -gt 0 ]; then
    JUST_FAILED=$(grep '\[FAILED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null | tail -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//')
    echo "[fusion] Task ${JUST_FAILED:-?} → FAILED. Apply 3-Strike protocol."
fi

exit 0
