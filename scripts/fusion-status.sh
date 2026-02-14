#!/bin/bash
# fusion-status.sh - Show current fusion status
set -euo pipefail

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat <<'USAGE'
Usage: fusion-status.sh [--json]
USAGE
}

emit_json_status() {
    local result="$1"
    local reason="$2"
    local status="$3"
    local phase="$4"

    if command -v jq &>/dev/null; then
        if [ -n "$reason" ]; then
            jq -nc --arg result "$result" --arg reason "$reason" --arg status "$status" --arg phase "$phase" '{result:$result,status:$status,phase:$phase,reason:$reason}'
        else
            jq -nc --arg result "$result" --arg status "$status" --arg phase "$phase" '{result:$result,status:$status,phase:$phase}'
        fi
        return 0
    fi

    if command -v python3 &>/dev/null; then
        python3 - "$result" "$reason" "$status" "$phase" <<'PYJSON'
import json
import sys

result = sys.argv[1]
reason = sys.argv[2]
status = sys.argv[3]
phase = sys.argv[4]

payload = {
    "result": result,
    "status": status,
    "phase": phase,
}
if reason:
    payload["reason"] = reason

print(json.dumps(payload, ensure_ascii=False))
PYJSON
        return 0
    fi

    if [ -n "$reason" ]; then
        printf '{"result":"%s","status":"%s","phase":"%s","reason":"%s"}
' "$result" "$status" "$phase" "$reason"
    else
        printf '{"result":"%s","status":"%s","phase":"%s"}
' "$result" "$status" "$phase"
    fi
}

emit_json_summary() {
    local status="$1"
    local phase="$2"
    local task_completed="$3"
    local task_pending="$4"
    local task_in_progress="$5"
    local task_failed="$6"
    local dependency_status="$7"
    local dependency_missing="$8"
    local achievement_completed_tasks="$9"
    local achievement_safe_total="${10}"
    local achievement_advisory_total="${11}"
    local owner_planner="${12}"
    local owner_coder="${13}"
    local owner_reviewer="${14}"
    local current_role="${15}"
    local current_role_task="${16}"
    local current_role_status="${17}"
    local backend_status="${18}"
    local backend_primary="${19}"
    local backend_fallback="${20}"

    if command -v jq &>/dev/null; then
        jq -nc             --arg result "ok"             --arg status "$status"             --arg phase "$phase"             --arg dependency_status "$dependency_status"             --arg dependency_missing "$dependency_missing"             --arg current_role "$current_role"             --arg current_role_task "$current_role_task"             --arg current_role_status "$current_role_status"             --arg backend_status "$backend_status"             --arg backend_primary "$backend_primary"             --arg backend_fallback "$backend_fallback"             --argjson task_completed "$task_completed"             --argjson task_pending "$task_pending"             --argjson task_in_progress "$task_in_progress"             --argjson task_failed "$task_failed"             --argjson achievement_completed_tasks "$achievement_completed_tasks"             --argjson achievement_safe_total "$achievement_safe_total"             --argjson achievement_advisory_total "$achievement_advisory_total"             --argjson owner_planner "$owner_planner"             --argjson owner_coder "$owner_coder"             --argjson owner_reviewer "$owner_reviewer"             '{
                result:$result,
                status:$status,
                phase:$phase,
                task_completed:$task_completed,
                task_pending:$task_pending,
                task_in_progress:$task_in_progress,
                task_failed:$task_failed,
                dependency_status:$dependency_status,
                dependency_missing:$dependency_missing,
                backend_status:$backend_status,
                backend_primary:$backend_primary,
                backend_fallback:$backend_fallback,
                achievement_completed_tasks:$achievement_completed_tasks,
                achievement_safe_total:$achievement_safe_total,
                achievement_advisory_total:$achievement_advisory_total,
                owner_planner:$owner_planner,
                owner_coder:$owner_coder,
                owner_reviewer:$owner_reviewer,
                current_role:$current_role,
                current_role_task:$current_role_task,
                current_role_status:$current_role_status
            }'
        return 0
    fi

    if command -v python3 &>/dev/null; then
        python3 - "$status" "$phase" "$task_completed" "$task_pending" "$task_in_progress" "$task_failed" "$dependency_status" "$dependency_missing" "$achievement_completed_tasks" "$achievement_safe_total" "$achievement_advisory_total" "$owner_planner" "$owner_coder" "$owner_reviewer" "$current_role" "$current_role_task" "$current_role_status" "$backend_status" "$backend_primary" "$backend_fallback" <<'PYJSON'
import json
import sys

payload = {
    "result": "ok",
    "status": sys.argv[1],
    "phase": sys.argv[2],
    "task_completed": int(sys.argv[3]),
    "task_pending": int(sys.argv[4]),
    "task_in_progress": int(sys.argv[5]),
    "task_failed": int(sys.argv[6]),
    "dependency_status": sys.argv[7],
    "dependency_missing": sys.argv[8],
    "achievement_completed_tasks": int(sys.argv[9]),
    "achievement_safe_total": int(sys.argv[10]),
    "achievement_advisory_total": int(sys.argv[11]),
    "owner_planner": int(sys.argv[12]),
    "owner_coder": int(sys.argv[13]),
    "owner_reviewer": int(sys.argv[14]),
    "current_role": sys.argv[15],
    "current_role_task": sys.argv[16],
    "current_role_status": sys.argv[17],
    "backend_status": sys.argv[18],
    "backend_primary": sys.argv[19],
    "backend_fallback": sys.argv[20],
}
print(json.dumps(payload, ensure_ascii=False))
PYJSON
        return 0
    fi

    printf '{"result":"ok","status":"%s","phase":"%s","task_completed":%s,"task_pending":%s,"task_in_progress":%s,"task_failed":%s,"dependency_status":"%s","dependency_missing":"%s","achievement_completed_tasks":%s,"achievement_safe_total":%s,"achievement_advisory_total":%s,"owner_planner":%s,"owner_coder":%s,"owner_reviewer":%s,"current_role":"%s","current_role_task":"%s","current_role_status":"%s","backend_status":"%s","backend_primary":"%s","backend_fallback":"%s"}\n'         "$status" "$phase" "$task_completed" "$task_pending" "$task_in_progress" "$task_failed" "$dependency_status" "$dependency_missing" "$achievement_completed_tasks" "$achievement_safe_total" "$achievement_advisory_total" "$owner_planner" "$owner_coder" "$owner_reviewer" "$current_role" "$current_role_task" "$current_role_status" "$backend_status" "$backend_primary" "$backend_fallback"
}

collect_owner_metrics() {
    local task_plan_path="$1"

    OWNER_PLANNER=0
    OWNER_CODER=0
    OWNER_REVIEWER=0
    CURRENT_ROLE=""
    CURRENT_ROLE_TASK=""
    CURRENT_ROLE_STATUS=""

    [ -f "$task_plan_path" ] || return 0

    local parsed
    parsed=""

    if command -v python3 &>/dev/null; then
        parsed=$(python3 - "$task_plan_path" <<'PY'
import re
import sys

path = sys.argv[1]
header_re = re.compile(r'^###\s+Task\s+\d+:\s*(.*?)\s+\[([A-Z_]+)\]\s*$')
type_re = re.compile(r'^-\s*Type:\s*(.+?)\s*$', re.IGNORECASE)
owner_re = re.compile(r'^-\s*(?:Owner|Role):\s*(.+?)\s*$', re.IGNORECASE)

def normalize_owner(raw_owner: str) -> str:
    value = (raw_owner or "").strip().lower()
    if not value:
        return ""
    mapping = {
        "planner": "planner",
        "plan": "planner",
        "planning": "planner",
        "coder": "coder",
        "code": "coder",
        "coding": "coder",
        "developer": "coder",
        "dev": "coder",
        "implementer": "coder",
        "reviewer": "reviewer",
        "review": "reviewer",
        "qa": "reviewer",
        "verifier": "reviewer",
        "verification": "reviewer",
    }
    return mapping.get(value, "")


def infer_owner(task_type: str) -> str:
    task_type = (task_type or "").strip().lower()
    if task_type == "verification":
        return "reviewer"
    if task_type in {"design", "research"}:
        return "planner"
    return "coder"


tasks = []
current = None
with open(path, "r", encoding="utf-8") as fh:
    for raw in fh:
        line = raw.rstrip("\n")
        header_match = header_re.match(line)
        if header_match:
            if current is not None:
                tasks.append(current)
            current = {
                "title": header_match.group(1).strip(),
                "status": header_match.group(2).strip(),
                "type": "",
                "owner": "",
            }
            continue

        if current is None:
            continue

        type_match = type_re.match(line)
        if type_match:
            current["type"] = type_match.group(1).strip()
            continue

        owner_match = owner_re.match(line)
        if owner_match:
            current["owner"] = owner_match.group(1).strip()

if current is not None:
    tasks.append(current)

counts = {"planner": 0, "coder": 0, "reviewer": 0}
current_role = ""
current_task = ""
current_status = ""
pending_role = ""
pending_task = ""
pending_status = ""

for task in tasks:
    owner = normalize_owner(task.get("owner", ""))
    if not owner:
        owner = infer_owner(task.get("type", ""))
    counts[owner] += 1

    status = task.get("status", "")
    if not current_role and status == "IN_PROGRESS":
        current_role = owner
        current_task = task.get("title", "")
        current_status = status

    if not pending_role and status == "PENDING":
        pending_role = owner
        pending_task = task.get("title", "")
        pending_status = status

if not current_role:
    current_role = pending_role
    current_task = pending_task
    current_status = pending_status

print(counts["planner"])
print(counts["coder"])
print(counts["reviewer"])
print(current_role)
print(current_task)
print(current_status)
PY
)
    else
        parsed=$(awk '
function trim(v) {
    gsub(/^[[:space:]]+/, "", v)
    gsub(/[[:space:]]+$/, "", v)
    return v
}
function normalize_owner(v, n) {
    n = tolower(trim(v))
    if (n == "planner" || n == "plan" || n == "planning") return "planner"
    if (n == "coder" || n == "code" || n == "coding" || n == "developer" || n == "dev" || n == "implementer") return "coder"
    if (n == "reviewer" || n == "review" || n == "qa" || n == "verifier" || n == "verification") return "reviewer"
    return ""
}
function infer_owner(v, n) {
    n = tolower(trim(v))
    if (n == "verification") return "reviewer"
    if (n == "design" || n == "research") return "planner"
    return "coder"
}
function flush_task(    owner) {
    if (task_title == "") return

    owner = normalize_owner(task_owner)
    if (owner == "") {
        owner = infer_owner(task_type)
    }

    if (owner == "planner") planner_count++
    else if (owner == "reviewer") reviewer_count++
    else coder_count++

    if (current_role == "" && task_status == "IN_PROGRESS") {
        current_role = owner
        current_task = task_title
        current_status = task_status
    }

    if (pending_role == "" && task_status == "PENDING") {
        pending_role = owner
        pending_task = task_title
        pending_status = task_status
    }

    task_title = ""
    task_status = ""
    task_type = ""
    task_owner = ""
}
BEGIN {
    planner_count = 0
    coder_count = 0
    reviewer_count = 0
    task_title = ""
    task_status = ""
    task_type = ""
    task_owner = ""
    current_role = ""
    current_task = ""
    current_status = ""
    pending_role = ""
    pending_task = ""
    pending_status = ""
}
/^### Task [0-9]+:/ {
    flush_task()

    line = $0
    sub(/^### Task [0-9]+: /, "", line)

    task_status = line
    sub(/^.*\[/, "", task_status)
    sub(/\].*$/, "", task_status)
    task_status = trim(task_status)

    task_title = line
    sub(/ \[?[A-Z_]+\]?[[:space:]]*$/, "", task_title)
    task_title = trim(task_title)
    next
}
/^- [Tt]ype:[[:space:]]*/ {
    if (task_title != "") {
        task_type = $0
        sub(/^- [Tt]ype:[[:space:]]*/, "", task_type)
        task_type = trim(task_type)
    }
    next
}
/^- ([Oo]wner|[Rr]ole):[[:space:]]*/ {
    if (task_title != "") {
        task_owner = $0
        sub(/^- ([Oo]wner|[Rr]ole):[[:space:]]*/, "", task_owner)
        task_owner = trim(task_owner)
    }
    next
}
END {
    flush_task()

    if (current_role == "") {
        current_role = pending_role
        current_task = pending_task
        current_status = pending_status
    }

    print planner_count + 0
    print coder_count + 0
    print reviewer_count + 0
    print current_role
    print current_task
    print current_status
}
' "$task_plan_path")
    fi

    OWNER_PLANNER=$(printf '%s\n' "$parsed" | sed -n '1p')
    OWNER_CODER=$(printf '%s\n' "$parsed" | sed -n '2p')
    OWNER_REVIEWER=$(printf '%s\n' "$parsed" | sed -n '3p')
    CURRENT_ROLE=$(printf '%s\n' "$parsed" | sed -n '4p')
    CURRENT_ROLE_TASK=$(printf '%s\n' "$parsed" | sed -n '5p')
    CURRENT_ROLE_STATUS=$(printf '%s\n' "$parsed" | sed -n '6p')

    [ -n "$OWNER_PLANNER" ] || OWNER_PLANNER=0
    [ -n "$OWNER_CODER" ] || OWNER_CODER=0
    [ -n "$OWNER_REVIEWER" ] || OWNER_REVIEWER=0
}

JSON_MODE=false
UNKNOWN_OPTION=""

for arg in "$@"; do
    case "$arg" in
        -h|--help)
            usage
            exit 0
            ;;
        --json)
            JSON_MODE=true
            ;;
        *)
            UNKNOWN_OPTION="$arg"
            break
            ;;
    esac
done

if [ -n "$UNKNOWN_OPTION" ]; then
    if [ "$JSON_MODE" = true ]; then
        emit_json_status "error" "Unknown option: $UNKNOWN_OPTION" "" ""
    else
        echo "Unknown option: $UNKNOWN_OPTION" >&2
        usage >&2
    fi
    exit 1
fi

if [ ! -d "$FUSION_DIR" ]; then
    local_msg="[fusion] No .fusion directory found. Run /fusion to start."
    if [ "$JSON_MODE" = true ]; then
        emit_json_status "error" "$local_msg" "" ""
    else
        echo "$local_msg"
    fi
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

if [ "$JSON_MODE" = true ]; then
    STATUS_JSON=""
    PHASE_JSON=""
    if [ -f "$FUSION_DIR/sessions.json" ]; then
        STATUS_JSON=$(json_get "$FUSION_DIR/sessions.json" ".status")
        PHASE_JSON=$(json_get "$FUSION_DIR/sessions.json" ".current_phase")
    fi

    TASK_COMPLETED=0
    TASK_PENDING=0
    TASK_IN_PROGRESS=0
    TASK_FAILED=0
    if [ -f "$FUSION_DIR/task_plan.md" ]; then
        TASK_COMPLETED=$(grep -c '\[COMPLETED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || TASK_COMPLETED=0
        TASK_PENDING=$(grep -c '\[PENDING\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || TASK_PENDING=0
        TASK_IN_PROGRESS=$(grep -c '\[IN_PROGRESS\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || TASK_IN_PROGRESS=0
        TASK_FAILED=$(grep -c '\[FAILED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || TASK_FAILED=0
    fi

    collect_owner_metrics "$FUSION_DIR/task_plan.md"

    DEP_STATUS=""
    DEP_MISSING=""
    if [ -f "$FUSION_DIR/dependency_report.json" ]; then
        DEP_STATUS=$(json_get "$FUSION_DIR/dependency_report.json" ".status")
        if command -v jq &>/dev/null; then
            DEP_MISSING=$(jq -r '.missing // [] | join(", ")' "$FUSION_DIR/dependency_report.json" 2>/dev/null || echo "")
        else
            DEP_MISSING=$(grep -o '"missing"[[:space:]]*:[[:space:]]*\[[^]]*\]' "$FUSION_DIR/dependency_report.json" 2>/dev/null | sed 's/.*\[//; s/\].*//; s/"//g' || true)
        fi
    fi

    BACKEND_STATUS=""
    BACKEND_PRIMARY=""
    BACKEND_FALLBACK=""
    if [ -f "$FUSION_DIR/backend_failure_report.json" ]; then
        BACKEND_STATUS=$(json_get "$FUSION_DIR/backend_failure_report.json" ".status")
        BACKEND_PRIMARY=$(json_get "$FUSION_DIR/backend_failure_report.json" ".primary_backend")
        BACKEND_FALLBACK=$(json_get "$FUSION_DIR/backend_failure_report.json" ".fallback_backend")
    fi

    ACH_COMPLETED_TASKS="$TASK_COMPLETED"
    ACH_SAFE_TOTAL=0
    ACH_ADVISORY_TOTAL=0
    if [ -f "$FUSION_DIR/events.jsonl" ]; then
        if command -v jq &>/dev/null; then
            ACH_SAFE_TOTAL=$(jq -s -r '[.[] | select(.type == "SAFE_BACKLOG_INJECTED") | (.payload.added // 0)] | add // 0' "$FUSION_DIR/events.jsonl" 2>/dev/null)
            ACH_ADVISORY_TOTAL=$(jq -s -r '[.[] | select(.type == "SUPERVISOR_ADVISORY")] | length' "$FUSION_DIR/events.jsonl" 2>/dev/null)
        else
            ACH_SAFE_TOTAL=$(grep '"type"[[:space:]]*:[[:space:]]*"SAFE_BACKLOG_INJECTED"' "$FUSION_DIR/events.jsonl" 2>/dev/null | grep -o '"added"[[:space:]]*:[[:space:]]*[0-9]*' | grep -o '[0-9]*' | awk '{s+=$1} END{print s+0}')
            ACH_ADVISORY_TOTAL=$(grep -c '"type"[[:space:]]*:[[:space:]]*"SUPERVISOR_ADVISORY"' "$FUSION_DIR/events.jsonl" 2>/dev/null) || ACH_ADVISORY_TOTAL=0
        fi
    fi

    [ -n "$ACH_SAFE_TOTAL" ] || ACH_SAFE_TOTAL=0
    [ -n "$ACH_ADVISORY_TOTAL" ] || ACH_ADVISORY_TOTAL=0

    emit_json_summary         "$STATUS_JSON" "$PHASE_JSON"         "$TASK_COMPLETED" "$TASK_PENDING" "$TASK_IN_PROGRESS" "$TASK_FAILED"         "$DEP_STATUS" "$DEP_MISSING"         "$ACH_COMPLETED_TASKS" "$ACH_SAFE_TOTAL" "$ACH_ADVISORY_TOTAL"         "$OWNER_PLANNER" "$OWNER_CODER" "$OWNER_REVIEWER"         "$CURRENT_ROLE" "$CURRENT_ROLE_TASK" "$CURRENT_ROLE_STATUS"         "$BACKEND_STATUS" "$BACKEND_PRIMARY" "$BACKEND_FALLBACK"
    exit 0
fi

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


print_achievement_leaderboard_top3() {
    local achievements_script="$SCRIPT_DIR/fusion-achievements.sh"
    [ -f "$achievements_script" ] || return 0

    local leaderboard_root="${FUSION_LEADERBOARD_ROOT:-$PWD}"
    [ -d "$leaderboard_root" ] || return 0

    local leaderboard_output

    if command -v timeout &>/dev/null; then
        if ! leaderboard_output=$(timeout 2 bash "$achievements_script" --leaderboard-only --root "$leaderboard_root" --top 3 2>/dev/null); then
            return 0
        fi
    else
        if ! leaderboard_output=$(bash "$achievements_script" --leaderboard-only --root "$leaderboard_root" --top 3 2>/dev/null); then
            return 0
        fi
    fi

    local rows
    rows=$(printf '%s\n' "$leaderboard_output" | awk 'BEGIN {start=0} /^## Achievement Leaderboard$/ {start=1; next} start==1 {print}')
    rows=$(printf '%s\n' "$rows" | sed '/^[[:space:]]*$/d')

    if [ -z "$rows" ]; then
        return 0
    fi

    echo ""
    echo "## Achievement Leaderboard (Top 3)"
    printf '%s\n' "$rows"
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
    grep "^|" "$FUSION_DIR/progress.md" | tail -12 || true
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

    echo ""
    echo "## Hook Debug"
    HOOK_DEBUG_ENABLED=false
    HOOK_DEBUG_ENV=$(printf '%s' "${FUSION_HOOK_DEBUG:-}" | tr '[:upper:]' '[:lower:]')
    case "$HOOK_DEBUG_ENV" in
        1|true|yes|on)
            HOOK_DEBUG_ENABLED=true
            ;;
    esac
    if [ -f "$FUSION_DIR/.hook_debug" ]; then
        HOOK_DEBUG_ENABLED=true
    fi

    echo "hook_debug.enabled: $HOOK_DEBUG_ENABLED"
    if [ -f "$FUSION_DIR/.hook_debug" ]; then
        echo "hook_debug.flag: $FUSION_DIR/.hook_debug"
    fi
    if [ -f "$FUSION_DIR/hook-debug.log" ]; then
        echo "hook_debug.log: $FUSION_DIR/hook-debug.log"
        echo "hook_debug.tail:"
        tail -n 5 "$FUSION_DIR/hook-debug.log" 2>/dev/null | sed 's/^/  /'
    else
        echo "hook_debug.log: (none yet)"
    fi
fi

if [ -f "$FUSION_DIR/task_plan.md" ]; then
    collect_owner_metrics "$FUSION_DIR/task_plan.md"

    echo ""
    echo "## Team Roles"
    echo "owner.planner: $OWNER_PLANNER"
    echo "owner.coder: $OWNER_CODER"
    echo "owner.reviewer: $OWNER_REVIEWER"

    if [ -n "$CURRENT_ROLE" ]; then
        echo "current_role: $CURRENT_ROLE"
    fi

    if [ -n "$CURRENT_ROLE_TASK" ]; then
        if [ -n "$CURRENT_ROLE_STATUS" ]; then
            echo "current_role_task: $CURRENT_ROLE_TASK [$CURRENT_ROLE_STATUS]"
        else
            echo "current_role_task: $CURRENT_ROLE_TASK"
        fi
    fi
fi

# Achievement summary
if [ -d "$FUSION_DIR" ]; then
    echo ""
    echo "## Achievements"

    ACH_FOUND=0

    if [ -f "$FUSION_DIR/sessions.json" ]; then
        ACH_STATUS=$(json_get "$FUSION_DIR/sessions.json" ".status")
        if [ "$ACH_STATUS" = "completed" ]; then
            echo "- 🎯 Workflow completed"
            ACH_FOUND=1
        fi
    fi

    if [ -f "$FUSION_DIR/task_plan.md" ]; then
        ACH_COMPLETED_COUNT=$(grep -c '\[COMPLETED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || ACH_COMPLETED_COUNT=0
        if [ "$ACH_COMPLETED_COUNT" -gt 0 ]; then
            echo "- ✅ Completed tasks: $ACH_COMPLETED_COUNT"
            grep '\[COMPLETED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null                 | sed 's/^### Task [0-9]*: //'                 | sed 's/ \[COMPLETED\].*$//'                 | while IFS= read -r task_title; do
                    [ -n "$task_title" ] && echo "- 🏆 $task_title"
                done
            ACH_FOUND=1
        fi
    fi

    if [ -f "$FUSION_DIR/events.jsonl" ]; then
        SAFE_TIMES=0
        SAFE_TOTAL=0
        ADVICE_TIMES=0

        if command -v jq &>/dev/null; then
            SAFE_TIMES=$(jq -s -r '[.[] | select(.type == "SAFE_BACKLOG_INJECTED")] | length' "$FUSION_DIR/events.jsonl" 2>/dev/null)
            SAFE_TOTAL=$(jq -s -r '[.[] | select(.type == "SAFE_BACKLOG_INJECTED") | (.payload.added // 0)] | add // 0' "$FUSION_DIR/events.jsonl" 2>/dev/null)
            ADVICE_TIMES=$(jq -s -r '[.[] | select(.type == "SUPERVISOR_ADVISORY")] | length' "$FUSION_DIR/events.jsonl" 2>/dev/null)
        else
            SAFE_TIMES=$(grep -c '"type"[[:space:]]*:[[:space:]]*"SAFE_BACKLOG_INJECTED"' "$FUSION_DIR/events.jsonl" 2>/dev/null) || SAFE_TIMES=0
            SAFE_TOTAL=$(grep '"type"[[:space:]]*:[[:space:]]*"SAFE_BACKLOG_INJECTED"' "$FUSION_DIR/events.jsonl" 2>/dev/null \
                | grep -o '"added"[[:space:]]*:[[:space:]]*[0-9]*' \
                | grep -o '[0-9]*' \
                | awk '{s+=$1} END{print s+0}')
            ADVICE_TIMES=$(grep -c '"type"[[:space:]]*:[[:space:]]*"SUPERVISOR_ADVISORY"' "$FUSION_DIR/events.jsonl" 2>/dev/null) || ADVICE_TIMES=0
        fi

        [ -n "$SAFE_TIMES" ] || SAFE_TIMES=0
        [ -n "$SAFE_TOTAL" ] || SAFE_TOTAL=0
        [ -n "$ADVICE_TIMES" ] || ADVICE_TIMES=0

        if [ "$SAFE_TIMES" -gt 0 ]; then
            echo "- 🧩 Safe backlog unlocked: +$SAFE_TOTAL tasks ($SAFE_TIMES rounds)"
            ACH_FOUND=1
        fi

        if [ "$ADVICE_TIMES" -gt 0 ]; then
            echo "- 🛡️ Supervisor advisories recorded: $ADVICE_TIMES"
            ACH_FOUND=1
        fi
    fi

    if [ "$ACH_FOUND" -eq 0 ]; then
        echo "- (no achievements yet)"
    fi
fi

SHOW_LEADERBOARD_RAW="${FUSION_STATUS_SHOW_LEADERBOARD:-1}"
SHOW_LEADERBOARD_NORMALIZED=$(printf '%s' "$SHOW_LEADERBOARD_RAW" | tr '[:upper:]' '[:lower:]')
case "$SHOW_LEADERBOARD_NORMALIZED" in
    0|false|no|off)
        ;;
    *)
        print_achievement_leaderboard_top3
        ;;
esac

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

    if [ -n "$DEP_MISSING" ]; then
        echo "missing: $DEP_MISSING"
    fi
    if [ -n "$DEP_NEXT" ]; then
        echo "next: $DEP_NEXT"
    fi
fi

# Show unresolved backend failure report (if present)
if [ -f "$FUSION_DIR/backend_failure_report.json" ]; then
    echo ""
    echo "## Backend Failure Report"

    BACKEND_STATUS=$(json_get "$FUSION_DIR/backend_failure_report.json" ".status")
    BACKEND_SOURCE=$(json_get "$FUSION_DIR/backend_failure_report.json" ".source")
    BACKEND_PRIMARY=$(json_get "$FUSION_DIR/backend_failure_report.json" ".primary_backend")
    BACKEND_FALLBACK=$(json_get "$FUSION_DIR/backend_failure_report.json" ".fallback_backend")
    BACKEND_PRIMARY_ERROR=$(json_get "$FUSION_DIR/backend_failure_report.json" ".primary_error")
    BACKEND_FALLBACK_ERROR=$(json_get "$FUSION_DIR/backend_failure_report.json" ".fallback_error")

    if [ -n "$BACKEND_STATUS" ]; then
        echo "status: $BACKEND_STATUS"
    fi
    if [ -n "$BACKEND_SOURCE" ]; then
        echo "source: $BACKEND_SOURCE"
    fi
    if [ -n "$BACKEND_PRIMARY" ]; then
        echo "primary_backend: $BACKEND_PRIMARY"
    fi
    if [ -n "$BACKEND_FALLBACK" ]; then
        echo "fallback_backend: $BACKEND_FALLBACK"
    fi
    if [ -n "$BACKEND_PRIMARY_ERROR" ]; then
        echo "primary_error: $BACKEND_PRIMARY_ERROR"
    fi
    if [ -n "$BACKEND_FALLBACK_ERROR" ]; then
        echo "fallback_error: $BACKEND_FALLBACK_ERROR"
    fi

    if command -v jq &>/dev/null; then
        BACKEND_NEXT=$(jq -r '.next_actions[0] // empty' "$FUSION_DIR/backend_failure_report.json" 2>/dev/null || echo "")
    else
        BACKEND_NEXT=$(grep -o '"next_actions"[[:space:]]*:[[:space:]]*\[[^]]*\]' "$FUSION_DIR/backend_failure_report.json" 2>/dev/null | sed 's/.*\[//; s/\].*//; s/"//g' | cut -d',' -f1 || true)
    fi
    if [ -n "${BACKEND_NEXT:-}" ]; then
        echo "next: $BACKEND_NEXT"
    fi
fi
