#!/bin/bash
# fusion-cancel.sh - Cancel the current workflow
set -euo pipefail

FUSION_DIR=".fusion"
STATE_LOCK="${FUSION_DIR}/.state.lock"
LOCK_STALE_SECONDS=300  # 5 minutes
LOCK_ACQUIRED=false

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
    echo "❌ No fusion workflow found in current directory"
    exit 1
fi

if [ ! -f "$FUSION_DIR/sessions.json" ]; then
    echo "❌ No active session found"
    exit 1
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

# Check current status (CAS: read before write)
if command -v jq &>/dev/null; then
    STATUS=$(jq -r '.status // "unknown"' "$FUSION_DIR/sessions.json")
    GOAL=$(jq -r '.goal // "unknown"' "$FUSION_DIR/sessions.json")
else
    STATUS=$(grep -o '"status"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "unknown")
    GOAL=$(grep -o '"goal"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "unknown")
fi

if [ "$STATUS" = "cancelled" ]; then
    echo "⚠️ Workflow is already cancelled"
    exit 0
fi

if [ "$STATUS" = "completed" ]; then
    echo "⚠️ Workflow is already completed"
    exit 0
fi

# Update status to cancelled (within lock - atomic)
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
UPDATE_SUCCESS=false

if command -v jq &>/dev/null; then
    TMP_FILE=$(mktemp "${FUSION_DIR}/.tmp.XXXXXX")
    if jq ".status = \"cancelled\" | .last_checkpoint = \"$TIMESTAMP\"" "$FUSION_DIR/sessions.json" > "$TMP_FILE" 2>/dev/null; then
        mv "$TMP_FILE" "$FUSION_DIR/sessions.json"
        UPDATE_SUCCESS=true
    else
        rm -f "$TMP_FILE" 2>/dev/null || true
    fi
else
    # Try GNU sed first, then BSD sed
    if sed -i "s/\"status\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"status\": \"cancelled\"/" "$FUSION_DIR/sessions.json" 2>/dev/null; then
        UPDATE_SUCCESS=true
    elif sed -i '' "s/\"status\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"status\": \"cancelled\"/" "$FUSION_DIR/sessions.json" 2>/dev/null; then
        UPDATE_SUCCESS=true
    fi
    # Verify change (CAS verification)
    if [ "$UPDATE_SUCCESS" = true ]; then
        VERIFY=$(grep -o '"status"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "")
        if [ "$VERIFY" != "cancelled" ]; then
            UPDATE_SUCCESS=false
        fi
    fi
fi

if [ "$UPDATE_SUCCESS" = false ]; then
    echo "❌ Failed to update workflow status"
    exit 1
fi

# Log to progress
if [ -f "$FUSION_DIR/progress.md" ]; then
    echo "| $TIMESTAMP | CANCELLED | User requested cancel | OK | Workflow terminated |" >> "$FUSION_DIR/progress.md"
fi

# Reset block count
rm -f "$FUSION_DIR/.block_count" 2>/dev/null || true

echo "❌ Workflow cancelled"
echo ""
echo "Goal: $GOAL"
echo ""
echo "Options:"
echo "  - Start fresh: /fusion \"<new goal>\""
echo "  - Clean up: rm -rf .fusion/"
