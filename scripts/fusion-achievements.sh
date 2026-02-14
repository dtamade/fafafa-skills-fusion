#!/bin/bash
# fusion-achievements.sh - Show Fusion achievements and leaderboard

set -uo pipefail

FUSION_DIR=".fusion"
LEADERBOARD_ROOT="${FUSION_LEADERBOARD_ROOT:-$HOME/projects}"
TOP_N=10
SHOW_LOCAL=1
SHOW_LEADERBOARD=1

usage() {
    cat <<'USAGE'
Usage: fusion-achievements.sh [options]

Options:
  --local-only            Show achievements for current workspace only
  --leaderboard-only      Show cross-project leaderboard only
  --root <path>           Leaderboard root directory (default: $FUSION_LEADERBOARD_ROOT or $HOME/projects)
  --top <n>               Number of leaderboard rows (default: 10)
  -h, --help              Show this help
USAGE
}
fail_with_usage() {
    local message="$1"
    echo "$message" >&2
    usage >&2
    exit 1
}


json_get() {
    local file="$1" key="$2"
    if [ ! -f "$file" ]; then
        echo ""
        return 0
    fi

    if command -v jq &>/dev/null; then
        jq -r "$key // empty" "$file" 2>/dev/null || echo ""
    else
        local clean_key="${key#.}"
        grep -o "\"$clean_key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$file" 2>/dev/null | head -1 | cut -d'"' -f4 || echo ""
    fi
}

count_completed_tasks() {
    local task_file="$1"
    if [ -f "$task_file" ]; then
        local count
        count=$(grep -c '\[COMPLETED\]' "$task_file" 2>/dev/null || true)
        [ -n "$count" ] && echo "$count" || echo "0"
    else
        echo "0"
    fi
}

iter_completed_titles() {
    local task_file="$1"
    [ -f "$task_file" ] || return 0
    grep '\[COMPLETED\]' "$task_file" 2>/dev/null \
        | sed 's/^### Task [0-9]*: //' \
        | sed 's/ \[COMPLETED\].*$//' \
        | awk 'NF > 0 {print}'
}

count_safe_rounds() {
    local events_file="$1"
    if [ ! -f "$events_file" ]; then
        echo "0"
        return 0
    fi

    if command -v jq &>/dev/null; then
        jq -s -r '[.[] | select(.type == "SAFE_BACKLOG_INJECTED")] | length' "$events_file" 2>/dev/null
    else
        local count
        count=$(grep -c '"type"[[:space:]]*:[[:space:]]*"SAFE_BACKLOG_INJECTED"' "$events_file" 2>/dev/null || true)
        [ -n "$count" ] && echo "$count" || echo "0"
    fi
}

sum_safe_added() {
    local events_file="$1"
    if [ ! -f "$events_file" ]; then
        echo "0"
        return 0
    fi

    if command -v jq &>/dev/null; then
        jq -s -r '[.[] | select(.type == "SAFE_BACKLOG_INJECTED") | (.payload.added // 0)] | add // 0' "$events_file" 2>/dev/null
    else
        grep '"type"[[:space:]]*:[[:space:]]*"SAFE_BACKLOG_INJECTED"' "$events_file" 2>/dev/null \
            | grep -o '"added"[[:space:]]*:[[:space:]]*[0-9]*' \
            | grep -o '[0-9]*' \
            | awk '{s+=$1} END{print s+0}'
    fi
}

count_advisory_events() {
    local events_file="$1"
    if [ ! -f "$events_file" ]; then
        echo "0"
        return 0
    fi

    if command -v jq &>/dev/null; then
        jq -s -r '[.[] | select(.type == "SUPERVISOR_ADVISORY")] | length' "$events_file" 2>/dev/null
    else
        local count
        count=$(grep -c '"type"[[:space:]]*:[[:space:]]*"SUPERVISOR_ADVISORY"' "$events_file" 2>/dev/null || true)
        [ -n "$count" ] && echo "$count" || echo "0"
    fi
}

metrics_for_fusion_dir() {
    local dir="$1"
    local sessions_file="$dir/sessions.json"
    local task_file="$dir/task_plan.md"
    local events_file="$dir/events.jsonl"

    local status
    status=$(json_get "$sessions_file" ".status")
    local completed_workflow=0
    if [ "$status" = "completed" ]; then
        completed_workflow=1
    fi

    local completed_tasks
    completed_tasks=$(count_completed_tasks "$task_file")
    [ -n "$completed_tasks" ] || completed_tasks=0

    local safe_rounds
    safe_rounds=$(count_safe_rounds "$events_file")
    [ -n "$safe_rounds" ] || safe_rounds=0

    local safe_total
    safe_total=$(sum_safe_added "$events_file")
    [ -n "$safe_total" ] || safe_total=0

    local advice_total
    advice_total=$(count_advisory_events "$events_file")
    [ -n "$advice_total" ] || advice_total=0

    local score=$((completed_workflow * 50 + completed_tasks * 10 + safe_total * 3 + advice_total * 2))

    printf '%s|%s|%s|%s|%s|%s|%s\n' "$completed_workflow" "$completed_tasks" "$safe_rounds" "$safe_total" "$advice_total" "$score" "$status"
}

print_local_achievements() {
    echo "## Current Workspace Achievements"

    if [ ! -d "$FUSION_DIR" ]; then
        echo "- (no .fusion workspace found)"
        return 0
    fi

    local metrics
    metrics=$(metrics_for_fusion_dir "$FUSION_DIR")

    local completed_workflow completed_tasks safe_rounds safe_total advice_total score status
    IFS='|' read -r completed_workflow completed_tasks safe_rounds safe_total advice_total score status <<< "$metrics"

    local project_name
    project_name=$(basename "$PWD")

    echo "project: $project_name"
    echo "status: ${status:-unknown}"
    echo "score=$score"

    local found=0

    if [ "$completed_workflow" -eq 1 ]; then
        echo "- 🎯 Workflow completed"
        found=1
    fi

    if [ "$completed_tasks" -gt 0 ]; then
        echo "- ✅ Completed tasks: $completed_tasks"
        iter_completed_titles "$FUSION_DIR/task_plan.md" | while IFS= read -r title; do
            echo "- 🏆 $title"
        done
        found=1
    fi

    if [ "$safe_rounds" -gt 0 ]; then
        echo "- 🧩 Safe backlog unlocked: +$safe_total tasks ($safe_rounds rounds)"
        found=1
    fi

    if [ "$advice_total" -gt 0 ]; then
        echo "- 🛡️ Supervisor advisories recorded: $advice_total"
        found=1
    fi

    if [ "$found" -eq 0 ]; then
        echo "- (no achievements yet)"
    fi
}

print_leaderboard() {
    echo "## Achievement Leaderboard"

    if [ ! -d "$LEADERBOARD_ROOT" ]; then
        echo "- (root not found: $LEADERBOARD_ROOT)"
        return 0
    fi

    local rows
    rows=$(
        find "$LEADERBOARD_ROOT" -type d -name .fusion 2>/dev/null | while IFS= read -r fusion_path; do
            local metrics
            metrics=$(metrics_for_fusion_dir "$fusion_path")

            local completed_workflow completed_tasks safe_rounds safe_total advice_total score status
            IFS='|' read -r completed_workflow completed_tasks safe_rounds safe_total advice_total score status <<< "$metrics"

            if [ "$score" -le 0 ]; then
                continue
            fi

            local project_path project_name
            project_path=$(dirname "$fusion_path")
            project_name=$(basename "$project_path")

            printf '%s|%s|%s|%s|%s|%s|%s\n' "$score" "$project_name" "$completed_workflow" "$completed_tasks" "$safe_total" "$advice_total" "$project_path"
        done | sort -t'|' -k1,1nr -k2,2 | head -n "$TOP_N"
    )

    if [ -z "$rows" ]; then
        echo "- (no achievements found under $LEADERBOARD_ROOT)"
        return 0
    fi

    local rank=0
    while IFS='|' read -r score name workflows tasks safe_total advice_total project_path; do
        [ -n "$name" ] || continue
        rank=$((rank + 1))
        printf '%s) %s | score=%s | workflows=%s | tasks=%s | safe=%s | advisory=%s\n' \
            "$rank" "$name" "$score" "$workflows" "$tasks" "$safe_total" "$advice_total"
    done <<< "$rows"
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --local-only)
            SHOW_LOCAL=1
            SHOW_LEADERBOARD=0
            ;;
        --leaderboard-only)
            SHOW_LOCAL=0
            SHOW_LEADERBOARD=1
            ;;
        --root)
            shift
            if [ "$#" -eq 0 ] || [ -z "${1:-}" ] || [[ "${1:-}" == --* ]]; then
                fail_with_usage "Missing value for --root"
            fi
            LEADERBOARD_ROOT="$1"
            ;;
        --root=*)
            LEADERBOARD_ROOT="${1#--root=}"
            if [ -z "$LEADERBOARD_ROOT" ]; then
                fail_with_usage "Missing value for --root"
            fi
            ;;
        --top)
            shift
            if [ "$#" -eq 0 ] || [ -z "${1:-}" ] || [[ "${1:-}" == --* ]]; then
                fail_with_usage "Missing value for --top"
            fi
            TOP_N="$1"
            ;;
        --top=*)
            TOP_N="${1#--top=}"
            if [ -z "$TOP_N" ]; then
                fail_with_usage "Missing value for --top"
            fi
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
    shift
done

if ! [[ "$TOP_N" =~ ^[1-9][0-9]*$ ]]; then
    fail_with_usage "--top must be a positive integer"
fi

echo "=== Fusion Achievements ==="
echo ""

if [ "$SHOW_LOCAL" -eq 1 ]; then
    print_local_achievements
fi

if [ "$SHOW_LOCAL" -eq 1 ] && [ "$SHOW_LEADERBOARD" -eq 1 ]; then
    echo ""
fi

if [ "$SHOW_LEADERBOARD" -eq 1 ]; then
    print_leaderboard
fi

exit 0
