#!/bin/bash
# loop-guardian.sh - LoopGuardian: Intelligent anti-deadloop protection
#
# Usage:
#   source scripts/loop-guardian.sh
#   if guardian_init; then
#       guardian_record_iteration "$phase" "$task" "$error"
#       decision=$(guardian_evaluate)
#   fi
#
# Decisions:
#   CONTINUE      - Normal execution, proceed
#   BACKOFF       - Slow down, add delay before next iteration
#   ESCALATE      - Ask user for guidance
#   ABORT_STUCK   - Mark as stuck and stop
#
# IMPORTANT: Uses shell + awk only. If awk is not available,
#            GUARDIAN_JQ_AVAILABLE=false and callers should use fallback.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FUSION_DIR="${FUSION_DIR:-.fusion}"
LOOP_CONTEXT_FILE="${FUSION_DIR}/loop_context.json"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"
source "$SCRIPT_DIR/lib/fusion-json.sh"
source "$SCRIPT_DIR/lib/fusion-task-plan.sh"
source "$SCRIPT_DIR/lib/fusion-loop-guardian-core.sh"

if command -v awk >/dev/null 2>&1; then
    GUARDIAN_RUNTIME_AVAILABLE=true
else
    GUARDIAN_RUNTIME_AVAILABLE=false
fi
# Backward-compatible variable name used by some docs/tests.
GUARDIAN_JQ_AVAILABLE="$GUARDIAN_RUNTIME_AVAILABLE"

# Configuration defaults
GUARDIAN_MAX_ITERATIONS="${GUARDIAN_MAX_ITERATIONS:-50}"
GUARDIAN_MAX_NO_PROGRESS="${GUARDIAN_MAX_NO_PROGRESS:-6}"
GUARDIAN_MAX_SAME_ACTION="${GUARDIAN_MAX_SAME_ACTION:-3}"
GUARDIAN_MAX_SAME_ERROR="${GUARDIAN_MAX_SAME_ERROR:-3}"
GUARDIAN_MAX_STATE_VISITS="${GUARDIAN_MAX_STATE_VISITS:-8}"
GUARDIAN_MAX_WALL_TIME_MS="${GUARDIAN_MAX_WALL_TIME_MS:-7200000}"  # 2 hours
GUARDIAN_BACKOFF_THRESHOLD="${GUARDIAN_BACKOFF_THRESHOLD:-3}"

guardian_bridge_loop_context_inspect() {
    local subcommand="$1"
    shift

    fusion_bridge_disabled && return 1
    command -v resolve_fusion_bridge_bin >/dev/null 2>&1 || return 1

    local bridge_bin
    bridge_bin="$(resolve_fusion_bridge_bin "${SCRIPT_DIR:-$(pwd)}")" || return 1

    "$bridge_bin" inspect loop-context --file "$LOOP_CONTEXT_FILE" "$subcommand" "$@"
}

guardian_bridge_loop_guardian_available() {
    fusion_bridge_disabled && return 1
    command -v resolve_fusion_bridge_bin >/dev/null 2>&1 || return 1

    local bridge_bin
    bridge_bin="$(resolve_fusion_bridge_bin "${SCRIPT_DIR:-$(pwd)}")" || return 1
    "$bridge_bin" loop-guardian --help >/dev/null 2>&1
}

guardian_bridge_loop_guardian() {
    local subcommand="$1"
    shift

    local bridge_bin
    bridge_bin="$(resolve_fusion_bridge_bin "${SCRIPT_DIR:-$(pwd)}")" || return 1
    "$bridge_bin" loop-guardian "$subcommand" --fusion-dir "$FUSION_DIR" "$@"
}

guardian_load_config
source "$SCRIPT_DIR/lib/fusion-loop-guardian-io.sh"
source "$SCRIPT_DIR/lib/fusion-loop-guardian-context.sh"

if guardian_bridge_loop_guardian_available; then
    guardian_init() {
        guardian_bridge_loop_guardian init
    }

    guardian_record_iteration() {
        guardian_bridge_loop_guardian record "${1:-EXECUTE}" "${2:-unknown}" "${3:-}"
    }

    guardian_evaluate() {
        guardian_bridge_loop_guardian evaluate
    }

    guardian_status() {
        guardian_bridge_loop_guardian status
    }

    guardian_reset() {
        guardian_bridge_loop_guardian reset
    }
fi
