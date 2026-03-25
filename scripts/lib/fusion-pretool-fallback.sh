#!/bin/bash

fusion_pretool_pretty_json_scalar() {
    local file="$1"
    local mode="$2"
    local target="$3"

    [ -f "$file" ] || {
        echo ""
        return 0
    }

    awk -v target="$target" -v mode="$mode" '
    function clear_deeper(indent,    key) {
        for (key in parts) {
            if ((key + 0) >= indent) {
                delete parts[key]
            }
        }
    }
    function extract_key(text,    key) {
        key = text
        sub(/^[[:space:]]*"/, "", key)
        sub(/".*$/, "", key)
        return key
    }
    function build_path(indent, key,    level, path) {
        path = ""
        for (level = 0; level < indent; level += 2) {
            if (parts[level] != "") {
                path = (path == "" ? parts[level] : path "." parts[level])
            }
        }
        return (path == "" ? key : path "." key)
    }
    {
        line = $0
        match(line, /^ */)
        indent = RLENGTH

        if (line ~ /^[[:space:]]*[}\]][[:space:]]*,?[[:space:]]*$/) {
            clear_deeper(indent)
            next
        }

        if (line ~ /^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*\{[[:space:]]*,?[[:space:]]*$/) {
            clear_deeper(indent)
            parts[indent] = extract_key(line)
            next
        }

        if (line ~ /^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*\[[[:space:]]*$/) {
            clear_deeper(indent + 2)
            next
        }

        if (mode == "string" && line ~ /^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*".*"[[:space:]]*,?[[:space:]]*$/) {
            clear_deeper(indent + 2)
            key = extract_key(line)
            value = line
            sub(/^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*"/, "", value)
            sub(/"[[:space:]]*,?[[:space:]]*$/, "", value)
            if (build_path(indent, key) == target) {
                print value
                exit
            }
            next
        }

        if (mode == "number" && line ~ /^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*-?[0-9]+[[:space:]]*,?[[:space:]]*$/) {
            clear_deeper(indent + 2)
            key = extract_key(line)
            value = line
            sub(/^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*/, "", value)
            sub(/[[:space:]]*,?[[:space:]]*$/, "", value)
            if (build_path(indent, key) == target) {
                print value
                exit
            }
            next
        }

        if (mode == "bool" && line ~ /^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*(true|false)[[:space:]]*,?[[:space:]]*$/) {
            clear_deeper(indent + 2)
            key = extract_key(line)
            value = line
            sub(/^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*/, "", value)
            sub(/[[:space:]]*,?[[:space:]]*$/, "", value)
            if (build_path(indent, key) == target) {
                print value
                exit
            }
        }
    }
    ' "$file" 2>/dev/null
}

fusion_pretool_pretty_json_string_array_csv() {
    local file="$1"
    local target="$2"

    [ -f "$file" ] || {
        echo ""
        return 0
    }

    awk -v target="$target" '
    function clear_deeper(indent,    key) {
        for (key in parts) {
            if ((key + 0) >= indent) {
                delete parts[key]
            }
        }
    }
    function extract_key(text,    key) {
        key = text
        sub(/^[[:space:]]*"/, "", key)
        sub(/".*$/, "", key)
        return key
    }
    function extract_item(text,    item) {
        item = text
        sub(/^[[:space:]]*"/, "", item)
        sub(/"[[:space:]]*,?[[:space:]]*$/, "", item)
        return item
    }
    function build_path(indent, key,    level, path) {
        path = ""
        for (level = 0; level < indent; level += 2) {
            if (parts[level] != "") {
                path = (path == "" ? parts[level] : path "." parts[level])
            }
        }
        return (path == "" ? key : path "." key)
    }
    {
        line = $0
        match(line, /^ */)
        indent = RLENGTH

        if (in_target) {
            if (line ~ /^[[:space:]]*\][[:space:]]*,?[[:space:]]*$/) {
                exit
            }
            if (line ~ /^[[:space:]]*".*"[[:space:]]*,?[[:space:]]*$/) {
                items[++count] = extract_item(line)
            }
            next
        }

        if (line ~ /^[[:space:]]*[}\]][[:space:]]*,?[[:space:]]*$/) {
            clear_deeper(indent)
            next
        }

        if (line ~ /^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*\{[[:space:]]*,?[[:space:]]*$/) {
            clear_deeper(indent)
            parts[indent] = extract_key(line)
            next
        }

        if (line ~ /^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*\[[[:space:]]*$/) {
            clear_deeper(indent + 2)
            if (build_path(indent, extract_key(line)) == target) {
                in_target = 1
            }
        }
    }
    END {
        if (count > 0) {
            printf "%s", items[1]
            for (i = 2; i <= count; i++) {
                printf ", %s", items[i]
            }
            printf "\n"
        }
    }
    ' "$file" 2>/dev/null
}

fusion_pretool_print_scheduler_summary() {
    local sessions_file="$1"
    local enabled=""
    local batch_id=0
    local parallel=0

    enabled=$(fusion_pretool_pretty_json_scalar "$sessions_file" "bool" "_runtime.scheduler.enabled")
    [ "$enabled" = "true" ] || return 0

    batch_id=$(fusion_pretool_pretty_json_scalar "$sessions_file" "number" "_runtime.scheduler.current_batch_id")
    parallel=$(fusion_pretool_pretty_json_scalar "$sessions_file" "number" "_runtime.scheduler.parallel_tasks")
    [[ "$batch_id" =~ ^-?[0-9]+$ ]] || batch_id=0
    [[ "$parallel" =~ ^-?[0-9]+$ ]] || parallel=0

    if [ "$batch_id" -gt 0 ] || [ "$parallel" -gt 0 ]; then
        echo "[fusion] Batch: ${batch_id} | Parallel: ${parallel} tasks"
    fi
}

fusion_pretool_print_agent_summary() {
    local sessions_file="$1"
    local enabled=""
    local batch_id=0
    local review_queue=0
    local roles=""
    local tasks=""
    local turn_role=""
    local turn_task_id=""
    local turn_kind=""
    local pending_reviews=""

    enabled=$(fusion_pretool_pretty_json_scalar "$sessions_file" "bool" "_runtime.agents.enabled")
    [ "$enabled" = "true" ] || return 0

    batch_id=$(fusion_pretool_pretty_json_scalar "$sessions_file" "number" "_runtime.agents.current_batch_id")
    review_queue=$(fusion_pretool_pretty_json_scalar "$sessions_file" "number" "_runtime.agents.review_queue_size")
    roles=$(fusion_pretool_pretty_json_string_array_csv "$sessions_file" "_runtime.agents.active_roles")
    tasks=$(fusion_pretool_pretty_json_string_array_csv "$sessions_file" "_runtime.agents.current_batch_tasks")
    turn_role=$(fusion_pretool_pretty_json_scalar "$sessions_file" "string" "_runtime.agents.collaboration.turn_role")
    turn_task_id=$(fusion_pretool_pretty_json_scalar "$sessions_file" "string" "_runtime.agents.collaboration.turn_task_id")
    turn_kind=$(fusion_pretool_pretty_json_scalar "$sessions_file" "string" "_runtime.agents.collaboration.turn_kind")
    pending_reviews=$(fusion_pretool_pretty_json_string_array_csv "$sessions_file" "_runtime.agents.collaboration.pending_reviews")

    [[ "$batch_id" =~ ^-?[0-9]+$ ]] || batch_id=0
    [[ "$review_queue" =~ ^-?[0-9]+$ ]] || review_queue=0
    turn_task_id="${turn_task_id:-unknown}"
    turn_kind="${turn_kind:-task}"

    if [ "$batch_id" -gt 0 ] || [ -n "$roles" ] || [ "$review_queue" -gt 0 ]; then
        echo "[fusion] Agent batch: ${batch_id} | Roles: ${roles} | Review queue: ${review_queue}"
    fi
    if [ -n "$tasks" ]; then
        echo "[fusion] Agent tasks: ${tasks}"
    fi
    if [ -n "$turn_role" ]; then
        echo "[fusion] Agent turn: ${turn_role} -> ${turn_task_id} (${turn_kind})"
    fi
    if [ -n "$pending_reviews" ]; then
        echo "[fusion] Pending reviews: ${pending_reviews}"
    fi
}

fusion_pretool_progress_bar() {
    local filled="$1"
    local total_slots=10
    local bar=""
    local i=0

    while [ "$i" -lt "$filled" ]; do
        bar="${bar}█"
        i=$((i + 1))
    done

    while [ "$i" -lt "$total_slots" ]; do
        bar="${bar}░"
        i=$((i + 1))
    done

    printf '%s\n' "$bar"
}

fusion_pretool_guidance_for_task() {
    local phase="$1"
    local task_name="$2"
    local task_type="$3"
    local normalized_type=""

    normalized_type=$(printf '%s' "$task_type" | tr '[:upper:]' '[:lower:]')

    case "$normalized_type" in
        implementation|verification)
            echo "TDD flow: RED → GREEN → REFACTOR"
            ;;
        design|documentation|configuration|research)
            echo "Direct execution"
            ;;
        *)
            if [ "$phase" = "EXECUTE" ] && [ -n "$task_name" ]; then
                echo "Check task type in task_plan.md"
            fi
            ;;
    esac
}

fusion_pretool_review_gate_guidance() {
    local task_id=""

    task_id=$(printf '%s' "$1" | tr -d '"\\\t\n\r')
    [ -n "$task_id" ] || task_id="current task"

    printf 'Review gate: reviewer approve %s before execution continues\n' "$task_id"
}

fusion_pretool_shell_fallback() {
    local fusion_dir="$1"
    local sessions_file="$fusion_dir/sessions.json"
    local task_plan="$fusion_dir/task_plan.md"

    local status
    status=$(json_get_field "$sessions_file" "status")
    if [ "$status" != "in_progress" ]; then
        hook_debug_log "skip: status=$status"
        return 0
    fi

    local goal phase phase_num
    goal=$(json_get_field "$sessions_file" "goal")
    goal=$(printf '%.60s' "$goal" | tr -d '"\\\t\n\r')
    phase=$(json_get_field "$sessions_file" "current_phase")
    phase="${phase:-EXECUTE}"
    phase_num=$(fusion_phase_num "$phase")

    local completed=0 pending=0 in_progress=0 failed=0 total=0
    local current_task="" current_task_type="" guidance="" pending_review_task_id=""

    if [ -f "$task_plan" ]; then
        IFS=':' read -r completed pending in_progress failed <<< "$(fusion_task_counts "$task_plan")"
        total=$((completed + pending + in_progress + failed))
        current_task=$(fusion_current_or_next_task "$task_plan")
        pending_review_task_id=$(fusion_first_pending_review_task_id "$task_plan")
        if [ -n "$current_task" ]; then
            current_task_type=$(fusion_task_type_for_title "$task_plan" "$current_task")
        fi
    fi

    if [ -n "$pending_review_task_id" ]; then
        guidance=$(fusion_pretool_review_gate_guidance "$pending_review_task_id")
    else
        guidance=$(fusion_pretool_guidance_for_task "$phase" "$current_task" "$current_task_type")
    fi

    hook_debug_log "active: phase=$phase completed=$completed pending=$pending in_progress=$in_progress failed=$failed task=${current_task:-none}"

    echo "[fusion] Goal: ${goal:-?} | Phase: $phase ($phase_num)"

    if [ "$total" -gt 0 ] && [ -n "$current_task" ]; then
        local task_status="PENDING"
        [ "$in_progress" -gt 0 ] && task_status="IN_PROGRESS"

        local task_index percent filled bar guardian_status type_display=""
        task_index=$((completed + 1))
        percent=$((completed * 100 / total))
        filled=$((completed * 10 / total))
        bar=$(fusion_pretool_progress_bar "$filled")
        guardian_status=$(fusion_guardian_status "$fusion_dir")
        [ -n "$current_task_type" ] && type_display=" (type: $current_task_type)"

        echo "[fusion] Task ${task_index}/${total}: ${current_task} [${task_status}]${type_display}"
        echo "[fusion] Progress: ${bar} ${percent}% | Guardian: ${guardian_status}"
    fi

    if [ -n "$guidance" ]; then
        echo "[fusion] → $guidance"
    fi

    fusion_pretool_print_scheduler_summary "$sessions_file"
    fusion_pretool_print_agent_summary "$sessions_file"

    hook_debug_log "done: emitted-summary"
    return 0
}
