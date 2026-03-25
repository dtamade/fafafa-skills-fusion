#!/bin/bash

fusion_stop_guard_shell_fallback() {
    # Use unified state lock (same as pause/cancel/resume)
    # This prevents concurrent writes to sessions.json

    # Check for stale lock and clean up
    stop_guard_cleanup_stale_lock "$STATE_LOCK" "$LOCK_STALE_SECONDS"

    # Reentry protection: acquire lock (mkdir is atomic)
    if mkdir "$STATE_LOCK" 2>/dev/null; then
        LOCK_ACQUIRED=true
    else
        # Another instance is running; block in a mode-compatible way.
        local lock_reason
        lock_reason="State operation already in progress. Continue executing the Fusion workflow and retry stop."
        local lock_sys_msg
        lock_sys_msg="🔒 Fusion state operation in progress"
        emit_block_response "$lock_reason" "$lock_sys_msg"
    fi

    # Read status
    local status
    status=$(json_get "$FUSION_DIR/sessions.json" "status")

    # Only block if workflow is in_progress
    if [ "$status" != "in_progress" ]; then
        hook_debug_log "allow: status=$status"
        reset_block_count
        exit 0
    fi

    # Count pending and in_progress tasks from task_plan.md
    local completed_count=0
    local pending_count=0
    local in_progress_count=0
    local failed_count=0
    local task_plan_exists=false

    if [ -f "$FUSION_DIR/task_plan.md" ]; then
        task_plan_exists=true
        IFS=':' read -r completed_count pending_count in_progress_count failed_count <<< "$(fusion_task_counts "$FUSION_DIR/task_plan.md")"
    fi

    # FAILED tasks count as remaining work (need user intervention)
    local total_remaining=$((pending_count + in_progress_count + failed_count))

    # Read current phase for decision making
    local current_phase
    current_phase=$(json_get "$FUSION_DIR/sessions.json" "current_phase")

    # --- Phase Coherence Validation ---
    # Detect and auto-correct phase inconsistencies.
    # If sessions.json says EXECUTE but all tasks are COMPLETED, advance to VERIFY.
    local phase_corrected=false
    if [ "$task_plan_exists" = true ] && [ "$total_remaining" -eq 0 ] && [ "$current_phase" = "EXECUTE" ]; then
        if [ "$completed_count" -gt 0 ]; then
            json_set "$FUSION_DIR/sessions.json" "current_phase" "VERIFY" || true
            current_phase="VERIFY"
            phase_corrected=true
        fi
    fi
    # If phase is VERIFY/REVIEW/COMMIT but tasks still PENDING, correct back to EXECUTE
    if [ "$task_plan_exists" = true ] && [ "$pending_count" -gt 0 ]; then
        case "$current_phase" in
            VERIFY|REVIEW|COMMIT|DELIVER)
                json_set "$FUSION_DIR/sessions.json" "current_phase" "EXECUTE" || true
                current_phase="EXECUTE"
                phase_corrected=true
                ;;
        esac
    fi

    # Get goal for context (sanitize to prevent JSON injection in sed fallback)
    local goal
    goal=$(json_get "$FUSION_DIR/sessions.json" "goal")
    goal=$(stop_guard_sanitize_inline "$goal")

    # IMPORTANT: If task_plan.md doesn't exist but status is in_progress,
    # check the phase to decide behavior
    if [ "$task_plan_exists" = false ]; then
        # Early phases (before tasks are created) - block and continue
        if [ "$current_phase" = "INITIALIZE" ] || [ "$current_phase" = "ANALYZE" ] || [ "$current_phase" = "DECOMPOSE" ] || [ -z "$current_phase" ]; then
            local iteration=0
            local next_action=""
            local prompt=""
            local sys_msg=""

            iteration=$(increment_block_count)
            if [ "$iteration" -gt "$MAX_CONSECUTIVE_BLOCKS" ]; then
                json_set "$FUSION_DIR/sessions.json" "status" "stuck" || true
                echo "⚠️ FUSION SAFETY LIMIT REACHED (no task_plan.md)" >&2
                exit 0
            fi

            next_action=$(stop_guard_create_task_plan_next_action)
            prompt=$(build_decompose_prompt "$goal" "$next_action")
            sys_msg=$(stop_guard_system_message "${current_phase:-DECOMPOSE}" "" "$next_action")
            emit_block_response "$prompt" "$sys_msg"
        fi

        # Later phases without task_plan.md is an error - mark as stuck
        json_set "$FUSION_DIR/sessions.json" "status" "stuck" || true
        echo "⚠️ FUSION ERROR: task_plan.md missing in phase $current_phase" >&2
        exit 0
    fi

    # Emergency shell fallback no longer performs safe_backlog injection.
    # Task exhaustion completion side-effects stay minimal here; richer continuation belongs to Rust-side orchestration paths.
    if [ "$total_remaining" -eq 0 ]; then
        hook_debug_log "allow: all tasks done phase=${current_phase:-unknown}"
        if json_set "$FUSION_DIR/sessions.json" "status" "completed"; then
            reset_block_count

            local timestamp
            timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            if [ -f "$FUSION_DIR/progress.md" ]; then
                echo "| $timestamp | COMPLETE | Workflow finished | OK | All tasks done |" >> "$FUSION_DIR/progress.md"
            fi

            exit 0
        fi

        echo "⚠️ Failed to update status to completed, but all tasks done" >&2
        exit 0
    fi

    # Get next task for context (sanitize to prevent JSON injection in sed fallback)
    local next_task_display=""
    next_task_display=$(fusion_current_or_next_task_display_with_status "$FUSION_DIR/task_plan.md")
    next_task_display=$(stop_guard_sanitize_inline "$next_task_display")
    local pending_review_task_id
    local pending_review_task_title=""
    pending_review_task_id=$(fusion_first_pending_review_task_id "$FUSION_DIR/task_plan.md")
    pending_review_task_id=$(stop_guard_sanitize_inline "$pending_review_task_id")
    if [ -n "$pending_review_task_id" ]; then
        pending_review_task_title=$(fusion_task_title_by_id "$FUSION_DIR/task_plan.md" "$pending_review_task_id")
        pending_review_task_title=$(stop_guard_sanitize_inline "$pending_review_task_title")
    fi

    local block_count
    block_count=$(increment_block_count)
    if [ "$block_count" -gt "$MAX_CONSECUTIVE_BLOCKS" ]; then
        json_set "$FUSION_DIR/sessions.json" "status" "stuck" || true
        echo "⚠️ FUSION SAFETY LIMIT REACHED ($block_count blocks). Status set to 'stuck'." >&2
        exit 0
    fi

    # Normal continuation within the shell emergency limit.
    local prompt
    local iteration
    local next_action
    iteration=$(stop_guard_read_block_count)
    if [ -n "$pending_review_task_id" ]; then
        next_action=$(stop_guard_review_gate_next_action "$pending_review_task_id")
        prompt=$(build_review_gate_prompt "$goal" "$current_phase" "$next_action" "$total_remaining" "$pending_review_task_id" "$pending_review_task_title")
    else
        next_action=$(stop_guard_continue_task_next_action "$next_task_display")
        prompt=$(build_continuation_prompt "$goal" "$current_phase" "$next_action" "$total_remaining" "$iteration")
    fi

    # Add phase correction notice if applicable
    if [ "$phase_corrected" = true ]; then
        prompt="${prompt}$(stop_guard_phase_correction_note "$current_phase")"
    fi

    # Build system message
    local sys_msg=""
    sys_msg=$(stop_guard_system_message "${current_phase:-EXECUTE}" "$total_remaining" "$next_action")

    # Output JSON to stdout (advanced stop hook API)
    if [ -n "$pending_review_task_id" ]; then
        hook_debug_log "block: phase=${current_phase:-EXECUTE} remaining=$total_remaining next=${next_action:-unknown}"
    else
        hook_debug_log "block: phase=${current_phase:-EXECUTE} remaining=$total_remaining next=${next_action:-unknown}"
    fi
    emit_block_response "$prompt" "$sys_msg"
}
