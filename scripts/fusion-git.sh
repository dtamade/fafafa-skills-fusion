#!/bin/bash
# fusion-git.sh - Thin wrapper around Rust fusion-bridge git
set -euo pipefail

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
    echo "Usage: fusion-git.sh {status|create-branch|commit|branch|changes|diff|cleanup}"
}

if [ "$#" -eq 0 ]; then
    ACTION="status"
else
    ACTION="$1"
    shift
fi

case "$ACTION" in
    -h|--help)
        usage
        exit 0
        ;;
    create-branch)
        if [ "$#" -ne 1 ]; then
            echo "Usage: fusion-git.sh create-branch <goal-slug>" >&2
            exit 1
        fi
        ;;
    commit)
        if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
            echo "Usage: fusion-git.sh commit <message> [task_id]" >&2
            exit 1
        fi
        ;;
    branch|changes|diff|status)
        if [ "$#" -ne 0 ]; then
            echo "Usage: fusion-git.sh $ACTION" >&2
            exit 1
        fi
        ;;
    cleanup)
        if [ "$#" -gt 1 ]; then
            echo "Usage: fusion-git.sh cleanup [original_branch]" >&2
            exit 1
        fi
        ;;
    *)
        echo "Unknown action: $ACTION" >&2
        usage >&2
        exit 1
        ;;
esac

if fusion_bridge_disabled; then
    echo "[fusion][deps] fusion-git.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release" >&2
    exit 127
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || {
    echo "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release" >&2
    exit 127
}

"$bridge_bin" git "$ACTION" "$@"
