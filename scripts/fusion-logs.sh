#!/bin/bash
# fusion-logs.sh - Thin wrapper around Rust fusion-bridge logs
set -euo pipefail

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINES="50"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
    echo "Usage: fusion-logs.sh [lines]"
}

if [ "$#" -gt 1 ]; then
    echo "Too many arguments" >&2
    usage >&2
    exit 1
fi

case "${1:-}" in
    -h|--help)
        usage
        exit 0
        ;;
    --*)
        echo "Unknown option: $1" >&2
        usage >&2
        exit 1
        ;;
    "")
        ;;
    *)
        LINES="$1"
        ;;
esac

if ! [[ "$LINES" =~ ^[1-9][0-9]*$ ]]; then
    echo "LINES must be a positive integer" >&2
    usage >&2
    exit 1
fi

if fusion_bridge_disabled; then
    echo "[fusion][deps] fusion-logs.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release" >&2
    exit 127
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || {
    echo "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release" >&2
    exit 127
}

"$bridge_bin" logs "$LINES" --fusion-dir "$FUSION_DIR"
