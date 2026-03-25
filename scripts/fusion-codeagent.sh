#!/bin/bash
# fusion-codeagent.sh - Thin wrapper around Rust fusion-bridge codeagent

set -euo pipefail

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
    cat <<'USAGE'
Usage: fusion-codeagent.sh [phase] [prompt...]

Examples:
  fusion-codeagent.sh EXECUTE
  fusion-codeagent.sh REVIEW
  fusion-codeagent.sh
USAGE
}

phase=""
prompt_args=()

case "${1:-}" in
    -h|--help)
        usage
        exit 0
        ;;
    --)
        shift || true
        ;;
    -*)
        echo "Unknown option: $1" >&2
        usage >&2
        exit 1
        ;;
    "")
        ;;
    *)
        phase="$1"
        shift || true
        ;;
esac

if [ "$#" -gt 0 ]; then
    prompt_args=("$@")
fi

if fusion_bridge_disabled; then
    echo "[fusion][deps] fusion-codeagent.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release" >&2
    exit 127
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || {
    echo "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release" >&2
    exit 127
}

bridge_args=(codeagent --fusion-dir "$FUSION_DIR")
if [ -n "$phase" ]; then
    bridge_args+=("$phase")
fi
if [ "${#prompt_args[@]}" -gt 0 ]; then
    bridge_args+=("${prompt_args[@]}")
fi

FUSION_CODEAGENT_REPORT_SOURCE="fusion-codeagent.sh" "$bridge_bin" "${bridge_args[@]}"
