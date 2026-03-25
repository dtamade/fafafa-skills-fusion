#!/bin/bash

fusion_bridge_inspect_json_field() {
    local key="$1"
    local mode="${2:-string}"
    local source="${3:-stdin}"

    if command -v fusion_bridge_disabled >/dev/null 2>&1 && fusion_bridge_disabled; then
        return 1
    fi

    command -v resolve_fusion_bridge_bin >/dev/null 2>&1 || return 1

    local bridge_bin
    bridge_bin="$(resolve_fusion_bridge_bin "${SCRIPT_DIR:-$(pwd)}")" || return 1

    if [ "$source" = "stdin" ]; then
        if [ "$mode" = "number" ]; then
            "$bridge_bin" inspect json-field --key "$key" --number
        elif [ "$mode" = "bool" ]; then
            "$bridge_bin" inspect json-field --key "$key" --bool
        else
            "$bridge_bin" inspect json-field --key "$key"
        fi
    else
        if [ "$mode" = "number" ]; then
            "$bridge_bin" inspect json-field --file "$source" --key "$key" --number
        elif [ "$mode" = "bool" ]; then
            "$bridge_bin" inspect json-field --file "$source" --key "$key" --bool
        else
            "$bridge_bin" inspect json-field --file "$source" --key "$key"
        fi
    fi
}

json_escape_string() {
    local value="$1"
    value=${value//\\/\\\\}
    value=${value//"/\\"}
    value=${value//$'\n'/\\n}
    value=${value//$'\r'/\\r}
    value=${value//$'\t'/\\t}
    printf '%s' "$value"
}

json_get_string_from_text() {
    local text="$1" key="$2"
    if command -v resolve_fusion_bridge_bin >/dev/null 2>&1; then
        local value=""
        value=$(printf '%s' "$text" | fusion_bridge_inspect_json_field "$key" "string" 2>/dev/null) || value=""
        if [ -n "$value" ]; then
            printf '%s\n' "$value"
            return 0
        fi
    fi
    printf '%s' "$text" | grep -o "\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" 2>/dev/null | head -1 | cut -d'"' -f4 || echo ""
}

json_get_number_from_text() {
    local text="$1" key="$2"
    if command -v resolve_fusion_bridge_bin >/dev/null 2>&1; then
        local value=""
        value=$(printf '%s' "$text" | fusion_bridge_inspect_json_field "$key" "number" 2>/dev/null) || value=""
        if [[ "$value" =~ ^[0-9]+$ ]]; then
            printf '%s\n' "$value"
            return 0
        fi
    fi
    printf '%s' "$text" | grep -o "\"$key\"[[:space:]]*:[[:space:]]*[0-9][0-9]*" 2>/dev/null | head -1 | grep -o '[0-9][0-9]*' || echo ""
}

json_get_bool_from_text() {
    local text="$1" key="$2"
    if command -v resolve_fusion_bridge_bin >/dev/null 2>&1; then
        local value=""
        value=$(printf '%s' "$text" | fusion_bridge_inspect_json_field "$key" "bool" 2>/dev/null) || value=""
        if [ "$value" = "true" ] || [ "$value" = "false" ]; then
            printf '%s\n' "$value"
            return 0
        fi
    fi
    printf '%s' "$text" | grep -Eo "\"$key\"[[:space:]]*:[[:space:]]*(true|false)" 2>/dev/null | head -1 | grep -Eo '(true|false)$' || echo ""
}

json_get_field() {
    local file="$1" key="$2" text=""
    [ -f "$file" ] || {
        echo ""
        return
    }

    if command -v resolve_fusion_bridge_bin >/dev/null 2>&1; then
        local value=""
        value=$(fusion_bridge_inspect_json_field "$key" "string" "$file" 2>/dev/null) || value=""
        if [ -n "$value" ]; then
            printf '%s\n' "$value"
            return 0
        fi
    fi

    text=$(cat "$file" 2>/dev/null || true)
    json_get_string_from_text "$text" "$key"
}

json_set_field() {
    local file="$1" key="$2" value="$3" tmp_root="${4:-$(dirname "$file")}" 

    [ -f "$file" ] || return 1
    command -v awk >/dev/null 2>&1 || return 1

    local tmp_file
    tmp_file=$(mktemp "${tmp_root}/.tmp.XXXXXX") || return 1

    if awk -v key="$key" -v value="$value" '
    function json_escape(v) {
        gsub(/\\/, "\\\\", v)
        gsub(/"/, "\\\"", v)
        gsub(/\r/, "", v)
        gsub(/\n/, "\\n", v)
        gsub(/\t/, "\\t", v)
        return v
    }
    BEGIN {
        found = 0
        inserted = 0
    }
    {
        lines[NR] = $0
    }
    END {
        esc = json_escape(value)
        replacement = "\"" key "\": \"" esc "\""

        for (i = 1; i <= NR; i++) {
            if (!found && sub("\"" key "\"[[:space:]]*:[[:space:]]*(null|\"[^\"]*\")", replacement, lines[i])) {
                found = 1
            }
        }

        if (found) {
            for (i = 1; i <= NR; i++) {
                print lines[i]
            }
            exit 0
        }

        if (NR == 1) {
            line = lines[1]
            if (line ~ /{[[:space:]]*}/) {
                sub(/{[[:space:]]*}/, "{" replacement "}", line)
                print line
                exit 0
            }
            if (line ~ /}[[:space:]]*$/) {
                sub(/[[:space:]]*}[[:space:]]*$/, ", " replacement "}", line)
                print line
                exit 0
            }
            print line
            exit 1
        }

        for (i = 1; i <= NR; i++) {
            if (!inserted && lines[i] ~ /^[[:space:]]*}[[:space:]]*$/) {
                prev = i - 1
                while (prev > 0 && lines[prev] ~ /^[[:space:]]*$/) {
                    prev--
                }
                if (prev > 0 && lines[prev] !~ /,[[:space:]]*$/) {
                    lines[prev] = lines[prev] ","
                }
                lines[i] = "  " replacement "\n" lines[i]
                inserted = 1
                break
            }
        }

        for (i = 1; i <= NR; i++) {
            print lines[i]
        }
        exit(inserted ? 0 : 1)
    }
    ' "$file" > "$tmp_file"; then
        mv "$tmp_file" "$file"
        return 0
    fi

    rm -f "$tmp_file" 2>/dev/null || true
    return 1
}

json_set_fields_atomically() {
    local file="$1" tmp_root="${2:-$(dirname "$file")}" 
    shift 2

    [ -f "$file" ] || return 1
    [ "$#" -ge 2 ] || return 1
    [ $(( $# % 2 )) -eq 0 ] || return 1

    local working_file
    working_file=$(mktemp "${tmp_root}/.tmp.XXXXXX") || return 1

    if ! cp "$file" "$working_file" 2>/dev/null; then
        rm -f "$working_file" 2>/dev/null || true
        return 1
    fi

    while [ "$#" -gt 0 ]; do
        local key="$1"
        local value="$2"
        shift 2
        if ! json_set_field "$working_file" "$key" "$value" "$tmp_root"; then
            rm -f "$working_file" 2>/dev/null || true
            return 1
        fi
    done

    mv "$working_file" "$file"
}
