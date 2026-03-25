#!/bin/bash

stop_guard_cleanup_lock() {
    local state_lock="$1"
    local lock_acquired="${2:-false}"

    if [ "$lock_acquired" = true ]; then
        rmdir "$state_lock" 2>/dev/null || true
    fi
}

json_get() {
    json_get_field "$1" "$2"
}

json_set() {
    local file="$1"
    local key="$2"
    local value="$3"
    local tmp_root="${FUSION_DIR:-$(dirname "$file")}" 

    json_set_field "$file" "$key" "$value" "$tmp_root"
}

increment_block_count() {
    local count_file="$FUSION_DIR/.block_count"
    local current=0

    if [ -f "$count_file" ]; then
        current=$(cat "$count_file" 2>/dev/null) || true
        if ! [[ "$current" =~ ^[0-9]+$ ]]; then
            current=0
        fi
    fi

    current=$((current + 1))
    echo "$current" > "$count_file"
    echo "$current"
}

reset_block_count() {
    rm -f "$FUSION_DIR/.block_count" 2>/dev/null || true
}

stop_guard_read_block_count() {
    local count_file="$FUSION_DIR/.block_count"
    local current=0

    if [ -f "$count_file" ]; then
        current=$(cat "$count_file" 2>/dev/null) || true
        if ! [[ "$current" =~ ^[0-9]+$ ]]; then
            current=0
        fi
    fi

    echo "$current"
}

is_lock_stale() {
    local lock_dir="$1"
    local stale_seconds="${2:-300}"

    if [ ! -d "$lock_dir" ]; then
        return 1
    fi

    local lock_mtime
    if stat --version &>/dev/null 2>&1; then
        lock_mtime=$(stat -c %Y "$lock_dir" 2>/dev/null) || return 1
    else
        lock_mtime=$(stat -f %m "$lock_dir" 2>/dev/null) || return 1
    fi

    local current_time
    current_time=$(date +%s)
    local age=$((current_time - lock_mtime))

    [ "$age" -gt "$stale_seconds" ]
}

stop_guard_cleanup_stale_lock() {
    local state_lock="$1"
    local stale_seconds="${2:-300}"

    if is_lock_stale "$state_lock" "$stale_seconds"; then
        echo "⚠️ Cleaning up stale lock (older than ${stale_seconds}s)" >&2
        rmdir "$state_lock" 2>/dev/null || true
    fi

    return 0
}

stop_guard_sanitize_inline() {
    printf '%s' "$1" | tr -d '"\\\t' | tr '\n' ' '
}

stop_guard_live_next_action_fallback() {
    echo "Inspect .fusion/task_plan.md and continue from the next live step"
}

stop_guard_review_gate_next_action() {
    local task_id
    task_id=$(stop_guard_sanitize_inline "$1")

    if [ -z "$task_id" ]; then
        task_id="current task"
    fi

    printf 'reviewer approve %s before execution continues\n' "$task_id"
}

stop_guard_review_gate_task_line() {
    local task_id
    local task_title

    task_id=$(stop_guard_sanitize_inline "$1")
    task_title=$(stop_guard_sanitize_inline "$2")

    [ -n "$task_id" ] || task_id="current task"
    [ -n "$task_title" ] || task_title="current task"

    printf 'Task: %s (%s)\n' "$task_id" "$task_title"
}

stop_guard_continue_task_next_action() {
    local task_display
    task_display=$(stop_guard_sanitize_inline "$1")

    if [ -z "$task_display" ]; then
        stop_guard_live_next_action_fallback
        return 0
    fi

    printf 'Continue task: %s\n' "$task_display"
}

stop_guard_create_task_plan_next_action() {
    echo "Create task plan and run the DECOMPOSE phase"
}

stop_guard_phase_correction_note() {
    local phase

    phase=$(stop_guard_sanitize_inline "$1")
    [ -n "$phase" ] || phase="?"

    printf '\n\nNote: Phase auto-corrected to %s based on task states.\n' "$phase"
}

stop_guard_system_message() {
    local phase="$1"
    local remaining="${2:-}"
    local next_action="$3"

    phase=$(stop_guard_sanitize_inline "$phase")
    next_action=$(stop_guard_sanitize_inline "$next_action")

    [ -n "$phase" ] || phase="?"
    [ -n "$next_action" ] || next_action="$(stop_guard_live_next_action_fallback)"

    if [ -n "$remaining" ]; then
        printf '🔄 Fusion | Phase: %s | Remaining: %s | Next: %s\n' "$phase" "$remaining" "$next_action"
    else
        printf '🔄 Fusion | Phase: %s | Next: %s\n' "$phase" "$next_action"
    fi
}

build_decompose_prompt() {
    local goal="$1"
    local next_action="$2"

    goal=$(stop_guard_sanitize_inline "$goal")
    next_action=$(stop_guard_sanitize_inline "$next_action")
    [ -n "$next_action" ] || next_action="$(stop_guard_create_task_plan_next_action)"

    cat << EOF2
Continue with task decomposition for goal: ${goal:-"(not set)"}.

Next action: $next_action
1. Break the goal into explicit tasks
2. Save them to .fusion/task_plan.md
EOF2
}

build_continuation_prompt() {
    local goal="$1"
    local phase="$2"
    local next_action="$3"
    local remaining="$4"
    local _block_count="${5:-0}"

    goal=$(stop_guard_sanitize_inline "$goal")
    next_action=$(stop_guard_sanitize_inline "$next_action")
    [ -n "$next_action" ] || next_action="$(stop_guard_live_next_action_fallback)"

    cat << EOF2
Continue executing the Fusion workflow.

Goal: ${goal:-"(not set)"}
Phase: ${phase:-"EXECUTE"}
Remaining: $remaining tasks
Next action: $next_action

Instructions:
1. Read .fusion/task_plan.md
2. Find next PENDING or IN_PROGRESS task
3. Execute based on task type:
   - implementation/verification → TDD flow (RED→GREEN→REFACTOR)
   - design/documentation/configuration/research → direct execution
4. Update task status to [COMPLETED]
5. Continue until all tasks done

Only ask user if 3-Strike exhausted.
EOF2
}

build_review_gate_prompt() {
    local goal="$1"
    local phase="$2"
    local next_action="$3"
    local remaining="$4"
    local task_id="$5"
    local task_title="$6"

    goal=$(stop_guard_sanitize_inline "$goal")
    phase=$(stop_guard_sanitize_inline "$phase")
    next_action=$(stop_guard_sanitize_inline "$next_action")
    task_id=$(stop_guard_sanitize_inline "$task_id")
    task_title=$(stop_guard_sanitize_inline "$task_title")
    [ -n "$phase" ] || phase="EXECUTE"
    [ -n "$next_action" ] || next_action="$(stop_guard_live_next_action_fallback)"

    cat << EOF2
Continue executing the Fusion workflow.

Goal: ${goal:-"(not set)"}
Phase: $phase
Remaining: $remaining tasks
Review gate: $next_action
$(stop_guard_review_gate_task_line "$task_id" "$task_title")

Reviewer instructions:
1. Read .fusion/task_plan.md
2. Review the task output and regressions only
3. If approved, set \`- Review-Status: approved\` and mark the task [COMPLETED]
4. If changes are required, set \`- Review-Status: changes_requested\` and return it to implementation
5. Continue only after the review decision is recorded
EOF2
}
