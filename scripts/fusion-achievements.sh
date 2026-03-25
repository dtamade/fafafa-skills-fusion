#!/bin/bash
# fusion-achievements.sh - Thin wrapper around Rust fusion-bridge achievements
set -euo pipefail

FUSION_DIR=".fusion"
LEADERBOARD_ROOT="${FUSION_LEADERBOARD_ROOT:-$HOME/projects}"
TOP_N=10
SHOW_LOCAL=1
SHOW_LEADERBOARD=1
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
    cat <<'USAGE'
Usage: fusion-achievements.sh [options]

Options:
  --local-only            Show achievements for current workspace only
  --leaderboard-only      Show cross-project leaderboard only
  --root <path>           Leaderboard root directory (default: $FUSION_LEADERBOARD_ROOT or $HOME/projects)
  --top <n>               Number of leaderboard rows (default: 10)
  -h, --help              Show this help
USAGE
}

fail_with_usage() {
    local message="$1"
    echo "$message" >&2
    usage >&2
    exit 1
}

fail_wrapper() {
    echo "$1" >&2
    exit 127
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --local-only)
            SHOW_LOCAL=1
            SHOW_LEADERBOARD=0
            ;;
        --leaderboard-only)
            SHOW_LOCAL=0
            SHOW_LEADERBOARD=1
            ;;
        --root)
            shift
            if [ "$#" -eq 0 ] || [ -z "${1:-}" ] || [[ "${1:-}" == --* ]]; then
                fail_with_usage "Missing value for --root"
            fi
            LEADERBOARD_ROOT="$1"
            ;;
        --root=*)
            LEADERBOARD_ROOT="${1#--root=}"
            if [ -z "$LEADERBOARD_ROOT" ]; then
                fail_with_usage "Missing value for --root"
            fi
            ;;
        --top)
            shift
            if [ "$#" -eq 0 ] || [ -z "${1:-}" ] || [[ "${1:-}" == --* ]]; then
                fail_with_usage "Missing value for --top"
            fi
            TOP_N="$1"
            ;;
        --top=*)
            TOP_N="${1#--top=}"
            if [ -z "$TOP_N" ]; then
                fail_with_usage "Missing value for --top"
            fi
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            fail_with_usage "Unknown option: $1"
            ;;
    esac
    shift
done

if ! [[ "$TOP_N" =~ ^[1-9][0-9]*$ ]]; then
    fail_with_usage "--top must be a positive integer"
fi

if fusion_bridge_disabled; then
    fail_wrapper "[fusion][deps] fusion-achievements.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release"
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || \
    fail_wrapper "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release"

bridge_args=(achievements --fusion-dir "$FUSION_DIR" --root "$LEADERBOARD_ROOT" --top "$TOP_N")
if [ "$SHOW_LOCAL" -eq 1 ] && [ "$SHOW_LEADERBOARD" -eq 0 ]; then
    bridge_args+=(--local-only)
elif [ "$SHOW_LOCAL" -eq 0 ] && [ "$SHOW_LEADERBOARD" -eq 1 ]; then
    bridge_args+=(--leaderboard-only)
fi

"$bridge_bin" "${bridge_args[@]}"
