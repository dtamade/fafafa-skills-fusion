#!/bin/bash
# fusion-init.sh - Thin wrapper around Rust fusion-bridge init
set -euo pipefail

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_DIR="$(dirname "$SCRIPT_DIR")/templates"
ENGINE="rust"
JSON_MODE=false

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
    echo "Usage: fusion-init.sh [--engine rust] [--json]"
}

json_escape() {
    local value="$1"
    value=${value//\\/\\\\}
    value=${value//"/\\"}
    value=${value//$'\n'/\\n}
    value=${value//$'\r'/\\r}
    value=${value//$'\t'/\\t}
    printf '%s' "$value"
}

fusion_dir_abs() {
    if [ -d "$FUSION_DIR" ]; then
        (
            cd "$FUSION_DIR" 2>/dev/null && pwd
        ) || printf '%s' "$FUSION_DIR"
        return 0
    fi

    printf '%s' "$FUSION_DIR"
}

emit_json() {
    local result="$1"
    local reason="${2:-}"
    local abs_dir
    abs_dir="$(fusion_dir_abs)"

    if [ -n "$reason" ]; then
        printf '{"result":"%s","engine":"%s","fusion_dir":"%s","reason":"%s"}\n' \
            "$(json_escape "$result")" \
            "$(json_escape "$ENGINE")" \
            "$(json_escape "$abs_dir")" \
            "$(json_escape "$reason")"
    else
        printf '{"result":"%s","engine":"%s","fusion_dir":"%s"}\n' \
            "$(json_escape "$result")" \
            "$(json_escape "$ENGINE")" \
            "$(json_escape "$abs_dir")"
    fi
}

fail_with_message() {
    local reason="$1"
    local exit_code="${2:-1}"
    if [ "$JSON_MODE" = true ]; then
        emit_json "error" "$reason"
    else
        echo "$reason" >&2
    fi
    exit "$exit_code"
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --json)
            JSON_MODE=true
            ;;
        --engine)
            shift
            if [ "$#" -eq 0 ]; then
                fail_with_message "Missing value for --engine"
            fi
            ENGINE="$1"
            ;;
        --engine=*)
            ENGINE="${1#--engine=}"
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            if [ "$JSON_MODE" = true ]; then
                emit_json "error" "Unknown option: $1"
            else
                echo "Unknown option: $1" >&2
                usage >&2
            fi
            exit 1
            ;;
    esac
    shift
done

case "$ENGINE" in
    rust)
        ;;
    *)
        fail_with_message "Invalid engine: $ENGINE (expected: rust)"
        ;;
esac

if fusion_bridge_disabled; then
    fail_with_message "[fusion][deps] fusion-init.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release" 127
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || \
    fail_with_message "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release" 127

bridge_args=(init --fusion-dir "$FUSION_DIR" --templates-dir "$TEMPLATE_DIR" --engine "$ENGINE")

if [ "$JSON_MODE" = true ]; then
    stderr_file="$(mktemp)"
    if "$bridge_bin" "${bridge_args[@]}" >/dev/null 2>"$stderr_file"; then
        rm -f "$stderr_file"
        emit_json "ok"
        exit 0
    fi
    reason="$(cat "$stderr_file" 2>/dev/null || true)"
    rm -f "$stderr_file"
    reason="${reason#"${reason%%[![:space:]]*}"}"
    reason="${reason%"${reason##*[![:space:]]}"}"
    [ -n "$reason" ] || reason="fusion-bridge init failed"
    emit_json "error" "$reason"
    exit 1
fi

"$bridge_bin" "${bridge_args[@]}"
