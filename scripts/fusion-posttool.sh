#!/bin/bash
# fusion-posttool.sh - Progress Monitor
#
# PostToolUse hook: runs AFTER every Write/Edit call.
# Detects progress changes and provides structured status updates.
#
# Design constraints:
#   - Must execute quickly (no jq, pure grep)
#   - Non-invasive: silent exit if no active Fusion workflow
#   - Fault-tolerant: all operations || true, never blocks Claude
#   - Maintains a snapshot file (.fusion/.progress_snapshot) for diff detection

FUSION_DIR=".fusion"
SNAPSHOT_FILE="$FUSION_DIR/.progress_snapshot"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"
source "$SCRIPT_DIR/lib/fusion-hook-common.sh"
source "$SCRIPT_DIR/lib/fusion-json.sh"
source "$SCRIPT_DIR/lib/fusion-task-plan.sh"
source "$SCRIPT_DIR/lib/fusion-hook-adapter.sh"
source "$SCRIPT_DIR/lib/fusion-posttool-fallback.sh"
fusion_hook_init_debug posttool "$FUSION_DIR"


if ! fusion_hook_require_session_context "$FUSION_DIR" "skip"; then
    exit 0
fi

hook_debug_log "invoked: cwd=$(pwd)"

# Read hook input from stdin (PostToolUse hook protocol)
# Input contains: {"tool_name": "...", "tool_input": {...}}
fusion_hook_consume_stdin

if fusion_should_prefer_bridge "$FUSION_DIR"; then
    hook_debug_log "runtime-adapter: bridge preferred"
    fusion_hook_run_bridge posttool "$FUSION_DIR" "$SCRIPT_DIR"
    exit $?
fi

# --- Runtime adapter ---
# Prefer Rust bridge whenever it is available, even if runtime.enabled=false.
# Stale non-rust runtime.engine values are normalized to rust on the live path.
# If Rust bridge is unavailable or disabled, fall back directly to the minimal shell path.
# runtime-only rich side effects now belong to the Rust bridge, not the shell adapter.
if fusion_hook_try_runtime_adapter posttool "$FUSION_DIR" "$SCRIPT_DIR" "runtime-adapter: bridge unavailable, fallback=shell"; then
    exit 0
fi
# Runtime adapter failed - fall through to Shell logic
fusion_posttool_shell_fallback "$FUSION_DIR"
exit 0
