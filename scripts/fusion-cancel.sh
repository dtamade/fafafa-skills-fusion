#!/bin/bash
# fusion-cancel.sh - Thin wrapper around Rust fusion-bridge cancel
set -euo pipefail

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
    cat <<'USAGE'
Usage: fusion-cancel.sh
USAGE
}

fail_wrapper() {
    echo "$1" >&2
    exit 127
}

if [ "$#" -gt 0 ]; then
    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
fi

if fusion_bridge_disabled; then
    fail_wrapper "[fusion][deps] fusion-cancel.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release"
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || \
    fail_wrapper "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release"

"$bridge_bin" cancel --fusion-dir "$FUSION_DIR"
