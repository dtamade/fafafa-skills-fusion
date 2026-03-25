#!/bin/bash

fusion_bridge_task_plan_inspect() {
    local task_plan="$1"
    shift

    if command -v fusion_bridge_disabled >/dev/null 2>&1 && fusion_bridge_disabled; then
        return 1
    fi

    command -v resolve_fusion_bridge_bin >/dev/null 2>&1 || return 1

    local bridge_bin
    bridge_bin="$(resolve_fusion_bridge_bin "${SCRIPT_DIR:-$(pwd)}")" || return 1

    "$bridge_bin" inspect task-plan --file "$task_plan" "$@"
}

fusion_count_pattern() {
    local pattern="$1"
    local file="$2"
    local count

    count=$(grep -c "$pattern" "$file" 2>/dev/null) || true
    if [[ "$count" =~ ^[0-9]+$ ]]; then
        echo "$count"
    else
        echo "0"
    fi
}

fusion_task_counts() {
    local task_plan="$1"

    if [ ! -f "$task_plan" ]; then
        echo "0:0:0:0"
        return 0
    fi

    local bridge_counts=""
    bridge_counts=$(fusion_bridge_task_plan_inspect "$task_plan" counts 2>/dev/null) || bridge_counts=""
    if [[ "$bridge_counts" =~ ^[0-9]+:[0-9]+:[0-9]+:[0-9]+$ ]]; then
        echo "$bridge_counts"
        return 0
    fi

    local completed pending in_progress failed
    completed=$(fusion_count_pattern '\[COMPLETED\]' "$task_plan")
    pending=$(fusion_count_pattern '\[PENDING\]' "$task_plan")
    in_progress=$(fusion_count_pattern '\[IN_PROGRESS\]' "$task_plan")
    failed=$(fusion_count_pattern '\[FAILED\]' "$task_plan")
    echo "${completed}:${pending}:${in_progress}:${failed}"
}

fusion_first_task_with_status() {
    local task_plan="$1"
    local status_tag="$2"

    [ -f "$task_plan" ] || {
        echo ""
        return 0
    }

    local bridge_task=""
    bridge_task=$(fusion_bridge_task_plan_inspect "$task_plan" first --status "$status_tag" 2>/dev/null) || bridge_task=""
    if [ -n "$bridge_task" ]; then
        printf '%s\n' "$bridge_task"
        return 0
    fi

    grep -F "$status_tag" "$task_plan" 2>/dev/null         | grep '^### Task '         | head -1         | sed 's/^### Task [0-9]*: //'         | sed 's/ \[.*//' || echo ""
}

fusion_first_task_display_with_status() {
    local task_plan="$1"
    local status_tag="$2"
    local status_name="${status_tag#[}"
    status_name="${status_name%]}"

    [ -f "$task_plan" ] || {
        echo ""
        return 0
    }

    local bridge_task=""
    bridge_task=$(fusion_bridge_task_plan_inspect "$task_plan" first --status "$status_tag" 2>/dev/null) || bridge_task=""
    if [ -n "$bridge_task" ]; then
        printf '%s [%s]\n' "$bridge_task" "$status_name"
        return 0
    fi

    grep -F "$status_tag" "$task_plan" 2>/dev/null \
        | grep '^### Task ' \
        | head -1 \
        | sed 's/^### Task [0-9]*: //' || echo ""
}

fusion_last_task_with_status() {
    local task_plan="$1"
    local status_tag="$2"

    [ -f "$task_plan" ] || {
        echo ""
        return 0
    }

    local bridge_task=""
    bridge_task=$(fusion_bridge_task_plan_inspect "$task_plan" last --status "$status_tag" 2>/dev/null) || bridge_task=""
    if [ -n "$bridge_task" ]; then
        printf '%s\n' "$bridge_task"
        return 0
    fi

    grep -F "$status_tag" "$task_plan" 2>/dev/null         | grep '^### Task '         | tail -1         | sed 's/^### Task [0-9]*: //'         | sed 's/ \[.*//' || echo ""
}

fusion_current_or_next_task() {
    local task_plan="$1"
    local task

    task=$(fusion_bridge_task_plan_inspect "$task_plan" next 2>/dev/null) || task=""
    if [ -n "$task" ]; then
        echo "$task"
        return 0
    fi

    task=$(fusion_first_task_with_status "$task_plan" "[IN_PROGRESS]")
    if [ -n "$task" ]; then
        echo "$task"
        return 0
    fi

    fusion_first_task_with_status "$task_plan" "[PENDING]"
}

fusion_current_or_next_task_display_with_status() {
    local task_plan="$1"
    local task

    task=$(fusion_first_task_display_with_status "$task_plan" "[IN_PROGRESS]")
    if [ -n "$task" ]; then
        echo "$task"
        return 0
    fi

    fusion_first_task_display_with_status "$task_plan" "[PENDING]"
}

fusion_first_pending_review_task_id() {
    local task_plan="$1"
    local current_task_id=""
    local current_review_status=""
    local line=""

    [ -f "$task_plan" ] || {
        echo ""
        return 0
    }

    while IFS= read -r line || [ -n "$line" ]; do
        if [[ "$line" =~ ^###\ Task\ ([0-9]+): ]]; then
            if [ "$current_review_status" = "pending" ] && [ -n "$current_task_id" ]; then
                printf '%s\n' "$current_task_id"
                return 0
            fi
            current_task_id="task_${BASH_REMATCH[1]}"
            current_review_status=""
            continue
        fi

        if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*Review-Status:[[:space:]]*(.+)$ ]]; then
            current_review_status=$(
                printf '%s' "${BASH_REMATCH[1]}" \
                    | tr '[:upper:]' '[:lower:]' \
                    | tr -d '[:space:]'
            )
        fi
    done < "$task_plan"

    if [ "$current_review_status" = "pending" ] && [ -n "$current_task_id" ]; then
        printf '%s\n' "$current_task_id"
    else
        echo ""
    fi
}

fusion_task_title_by_id() {
    local task_plan="$1"
    local task_id="$2"
    local task_number=""

    [ -f "$task_plan" ] || {
        echo ""
        return 0
    }
    [ -n "$task_id" ] || {
        echo ""
        return 0
    }

    task_number="${task_id#task_}"
    if ! [[ "$task_number" =~ ^[0-9]+$ ]]; then
        echo ""
        return 0
    fi

    awk -v task_number="$task_number" '
        /^### Task / {
            header = $0
            current_number = header
            sub(/^### Task /, "", current_number)
            sub(/: .*$/, "", current_number)
            if (current_number == task_number) {
                sub(/^### Task [0-9]+: /, "", header)
                sub(/ \[[^][]*\]$/, "", header)
                print header
                exit
            }
        }
    ' "$task_plan" 2>/dev/null || echo ""
}

fusion_task_type_for_title() {
    local task_plan="$1"
    local task_title="$2"

    [ -f "$task_plan" ] || {
        echo ""
        return 0
    }
    [ -n "$task_title" ] || {
        echo ""
        return 0
    }

    local bridge_type=""
    bridge_type=$(fusion_bridge_task_plan_inspect "$task_plan" task-type --title "$task_title" 2>/dev/null) || bridge_type=""
    if [ -n "$bridge_type" ]; then
        printf '%s\n' "$bridge_type"
        return 0
    fi

    awk -v task_title="$task_title" '
        /^### Task / {
            header = $0
            sub(/^### Task [0-9]+: /, "", header)
            sub(/ \[[^][]*\]$/, "", header)
            in_target = (header == task_title)
            next
        }
        in_target && /Type:[[:space:]]*[a-z]+/ {
            if (match($0, /Type:[[:space:]]*[a-z]+/)) {
                type = substr($0, RSTART, RLENGTH)
                sub(/^Type:[[:space:]]*/, "", type)
                print type
            }
            exit
        }
    ' "$task_plan" 2>/dev/null || echo ""
}

fusion_execution_mode_for_task_type() {
    case "$1" in
        implementation|verification) echo "TDD" ;;
        *) echo "Direct" ;;
    esac
}

fusion_guardian_status() {
    local fusion_dir="$1"
    local loop_context="$fusion_dir/loop_context.json"
    local no_progress=0
    local same_action=0
    local value=""

    if [ -f "$loop_context" ]; then
        value=$(fusion_bridge_inspect_json_field "no_progress_rounds" "number" "$loop_context" 2>/dev/null) || value=""
        if [[ "$value" =~ ^[0-9]+$ ]]; then
            no_progress="$value"
        else
            no_progress=$(grep -o '"no_progress_rounds"[[:space:]]*:[[:space:]]*[0-9]*' "$loop_context" 2>/dev/null | grep -o '[0-9]*$') || true
        fi

        value=$(fusion_bridge_inspect_json_field "same_action_count" "number" "$loop_context" 2>/dev/null) || value=""
        if [[ "$value" =~ ^[0-9]+$ ]]; then
            same_action="$value"
        else
            same_action=$(grep -o '"same_action_count"[[:space:]]*:[[:space:]]*[0-9]*' "$loop_context" 2>/dev/null | grep -o '[0-9]*$') || true
        fi
    fi

    if [ "${no_progress:-0}" -ge 4 ] || [ "${same_action:-0}" -ge 2 ]; then
        echo "⚠ BACKOFF"
    elif [ "${no_progress:-0}" -ge 2 ]; then
        echo "~"
    else
        echo "OK"
    fi
}

fusion_phase_num() {
    case "$1" in
        UNDERSTAND) echo "0/8" ;;
        INITIALIZE) echo "1/8" ;;
        ANALYZE) echo "2/8" ;;
        DECOMPOSE) echo "3/8" ;;
        EXECUTE) echo "4/8" ;;
        VERIFY) echo "5/8" ;;
        REVIEW) echo "6/8" ;;
        COMMIT) echo "7/8" ;;
        DELIVER) echo "8/8" ;;
        *) echo "?/8" ;;
    esac
}
