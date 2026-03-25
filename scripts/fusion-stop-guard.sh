#!/bin/bash
# fusion-stop-guard.sh - Stop hook to prevent premature stopping
#
# CRITICAL: This is the ONLY stop hook. Do not add checkpoint logic here.
#
# Exit codes:
#   0 = Allow stop (all tasks complete or workflow not active)
#   2 = Block stop (tasks remaining) - compatibility behavior
#
# Advanced API (stdout JSON):
#   When blocking, outputs JSON to stdout:
#   {"decision":"block","reason":"<prompt>","systemMessage":"<status>"}
#
# Safety features:
#   - Reentry protection via lock directory (only owner cleans up)
#   - Atomic state operations
#   - Stale lock detection and cleanup
#   - Simple shell fallback block-count limit

set -euo pipefail

FUSION_DIR=".fusion"
LOCK_STALE_SECONDS=300     # Consider lock stale after 5 minutes
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"
source "$SCRIPT_DIR/lib/fusion-hook-common.sh"
source "$SCRIPT_DIR/lib/fusion-json.sh"
source "$SCRIPT_DIR/lib/fusion-task-plan.sh"
source "$SCRIPT_DIR/lib/fusion-hook-adapter.sh"
source "$SCRIPT_DIR/lib/fusion-stop-guard-common.sh"
source "$SCRIPT_DIR/lib/fusion-stop-guard-fallback.sh"
fusion_hook_init_debug stop "$FUSION_DIR"


# Use the same state lock as pause/cancel/resume for unified protection
STATE_LOCK="${FUSION_DIR}/.state.lock"

# Track if we acquired the lock (only owner should clean up)
LOCK_ACQUIRED=false

# Read hook input from stdin (advanced stop hook API)
fusion_hook_consume_stdin
hook_debug_log "invoked: mode=${FUSION_STOP_HOOK_MODE:-auto} cwd=$(pwd)"

# Emergency shell fallback uses only a simple block-count limit.
MAX_CONSECUTIVE_BLOCKS=50

# Cleanup lock on exit - ONLY if we acquired it
cleanup() {
    stop_guard_cleanup_lock "$STATE_LOCK" "$LOCK_ACQUIRED"
}
trap cleanup EXIT

# Main logic
main() {
    if ! fusion_hook_require_session_context "$FUSION_DIR" "allow"; then
        exit 0
    fi

    # Preserve shell-level lock contention semantics before bridge fallback.
    # Stop should not bypass an active state operation just because Rust bridge is available.
    stop_guard_cleanup_stale_lock "$STATE_LOCK" "$LOCK_STALE_SECONDS"

    if [ -d "$STATE_LOCK" ] || [ -f "$STATE_LOCK" ]; then
        local lock_reason
        lock_reason="State operation already in progress. Continue executing the Fusion workflow and retry stop."
        local lock_sys_msg
        lock_sys_msg="🔒 Fusion state operation in progress; retry stop after the current state update finishes"
        emit_block_response "$lock_reason" "$lock_sys_msg"
    fi

    if fusion_should_prefer_bridge "$FUSION_DIR"; then
        hook_debug_log "runtime-adapter: bridge preferred"
        local runtime_output
        runtime_output="$(fusion_hook_run_bridge stop-guard "$FUSION_DIR" "$SCRIPT_DIR")"
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

    # --- Runtime adapter ---
    # Prefer Rust bridge whenever it is available, even if runtime.enabled=false.
    # Stale non-rust runtime.engine values are normalized to rust on the live path.
    # If Rust bridge is unavailable or disabled, stop-guard now falls back directly to the minimal shell path.
    if try_stop_guard_runtime_adapter "$FUSION_DIR" "$SCRIPT_DIR"; then
        exit 0
    fi
    # Runtime adapter failed or disabled - fall through to Shell logic

    fusion_stop_guard_shell_fallback

}

main "$@"
