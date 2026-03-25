#!/bin/bash

guardian_backend_available() {
    [ "${GUARDIAN_RUNTIME_AVAILABLE:-false}" = true ]
}

guardian_bridge_loop_guardian_config_field() {
    local field="$1"

    fusion_bridge_disabled && return 1
    command -v resolve_fusion_bridge_bin >/dev/null 2>&1 || return 1

    local bridge_bin
    bridge_bin="$(resolve_fusion_bridge_bin "${SCRIPT_DIR:-$(pwd)}")" || return 1

    "$bridge_bin" inspect loop-guardian-config --fusion-dir "$FUSION_DIR" --field "$field"
}

guardian_load_config() {
    local config_file="${FUSION_DIR}/config.yaml"
    [ -f "$config_file" ] || return 0

    local val
    val=$(guardian_bridge_loop_guardian_config_field "max_iterations" 2>/dev/null || true)
    if [ -z "$val" ]; then
        val=$(grep -E '^[[:space:]]*max_iterations:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    fi
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_ITERATIONS="$val"

    val=$(guardian_bridge_loop_guardian_config_field "max_no_progress" 2>/dev/null || true)
    if [ -z "$val" ]; then
        val=$(grep -E '^[[:space:]]*max_no_progress:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    fi
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_NO_PROGRESS="$val"

    val=$(guardian_bridge_loop_guardian_config_field "max_same_action" 2>/dev/null || true)
    if [ -z "$val" ]; then
        val=$(grep -E '^[[:space:]]*max_same_action:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    fi
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_SAME_ACTION="$val"

    val=$(guardian_bridge_loop_guardian_config_field "max_same_error" 2>/dev/null || true)
    if [ -z "$val" ]; then
        val=$(grep -E '^[[:space:]]*max_same_error:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    fi
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_SAME_ERROR="$val"

    val=$(guardian_bridge_loop_guardian_config_field "max_state_visits" 2>/dev/null || true)
    if [ -z "$val" ]; then
        val=$(grep -E '^[[:space:]]*max_state_visits:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    fi
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_STATE_VISITS="$val"

    val=$(guardian_bridge_loop_guardian_config_field "max_wall_time_ms" 2>/dev/null || true)
    if [ -z "$val" ]; then
        val=$(grep -E '^[[:space:]]*max_wall_time_ms:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    fi
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_WALL_TIME_MS="$val"

    val=$(guardian_bridge_loop_guardian_config_field "backoff_threshold" 2>/dev/null || true)
    if [ -z "$val" ]; then
        val=$(grep -E '^[[:space:]]*backoff_threshold:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    fi
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_BACKOFF_THRESHOLD="$val"

    return 0
}

get_timestamp_ms() {
    local ts
    ts=$(date +%s%3N 2>/dev/null)
    if [[ "$ts" =~ ^[0-9]+$ ]]; then
        echo "$ts"
    else
        echo "$(date +%s)000"
    fi
}

compute_md5() {
    local input="$1"
    if command -v md5sum &>/dev/null; then
        printf '%s' "$input" | md5sum | cut -d' ' -f1
    elif command -v md5 &>/dev/null; then
        printf '%s' "$input" | md5 -q
    else
        printf '%s' "$input"
    fi
}

guardian_json_escape() {
    local value="$1"
    value=${value//\\/\\\\}
    value=${value//"/\\"}
    value=${value//$'\n'/\\n}
    value=${value//$'\r'/\\r}
    value=${value//$'\t'/\\t}
    printf '%s' "$value"
}

guardian_json_string_or_null() {
    local value="${1:-}"
    if [ -n "$value" ]; then
        printf '"%s"' "$(guardian_json_escape "$value")"
    else
        printf 'null'
    fi
}

guardian_numeric_or_default() {
    local value="${1:-}"
    local default_value="${2:-0}"
    if [[ "$value" =~ ^[0-9]+$ ]]; then
        printf '%s' "$value"
    else
        printf '%s' "$default_value"
    fi
}
