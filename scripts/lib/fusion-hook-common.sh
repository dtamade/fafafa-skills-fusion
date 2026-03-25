#!/bin/bash

fusion_hook_init_debug() {
    FUSION_HOOK_DEBUG_NAME="$1"
    FUSION_HOOK_DEBUG_DIR="$2"
    HOOK_DEBUG=false
    if fusion_is_truthy "${FUSION_HOOK_DEBUG:-}" || [ -f "$FUSION_HOOK_DEBUG_DIR/.hook_debug" ]; then
        HOOK_DEBUG=true
    fi
}

hook_debug_log() {
    [ "${HOOK_DEBUG:-false}" = true ] || return 0
    local message="$1"
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local line="[fusion][hook-debug][${FUSION_HOOK_DEBUG_NAME:-hook}][$ts] $message"
    echo "$line" >&2
    if [ -d "${FUSION_HOOK_DEBUG_DIR:-.fusion}" ]; then
        echo "$line" >> "${FUSION_HOOK_DEBUG_DIR}/hook-debug.log" 2>/dev/null || true
    fi
}

fusion_hook_require_session_context() {
    local fusion_dir="$1"
    local verb="${2:-skip}"

    if [ ! -d "$fusion_dir" ]; then
        hook_debug_log "$verb: .fusion missing"
        return 1
    fi

    if [ ! -f "$fusion_dir/sessions.json" ]; then
        hook_debug_log "$verb: sessions.json missing"
        return 1
    fi

    return 0
}

fusion_hook_consume_stdin() {
    HOOK_INPUT=$(cat)
}
