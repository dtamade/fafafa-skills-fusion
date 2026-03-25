#!/bin/bash
# fusion-pretool.sh - Attention Injection Engine
#
# PreToolUse hook: runs BEFORE every Write/Edit/Bash/Read/Glob/Grep call.
# Outputs a compact context summary to keep Claude focused on the current task.
#
# Design constraints:
#   - Must execute in < 50ms (no jq, pure grep/awk)
#   - Non-invasive: silent exit if no active Fusion workflow
#   - Fault-tolerant: all operations || true, never blocks Claude
#
# Output goes to stdout and appears in Claude's context window,
# implementing the Manus "attention manipulation through recitation" pattern.

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"
source "$SCRIPT_DIR/lib/fusion-hook-common.sh"
source "$SCRIPT_DIR/lib/fusion-json.sh"
source "$SCRIPT_DIR/lib/fusion-task-plan.sh"
source "$SCRIPT_DIR/lib/fusion-hook-adapter.sh"
source "$SCRIPT_DIR/lib/fusion-pretool-fallback.sh"
fusion_hook_init_debug pretool "$FUSION_DIR"


if ! fusion_hook_require_session_context "$FUSION_DIR" "skip"; then
    exit 0
fi

hook_debug_log "invoked: cwd=$(pwd)"

# Read hook input from stdin (PreToolUse hook protocol)
# Input contains: {"tool_name": "...", "tool_input": {...}}
fusion_hook_consume_stdin

if fusion_should_prefer_bridge "$FUSION_DIR"; then
    hook_debug_log "runtime-adapter: bridge preferred"
    fusion_hook_run_bridge pretool "$FUSION_DIR" "$SCRIPT_DIR"
    exit $?
fi

# --- Runtime adapter ---
if fusion_hook_try_runtime_adapter pretool "$FUSION_DIR" "$SCRIPT_DIR"; then
    exit 0
fi
# Runtime adapter failed - fall through to Shell logic
fusion_pretool_shell_fallback "$FUSION_DIR"
exit 0
