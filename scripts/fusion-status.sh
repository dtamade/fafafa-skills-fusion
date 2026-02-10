#!/bin/bash
# fusion-status.sh - Show current fusion status
set -euo pipefail

FUSION_DIR=".fusion"

if [ ! -d "$FUSION_DIR" ]; then
    echo "[fusion] No .fusion directory found. Run /fusion to start."
    exit 1
fi

json_get() {
    local file="$1" key="$2"
    if command -v jq &>/dev/null; then
        jq -r "$key // empty" "$file" 2>/dev/null || echo ""
    else
        local clean_key="${key#.}"
        grep -o "\"$clean_key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$file" 2>/dev/null | head -1 | cut -d'"' -f4 || echo ""
    fi
}

format_epoch_utc_iso() {
    local ts="$1"

    [ -n "$ts" ] || return 0

    if command -v python3 &>/dev/null; then
        python3 - "$ts" <<'PY'
import datetime
import sys

try:
    value = float(sys.argv[1])
except Exception:
    raise SystemExit(0)

dt = datetime.datetime.fromtimestamp(value, tz=datetime.timezone.utc).replace(microsecond=0)
print(dt.isoformat().replace("+00:00", "Z"))
PY
        return 0
    fi

    if date -u -d "@0" +%Y-%m-%dT%H:%M:%SZ >/dev/null 2>&1; then
        local sec
        sec=$(printf '%.0f' "$ts" 2>/dev/null || true)
        [ -n "$sec" ] && date -u -d "@$sec" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || true
        return 0
    fi

    if date -u -r 0 +%Y-%m-%dT%H:%M:%SZ >/dev/null 2>&1; then
        local sec
        sec=$(printf '%.0f' "$ts" 2>/dev/null || true)
        [ -n "$sec" ] && date -u -r "$sec" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || true
    fi
}

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

    # Runtime & scheduler summary
    echo ""
    echo "## Runtime"
    STATUS=$(json_get "$FUSION_DIR/sessions.json" ".status")
    PHASE=$(json_get "$FUSION_DIR/sessions.json" ".current_phase")
    LAST_EVENT=$(json_get "$FUSION_DIR/sessions.json" "._runtime.last_event_id")
    EVENT_COUNTER=$(json_get "$FUSION_DIR/sessions.json" "._runtime.last_event_counter")
    SCHED_ENABLED=$(json_get "$FUSION_DIR/sessions.json" "._runtime.scheduler.enabled")
    SCHED_BATCH=$(json_get "$FUSION_DIR/sessions.json" "._runtime.scheduler.current_batch_id")
    SCHED_PARALLEL=$(json_get "$FUSION_DIR/sessions.json" "._runtime.scheduler.parallel_tasks")

    [ -n "$STATUS" ] && echo "status: $STATUS"
    [ -n "$PHASE" ] && echo "phase: $PHASE"
    [ -n "$LAST_EVENT" ] && echo "last_event_id: $LAST_EVENT"
    [ -n "$EVENT_COUNTER" ] && echo "event_counter: $EVENT_COUNTER"
    if [ -n "$SCHED_ENABLED" ]; then
        echo "scheduler.enabled: $SCHED_ENABLED"
        [ -n "$SCHED_BATCH" ] && echo "scheduler.batch_id: $SCHED_BATCH"
        [ -n "$SCHED_PARALLEL" ] && echo "scheduler.parallel_tasks: $SCHED_PARALLEL"
    fi

    # Safe backlog latest injection summary
    if [ -f "$FUSION_DIR/events.jsonl" ]; then
        if command -v jq &>/dev/null; then
            SAFE_EVENT=$(jq -c 'select(.type == "SAFE_BACKLOG_INJECTED")' "$FUSION_DIR/events.jsonl" 2>/dev/null | tail -1)
            if [ -n "$SAFE_EVENT" ]; then
                SAFE_ADDED=$(printf '%s\n' "$SAFE_EVENT" | jq -r '.payload.added // empty' 2>/dev/null || echo "")
                SAFE_TS=$(printf '%s\n' "$SAFE_EVENT" | jq -r '.timestamp // empty' 2>/dev/null || echo "")
                [ -n "$SAFE_ADDED" ] && echo "safe_backlog.last_added: $SAFE_ADDED"
                if [ -n "$SAFE_TS" ]; then
                    echo "safe_backlog.last_injected_at: $SAFE_TS"
                    SAFE_TS_ISO=$(format_epoch_utc_iso "$SAFE_TS")
                    [ -n "$SAFE_TS_ISO" ] && echo "safe_backlog.last_injected_at_iso: $SAFE_TS_ISO"
                fi
            fi
        else
            SAFE_LINE=$(grep '"type"[[:space:]]*:[[:space:]]*"SAFE_BACKLOG_INJECTED"' "$FUSION_DIR/events.jsonl" 2>/dev/null | tail -1 || true)
            if [ -n "$SAFE_LINE" ]; then
                SAFE_ADDED=$(printf '%s\n' "$SAFE_LINE" | grep -o '"added"[[:space:]]*:[[:space:]]*[0-9]*' | head -1 | grep -o '[0-9]*' || true)
                SAFE_TS=$(printf '%s\n' "$SAFE_LINE" | grep -o '"timestamp"[[:space:]]*:[[:space:]]*[0-9.]*' | head -1 | sed 's/.*:[[:space:]]*//' || true)
                [ -n "$SAFE_ADDED" ] && echo "safe_backlog.last_added: $SAFE_ADDED"
                if [ -n "$SAFE_TS" ]; then
                    echo "safe_backlog.last_injected_at: $SAFE_TS"
                    SAFE_TS_ISO=$(format_epoch_utc_iso "$SAFE_TS")
                    [ -n "$SAFE_TS_ISO" ] && echo "safe_backlog.last_injected_at_iso: $SAFE_TS_ISO"
                fi
            fi
        fi
    fi
fi

# Show unresolved dependency report (if present)
if [ -f "$FUSION_DIR/dependency_report.json" ]; then
    echo ""
    echo "## Dependency Report"

    DEP_STATUS=$(json_get "$FUSION_DIR/dependency_report.json" ".status")
    DEP_SOURCE=$(json_get "$FUSION_DIR/dependency_report.json" ".source")
    DEP_REASON=$(json_get "$FUSION_DIR/dependency_report.json" ".reason")

    [ -n "$DEP_STATUS" ] && echo "status: $DEP_STATUS"
    [ -n "$DEP_SOURCE" ] && echo "source: $DEP_SOURCE"
    [ -n "$DEP_REASON" ] && echo "reason: $DEP_REASON"

    if command -v jq &>/dev/null; then
        DEP_MISSING=$(jq -r '.missing // [] | join(", ")' "$FUSION_DIR/dependency_report.json" 2>/dev/null || echo "")
        DEP_NEXT=$(jq -r '.next_actions[0] // empty' "$FUSION_DIR/dependency_report.json" 2>/dev/null || echo "")
    else
        DEP_MISSING=$(grep -o '"missing"[[:space:]]*:[[:space:]]*\[[^]]*\]' "$FUSION_DIR/dependency_report.json" 2>/dev/null | sed 's/.*\[//; s/\].*//; s/"//g' || true)
        DEP_NEXT=$(grep -o '"next_actions"[[:space:]]*:[[:space:]]*\[[^]]*\]' "$FUSION_DIR/dependency_report.json" 2>/dev/null | sed 's/.*\[//; s/\].*//; s/"//g' | cut -d',' -f1 || true)
    fi

    [ -n "$DEP_MISSING" ] && echo "missing: $DEP_MISSING"
    [ -n "$DEP_NEXT" ] && echo "next: $DEP_NEXT"
fi
