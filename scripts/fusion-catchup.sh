#!/bin/bash
# fusion-catchup.sh - Session recovery via Rust bridge

set -euo pipefail

FUSION_DIR=".fusion"
PROJECT_PATH=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_MANIFEST="$SCRIPT_DIR/../rust/Cargo.toml"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
    cat <<'USAGE'
Usage: fusion-catchup.sh [options] [project-path]

Options:
  --fusion-dir <path>     Fusion working directory (default: .fusion)
  --project-path <path>   Project root to recover (default: current directory)
  -h, --help              Show this help
USAGE
}

fail_with_usage() {
    local message="$1"
    echo "$message" >&2
    usage >&2
    exit 1
}

while [ $# -gt 0 ]; do
    case "$1" in
        --fusion-dir)
            [ $# -ge 2 ] || fail_with_usage "Missing value for --fusion-dir"
            FUSION_DIR="$2"
            shift 2
            ;;
        --project-path)
            [ $# -ge 2 ] || fail_with_usage "Missing value for --project-path"
            PROJECT_PATH="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            break
            ;;
        -*)
            fail_with_usage "Unknown option: $1"
            ;;
        *)
            if [ -n "$PROJECT_PATH" ]; then
                fail_with_usage "Unexpected extra argument: $1"
            fi
            PROJECT_PATH="$1"
            shift
            ;;
    esac
done

if [ $# -gt 0 ]; then
    fail_with_usage "Unexpected extra arguments: $*"
fi

if [ -z "$PROJECT_PATH" ]; then
    PROJECT_PATH="$PWD"
fi

if bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")"; then
    exec "$bridge_bin" catchup --fusion-dir "$FUSION_DIR" --project-path "$PROJECT_PATH"
fi

if command -v cargo >/dev/null 2>&1 && [ -f "$RUST_MANIFEST" ]; then
    exec env CARGO_HOME="${CARGO_HOME:-$SCRIPT_DIR/../rust/.cargo-codex}" \
        cargo run --manifest-path "$RUST_MANIFEST" --release -q -p fusion-cli -- \
        catchup --fusion-dir "$FUSION_DIR" --project-path "$PROJECT_PATH"
fi

echo "fusion-catchup.sh requires the Rust bridge binary or cargo --release to be available." >&2
exit 1
