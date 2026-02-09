#!/bin/bash
# fusion-stop-guard.sh - Stop hook to prevent premature stopping
#
# CRITICAL: This is the ONLY stop hook. Do not add checkpoint logic here.
#
# Exit codes:
#   0 = Allow stop (all tasks complete or workflow not active)
#   2 = Block stop (tasks remaining) - legacy compatibility
#
# Advanced API (stdout JSON):
#   When blocking, outputs JSON to stdout:
#   {"decision":"block","reason":"<prompt>","systemMessage":"<status>"}
#
# Safety features:
#   - LoopGuardian: intelligent anti-deadloop protection
#   - Reentry protection via lock directory (only owner cleans up)
#   - Atomic state operations
#   - Stale lock detection and cleanup

set -euo pipefail

FUSION_DIR=".fusion"
LOCK_STALE_SECONDS=300     # Consider lock stale after 5 minutes

# Use the same state lock as pause/cancel/resume for unified protection
STATE_LOCK="${FUSION_DIR}/.state.lock"

# Track if we acquired the lock (only owner should clean up)
LOCK_ACQUIRED=false

# Read hook input from stdin (advanced stop hook API)
HOOK_INPUT=$(cat)

# Source LoopGuardian for intelligent loop protection
# IMPORTANT: Guardian requires jq. If not available, use simple block count fallback.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/loop-guardian.sh" ] && command -v jq &>/dev/null; then
    # shellcheck source=loop-guardian.sh
    source "$SCRIPT_DIR/loop-guardian.sh"
    if [ "$GUARDIAN_JQ_AVAILABLE" = true ]; then
        GUARDIAN_ENABLED=true
    else
        GUARDIAN_ENABLED=false
    fi
else
    # Guardian requires jq - fallback to simple block count
    GUARDIAN_ENABLED=false
fi

# Fallback: simple block count (used when jq not available)
MAX_CONSECUTIVE_BLOCKS=50

# Cleanup lock on exit - ONLY if we acquired it
cleanup() {
    if [ "$LOCK_ACQUIRED" = true ]; then
        rmdir "$STATE_LOCK" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Atomic JSON read using jq if available, fallback to grep
json_get() {
    local file="$1"
    local key="$2"

    if command -v jq &>/dev/null; then
        jq -r ".$key // empty" "$file" 2>/dev/null || echo ""
    else
        # Fallback: simple grep (less reliable but works)
        grep -o "\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$file" 2>/dev/null | head -1 | cut -d'"' -f4 || echo ""
    fi
}

# Atomic JSON update using jq if available
json_set() {
    local file="$1"
    local key="$2"
    local value="$3"

    if command -v jq &>/dev/null; then
        local tmp_file
        tmp_file=$(mktemp "${FUSION_DIR}/.tmp.XXXXXX")
        if jq ".$key = \"$value\"" "$file" > "$tmp_file" 2>/dev/null; then
            mv "$tmp_file" "$file"
            return 0
        else
            rm -f "$tmp_file" 2>/dev/null || true
            return 1
        fi
    else
        # Fallback: sed (fail-close - verify the change happened)
        local before after
        before=$(grep -o "\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$file" 2>/dev/null | head -1 || echo "")

        # Try sed -i (GNU style first, then BSD style)
        if sed -i "s/\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"$key\": \"$value\"/" "$file" 2>/dev/null; then
            : # GNU sed succeeded
        elif sed -i '' "s/\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"$key\": \"$value\"/" "$file" 2>/dev/null; then
            : # BSD sed succeeded
        else
            return 1  # Both failed
        fi

        # Verify the change actually happened
        after=$(grep -o "\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$file" 2>/dev/null | head -1 || echo "")
        if [ "$after" = "\"$key\": \"$value\"" ]; then
            return 0
        else
            return 1  # Change didn't take effect
        fi
    fi
}

# Safe grep count - returns numeric 0 on no match
grep_count() {
    local pattern="$1"
    local file="$2"
    local count
    count=$(grep -c "$pattern" "$file" 2>/dev/null) || true
    # Ensure we have a valid number
    if [[ "$count" =~ ^[0-9]+$ ]]; then
        echo "$count"
    else
        echo "0"
    fi
}

# Atomic increment of block count
increment_block_count() {
    local count_file="$FUSION_DIR/.block_count"
    local current=0

    if [ -f "$count_file" ]; then
        current=$(cat "$count_file" 2>/dev/null) || true
        # Validate it's a number
        if ! [[ "$current" =~ ^[0-9]+$ ]]; then
            current=0
        fi
    fi

    current=$((current + 1))
    echo "$current" > "$count_file"
    echo "$current"
}

# Reset block count
reset_block_count() {
    rm -f "$FUSION_DIR/.block_count" 2>/dev/null || true
}

# Check if lock is stale (older than LOCK_STALE_SECONDS)
is_lock_stale() {
    local lock_dir="$1"

    if [ ! -d "$lock_dir" ]; then
        return 1  # Not stale if doesn't exist
    fi

    # Get lock directory modification time
    local lock_mtime
    if stat --version &>/dev/null 2>&1; then
        # GNU stat
        lock_mtime=$(stat -c %Y "$lock_dir" 2>/dev/null) || return 1
    else
        # BSD stat (macOS)
        lock_mtime=$(stat -f %m "$lock_dir" 2>/dev/null) || return 1
    fi

    local current_time
    current_time=$(date +%s)
    local age=$((current_time - lock_mtime))

    [ "$age" -gt "$LOCK_STALE_SECONDS" ]
}

# Output JSON block response to stdout (Ralph-style advanced API)
output_block_json() {
    local reason="$1"
    local system_msg="$2"

    if command -v jq &>/dev/null; then
        jq -n \
            --arg reason "$reason" \
            --arg msg "$system_msg" \
            '{
                "decision": "block",
                "reason": $reason,
                "systemMessage": $msg
            }'
    else
        # Fallback: manual JSON (escape quotes and newlines in reason)
        local escaped_reason
        escaped_reason=$(printf '%s' "$reason" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\t/\\t/g' | awk '{printf "%s\\n", $0}' | sed 's/\\n$//')
        local escaped_msg
        escaped_msg=$(printf '%s' "$system_msg" | sed 's/\\/\\\\/g; s/"/\\"/g')
        echo "{\"decision\":\"block\",\"reason\":\"$escaped_reason\",\"systemMessage\":\"$escaped_msg\"}"
    fi
}

# Build the prompt/reason for Claude to continue
build_continuation_prompt() {
    local goal="$1"
    local phase="$2"
    local next_task="$3"
    local remaining="$4"
    local block_count="$5"

    cat << EOF
Continue executing the Fusion workflow.

Goal: ${goal:-"(not set)"}
Phase: ${phase:-"EXECUTE"}
Remaining: $remaining tasks
Next task: $next_task

Instructions:
1. Read .fusion/task_plan.md
2. Find next PENDING or IN_PROGRESS task
3. Execute based on task type:
   - implementation/verification → TDD flow (RED→GREEN→REFACTOR)
   - design/documentation/configuration/research → direct execution
4. Update task status to [COMPLETED]
5. Continue until all tasks done

Only ask user if 3-Strike exhausted.
EOF
}

# Main logic
main() {
    # If no fusion directory, allow stop (not in a fusion workflow)
    if [ ! -d "$FUSION_DIR" ]; then
        exit 0
    fi

    # Check sessions.json for workflow status
    if [ ! -f "$FUSION_DIR/sessions.json" ]; then
        exit 0
    fi

    # Use unified state lock (same as pause/cancel/resume)
    # This prevents concurrent writes to sessions.json

    # Check for stale lock and clean up
    if is_lock_stale "$STATE_LOCK"; then
        echo "⚠️ Cleaning up stale lock (older than ${LOCK_STALE_SECONDS}s)" >&2
        rmdir "$STATE_LOCK" 2>/dev/null || true
    fi

    # Reentry protection: acquire lock (mkdir is atomic)
    if mkdir "$STATE_LOCK" 2>/dev/null; then
        LOCK_ACQUIRED=true
    else
        # Another instance is running - but don't just allow stop!
        # Output warning and still block to be safe
        echo "⚠️ Another state operation in progress, blocking to be safe" >&2
        exit 2
    fi

    # Read status
    local status
    status=$(json_get "$FUSION_DIR/sessions.json" "status")

    # Only block if workflow is in_progress
    if [ "$status" != "in_progress" ]; then
        reset_block_count
        exit 0
    fi

    # Count pending and in_progress tasks from task_plan.md
    local pending_count=0
    local in_progress_count=0
    local failed_count=0
    local task_plan_exists=false

    if [ -f "$FUSION_DIR/task_plan.md" ]; then
        task_plan_exists=true
        pending_count=$(grep_count "\[PENDING\]" "$FUSION_DIR/task_plan.md")
        in_progress_count=$(grep_count "\[IN_PROGRESS\]" "$FUSION_DIR/task_plan.md")
        # Also count FAILED tasks - they are NOT completed
        failed_count=$(grep_count "\[FAILED\]" "$FUSION_DIR/task_plan.md")
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
        local completed_count
        completed_count=$(grep_count "\[COMPLETED\]" "$FUSION_DIR/task_plan.md")
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

    # Get goal for context
    local goal
    goal=$(json_get "$FUSION_DIR/sessions.json" "goal")

    # IMPORTANT: If task_plan.md doesn't exist but status is in_progress,
    # check the phase to decide behavior
    if [ "$task_plan_exists" = false ]; then
        # Early phases (before tasks are created) - block and continue
        if [ "$current_phase" = "INITIALIZE" ] || [ "$current_phase" = "ANALYZE" ] || [ "$current_phase" = "DECOMPOSE" ] || [ -z "$current_phase" ]; then
            local iteration=0

            if [ "$GUARDIAN_ENABLED" = true ]; then
                guardian_init || true
                guardian_record_iteration "${current_phase:-DECOMPOSE}" "create_task_plan" "" || true
                local guardian_decision
                guardian_decision=$(guardian_evaluate)

                if [ "$guardian_decision" = "ABORT_STUCK" ]; then
                    json_set "$FUSION_DIR/sessions.json" "status" "stuck" || true
                    echo "⚠️ LOOPGUARDIAN: Stuck in early phase (no task_plan.md)" >&2
                    exit 0
                fi

                iteration=$(guardian_get '.iteration')
            else
                # Fallback to simple counter
                iteration=$(increment_block_count)
                if [ "$iteration" -gt "$MAX_CONSECUTIVE_BLOCKS" ]; then
                    json_set "$FUSION_DIR/sessions.json" "status" "stuck" || true
                    echo "⚠️ FUSION SAFETY LIMIT REACHED (no task_plan.md)" >&2
                    exit 0
                fi
            fi

            # Output JSON to stdout for advanced API
            local prompt="Continue with task decomposition for goal: ${goal:-'(not set)'}. Create .fusion/task_plan.md with tasks."
            local sys_msg="🔄 Fusion iteration $iteration | Phase: ${current_phase:-DECOMPOSE} | Create task_plan.md"
            output_block_json "$prompt" "$sys_msg"
            exit 0
        fi

        # Later phases without task_plan.md is an error - mark as stuck
        json_set "$FUSION_DIR/sessions.json" "status" "stuck" || true
        echo "⚠️ FUSION ERROR: task_plan.md missing in phase $current_phase" >&2
        exit 0
    fi

    # If no remaining tasks, allow stop and update status
    if [ "$total_remaining" -eq 0 ]; then
        if json_set "$FUSION_DIR/sessions.json" "status" "completed"; then
            # Reset guardian for next workflow
            if [ "$GUARDIAN_ENABLED" = true ]; then
                guardian_reset
            fi

            # Save final checkpoint
            local timestamp
            timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            if [ -f "$FUSION_DIR/progress.md" ]; then
                echo "| $timestamp | COMPLETE | Workflow finished | OK | All tasks done |" >> "$FUSION_DIR/progress.md"
            fi

            exit 0
        else
            # JSON update failed - report error but still allow stop
            echo "⚠️ Failed to update status to completed, but all tasks done" >&2
            exit 0
        fi
    fi

    # Get next task for context (needed for guardian)
    local next_task
    next_task=$(grep -B1 "\[PENDING\]\|\[IN_PROGRESS\]" "$FUSION_DIR/task_plan.md" 2>/dev/null | grep "### Task" | head -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//' || echo "unknown")

    # LoopGuardian: Intelligent anti-deadloop protection
    if [ "$GUARDIAN_ENABLED" = true ]; then
        # Initialize guardian if needed
        guardian_init || true

        # Record this iteration
        guardian_record_iteration "$current_phase" "$next_task" "" || true

        # Evaluate and decide
        local guardian_decision
        guardian_decision=$(guardian_evaluate)

        case "$guardian_decision" in
            "ABORT_STUCK")
                # Guardian detected stuck state
                json_set "$FUSION_DIR/sessions.json" "status" "stuck" || true
                local guardian_status
                guardian_status=$(guardian_status 2>/dev/null || echo "")
                echo "⚠️ LOOPGUARDIAN: Detected stuck state" >&2
                echo "$guardian_status" >&2
                exit 0
                ;;
            "ESCALATE")
                # Need user intervention
                local prompt="⚠️ LoopGuardian detected a pattern that may indicate a problem.

$(guardian_status)

Please review:
1. Check .fusion/task_plan.md for stuck tasks
2. Check .fusion/progress.md for errors
3. Decide: continue (adjust approach), pause, or cancel

What would you like to do?"
                local sys_msg="⚠️ ESCALATE: Pattern detected - asking user for guidance"
                output_block_json "$prompt" "$sys_msg"
                exit 0
                ;;
            "BACKOFF")
                # Warning sign - add context to prompt
                local prompt
                prompt=$(build_continuation_prompt "$goal" "$current_phase" "$next_task" "$total_remaining" "$(guardian_get '.iteration')")
                prompt="$prompt

⚠️ Warning: LoopGuardian detected potential stagnation.
Consider: trying a different approach, checking for blockers, or asking for help."
                local sys_msg="🔄 Fusion (BACKOFF) | Phase: ${current_phase:-EXECUTE} | Remaining: $total_remaining | $(guardian_get '.metrics.no_progress_rounds') no-progress rounds"
                output_block_json "$prompt" "$sys_msg"
                exit 0
                ;;
            *)
                # CONTINUE - normal flow
                ;;
        esac
    else
        # FALLBACK: Guardian not available (no jq) - use simple block count
        local block_count
        block_count=$(increment_block_count)

        if [ "$block_count" -gt "$MAX_CONSECUTIVE_BLOCKS" ]; then
            # Safety limit reached
            json_set "$FUSION_DIR/sessions.json" "status" "stuck" || true
            echo "⚠️ FUSION SAFETY LIMIT REACHED ($block_count blocks). Status set to 'stuck'." >&2
            exit 0
        fi
    fi

    # Normal continuation (guardian CONTINUE or fallback within limits)
    # Build continuation prompt
    local prompt
    local iteration=0
    if [ "$GUARDIAN_ENABLED" = true ]; then
        iteration=$(guardian_get '.iteration')
    else
        iteration=$(cat "$FUSION_DIR/.block_count" 2>/dev/null || echo "0")
    fi
    prompt=$(build_continuation_prompt "$goal" "$current_phase" "$next_task" "$total_remaining" "$iteration")

    # Add phase correction notice if applicable
    if [ "$phase_corrected" = true ]; then
        prompt="$prompt

Note: Phase auto-corrected to $current_phase based on task states."
    fi

    # Build system message
    local sys_msg="🔄 Fusion iteration $iteration | Phase: ${current_phase:-EXECUTE} | Remaining: $total_remaining | Next: $next_task"

    # Output JSON to stdout (advanced stop hook API)
    output_block_json "$prompt" "$sys_msg"

    # Exit 0 for successful hook execution (JSON output handles the block)
    exit 0
}

main "$@"
