#!/bin/bash
# fusion-resume.sh - Resume a Fusion workflow from checkpoint
set -euo pipefail

FUSION_DIR=".fusion"
STATE_LOCK="${FUSION_DIR}/.state.lock"
LOCK_STALE_SECONDS=300  # 5 minutes
LOCK_ACQUIRED=false

# Get script directory (cross-platform)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if lock is stale
is_lock_stale() {
    local lock_dir="$1"
    if [ ! -d "$lock_dir" ]; then return 1; fi
    local lock_mtime
    if stat --version &>/dev/null 2>&1; then
        lock_mtime=$(stat -c %Y "$lock_dir" 2>/dev/null) || return 1
    else
        lock_mtime=$(stat -f %m "$lock_dir" 2>/dev/null) || return 1
    fi
    local current_time=$(date +%s)
    local age=$((current_time - lock_mtime))
    [ "$age" -gt "$LOCK_STALE_SECONDS" ]
}

# Cleanup lock on exit - ONLY if we acquired it
cleanup() {
    if [ "$LOCK_ACQUIRED" = true ]; then
        rmdir "$STATE_LOCK" 2>/dev/null || true
    fi
}
trap cleanup EXIT

if [ ! -d "$FUSION_DIR" ]; then
    echo "❌ No .fusion directory found. Nothing to resume."
    exit 1
fi

if [ ! -f "$FUSION_DIR/sessions.json" ]; then
    echo "❌ No sessions.json found. Cannot resume."
    exit 1
fi

# Read session info (before lock, for display purposes)
if command -v jq &>/dev/null; then
    STATUS=$(jq -r '.status // "unknown"' "$FUSION_DIR/sessions.json")
    GOAL=$(jq -r '.goal // "unknown"' "$FUSION_DIR/sessions.json")
    CURRENT_PHASE=$(jq -r '.current_phase // "EXECUTE"' "$FUSION_DIR/sessions.json")
    CODEX_SESSION=$(jq -r '.codex_session // ""' "$FUSION_DIR/sessions.json")
    LAST_CHECKPOINT=$(jq -r '.last_checkpoint // "unknown"' "$FUSION_DIR/sessions.json")
else
    STATUS=$(grep -o '"status"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "unknown")
    GOAL=$(grep -o '"goal"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "unknown")
    CURRENT_PHASE=$(grep -o '"current_phase"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "EXECUTE")
    CODEX_SESSION=$(grep -o '"codex_session"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "")
    LAST_CHECKPOINT=$(grep -o '"last_checkpoint"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "unknown")
fi

echo "═══════════════════════════════════════════════════════════════"
echo "                    FUSION WORKFLOW RESUME"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "📋 Session Info:"
echo "   Goal: $GOAL"
echo "   Status: $STATUS"
echo "   Phase: $CURRENT_PHASE"
echo "   Last checkpoint: $LAST_CHECKPOINT"
if [ -n "$CODEX_SESSION" ]; then
    echo "   Codex Session: $CODEX_SESSION"
fi
echo ""

# Check if workflow can be resumed (before lock)
case "$STATUS" in
    "completed")
        echo "✅ Workflow already completed. Nothing to resume."
        exit 0
        ;;
    "cancelled")
        echo "❌ Workflow was cancelled. Start a new workflow with:"
        echo "   /fusion \"<new goal>\""
        exit 1
        ;;
    "stuck")
        echo "⚠️ Workflow is stuck. Please investigate:"
        echo "   - Check .fusion/progress.md for errors"
        echo "   - Fix the issue and run /fusion resume again"
        echo "   - Or cancel with: ./scripts/fusion-cancel.sh"
        ;;
esac

# Count tasks (safe grep count)
if [ -f "$FUSION_DIR/task_plan.md" ]; then
    TOTAL=$(grep -c "^### Task" "$FUSION_DIR/task_plan.md" 2>/dev/null) || TOTAL=0
    COMPLETED=$(grep -c "\[COMPLETED\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || COMPLETED=0
    PENDING=$(grep -c "\[PENDING\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || PENDING=0
    IN_PROGRESS=$(grep -c "\[IN_PROGRESS\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || IN_PROGRESS=0

    echo "📊 Task Progress:"
    echo "   Total: $TOTAL"
    echo "   ✅ Completed: $COMPLETED"
    echo "   🔄 In Progress: $IN_PROGRESS"
    echo "   ⏳ Pending: $PENDING"
    echo ""

    # Show next task
    NEXT_TASK=$(grep -B1 "\[PENDING\]\|\[IN_PROGRESS\]" "$FUSION_DIR/task_plan.md" 2>/dev/null | grep "^### Task" | head -1 || echo "")
    if [ -n "$NEXT_TASK" ]; then
        echo "📝 Next Task:"
        echo "   $NEXT_TASK"
        echo ""
    fi
fi

# Acquire state lock (atomic via mkdir) with stale detection
if is_lock_stale "$STATE_LOCK"; then
    echo "⚠️ Cleaning up stale lock" >&2
    rmdir "$STATE_LOCK" 2>/dev/null || true
fi
if ! mkdir "$STATE_LOCK" 2>/dev/null; then
    echo "⚠️ Another state operation in progress, please retry"
    exit 1
fi
LOCK_ACQUIRED=true

# Re-check status (CAS: read within lock to prevent race)
if command -v jq &>/dev/null; then
    STATUS=$(jq -r '.status // "unknown"' "$FUSION_DIR/sessions.json")
else
    STATUS=$(grep -o '"status"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "unknown")
fi

# Verify status is resumable (within lock)
if [ "$STATUS" = "completed" ] || [ "$STATUS" = "cancelled" ]; then
    echo "⚠️ Status changed to '$STATUS' by another process. Cannot resume."
    exit 1
fi

# Update status to in_progress (within lock - atomic)
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
UPDATE_SUCCESS=false

if command -v jq &>/dev/null; then
    TMP_FILE=$(mktemp "${FUSION_DIR}/.tmp.XXXXXX")
    if jq ".status = \"in_progress\" | .last_checkpoint = \"$TIMESTAMP\"" "$FUSION_DIR/sessions.json" > "$TMP_FILE" 2>/dev/null; then
        mv "$TMP_FILE" "$FUSION_DIR/sessions.json"
        UPDATE_SUCCESS=true
    else
        rm -f "$TMP_FILE" 2>/dev/null || true
    fi
else
    # Try GNU sed first, then BSD sed
    if sed -i "s/\"status\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"status\": \"in_progress\"/" "$FUSION_DIR/sessions.json" 2>/dev/null; then
        UPDATE_SUCCESS=true
    elif sed -i '' "s/\"status\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"status\": \"in_progress\"/" "$FUSION_DIR/sessions.json" 2>/dev/null; then
        UPDATE_SUCCESS=true
    fi
    # Verify change (CAS verification)
    if [ "$UPDATE_SUCCESS" = true ]; then
        VERIFY=$(grep -o '"status"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "")
        if [ "$VERIFY" != "in_progress" ]; then
            UPDATE_SUCCESS=false
        fi
    fi
fi

if [ "$UPDATE_SUCCESS" = false ]; then
    echo "❌ Failed to update workflow status"
    exit 1
fi

# Reset block count
rm -f "$FUSION_DIR/.block_count" 2>/dev/null || true

# Log resume to progress
if [ -f "$FUSION_DIR/progress.md" ]; then
    echo "| $TIMESTAMP | RESUME | Workflow resumed | OK | Continuing from checkpoint |" >> "$FUSION_DIR/progress.md"
fi

echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "🚀 READY TO RESUME"
echo ""
echo "Status has been set to 'in_progress'."
echo "Claude will automatically continue executing when the stop hook runs."
echo ""

# Session catchup: recover context from previous session
# Cross-platform Python 3 detection
# Priority: python3 > py -3 (Windows launcher) > python (if it's Python 3)
CATCHUP_SCRIPT="$SCRIPT_DIR/fusion-catchup.py"
PYTHON_CMD=""
if command -v python3 &>/dev/null; then
    PYTHON_CMD="python3"
elif command -v py &>/dev/null && py -3 --version &>/dev/null 2>&1; then
    # Windows Python Launcher
    PYTHON_CMD="py -3"
elif command -v python &>/dev/null; then
    # Check if python is Python 3 (not Python 2)
    if python --version 2>&1 | grep -q "Python 3"; then
        PYTHON_CMD="python"
    fi
fi

if [ -f "$CATCHUP_SCRIPT" ] && [ -n "$PYTHON_CMD" ]; then
    $PYTHON_CMD "$CATCHUP_SCRIPT" "$(pwd)" 2>/dev/null || true
fi

echo "Or you can instruct Claude directly:"
echo ""
echo "  Read .fusion/task_plan.md and continue executing pending tasks."
echo "  For implementation tasks, use TDD flow (RED→GREEN→REFACTOR)."
echo "  For other tasks, execute directly."
if [ -n "$CODEX_SESSION" ]; then
    echo "  Use session ID: $CODEX_SESSION for codeagent-wrapper resume."
fi
echo ""
echo "═══════════════════════════════════════════════════════════════"
