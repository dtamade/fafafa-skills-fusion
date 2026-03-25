#!/bin/bash

fusion_is_truthy() {
    case "$(printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]')" in
        1|true|yes|on)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

fusion_bridge_disabled() {
    fusion_is_truthy "${FUSION_BRIDGE_DISABLE:-}"
}

fusion_bridge_runtime_config_field() {
    local fusion_dir="$1"
    local field="$2"

    fusion_bridge_disabled && return 1
    command -v resolve_fusion_bridge_bin >/dev/null 2>&1 || return 1

    local bridge_bin
    bridge_bin="$(resolve_fusion_bridge_bin "${SCRIPT_DIR:-$(pwd)}")" || return 1

    "$bridge_bin" inspect runtime-config --fusion-dir "$fusion_dir" --field "$field"
}

fusion_runtime_yaml_scalar() {
    local fusion_dir="$1"
    local field="$2"

    [ -f "$fusion_dir/config.yaml" ] || return 1

    awk -v wanted="$field" '
    BEGIN { in_runtime = 0 }
    /^[[:space:]]*#/ { next }
    /^[^[:space:]#][^:]*:[[:space:]]*$/ {
        key = $0
        sub(/[[:space:]]*:[[:space:]]*$/, "", key)
        in_runtime = (key == "runtime")
        next
    }
    in_runtime {
        pattern = "^[[:space:]]+" wanted ":[[:space:]]*"
        if ($0 ~ pattern) {
            value = $0
            sub(pattern, "", value)
            sub(/[[:space:]]*#.*/, "", value)
            gsub(/["[:space:]]/, "", value)
            print value
            exit
        }
    }
    /^[^[:space:]#]/ {
        in_runtime = 0
    }
    ' "$fusion_dir/config.yaml" 2>/dev/null
}

fusion_runtime_enabled() {
    local fusion_dir="$1"
    local bridge_value=""
    bridge_value=$(fusion_bridge_runtime_config_field "$fusion_dir" "enabled" 2>/dev/null) || bridge_value=""
    if [ "$bridge_value" = "true" ]; then
        return 0
    elif [ "$bridge_value" = "false" ]; then
        return 1
    fi

    [ -f "$fusion_dir/config.yaml" ] || return 1

    awk '
    BEGIN { in_runtime = 0; found = 0 }
    /^[[:space:]]*#/ { next }
    /^[^[:space:]#][^:]*:[[:space:]]*$/ {
        key = $0
        sub(/[[:space:]]*:[[:space:]]*$/, "", key)
        in_runtime = (key == "runtime")
        next
    }
    in_runtime && /^[[:space:]]+enabled:[[:space:]]*/ {
        value = $0
        sub(/^[[:space:]]+enabled:[[:space:]]*/, "", value)
        sub(/[[:space:]]*#.*/, "", value)
        gsub(/[[:space:]\"]/, "", value)
        if (tolower(value) == "true") {
            found = 1
        }
        exit
    }
    /^[^[:space:]#]/ {
        in_runtime = 0
    }
    END { exit(found ? 0 : 1) }
    ' "$fusion_dir/config.yaml" 2>/dev/null
}

fusion_runtime_engine() {
    local fusion_dir="$1"
    local bridge_value=""
    bridge_value=$(fusion_bridge_runtime_config_field "$fusion_dir" "engine" 2>/dev/null) || bridge_value=""
    case "$(printf '%s' "${bridge_value:-}" | tr '[:upper:]' '[:lower:]')" in
        rust)
            echo "rust"
            return 0
            ;;
    esac

    if [ ! -f "$fusion_dir/config.yaml" ]; then
        echo "rust"
        return 0
    fi

    local raw normalized
    raw=$(fusion_runtime_yaml_scalar "$fusion_dir" "engine")
    normalized=$(printf '%s' "${raw:-rust}" | tr '[:upper:]' '[:lower:]')
    case "$normalized" in
        rust)
            echo "rust"
            ;;
        *)
            echo "rust"
            ;;
    esac
}

fusion_runtime_engine_is_rust() {
    local fusion_dir="$1"
    [ "$(fusion_runtime_engine "$fusion_dir")" = "rust" ]
}

fusion_runtime_compat_mode() {
    local fusion_dir="$1"
    local bridge_value=""
    bridge_value=$(fusion_bridge_runtime_config_field "$fusion_dir" "compat_mode" 2>/dev/null) || bridge_value=""
    if [ "$bridge_value" = "true" ] || [ "$bridge_value" = "false" ]; then
        echo "$bridge_value"
        return 0
    fi

    [ -f "$fusion_dir/config.yaml" ] || {
        echo "true"
        return 0
    }

    awk '
    BEGIN { in_runtime = 0; value = "true" }
    /^[[:space:]]*#/ { next }
    /^[^[:space:]#][^:]*:[[:space:]]*$/ {
        key = $0
        sub(/[[:space:]]*:[[:space:]]*$/, "", key)
        in_runtime = (key == "runtime")
        next
    }
    in_runtime && /^[[:space:]]+compat_mode:[[:space:]]*/ {
        value = $0
        sub(/^[[:space:]]+compat_mode:[[:space:]]*/, "", value)
        sub(/[[:space:]]*#.*/, "", value)
        gsub(/[[:space:]\"]/, "", value)
        value = tolower(value)
        if (value != "true" && value != "false") {
            value = "true"
        }
        print value
        exit
    }
    END { print value }
    ' "$fusion_dir/config.yaml" 2>/dev/null | head -1
}

resolve_fusion_bridge_bin() {
    local script_dir="${1:-${SCRIPT_DIR:-$(pwd)}}"

    if [ -n "${FUSION_BRIDGE_BIN:-}" ] && [ -x "$FUSION_BRIDGE_BIN" ]; then
        echo "$FUSION_BRIDGE_BIN"
        return 0
    fi

    if command -v fusion-bridge >/dev/null 2>&1; then
        command -v fusion-bridge
        return 0
    fi

    local candidates=(
        "$script_dir/../rust/target/release/fusion-bridge"
        "$script_dir/../rust/target/release/fusion-bridge.exe"
    )

    local candidate
    for candidate in "${candidates[@]}"; do
        if [ -x "$candidate" ]; then
            echo "$candidate"
            return 0
        fi
    done

    return 1
}

fusion_should_prefer_bridge() {
    local fusion_dir="$1"

    fusion_bridge_disabled && return 1
    resolve_fusion_bridge_bin "$SCRIPT_DIR" >/dev/null 2>&1
}

runtime_enabled_in_config() {
    fusion_runtime_enabled "${1:-$FUSION_DIR}"
}

runtime_engine_is_rust() {
    fusion_runtime_engine_is_rust "${1:-$FUSION_DIR}"
}
