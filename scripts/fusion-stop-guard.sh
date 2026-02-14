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
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

is_truthy() {
    case "$(printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]')" in
        1|true|yes|on)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

HOOK_DEBUG=false
if is_truthy "${FUSION_HOOK_DEBUG:-}" || [ -f "$FUSION_DIR/.hook_debug" ]; then
    HOOK_DEBUG=true
fi

hook_debug_log() {
    [ "$HOOK_DEBUG" = true ] || return 0
    local message="$1"
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local line="[fusion][hook-debug][stop][$ts] $message"
    echo "$line" >&2
    if [ -d "$FUSION_DIR" ]; then
        echo "$line" >> "$FUSION_DIR/hook-debug.log" 2>/dev/null || true
    fi
}

resolve_fusion_bridge_bin() {
    if [ -n "${FUSION_BRIDGE_BIN:-}" ] && [ -x "$FUSION_BRIDGE_BIN" ]; then
        echo "$FUSION_BRIDGE_BIN"
        return 0
    fi

    if command -v fusion-bridge >/dev/null 2>&1; then
        command -v fusion-bridge
        return 0
    fi

    local candidates=(
        "$SCRIPT_DIR/../rust/target/release/fusion-bridge"
        "$SCRIPT_DIR/../rust/target/debug/fusion-bridge"
    )

    local candidate
    for candidate in "${candidates[@]}"; do
        if [ -x "$candidate" ]; then
            echo "$candidate"
            return 0
        fi
    done

    return 1
}

runtime_engine_is_rust() {
    [ -f "$FUSION_DIR/config.yaml" ] || return 1
    grep -Eq 'engine:[[:space:]]*"?rust"?' "$FUSION_DIR/config.yaml" 2>/dev/null
}

runtime_enabled_in_config() {
    [ -f "$FUSION_DIR/config.yaml" ] || return 1

    awk '
    BEGIN { in_runtime = 0; found = 0 }
    /^[[:space:]]*#/ { next }
    /^[^[:space:]#][^:]*:[[:space:]]*$/ {
        key = $0
        sub(/[[:space:]]*:[[:space:]]*$/, "", key)
        in_runtime = (key == "runtime")
        next
    }
    in_runtime && /^[[:space:]]+enabled:[[:space:]]*/ {
        value = $0
        sub(/^[[:space:]]+enabled:[[:space:]]*/, "", value)
        sub(/[[:space:]]*#.*/, "", value)
        gsub(/[[:space:]\"]/, "", value)
        if (tolower(value) == "true") {
            found = 1
        }
        exit
    }
    /^[^[:space:]#]/ {
        in_runtime = 0
    }
    END { exit(found ? 0 : 1) }
    ' "$FUSION_DIR/config.yaml" 2>/dev/null
}

# Use the same state lock as pause/cancel/resume for unified protection
STATE_LOCK="${FUSION_DIR}/.state.lock"

# Track if we acquired the lock (only owner should clean up)
LOCK_ACQUIRED=false

# Read hook input from stdin (advanced stop hook API)
HOOK_INPUT=$(cat)
hook_debug_log "invoked: mode=${FUSION_STOP_HOOK_MODE:-auto} cwd=$(pwd)"

# Source LoopGuardian for intelligent loop protection
# IMPORTANT: Guardian requires jq. If not available, use simple block count fallback.
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
        # Use --arg to safely pass value (prevents injection)
        if jq --arg v "$value" ".$key = \$v" "$file" > "$tmp_file" 2>/dev/null; then
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
        # Fallback: use Python json.dumps for reliable escaping (covers all control chars)
        # If Python unavailable, fall back to sed (best effort)
        if command -v python3 &>/dev/null; then
            python3 -c "import json,sys; print(json.dumps({'decision':'block','reason':sys.argv[1],'systemMessage':sys.argv[2]}))" "$reason" "$system_msg"
        else
            # Last resort: sed-based escaping (may miss some control chars)
            local escaped_reason
            escaped_reason=$(printf '%s' "$reason" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\t/\\t/g; s/\r/\\r/g' | awk '{printf "%s\\n", $0}' | sed 's/\\n$//')
            local escaped_msg
            escaped_msg=$(printf '%s' "$system_msg" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\t/\\t/g; s/\r/\\r/g; s/\n/\\n/g')
            echo "{\"decision\":\"block\",\"reason\":\"$escaped_reason\",\"systemMessage\":\"$escaped_msg\"}"
        fi
    fi
}

stop_hook_supports_json_block() {
    local mode
    mode="${FUSION_STOP_HOOK_MODE:-auto}"

    case "$mode" in
        json|modern|structured)
            return 0
            ;;
        legacy|exit2)
            return 1
            ;;
        auto|"")
            # Default to structured responses for modern hook runtimes.
            # Legacy behavior remains available via explicit FUSION_STOP_HOOK_MODE=legacy.
            return 0
            ;;
        *)
            echo "[fusion] unknown FUSION_STOP_HOOK_MODE='$mode', defaulting to structured" >&2
            return 0
            ;;
    esac
}

extract_json_field() {
    local json_payload="$1"
    local field="$2"

    if command -v python3 &>/dev/null; then
        printf '%s' "$json_payload" | python3 -c 'import json,sys
field=sys.argv[1]
try:
    payload=json.load(sys.stdin)
except Exception:
    raise SystemExit(0)
value=payload.get(field, "")
if value is None:
    value=""
print(value)' "$field" 2>/dev/null || true
        return 0
    fi

    if command -v jq &>/dev/null; then
        printf '%s' "$json_payload" | jq -r --arg field "$field" '.[$field] // empty' 2>/dev/null || true
        return 0
    fi

    echo ""
}

emit_block_response() {
    local reason="$1"
    local system_msg="$2"

    if stop_hook_supports_json_block; then
        hook_debug_log "decision=block mode=structured msg=${system_msg}"
        output_block_json "$reason" "$system_msg"
        exit 0
    fi

    hook_debug_log "decision=block mode=legacy msg=${system_msg}"
    echo "[fusion] stop blocked: $system_msg" >&2
    echo "$reason" >&2
    exit 2
}

emit_runtime_block_response() {
    local runtime_json="$1"

    local reason
    reason=$(extract_json_field "$runtime_json" "reason")
    local sys_msg
    sys_msg=$(extract_json_field "$runtime_json" "systemMessage")

    [ -n "$reason" ] || reason="Continue executing the Fusion workflow."
    [ -n "$sys_msg" ] || sys_msg="🔄 Fusion | Workflow in progress"

    emit_block_response "$reason" "$sys_msg"
}

# Build the prompt/reason for Claude to continue
build_continuation_prompt() {
    local goal="$1"
    local phase="$2"
    local next_task="$3"
    local remaining="$4"
    local block_count="$5"

    # Sanitize user-sourced data to prevent JSON injection in sed fallback path
    goal=$(printf '%s' "$goal" | tr -d '"\\\t' | tr '\n' ' ')
    next_task=$(printf '%s' "$next_task" | tr -d '"\\\t' | tr '\n' ' ')

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
        hook_debug_log "allow: .fusion missing"
        exit 0
    fi

    # Check sessions.json for workflow status
    if [ ! -f "$FUSION_DIR/sessions.json" ]; then
        hook_debug_log "allow: sessions.json missing"
        exit 0
    fi

    # --- Runtime adapter ---
    # If runtime.enabled is true, prefer Rust bridge when runtime.engine=rust, else Python compat_v2.
    # Falls back to Shell logic if runtime call fails.
    if runtime_enabled_in_config; then
        hook_debug_log "runtime-adapter: enabled"
        local runtime_output

        if runtime_engine_is_rust; then
            hook_debug_log "runtime-adapter: engine=rust"
            local bridge_bin
            if bridge_bin="$(resolve_fusion_bridge_bin 2>/dev/null)"; then
                if runtime_output=$("$bridge_bin" hook stop-guard --fusion-dir "$FUSION_DIR" 2>/dev/null); then
                    local decision
                    decision=$(extract_json_field "$runtime_output" "decision")
                    [ -n "$decision" ] || decision="allow"

                    if [ "$decision" = "allow" ]; then
                        hook_debug_log "runtime-adapter: rust decision=allow"
                        exit 0
                    fi

                    hook_debug_log "runtime-adapter: rust decision=block"
                    emit_runtime_block_response "$runtime_output"
                fi
                hook_debug_log "runtime-adapter: rust bridge failed"
            else
                hook_debug_log "runtime-adapter: rust bridge missing"
            fi
        fi

        if runtime_output=$(PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" python3 -m runtime.compat_v2 stop-guard "$FUSION_DIR" 2>/dev/null); then
            local decision
            decision=$(extract_json_field "$runtime_output" "decision")
            [ -n "$decision" ] || decision="allow"

            if [ "$decision" = "allow" ]; then
                hook_debug_log "runtime-adapter: python decision=allow"
                exit 0
            fi

            hook_debug_log "runtime-adapter: python decision=block"
            emit_runtime_block_response "$runtime_output"
        fi
        hook_debug_log "runtime-adapter: failed, fallback=shell"
        # Runtime adapter failed - fall through to Shell logic
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

    # Get goal for context (sanitize to prevent JSON injection in sed fallback)
    local goal
    goal=$(json_get "$FUSION_DIR/sessions.json" "goal")
    goal=$(printf '%s' "$goal" | tr -d '"\\\t' | tr '\n' ' ')

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
            emit_block_response "$prompt" "$sys_msg"
        fi

        # Later phases without task_plan.md is an error - mark as stuck
        json_set "$FUSION_DIR/sessions.json" "status" "stuck" || true
        echo "⚠️ FUSION ERROR: task_plan.md missing in phase $current_phase" >&2
        exit 0
    fi

    # If no remaining tasks, try to inject safe_backlog tasks before allowing stop
    if [ "$total_remaining" -eq 0 ]; then
        hook_debug_log "all tasks done, checking safe_backlog injection"

        # Try to inject safe_backlog tasks if configured
        local safe_backlog_injected=false
        if command -v python3 &>/dev/null && [ -f "$SCRIPT_DIR/runtime/safe_backlog.py" ]; then
            # Check if safe_backlog injection on task exhausted is enabled
            local inject_enabled
            inject_enabled=$(PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" python3 -c "
import sys
from runtime.config import load_fusion_config
try:
    cfg = load_fusion_config('$FUSION_DIR')
    enabled = bool(cfg.get('safe_backlog', {}).get('enabled', False))
    inject = bool(cfg.get('safe_backlog', {}).get('inject_on_task_exhausted', True))
    print('true' if (enabled and inject) else 'false')
except Exception:
    print('false')
" 2>/dev/null || echo "false")

            if [ "$inject_enabled" = "true" ]; then
                hook_debug_log "safe_backlog injection enabled, generating tasks"
                # Try to generate safe_backlog tasks
                local backlog_result
                backlog_result=$(PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" python3 -c "
import sys
from runtime.safe_backlog import generate_safe_backlog
try:
    result = generate_safe_backlog('$FUSION_DIR', '.')
    if result.get('added', 0) > 0:
        print('injected')
    else:
        print('none')
except Exception as e:
    print('error')
    print(str(e), file=sys.stderr)
" 2>/dev/null || echo "error")

                if [ "$backlog_result" = "injected" ]; then
                    safe_backlog_injected=true
                    hook_debug_log "safe_backlog tasks injected, continuing workflow"

                    # Re-count tasks after injection
                    pending_count=$(grep_count "\[PENDING\]" "$FUSION_DIR/task_plan.md")
                    total_remaining=$((pending_count + in_progress_count + failed_count))

                    # Build continuation prompt with safe_backlog context
                    local prompt
                    prompt="Continue executing the Fusion workflow.

Goal: ${goal:-"(not set)"}
Phase: ${current_phase:-"EXECUTE"}
Remaining: $total_remaining tasks (safe_backlog tasks injected)

Safe backlog tasks have been automatically added to maintain continuous development.
These are low-risk quality/documentation/optimization tasks.

Instructions:
1. Read .fusion/task_plan.md
2. Find next PENDING or IN_PROGRESS task
3. Execute based on task type:
   - implementation/verification → TDD flow (RED→GREEN→REFACTOR)
   - design/documentation/configuration/research → direct execution
4. Update task status to [COMPLETED]
5. Continue until all tasks done

Only ask user if 3-Strike exhausted."

                    local sys_msg="🔄 Fusion (safe_backlog injected) | Phase: ${current_phase:-EXECUTE} | Remaining: $total_remaining"
                    emit_block_response "$prompt" "$sys_msg"
                fi
            fi
        fi

        # If safe_backlog injection failed or disabled, allow stop
        if [ "$safe_backlog_injected" = false ]; then
            hook_debug_log "allow: all tasks done, no safe_backlog injection phase=${current_phase:-unknown}"
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
    fi

    # Get next task for context (sanitize to prevent JSON injection in sed fallback)
    local next_task
    next_task=$(grep -B1 "\[PENDING\]\|\[IN_PROGRESS\]" "$FUSION_DIR/task_plan.md" 2>/dev/null | grep "### Task" | head -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//' || echo "unknown")
    next_task=$(printf '%s' "$next_task" | tr -d '"\\\t' | tr '\n' ' ')

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
                emit_block_response "$prompt" "$sys_msg"
                ;;
            "BACKOFF")
                # Warning sign - add context to prompt
                local prompt
                prompt=$(build_continuation_prompt "$goal" "$current_phase" "$next_task" "$total_remaining" "$(guardian_get '.iteration')")
                prompt="$prompt

⚠️ Warning: LoopGuardian detected potential stagnation.
Consider: trying a different approach, checking for blockers, or asking for help."
                local sys_msg="🔄 Fusion (BACKOFF) | Phase: ${current_phase:-EXECUTE} | Remaining: $total_remaining | $(guardian_get '.metrics.no_progress_rounds') no-progress rounds"
                emit_block_response "$prompt" "$sys_msg"
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
    hook_debug_log "block: phase=${current_phase:-EXECUTE} remaining=$total_remaining next=${next_task:-unknown}"
    emit_block_response "$prompt" "$sys_msg"
}

main "$@"
