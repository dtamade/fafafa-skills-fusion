#!/bin/bash

guardian_extract_string_from_file() {
    local key="$1"
    [ -f "$LOOP_CONTEXT_FILE" ] || {
        echo ""
        return 0
    }
    local value=""
    value=$(fusion_bridge_inspect_json_field "$key" "string" "$LOOP_CONTEXT_FILE" 2>/dev/null) || value=""
    if [ -n "$value" ]; then
        printf '%s\n' "$value"
        return 0
    fi
    grep -o "\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$LOOP_CONTEXT_FILE" 2>/dev/null | head -1 | cut -d'"' -f4 || echo ""
}

guardian_extract_number_from_file() {
    local key="$1"
    [ -f "$LOOP_CONTEXT_FILE" ] || {
        echo ""
        return 0
    }
    local value=""
    value=$(fusion_bridge_inspect_json_field "$key" "number" "$LOOP_CONTEXT_FILE" 2>/dev/null) || value=""
    if [[ "$value" =~ ^[0-9]+$ ]]; then
        printf '%s\n' "$value"
        return 0
    fi
    grep -o "\"$key\"[[:space:]]*:[[:space:]]*[0-9][0-9]*" "$LOOP_CONTEXT_FILE" 2>/dev/null | head -1 | grep -o '[0-9][0-9]*' || echo ""
}

guardian_extract_array_values() {
    local key="$1"
    [ -f "$LOOP_CONTEXT_FILE" ] || return 0
    local bridge_values=""
    bridge_values=$(guardian_bridge_loop_context_inspect array-values --key "$key" 2>/dev/null) || bridge_values=""
    if [ -n "$bridge_values" ]; then
        printf '%s\n' "$bridge_values"
        return 0
    fi
    awk -v key="\"$key\"" '
        $0 ~ key "[[:space:]]*:[[:space:]]*" && index($0, "[") > 0 {
            if ($0 ~ /\[[[:space:]]*]/) {
                exit
            }
            in_array = 1
            next
        }
        in_array && /^[[:space:]]*]/ {
            exit
        }
        in_array {
            line = $0
            gsub(/^[[:space:]]+/, "", line)
            sub(/,[[:space:]]*$/, "", line)
            sub(/[[:space:]]+$/, "", line)
            if (line == "") {
                next
            }
            if (line ~ /^"/) {
                sub(/^"/, "", line)
                sub(/"$/, "", line)
            }
            print line
        }
    ' "$LOOP_CONTEXT_FILE" 2>/dev/null || true
}

guardian_extract_state_visits() {
    [ -f "$LOOP_CONTEXT_FILE" ] || return 0
    local bridge_values=""
    bridge_values=$(guardian_bridge_loop_context_inspect state-visits 2>/dev/null) || bridge_values=""
    if [ -n "$bridge_values" ]; then
        printf '%s\n' "$bridge_values"
        return 0
    fi
    awk '
        /"state_visits"[[:space:]]*:[[:space:]]*{/ {
            if ($0 ~ /{[[:space:]]*}/) {
                exit
            }
            in_object = 1
            next
        }
        in_object && /^[[:space:]]*}[[:space:]]*,?[[:space:]]*$/ {
            exit
        }
        in_object {
            line = $0
            gsub(/^[[:space:]]+/, "", line)
            sub(/,[[:space:]]*$/, "", line)
            if (line !~ /^"/) {
                next
            }
            key = line
            sub(/^"/, "", key)
            sub(/".*/, "", key)
            value = line
            sub(/.*:[[:space:]]*/, "", value)
            print key "=" value
        }
    ' "$LOOP_CONTEXT_FILE" 2>/dev/null || true
}

guardian_extract_decision_history() {
    [ -f "$LOOP_CONTEXT_FILE" ] || return 0
    local bridge_values=""
    bridge_values=$(guardian_bridge_loop_context_inspect decision-history 2>/dev/null) || bridge_values=""
    if [ -n "$bridge_values" ]; then
        printf '%s\n' "$bridge_values"
        return 0
    fi
    awk '
        /"decision_history"[[:space:]]*:[[:space:]]*/ && index($0, "[") > 0 {
            if ($0 ~ /\[[[:space:]]*]/) {
                exit
            }
            in_array = 1
            next
        }
        in_array && /^[[:space:]]*][[:space:]]*,?[[:space:]]*$/ {
            exit
        }
        !in_array {
            next
        }
        {
            line = $0
            gsub(/^[[:space:]]+/, "", line)
            sub(/,[[:space:]]*$/, "", line)
            if (line == "{") {
                collecting = 1
                current = "{"
                next
            }
            if (collecting) {
                if (line == "}") {
                    current = current " }"
                    gsub(/[[:space:]]+/, " ", current)
                    print current
                    collecting = 0
                    current = ""
                    next
                }
                current = current " " line
                next
            }
            if (line ~ /^\{.*\}$/) {
                print line
            }
        }
    ' "$LOOP_CONTEXT_FILE" 2>/dev/null || true
}

guardian_count_list_items() {
    local items="${1:-}"
    if [ -z "$items" ]; then
        echo "0"
        return 0
    fi
    printf '%s\n' "$items" | awk 'length($0) > 0 { count++ } END { print count + 0 }'
}

guardian_last_list_item() {
    local items="${1:-}"
    if [ -z "$items" ]; then
        echo ""
        return 0
    fi
    printf '%s\n' "$items" | awk 'length($0) > 0 { last = $0 } END { if (last != "") print last }'
}

guardian_append_list_with_limit() {
    local items="${1:-}"
    local item="${2:-}"
    local limit="${3:-1}"

    {
        [ -n "$items" ] && printf '%s\n' "$items"
        [ -n "$item" ] && printf '%s\n' "$item"
    } | awk -v limit="$limit" '
        length($0) > 0 {
            values[++count] = $0
        }
        END {
            start = count - limit + 1
            if (start < 1) {
                start = 1
            }
            for (i = start; i <= count; i++) {
                print values[i]
            }
        }
    '
}

guardian_state_visits_get() {
    local state_visits="${1:-}"
    local state="$2"
    if [ -z "$state_visits" ]; then
        echo "0"
        return 0
    fi
    printf '%s\n' "$state_visits" | awk -F '=' -v target="$state" '
        $1 == target {
            print $2
            found = 1
            exit
        }
        END {
            if (!found) {
                print 0
            }
        }
    '
}

guardian_state_visits_increment() {
    local state="$1"
    local updated=""
    local found=false
    local max_visits=0
    local name=""
    local value=""
    local current_visits=0

    while IFS='=' read -r name value; do
        [ -n "${name:-}" ] || continue
        current_visits=$(guardian_numeric_or_default "$value" 0)
        if [ "$name" = "$state" ]; then
            current_visits=$((current_visits + 1))
            found=true
        fi
        if [ "$current_visits" -gt "$max_visits" ]; then
            max_visits="$current_visits"
        fi
        updated+="${name}=${current_visits}"$'\n'
    done <<< "${CTX_STATE_VISITS:-}"

    if [ "$found" = false ]; then
        updated+="${state}=1"$'\n'
        if [ "$max_visits" -lt 1 ]; then
            max_visits=1
        fi
    fi

    CTX_STATE_VISITS="${updated%$'\n'}"
    CTX_MAX_STATE_VISIT_COUNT="$max_visits"
}

guardian_write_number_array() {
    local indent="$1"
    local key="$2"
    local items="${3:-}"
    local trailing_comma="${4:-,}"
    local total=0
    local index=0
    local item=""
    local comma=""

    total=$(guardian_count_list_items "$items")
    if [ "$total" -eq 0 ]; then
        printf '%s"%s": []%s\n' "$indent" "$key" "$trailing_comma"
        return 0
    fi

    printf '%s"%s": [\n' "$indent" "$key"
    while IFS= read -r item; do
        [ -n "$item" ] || continue
        index=$((index + 1))
        comma=""
        if [ "$index" -lt "$total" ]; then
            comma=","
        fi
        printf '%s  %s%s\n' "$indent" "$item" "$comma"
    done <<< "$items"
    printf '%s]%s\n' "$indent" "$trailing_comma"
}

guardian_write_string_array() {
    local indent="$1"
    local key="$2"
    local items="${3:-}"
    local trailing_comma="${4:-,}"
    local total=0
    local index=0
    local item=""
    local comma=""

    total=$(guardian_count_list_items "$items")
    if [ "$total" -eq 0 ]; then
        printf '%s"%s": []%s\n' "$indent" "$key" "$trailing_comma"
        return 0
    fi

    printf '%s"%s": [\n' "$indent" "$key"
    while IFS= read -r item; do
        [ -n "$item" ] || continue
        index=$((index + 1))
        comma=""
        if [ "$index" -lt "$total" ]; then
            comma=","
        fi
        printf '%s  "%s"%s\n' "$indent" "$(guardian_json_escape "$item")" "$comma"
    done <<< "$items"
    printf '%s]%s\n' "$indent" "$trailing_comma"
}

guardian_write_state_visits() {
    local indent="$1"
    local key="$2"
    local state_visits="${3:-}"
    local trailing_comma="${4:-,}"
    local total=0
    local index=0
    local state=""
    local count=""
    local comma=""

    total=$(guardian_count_list_items "$state_visits")
    if [ "$total" -eq 0 ]; then
        printf '%s"%s": {}%s\n' "$indent" "$key" "$trailing_comma"
        return 0
    fi

    printf '%s"%s": {\n' "$indent" "$key"
    while IFS='=' read -r state count; do
        [ -n "${state:-}" ] || continue
        index=$((index + 1))
        comma=""
        if [ "$index" -lt "$total" ]; then
            comma=","
        fi
        printf '%s  "%s": %s%s\n' "$indent" "$(guardian_json_escape "$state")" "$(guardian_numeric_or_default "$count" 0)" "$comma"
    done <<< "$state_visits"
    printf '%s}%s\n' "$indent" "$trailing_comma"
}

guardian_write_decision_history() {
    local indent="$1"
    local key="$2"
    local history="${3:-}"
    local trailing_comma="${4:-}"
    local total=0
    local index=0
    local entry=""
    local comma=""

    total=$(guardian_count_list_items "$history")
    if [ "$total" -eq 0 ]; then
        printf '%s"%s": []%s\n' "$indent" "$key" "$trailing_comma"
        return 0
    fi

    printf '%s"%s": [\n' "$indent" "$key"
    while IFS= read -r entry; do
        [ -n "$entry" ] || continue
        index=$((index + 1))
        comma=""
        if [ "$index" -lt "$total" ]; then
            comma=","
        fi
        printf '%s  %s%s\n' "$indent" "$entry" "$comma"
    done <<< "$history"
    printf '%s]%s\n' "$indent" "$trailing_comma"
}
