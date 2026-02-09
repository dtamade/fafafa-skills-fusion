#!/bin/bash
# fusion-continue.sh - Optional helper to add continuation markers to progress.md
#
# NOTE: This is NOT a required hook. The Stop hook (fusion-stop-guard.sh) handles
# the main execution loop. This script can be used manually or as a PostToolUse
# hook to add visual markers in progress.md.
#
# Usage:
#   ./scripts/fusion-continue.sh   # Run manually to add a marker
#
# The real execution loop is enforced by:
#   1. Stop hook blocking premature exit
#   2. SKILL.md instructions telling Claude to check task_plan.md

FUSION_DIR=".fusion"

# Exit early if no fusion directory (not in a fusion workflow)
if [ ! -d "$FUSION_DIR" ]; then
    exit 0
fi

# Check if sessions.json exists and has active workflow
if [ ! -f "$FUSION_DIR/sessions.json" ]; then
    exit 0
fi

# Read status from sessions.json
STATUS=$(grep -o '"status"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4)

# Only add marker if workflow is in_progress
if [ "$STATUS" != "in_progress" ]; then
    exit 0
fi

# Get current phase
CURRENT_PHASE=$(grep -o '"current_phase"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4)

# Count pending tasks from task_plan.md
if [ -f "$FUSION_DIR/task_plan.md" ]; then
    PENDING_COUNT=$(grep -c "\[PENDING\]\|\[IN_PROGRESS\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || PENDING_COUNT=0
else
    PENDING_COUNT="?"
fi

# Append a continuation marker to progress.md (avoid spam by checking last line)
if [ -f "$FUSION_DIR/progress.md" ]; then
    LAST_LINE=$(tail -1 "$FUSION_DIR/progress.md")
    if [[ "$LAST_LINE" != *"[CONTINUE]"* ]]; then
        echo "" >> "$FUSION_DIR/progress.md"
        echo "<!-- [CONTINUE] Phase: $CURRENT_PHASE | Pending: $PENDING_COUNT | Check task_plan.md and continue -->" >> "$FUSION_DIR/progress.md"
    fi
fi

exit 0
