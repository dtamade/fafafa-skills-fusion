#!/bin/bash
# fusion-logs.sh - Show detailed execution logs
set -euo pipefail

FUSION_DIR=".fusion"

is_truthy() {
    case "$(printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]')" in
        1|true|yes|on)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

hook_debug_enabled() {
    if is_truthy "${FUSION_HOOK_DEBUG:-}"; then
        return 0
    fi

    [ -f "$FUSION_DIR/.hook_debug" ]
}

usage() {
    echo "Usage: fusion-logs.sh [lines]"
}

if [ "$#" -gt 1 ]; then
    echo "❌ Too many arguments" >&2
    usage >&2
    exit 1
fi

case "${1:-}" in
    -h|--help)
        usage
        exit 0
        ;;
    --*)
        echo "❌ Unknown option: $1" >&2
        usage >&2
        exit 1
        ;;
    "")
        LINES="50"
        ;;
    *)
        LINES="$1"
        ;;
esac

if ! [[ "$LINES" =~ ^[1-9][0-9]*$ ]]; then
    echo "❌ LINES must be a positive integer" >&2
    usage >&2
    exit 1
fi

if [ ! -d "$FUSION_DIR" ]; then
    echo "❌ No fusion workflow found in current directory"
    exit 1
fi

echo "═══════════════════════════════════════════════════════════════"
echo "                    FUSION WORKFLOW LOGS"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Session Info
if [ -f "$FUSION_DIR/sessions.json" ]; then
    echo "📋 SESSION INFO"
    echo "───────────────────────────────────────────────────────────────"
    if command -v jq &>/dev/null; then
        jq -r '
            "Goal: \(.goal // "N/A")",
            "Status: \(.status // "N/A")",
            "Phase: \(.current_phase // "N/A")",
            "Started: \(.started_at // "N/A")",
            "Last checkpoint: \(.last_checkpoint // "N/A")"
        ' "$FUSION_DIR/sessions.json" 2>/dev/null || cat "$FUSION_DIR/sessions.json"
    else
        cat "$FUSION_DIR/sessions.json"
    fi
    echo ""
fi

# Hook debug summary
echo "🪝 HOOK DEBUG"
echo "───────────────────────────────────────────────────────────────"
if hook_debug_enabled; then
    echo "enabled: true"
else
    echo "enabled: false"
fi

if [ -f "$FUSION_DIR/.hook_debug" ]; then
    echo "flag: $FUSION_DIR/.hook_debug"
fi

if [ -f "$FUSION_DIR/hook-debug.log" ]; then
    echo "log: $FUSION_DIR/hook-debug.log"
    echo "recent (last 5):"
    tail -n 5 "$FUSION_DIR/hook-debug.log" 2>/dev/null | sed 's/^/  /'
else
    echo "log: (none)"
fi

echo ""

# Task Plan Summary (safe grep count)
if [ -f "$FUSION_DIR/task_plan.md" ]; then
    echo "📝 TASK SUMMARY"
    echo "───────────────────────────────────────────────────────────────"
    TOTAL=$(grep -c "^### Task" "$FUSION_DIR/task_plan.md" 2>/dev/null) || TOTAL=0
    COMPLETED=$(grep -c "\[COMPLETED\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || COMPLETED=0
    IN_PROGRESS=$(grep -c "\[IN_PROGRESS\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || IN_PROGRESS=0
    PENDING=$(grep -c "\[PENDING\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || PENDING=0
    FAILED=$(grep -c "\[FAILED\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || FAILED=0
    SKIPPED=$(grep -c "\[SKIPPED\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || SKIPPED=0

    echo "Total tasks: $TOTAL"
    echo "  ✅ Completed: $COMPLETED"
    echo "  🔄 In Progress: $IN_PROGRESS"
    echo "  ⏳ Pending: $PENDING"
    echo "  ❌ Failed: $FAILED"
    echo "  ⏭️ Skipped: $SKIPPED"
    echo ""

    # Show current/next task
    CURRENT=$(grep -A2 "\[IN_PROGRESS\]" "$FUSION_DIR/task_plan.md" 2>/dev/null | head -3 || echo "")
    if [ -n "$CURRENT" ]; then
        echo "Current task:"
        echo "$CURRENT" | sed 's/^/  /'
        echo ""
    fi
fi

# Progress Timeline (last N entries)
if [ -f "$FUSION_DIR/progress.md" ]; then
    echo "📊 PROGRESS TIMELINE (last $LINES entries)"
    echo "───────────────────────────────────────────────────────────────"
    # Skip header lines and show last N table rows
    grep "^|" "$FUSION_DIR/progress.md" | grep -v "^| Timestamp" | grep -v "^|---" | tail -n "$LINES"
    echo ""
fi

# Findings
if [ -f "$FUSION_DIR/findings.md" ]; then
    FINDINGS_COUNT=$(grep -c "^##" "$FUSION_DIR/findings.md" 2>/dev/null) || FINDINGS_COUNT=0
    if [ "$FINDINGS_COUNT" -gt 0 ]; then
        echo "🔍 FINDINGS ($FINDINGS_COUNT entries)"
        echo "───────────────────────────────────────────────────────────────"
        grep "^##" "$FUSION_DIR/findings.md" | head -10
        echo ""
    fi
fi

# Error summary
if [ -f "$FUSION_DIR/progress.md" ]; then
    ERRORS=$(grep -i "ERROR\|FAILED\|Strike" "$FUSION_DIR/progress.md" 2>/dev/null | tail -5 || echo "")
    if [ -n "$ERRORS" ]; then
        echo "⚠️ RECENT ERRORS"
        echo "───────────────────────────────────────────────────────────────"
        echo "$ERRORS"
        echo ""
    fi
fi

echo "═══════════════════════════════════════════════════════════════"
echo "For full details:"
echo "  - Task plan: cat $FUSION_DIR/task_plan.md"
echo "  - Progress: cat $FUSION_DIR/progress.md"
echo "  - Findings: cat $FUSION_DIR/findings.md"
echo "═══════════════════════════════════════════════════════════════"
