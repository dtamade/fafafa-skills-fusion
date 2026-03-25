#!/bin/bash

fusion_hook_run_bridge() {
    local hook_name="$1"
    local fusion_dir="$2"
    local script_dir="$3"

    local bridge_bin
    bridge_bin="$(resolve_fusion_bridge_bin "$script_dir")" || return 1

    "$bridge_bin" hook "$hook_name" --fusion-dir "$fusion_dir"
}

fusion_hook_try_runtime_adapter() {
    local hook_name="$1"
    local fusion_dir="$2"
    local script_dir="$3"
    local failure_log="${4:-runtime-adapter: bridge unavailable, fallback=shell}"

    if runtime_enabled_in_config "$fusion_dir"; then
        hook_debug_log "runtime-adapter: enabled"
    else
        hook_debug_log "runtime-adapter: disabled"
    fi

    if fusion_should_prefer_bridge "$fusion_dir"; then
        hook_debug_log "runtime-adapter: bridge preferred"
        local output
        if output=$(fusion_hook_run_bridge "$hook_name" "$fusion_dir" "$script_dir" 2>/dev/null); then
            hook_debug_log "runtime-adapter: rust bridge ok"
            [ -n "$output" ] && echo "$output"
            hook_debug_log "runtime-adapter: done"
            return 0
        fi
        hook_debug_log "runtime-adapter: rust bridge unavailable"
    elif fusion_bridge_disabled; then
        hook_debug_log "runtime-adapter: rust bridge disabled"
    fi

    hook_debug_log "$failure_log"
    return 1
}

output_block_json() {
    local reason="$1"
    local system_msg="$2"
    local escaped_reason escaped_msg

    escaped_reason="$reason"
    escaped_reason=${escaped_reason//\\/\\\\}
    escaped_reason=${escaped_reason//"/\\"}
    escaped_reason=${escaped_reason//$'\n'/\\n}
    escaped_reason=${escaped_reason//$'\r'/\\r}
    escaped_reason=${escaped_reason//$'\t'/\\t}

    escaped_msg="$system_msg"
    escaped_msg=${escaped_msg//\\/\\\\}
    escaped_msg=${escaped_msg//"/\\"}
    escaped_msg=${escaped_msg//$'\n'/\\n}
    escaped_msg=${escaped_msg//$'\r'/\\r}
    escaped_msg=${escaped_msg//$'\t'/\\t}

    printf '{"decision":"block","reason":"%s","systemMessage":"%s"}\n' "$escaped_reason" "$escaped_msg"
}

stop_hook_supports_json_block() {
    local mode
    mode="${FUSION_STOP_HOOK_MODE:-auto}"

    case "$mode" in
        json|modern|structured)
            return 0
            ;;
        legacy|exit2)
            return 1
            ;;
        auto|"")
            return 0
            ;;
        *)
            echo "[fusion] unknown FUSION_STOP_HOOK_MODE='$mode', defaulting to structured" >&2
            return 0
            ;;
    esac
}

extract_json_field() {
    local json_payload="$1"
    local field="$2"
    FUSION_BRIDGE_DISABLE=1 json_get_string_from_text "$json_payload" "$field"
}

emit_block_response() {
    local reason="$1"
    local system_msg="$2"

    if stop_hook_supports_json_block; then
        hook_debug_log "decision=block mode=structured msg=${system_msg}"
        output_block_json "$reason" "$system_msg"
        exit 0
    fi

    hook_debug_log "decision=block mode=legacy msg=${system_msg}"
    echo "[fusion] stop blocked: $system_msg" >&2
    echo "$reason" >&2
    exit 2
}

emit_runtime_block_response() {
    local runtime_json="$1"

    local reason
    reason=$(extract_json_field "$runtime_json" "reason")
    local sys_msg
    sys_msg=$(extract_json_field "$runtime_json" "systemMessage")

    [ -n "$reason" ] || reason="Continue executing the Fusion workflow."
    [ -n "$sys_msg" ] || sys_msg="🔄 Fusion | Workflow in progress"

    emit_block_response "$reason" "$sys_msg"
}

try_stop_guard_runtime_adapter() {
    local fusion_dir="$1"
    local script_dir="$2"

    if runtime_enabled_in_config "$fusion_dir"; then
        hook_debug_log "runtime-adapter: enabled"
    else
        hook_debug_log "runtime-adapter: disabled"
    fi

    local runtime_output
    if fusion_should_prefer_bridge "$fusion_dir"; then
        hook_debug_log "runtime-adapter: bridge preferred"
        if runtime_output=$(fusion_hook_run_bridge stop-guard "$fusion_dir" "$script_dir" 2>/dev/null); then
            local decision
            decision=$(extract_json_field "$runtime_output" "decision")
            [ -n "$decision" ] || decision="allow"

            if [ "$decision" = "allow" ]; then
                hook_debug_log "runtime-adapter: rust decision=allow"
                return 0
            fi

            hook_debug_log "runtime-adapter: rust decision=block"
            emit_runtime_block_response "$runtime_output"
        fi
        hook_debug_log "runtime-adapter: rust bridge unavailable"
    elif fusion_bridge_disabled; then
        hook_debug_log "runtime-adapter: rust bridge disabled"
    fi

    hook_debug_log "runtime-adapter: stop-guard bridge unavailable, fallback=shell"
    return 1
}
