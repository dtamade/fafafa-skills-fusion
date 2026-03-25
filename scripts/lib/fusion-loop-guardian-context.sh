#!/bin/bash

guardian_context_defaults() {
    local now_ms="${1:-$(get_timestamp_ms)}"
    CTX_ITERATION=0
    CTX_LAST_TASK_SNAPSHOT=""
    CTX_LAST_COMPLETED_COUNT=0
    CTX_LAST_ACTION_SIGNATURE=""
    CTX_LAST_ERROR_FINGERPRINT=""
    CTX_COMPLETED_COUNT_HISTORY=""
    CTX_ACTION_SIGNATURES=""
    CTX_ERROR_FINGERPRINTS=""
    CTX_STATE_VISITS=""
    CTX_STARTED_AT="$now_ms"
    CTX_LAST_PROGRESS_AT="$now_ms"
    CTX_TOTAL_ITERATIONS=0
    CTX_NO_PROGRESS_ROUNDS=0
    CTX_SAME_ACTION_COUNT=0
    CTX_SAME_ERROR_COUNT=0
    CTX_WALL_TIME_MS=0
    CTX_MAX_STATE_VISIT_COUNT=0
    CTX_DECISION_HISTORY=""
}

guardian_load_context() {
    local now_ms=""
    local value=""
    local history_last=""

    now_ms=$(get_timestamp_ms)
    guardian_context_defaults "$now_ms"
    [ -f "$LOOP_CONTEXT_FILE" ] || return 0

    value=$(guardian_extract_number_from_file "started_at")
    [ -n "$value" ] && CTX_STARTED_AT="$value"

    value=$(guardian_extract_number_from_file "iteration")
    [ -n "$value" ] && CTX_ITERATION="$value"

    CTX_LAST_TASK_SNAPSHOT=$(guardian_extract_string_from_file "last_task_snapshot")
    CTX_COMPLETED_COUNT_HISTORY=$(guardian_extract_array_values "completed_count_history")
    CTX_ACTION_SIGNATURES=$(guardian_extract_array_values "action_signatures")
    CTX_ERROR_FINGERPRINTS=$(guardian_extract_array_values "error_fingerprints")
    CTX_STATE_VISITS=$(guardian_extract_state_visits)
    CTX_DECISION_HISTORY=$(guardian_extract_decision_history)

    value=$(guardian_extract_number_from_file "last_progress_at")
    [ -n "$value" ] && CTX_LAST_PROGRESS_AT="$value"

    value=$(guardian_extract_number_from_file "total_iterations")
    if [ -n "$value" ]; then
        CTX_TOTAL_ITERATIONS="$value"
    else
        CTX_TOTAL_ITERATIONS="$CTX_ITERATION"
    fi

    value=$(guardian_extract_number_from_file "no_progress_rounds")
    [ -n "$value" ] && CTX_NO_PROGRESS_ROUNDS="$value"

    value=$(guardian_extract_number_from_file "same_action_count")
    [ -n "$value" ] && CTX_SAME_ACTION_COUNT="$value"

    value=$(guardian_extract_number_from_file "same_error_count")
    [ -n "$value" ] && CTX_SAME_ERROR_COUNT="$value"

    value=$(guardian_extract_number_from_file "wall_time_ms")
    [ -n "$value" ] && CTX_WALL_TIME_MS="$value"

    value=$(guardian_extract_number_from_file "max_state_visit_count")
    [ -n "$value" ] && CTX_MAX_STATE_VISIT_COUNT="$value"

    value=$(guardian_extract_number_from_file "last_completed_count")
    if [ -n "$value" ]; then
        CTX_LAST_COMPLETED_COUNT="$value"
    else
        history_last=$(guardian_last_list_item "$CTX_COMPLETED_COUNT_HISTORY")
        if [ -n "$history_last" ]; then
            CTX_LAST_COMPLETED_COUNT=$(guardian_numeric_or_default "$history_last" 0)
        elif [ -n "$CTX_LAST_TASK_SNAPSHOT" ]; then
            CTX_LAST_COMPLETED_COUNT=$(guardian_numeric_or_default "${CTX_LAST_TASK_SNAPSHOT%%:*}" 0)
        fi
    fi

    value=$(guardian_extract_string_from_file "last_action_signature")
    if [ -n "$value" ]; then
        CTX_LAST_ACTION_SIGNATURE="$value"
    else
        CTX_LAST_ACTION_SIGNATURE=$(guardian_last_list_item "$CTX_ACTION_SIGNATURES")
    fi

    value=$(guardian_extract_string_from_file "last_error_fingerprint")
    if [ -n "$value" ]; then
        CTX_LAST_ERROR_FINGERPRINT="$value"
    else
        CTX_LAST_ERROR_FINGERPRINT=$(guardian_last_list_item "$CTX_ERROR_FINGERPRINTS")
    fi
}

guardian_write_context() {
    guardian_backend_available || return 1
    mkdir -p "$FUSION_DIR"

    local tmp_file
    tmp_file=$(mktemp "${FUSION_DIR}/.tmp.XXXXXX") || return 1

    {
        echo "{"
        printf '  "iteration": %s,\n' "$(guardian_numeric_or_default "$CTX_ITERATION" 0)"
        printf '  "last_task_snapshot": %s,\n' "$(guardian_json_string_or_null "$CTX_LAST_TASK_SNAPSHOT")"
        printf '  "last_completed_count": %s,\n' "$(guardian_numeric_or_default "$CTX_LAST_COMPLETED_COUNT" 0)"
        printf '  "last_action_signature": %s,\n' "$(guardian_json_string_or_null "$CTX_LAST_ACTION_SIGNATURE")"
        printf '  "last_error_fingerprint": %s,\n' "$(guardian_json_string_or_null "$CTX_LAST_ERROR_FINGERPRINT")"
        guardian_write_number_array '  ' 'completed_count_history' "$CTX_COMPLETED_COUNT_HISTORY" ','
        guardian_write_string_array '  ' 'action_signatures' "$CTX_ACTION_SIGNATURES" ','
        guardian_write_string_array '  ' 'error_fingerprints' "$CTX_ERROR_FINGERPRINTS" ','
        guardian_write_state_visits '  ' 'state_visits' "$CTX_STATE_VISITS" ','
        printf '  "started_at": %s,\n' "$(guardian_numeric_or_default "$CTX_STARTED_AT" 0)"
        printf '  "last_progress_at": %s,\n' "$(guardian_numeric_or_default "$CTX_LAST_PROGRESS_AT" 0)"
        printf '  "total_iterations": %s,\n' "$(guardian_numeric_or_default "$CTX_TOTAL_ITERATIONS" 0)"
        printf '  "no_progress_rounds": %s,\n' "$(guardian_numeric_or_default "$CTX_NO_PROGRESS_ROUNDS" 0)"
        printf '  "same_action_count": %s,\n' "$(guardian_numeric_or_default "$CTX_SAME_ACTION_COUNT" 0)"
        printf '  "same_error_count": %s,\n' "$(guardian_numeric_or_default "$CTX_SAME_ERROR_COUNT" 0)"
        printf '  "wall_time_ms": %s,\n' "$(guardian_numeric_or_default "$CTX_WALL_TIME_MS" 0)"
        printf '  "max_state_visit_count": %s,\n' "$(guardian_numeric_or_default "$CTX_MAX_STATE_VISIT_COUNT" 0)"
        echo '  "metrics": {'
        printf '    "total_iterations": %s,\n' "$(guardian_numeric_or_default "$CTX_TOTAL_ITERATIONS" 0)"
        printf '    "no_progress_rounds": %s,\n' "$(guardian_numeric_or_default "$CTX_NO_PROGRESS_ROUNDS" 0)"
        printf '    "same_action_count": %s,\n' "$(guardian_numeric_or_default "$CTX_SAME_ACTION_COUNT" 0)"
        printf '    "same_error_count": %s,\n' "$(guardian_numeric_or_default "$CTX_SAME_ERROR_COUNT" 0)"
        printf '    "wall_time_ms": %s,\n' "$(guardian_numeric_or_default "$CTX_WALL_TIME_MS" 0)"
        printf '    "max_state_visit_count": %s\n' "$(guardian_numeric_or_default "$CTX_MAX_STATE_VISIT_COUNT" 0)"
        echo '  },'
        guardian_write_decision_history '  ' 'decision_history' "$CTX_DECISION_HISTORY"
        echo "}"
    } > "$tmp_file"

    mv "$tmp_file" "$LOOP_CONTEXT_FILE"
}

guardian_append_decision_history() {
    local decision="$1"
    local reason="$2"
    local timestamp="${3:-$(date +%s)}"
    local entry
    entry=$(printf '{"decision": "%s", "reason": "%s", "timestamp": %s}' \
        "$(guardian_json_escape "$decision")" \
        "$(guardian_json_escape "$reason")" \
        "$(guardian_numeric_or_default "$timestamp" 0)")
    CTX_DECISION_HISTORY=$(guardian_append_list_with_limit "$CTX_DECISION_HISTORY" "$entry" 20)
}

guardian_context_needs_normalization() {
    [ -f "$LOOP_CONTEXT_FILE" ] || return 1
    grep -q '"last_completed_count"' "$LOOP_CONTEXT_FILE" 2>/dev/null && return 1
    return 0
}

guardian_init() {
    guardian_backend_available || return 1

    mkdir -p "$FUSION_DIR"

    if [ ! -f "$LOOP_CONTEXT_FILE" ]; then
        guardian_context_defaults "$(get_timestamp_ms)"
        guardian_write_context || return 1
    elif guardian_context_needs_normalization; then
        guardian_load_context
        guardian_write_context || return 1
    fi
    return 0
}

guardian_bridge_get_scalar() {
    local normalized_key="${1:-}"
    local bridge_key=""
    local mode=""
    local value=""

    case "$normalized_key" in
        iteration|last_completed_count|started_at|last_progress_at|total_iterations|no_progress_rounds|same_action_count|same_error_count|wall_time_ms|max_state_visit_count)
            bridge_key="$normalized_key"
            mode="number"
            ;;
        metrics.total_iterations)
            bridge_key="total_iterations"
            mode="number"
            ;;
        metrics.no_progress_rounds)
            bridge_key="no_progress_rounds"
            mode="number"
            ;;
        metrics.same_action_count)
            bridge_key="same_action_count"
            mode="number"
            ;;
        metrics.same_error_count)
            bridge_key="same_error_count"
            mode="number"
            ;;
        metrics.wall_time_ms)
            bridge_key="wall_time_ms"
            mode="number"
            ;;
        metrics.max_state_visit_count)
            bridge_key="max_state_visit_count"
            mode="number"
            ;;
        last_task_snapshot|last_action_signature|last_error_fingerprint)
            bridge_key="$normalized_key"
            mode="string"
            ;;
        *)
            return 1
            ;;
    esac

    value=$(fusion_bridge_inspect_json_field "$bridge_key" "$mode" "$LOOP_CONTEXT_FILE" 2>/dev/null) || value=""
    case "$mode" in
        number)
            [[ "$value" =~ ^[0-9]+$ ]] || return 1
            ;;
        string)
            [ -n "$value" ] || return 1
            ;;
    esac

    printf '%s\n' "$value"
}

guardian_get() {
    local key="${1:-}"
    local normalized_key=""
    local state_name=""
    local bridge_value=""

    guardian_backend_available || {
        echo ""
        return 0
    }
    [ -f "$LOOP_CONTEXT_FILE" ] || {
        echo ""
        return 0
    }

    guardian_load_context
    normalized_key="${key#.}"

    if bridge_value=$(guardian_bridge_get_scalar "$normalized_key"); then
        printf '%s\n' "$bridge_value"
        return 0
    fi

    case "$normalized_key" in
        iteration)
            echo "$CTX_ITERATION"
            ;;
        last_task_snapshot)
            echo "$CTX_LAST_TASK_SNAPSHOT"
            ;;
        last_completed_count)
            echo "$CTX_LAST_COMPLETED_COUNT"
            ;;
        last_action_signature)
            echo "$CTX_LAST_ACTION_SIGNATURE"
            ;;
        last_error_fingerprint)
            echo "$CTX_LAST_ERROR_FINGERPRINT"
            ;;
        started_at)
            echo "$CTX_STARTED_AT"
            ;;
        last_progress_at)
            echo "$CTX_LAST_PROGRESS_AT"
            ;;
        total_iterations|metrics.total_iterations)
            echo "$CTX_TOTAL_ITERATIONS"
            ;;
        no_progress_rounds|metrics.no_progress_rounds)
            echo "$CTX_NO_PROGRESS_ROUNDS"
            ;;
        same_action_count|metrics.same_action_count)
            echo "$CTX_SAME_ACTION_COUNT"
            ;;
        same_error_count|metrics.same_error_count)
            echo "$CTX_SAME_ERROR_COUNT"
            ;;
        wall_time_ms|metrics.wall_time_ms)
            echo "$CTX_WALL_TIME_MS"
            ;;
        max_state_visit_count|metrics.max_state_visit_count)
            echo "$CTX_MAX_STATE_VISIT_COUNT"
            ;;
        state_visits.*)
            state_name="${normalized_key#state_visits.}"
            echo "$(guardian_state_visits_get "$CTX_STATE_VISITS" "$state_name")"
            ;;
        *)
            echo ""
            ;;
    esac
}

compute_task_snapshot() {
    if [ -f "$FUSION_DIR/task_plan.md" ]; then
        local completed pending in_progress failed
        IFS=':' read -r completed pending in_progress failed <<< "$(fusion_task_counts "$FUSION_DIR/task_plan.md")"
        echo "${completed}:${pending}:${in_progress}"
    else
        echo "0:0:0"
    fi
}

compute_action_signature() {
    local phase="$1"
    local next_task="$2"
    compute_md5 "${phase}:${next_task}"
}

guardian_record_iteration() {
    local phase="${1:-EXECUTE}"
    local next_task="${2:-unknown}"
    local error_msg="${3:-}"
    local now_ms=""
    local snapshot=""
    local action_sig=""
    local error_fp=""
    local completed_count="0"
    local previous_iterations=0
    local previous_completed=0

    guardian_backend_available || return 1
    [ -f "$LOOP_CONTEXT_FILE" ] || guardian_init || return 1

    guardian_load_context

    previous_iterations=$(guardian_numeric_or_default "$CTX_TOTAL_ITERATIONS" 0)
    previous_completed=$(guardian_numeric_or_default "$CTX_LAST_COMPLETED_COUNT" 0)
    now_ms=$(get_timestamp_ms)
    snapshot=$(compute_task_snapshot)
    completed_count=$(guardian_numeric_or_default "${snapshot%%:*}" 0)
    action_sig=$(compute_action_signature "$phase" "$next_task")
    if [ -n "$error_msg" ]; then
        error_fp=$(compute_md5 "$error_msg")
    fi

    CTX_ITERATION=$((CTX_ITERATION + 1))
    CTX_TOTAL_ITERATIONS=$((CTX_TOTAL_ITERATIONS + 1))
    CTX_WALL_TIME_MS=$((now_ms - CTX_STARTED_AT))

    CTX_COMPLETED_COUNT_HISTORY=$(guardian_append_list_with_limit "$CTX_COMPLETED_COUNT_HISTORY" "$completed_count" 10)
    if [ "$previous_iterations" -eq 0 ] || [ "$completed_count" -gt "$previous_completed" ]; then
        CTX_NO_PROGRESS_ROUNDS=0
        CTX_LAST_PROGRESS_AT="$now_ms"
    else
        CTX_NO_PROGRESS_ROUNDS=$((CTX_NO_PROGRESS_ROUNDS + 1))
    fi
    CTX_LAST_COMPLETED_COUNT="$completed_count"

    if [ -n "$CTX_LAST_ACTION_SIGNATURE" ] && [ "$CTX_LAST_ACTION_SIGNATURE" = "$action_sig" ]; then
        CTX_SAME_ACTION_COUNT=$((CTX_SAME_ACTION_COUNT + 1))
    else
        CTX_SAME_ACTION_COUNT=1
    fi
    CTX_LAST_ACTION_SIGNATURE="$action_sig"
    CTX_ACTION_SIGNATURES=$(guardian_append_list_with_limit "$CTX_ACTION_SIGNATURES" "$action_sig" 5)

    if [ -n "$error_fp" ]; then
        if [ -n "$CTX_LAST_ERROR_FINGERPRINT" ] && [ "$CTX_LAST_ERROR_FINGERPRINT" = "$error_fp" ] && [ "$CTX_SAME_ERROR_COUNT" -gt 0 ]; then
            CTX_SAME_ERROR_COUNT=$((CTX_SAME_ERROR_COUNT + 1))
        else
            CTX_SAME_ERROR_COUNT=1
        fi
        CTX_LAST_ERROR_FINGERPRINT="$error_fp"
        CTX_ERROR_FINGERPRINTS=$(guardian_append_list_with_limit "$CTX_ERROR_FINGERPRINTS" "$error_fp" 5)
    else
        CTX_SAME_ERROR_COUNT=0
        CTX_LAST_ERROR_FINGERPRINT=""
    fi

    guardian_state_visits_increment "$phase"
    CTX_LAST_TASK_SNAPSHOT="$snapshot"

    guardian_write_context
}

guardian_evaluate() {
    local decision="CONTINUE"
    local reason=""

    guardian_backend_available || {
        echo "CONTINUE"
        return 0
    }
    [ -f "$LOOP_CONTEXT_FILE" ] || {
        echo "CONTINUE"
        return 0
    }

    guardian_load_context

    if [ "$CTX_TOTAL_ITERATIONS" -ge "$GUARDIAN_MAX_ITERATIONS" ]; then
        decision="ABORT_STUCK"
        reason="Max iterations ($GUARDIAN_MAX_ITERATIONS) reached"
    elif [ "$CTX_WALL_TIME_MS" -ge "$GUARDIAN_MAX_WALL_TIME_MS" ]; then
        decision="ABORT_STUCK"
        reason="Max wall time (${GUARDIAN_MAX_WALL_TIME_MS}ms) exceeded"
    elif [ "$CTX_NO_PROGRESS_ROUNDS" -ge "$GUARDIAN_MAX_NO_PROGRESS" ]; then
        decision="ABORT_STUCK"
        reason="No progress for $CTX_NO_PROGRESS_ROUNDS rounds (max: $GUARDIAN_MAX_NO_PROGRESS)"
    elif [ "$CTX_SAME_ACTION_COUNT" -ge "$GUARDIAN_MAX_SAME_ACTION" ]; then
        decision="ESCALATE"
        reason="Same action repeated $CTX_SAME_ACTION_COUNT times (max: $GUARDIAN_MAX_SAME_ACTION)"
    elif [ "$CTX_SAME_ERROR_COUNT" -ge "$GUARDIAN_MAX_SAME_ERROR" ]; then
        decision="ESCALATE"
        reason="Same error repeated $CTX_SAME_ERROR_COUNT times (max: $GUARDIAN_MAX_SAME_ERROR)"
    elif [ "$CTX_MAX_STATE_VISIT_COUNT" -ge "$GUARDIAN_MAX_STATE_VISITS" ]; then
        decision="ESCALATE"
        reason="State visited $CTX_MAX_STATE_VISIT_COUNT times (max: $GUARDIAN_MAX_STATE_VISITS)"
    elif [ "$CTX_NO_PROGRESS_ROUNDS" -ge "$GUARDIAN_BACKOFF_THRESHOLD" ]; then
        decision="BACKOFF"
        reason="No progress for $CTX_NO_PROGRESS_ROUNDS rounds, slowing down"
    elif [ "$CTX_SAME_ACTION_COUNT" -ge 2 ]; then
        decision="BACKOFF"
        reason="Same action repeated $CTX_SAME_ACTION_COUNT times, slowing down"
    fi

    if [ -n "$reason" ]; then
        guardian_append_decision_history "$decision" "$reason"
        guardian_write_context >/dev/null 2>&1 || true
    fi

    echo "$decision"
}

guardian_status() {
    guardian_backend_available || {
        echo "LoopGuardian: unavailable"
        return 0
    }
    [ -f "$LOOP_CONTEXT_FILE" ] || {
        echo "LoopGuardian: not initialized"
        return 0
    }

    guardian_load_context

    printf 'LoopGuardian Status:\n'
    printf '  Iterations: %s/%s\n' "$CTX_TOTAL_ITERATIONS" "$GUARDIAN_MAX_ITERATIONS"
    printf '  No-Progress Rounds: %s/%s\n' "$CTX_NO_PROGRESS_ROUNDS" "$GUARDIAN_MAX_NO_PROGRESS"
    printf '  Same Action Count: %s/%s\n' "$CTX_SAME_ACTION_COUNT" "$GUARDIAN_MAX_SAME_ACTION"
    printf '  Same Error Count: %s/%s\n' "$CTX_SAME_ERROR_COUNT" "$GUARDIAN_MAX_SAME_ERROR"
    printf '  State Visits: %s/%s\n' "$CTX_MAX_STATE_VISIT_COUNT" "$GUARDIAN_MAX_STATE_VISITS"
    printf '  Wall Time: %ss/%ss\n' "$((CTX_WALL_TIME_MS / 1000))" "$((GUARDIAN_MAX_WALL_TIME_MS / 1000))"
}

guardian_reset() {
    rm -f "$LOOP_CONTEXT_FILE" 2>/dev/null || true
    guardian_init
}
