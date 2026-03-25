#!/bin/bash
# fusion-status.sh - Thin wrapper around Rust fusion-bridge status
set -euo pipefail

FUSION_DIR=".fusion"
JSON_MODE=false
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
    cat <<'USAGE'
Usage: fusion-status.sh [--json]
USAGE
}

emit_json_error() {
    local reason="$1"
    reason=${reason//\\/\\\\}
    reason=${reason//"/\\"}
    reason=${reason//$'\n'/\\n}
    reason=${reason//$'\r'/\\r}
    reason=${reason//$'\t'/\\t}
    printf '{"result":"error","status":"","phase":"","reason":"%s"}\n' "$reason"
}

fail_wrapper() {
    local reason="$1"
    if [ "$JSON_MODE" = true ]; then
        emit_json_error "$reason"
    else
        echo "$reason" >&2
    fi
    exit 127
}

for arg in "$@"; do
    case "$arg" in
        -h|--help)
            usage
            exit 0
            ;;
        --json)
            JSON_MODE=true
            ;;
        *)
            if [ "$JSON_MODE" = true ]; then
                emit_json_error "Unknown option: $arg"
            else
                echo "Unknown option: $arg" >&2
                usage >&2
            fi
            exit 1
            ;;
    esac
done

if fusion_bridge_disabled; then
    fail_wrapper "[fusion][deps] fusion-status.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release"
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || \
    fail_wrapper "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release"

bridge_args=(status --fusion-dir "$FUSION_DIR")
if [ "$JSON_MODE" = true ]; then
    bridge_args+=(--json)
fi

"$bridge_bin" "${bridge_args[@]}"
