#!/bin/bash

fusion_posttool_sanitize_inline() {
    printf '%s' "$1" | tr -d '"\\\t' | tr '\n' ' '
}

fusion_posttool_continue_task_next_action() {
    local task_display
    task_display=$(fusion_posttool_sanitize_inline "$1")
    [ -n "$task_display" ] || return 0
    printf 'Continue task: %s\n' "$task_display"
}

fusion_posttool_review_gate_next_action() {
    local task_id
    task_id=$(fusion_posttool_sanitize_inline "$1")
    [ -n "$task_id" ] || task_id="current task"
    printf 'reviewer approve %s before execution continues\n' "$task_id"
}

fusion_posttool_proceed_to_verify_next_action() {
    echo "Proceed to VERIFY phase"
}

fusion_posttool_next_action_line() {
    local next_action
    next_action=$(fusion_posttool_sanitize_inline "$1")
    [ -n "$next_action" ] || next_action="Inspect .fusion/task_plan.md and continue from the next live step"
    printf '[fusion] Next action: %s\n' "$next_action"
}

fusion_posttool_next_action_with_mode() {
    local next_action="$1"
    local mode
    mode=$(fusion_posttool_sanitize_inline "$2")

    if [ -n "$mode" ]; then
        printf '%s | Mode: %s\n' "$(fusion_posttool_next_action_line "$next_action")" "$mode"
    else
        fusion_posttool_next_action_line "$next_action"
    fi
}

fusion_posttool_shell_fallback() {
    local fusion_dir="$1"
    local task_plan="$fusion_dir/task_plan.md"
    local snapshot_file="$fusion_dir/.progress_snapshot"
    local stale_file="$fusion_dir/.snapshot_unchanged_count"

    local status
    status=$(json_get_field "$fusion_dir/sessions.json" "status")
    if [ "$status" != "in_progress" ]; then
        hook_debug_log "skip: status=$status"
        return 0
    fi

    local completed=0 pending=0 in_progress=0 failed=0
    if [ -f "$task_plan" ]; then
        IFS=':' read -r completed pending in_progress failed <<< "$(fusion_task_counts "$task_plan")"
    fi

    local total current_snapshot prev_snapshot=""
    total=$((completed + pending + in_progress + failed))
    current_snapshot="${completed}:${pending}:${in_progress}:${failed}"

    if [ -f "$snapshot_file" ]; then
        prev_snapshot=$(cat "$snapshot_file" 2>/dev/null) || true
    fi
    echo "$current_snapshot" > "$snapshot_file" 2>/dev/null || true

    hook_debug_log "active: snapshot=$current_snapshot prev=${prev_snapshot:-none}"
    if [ "$current_snapshot" = "$prev_snapshot" ]; then
        local unchanged=0 current_task=""
        if [ -f "$stale_file" ]; then
            unchanged=$(cat "$stale_file" 2>/dev/null) || unchanged=0
            if ! [[ "$unchanged" =~ ^[0-9]+$ ]]; then
                unchanged=0
            fi
        fi
        unchanged=$((unchanged + 1))
        echo "$unchanged" > "$stale_file" 2>/dev/null || true

        if [ "$unchanged" -ge 5 ] && [ "$total" -gt 0 ]; then
            hook_debug_log "no-progress: unchanged=$unchanged total=$total"
            current_task=$(fusion_first_task_with_status "$task_plan" "[IN_PROGRESS]")
            echo "[fusion] Info: ${unchanged} file edits since last task status change."
            if [ -n "$current_task" ]; then
                echo "[fusion] Current: ${current_task} [IN_PROGRESS] | When done, mark [COMPLETED] in task_plan.md"
            fi
        fi
        hook_debug_log "done: no task status changes"
        return 0
    fi

    rm -f "$stale_file" 2>/dev/null || true

    local prev_completed=0 prev_failed=0
    if [ -n "$prev_snapshot" ]; then
        IFS=':' read -r prev_completed _ _ prev_failed <<< "$prev_snapshot"
    fi
    [[ "$prev_completed" =~ ^[0-9]+$ ]] || prev_completed=0
    [[ "$prev_failed" =~ ^[0-9]+$ ]] || prev_failed=0

    local completed_delta failed_delta
    completed_delta=$((completed - prev_completed))
    failed_delta=$((failed - prev_failed))

    if [ "$completed_delta" -gt 0 ]; then
        hook_debug_log "task-delta: completed_delta=$completed_delta"
        local just_completed next_task next_task_display next_type execution_mode next_action pending_review_task_id=""
        just_completed=$(fusion_last_task_with_status "$task_plan" "[COMPLETED]")
        echo "[fusion] Task ${just_completed:-?} → COMPLETED (${completed}/${total} done)"

        pending_review_task_id=$(fusion_first_pending_review_task_id "$task_plan")
        if [ -n "$pending_review_task_id" ]; then
            next_action=$(fusion_posttool_review_gate_next_action "$pending_review_task_id")
            fusion_posttool_next_action_line "$next_action"
        else
            next_task=$(fusion_current_or_next_task "$task_plan")
            next_task_display=$(fusion_current_or_next_task_display_with_status "$task_plan")
        fi
        if [ -z "$pending_review_task_id" ] && [ -n "$next_task" ]; then
            next_type=$(fusion_task_type_for_title "$task_plan" "$next_task")
            execution_mode=$(fusion_execution_mode_for_task_type "$next_type")
            next_action=$(fusion_posttool_continue_task_next_action "$next_task_display")
            fusion_posttool_next_action_with_mode "$next_action" "$execution_mode"
        elif [ "$pending" -eq 0 ] && [ "$in_progress" -eq 0 ]; then
            fusion_posttool_next_action_line "$(fusion_posttool_proceed_to_verify_next_action)"
        fi
    fi

    if [ "$failed_delta" -gt 0 ]; then
        hook_debug_log "task-delta: failed_delta=$failed_delta"
        local just_failed
        just_failed=$(fusion_last_task_with_status "$task_plan" "[FAILED]")
        echo "[fusion] Task ${just_failed:-?} → FAILED. Apply 3-Strike protocol."
    fi

    hook_debug_log "done: progress processed"
    return 0
}
