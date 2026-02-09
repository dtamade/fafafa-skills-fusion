#!/bin/bash
# fusion-status.sh - Show current fusion status
set -euo pipefail

FUSION_DIR=".fusion"

if [ ! -d "$FUSION_DIR" ]; then
    echo "[fusion] No .fusion directory found. Run /fusion to start."
    exit 1
fi

echo "=== Fusion Status ==="
echo ""

# Show current status from task_plan.md
if [ -f "$FUSION_DIR/task_plan.md" ]; then
    echo "## Task Plan"
    grep -A 5 "^## Status" "$FUSION_DIR/task_plan.md" 2>/dev/null || echo "No status found"
    echo ""
fi

# Show recent progress
if [ -f "$FUSION_DIR/progress.md" ]; then
    echo "## Recent Progress (last 10 entries)"
    grep "^|" "$FUSION_DIR/progress.md" | tail -12
    echo ""
fi

# Show any errors
if [ -f "$FUSION_DIR/progress.md" ]; then
    error_count=$(grep -c "ERROR\|FAILED" "$FUSION_DIR/progress.md" 2>/dev/null) || error_count=0
    if [ "$error_count" -gt 0 ]; then
        echo "## Errors: $error_count found"
        grep "ERROR\|FAILED" "$FUSION_DIR/progress.md" | tail -5
    fi
fi

# Show sessions
if [ -f "$FUSION_DIR/sessions.json" ]; then
    echo "## Active Sessions"
    cat "$FUSION_DIR/sessions.json" | head -5
fi
