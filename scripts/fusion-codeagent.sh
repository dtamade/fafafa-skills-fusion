#!/bin/bash
# fusion-codeagent.sh - 统一 codeagent-wrapper 适配层

set -euo pipefail

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEAGENT_WRAPPER_BIN=""
PYTHON_CMD=""

usage() {
    cat <<'USAGE'
Usage: fusion-codeagent.sh [phase] [prompt...]

Examples:
  fusion-codeagent.sh EXECUTE
  fusion-codeagent.sh REVIEW
  fusion-codeagent.sh
USAGE
}

ensure_fusion() {
    [ -d "$FUSION_DIR" ] || { echo "[fusion] .fusion not found" >&2; exit 1; }
    [ -f "$FUSION_DIR/sessions.json" ] || { echo "[fusion] sessions.json not found" >&2; exit 1; }
}

resolve_python_cmd() {
    if [ -n "${PYTHON_CMD:-}" ] && command -v "$PYTHON_CMD" >/dev/null 2>&1; then
        echo "$PYTHON_CMD"
        return 0
    fi
    if command -v python3 >/dev/null 2>&1; then
        echo "python3"
        return 0
    fi
    if command -v python >/dev/null 2>&1; then
        echo "python"
        return 0
    fi
    return 1
}

write_dependency_report() {
    local missing="$1"
    local reason="$2"
    local report_file="$FUSION_DIR/dependency_report.json"
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    if [ -n "${PYTHON_CMD:-}" ]; then
        "$PYTHON_CMD" <<PYEOF
import json
from pathlib import Path

report = {
    "status": "blocked",
    "source": "fusion-codeagent.sh",
    "timestamp": "$ts",
    "missing": ["$missing"],
    "reason": "$reason",
    "auto_attempted": [
        "${CODEAGENT_WRAPPER_BIN:-}",
        "codeagent-wrapper in PATH",
        "./node_modules/.bin/codeagent-wrapper",
        "~/.local/bin/codeagent-wrapper",
        "~/.npm-global/bin/codeagent-wrapper"
    ],
    "next_actions": [
        "Install or expose codeagent-wrapper in PATH.",
        "Or set CODEAGENT_WRAPPER_BIN to an executable path.",
        "Re-run: bash scripts/fusion-codeagent.sh EXECUTE"
    ],
    "agent_prompt": "Dependency missing: codeagent-wrapper. Resolve installation/path and retry fusion-codeagent.sh."
}
Path("$report_file").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
PYEOF
        return 0
    fi

    cat > "$report_file" <<REPORT_EOF
{
  "status": "blocked",
  "source": "fusion-codeagent.sh",
  "timestamp": "$ts",
  "missing": ["$missing"],
  "reason": "$reason",
  "auto_attempted": [
    "${CODEAGENT_WRAPPER_BIN:-}",
    "codeagent-wrapper in PATH",
    "./node_modules/.bin/codeagent-wrapper",
    "~/.local/bin/codeagent-wrapper",
    "~/.npm-global/bin/codeagent-wrapper"
  ],
  "next_actions": [
    "Install or expose codeagent-wrapper in PATH.",
    "Or set CODEAGENT_WRAPPER_BIN to an executable path.",
    "Re-run: bash scripts/fusion-codeagent.sh EXECUTE"
  ],
  "agent_prompt": "Dependency missing: codeagent-wrapper. Resolve installation/path and retry fusion-codeagent.sh."
}
REPORT_EOF
}

json_escape_fallback() {
    printf '%s' "$1" | sed ':a;N;$!ba;s/\n/\\n/g; s/\\/\\\\/g; s/"/\\"/g; s/\r//g'
}

write_backend_failure_report() {
    local primary_backend="$1"
    local fallback_backend="$2"
    local primary_error="$3"
    local fallback_error="$4"
    local report_file="$FUSION_DIR/backend_failure_report.json"
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    if command -v jq >/dev/null 2>&1; then
        jq -nc \
            --arg status "blocked" \
            --arg source "fusion-codeagent.sh" \
            --arg timestamp "$ts" \
            --arg primary_backend "$primary_backend" \
            --arg fallback_backend "$fallback_backend" \
            --arg primary_error "$primary_error" \
            --arg fallback_error "$fallback_error" \
            --arg next_action "Check backend network/credentials and retry with explicit backend override." \
            '{
                status:$status,
                source:$source,
                timestamp:$timestamp,
                primary_backend:$primary_backend,
                fallback_backend:$fallback_backend,
                primary_error:$primary_error,
                fallback_error:$fallback_error,
                next_actions:[$next_action]
            }' > "$report_file"
        return 0
    fi

    if [ -n "${PYTHON_CMD:-}" ]; then
        "$PYTHON_CMD" <<PYEOF
import json
from pathlib import Path

report = {
    "status": "blocked",
    "source": "fusion-codeagent.sh",
    "timestamp": "$ts",
    "primary_backend": "$primary_backend",
    "fallback_backend": "$fallback_backend",
    "primary_error": """$primary_error""",
    "fallback_error": """$fallback_error""",
    "next_actions": ["Check backend network/credentials and retry with explicit backend override."],
}
Path("$report_file").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
PYEOF
        return 0
    fi

    local escaped_primary
    local escaped_fallback
    escaped_primary=$(json_escape_fallback "$primary_error")
    escaped_fallback=$(json_escape_fallback "$fallback_error")

    cat > "$report_file" <<REPORT_EOF
{
  "status": "blocked",
  "source": "fusion-codeagent.sh",
  "timestamp": "$ts",
  "primary_backend": "$primary_backend",
  "fallback_backend": "$fallback_backend",
  "primary_error": "$escaped_primary",
  "fallback_error": "$escaped_fallback",
  "next_actions": ["Check backend network/credentials and retry with explicit backend override."]
}
REPORT_EOF
}

resolve_codeagent_wrapper_bin() {
    if [ -n "${CODEAGENT_WRAPPER_BIN:-}" ] && [ -x "$CODEAGENT_WRAPPER_BIN" ]; then
        echo "$CODEAGENT_WRAPPER_BIN"
        return 0
    fi

    if [ -n "${CODEAGENT_WRAPPER_BIN:-}" ] && [ ! -x "$CODEAGENT_WRAPPER_BIN" ]; then
        echo "[fusion][deps] CODEAGENT_WRAPPER_BIN is set but not executable: $CODEAGENT_WRAPPER_BIN" >&2
    fi

    if command -v codeagent-wrapper >/dev/null 2>&1; then
        command -v codeagent-wrapper
        return 0
    fi

    local candidates=(
        "$PWD/node_modules/.bin/codeagent-wrapper"
        "$HOME/.local/bin/codeagent-wrapper"
        "$HOME/.npm-global/bin/codeagent-wrapper"
    )

    for candidate in "${candidates[@]}"; do
        if [ -x "$candidate" ]; then
            echo "$candidate"
            return 0
        fi
    done

    return 1
}

json_get() {
    local file="$1" key="$2"
    if command -v jq >/dev/null 2>&1; then
        jq -r ".$key // empty" "$file" 2>/dev/null || echo ""
    else
        grep -o "\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$file" 2>/dev/null | head -1 | cut -d'"' -f4 || echo ""
    fi
}

json_set() {
    local file="$1" key="$2" value="$3"
    if command -v jq >/dev/null 2>&1; then
        local tmp_file
        tmp_file=$(mktemp "${FUSION_DIR}/.tmp.XXXXXX")
        if jq --arg v "$value" ".$key = \$v" "$file" > "$tmp_file" 2>/dev/null; then
            mv "$tmp_file" "$file"
            return 0
        fi
        rm -f "$tmp_file" 2>/dev/null || true
        return 1
    fi

    if [ -z "${PYTHON_CMD:-}" ]; then
        return 1
    fi

    "$PYTHON_CMD" <<PYEOF
import json
f = "$file"
k = "$key"
v = "$value"
with open(f, "r", encoding="utf-8") as fp:
    data = json.load(fp)
data[k] = v
with open(f, "w", encoding="utf-8") as fp:
    json.dump(data, fp, ensure_ascii=False, indent=2)
PYEOF
}


extract_next_task_type() {
    local task_plan="$1"
    [ -f "$task_plan" ] || return 0

    awk '
    /^### Task / {
        active = ($0 ~ /\[IN_PROGRESS\]|\[PENDING\]/)
        next
    }
    active && /^[[:space:]]*-[[:space:]]*Type:[[:space:]]*/ {
        line = $0
        sub(/^[[:space:]]*-[[:space:]]*Type:[[:space:]]*/, "", line)
        gsub(/[[:space:]]+/, "", line)
        print tolower(line)
        exit
    }
    ' "$task_plan" 2>/dev/null || true
}

extract_next_task_owner() {
    local task_plan="$1"
    [ -f "$task_plan" ] || return 0

    awk '
    /^### Task / {
        active = ($0 ~ /\[IN_PROGRESS\]|\[PENDING\]/)
        next
    }
    active && /^[[:space:]]*-[[:space:]]*(Owner|Role):[[:space:]]*/ {
        line = $0
        sub(/^[[:space:]]*-[[:space:]]*(Owner|Role):[[:space:]]*/, "", line)
        gsub(/[[:space:]]+/, "", line)
        print tolower(line)
        exit
    }
    ' "$task_plan" 2>/dev/null || true
}

normalize_task_plan_owners() {
    local task_plan="$1"
    [ -f "$task_plan" ] || return 0

    local tmp_file
    tmp_file=$(mktemp "${FUSION_DIR}/.task_plan_owner.XXXXXX")

    awk '
    function owner_for_type(type_raw, type_value) {
        type_value = tolower(type_raw)
        gsub(/[[:space:]]+/, "", type_value)

        if (type_value == "verification") return "reviewer"
        if (type_value == "design" || type_value == "research") return "planner"
        if (type_value == "implementation" || type_value == "documentation" || type_value == "configuration") return "coder"
        return "coder"
    }

    function reset_block() {
        block_count = 0
    }

    function flush_block(    i, has_owner, type_idx, task_type, owner, injected) {
        if (block_count == 0) {
            return
        }

        has_owner = 0
        type_idx = 0
        task_type = "implementation"

        for (i = 1; i <= block_count; i++) {
            if (block[i] ~ /^[[:space:]]*-[[:space:]]*(Owner|Role):[[:space:]]*/) {
                has_owner = 1
            }
            if (type_idx == 0 && block[i] ~ /^[[:space:]]*-[[:space:]]*Type:[[:space:]]*/) {
                type_idx = i
                task_type = block[i]
                sub(/^[[:space:]]*-[[:space:]]*Type:[[:space:]]*/, "", task_type)
            }
        }

        owner = owner_for_type(task_type)
        injected = 0

        for (i = 1; i <= block_count; i++) {
            print block[i]
            if (!has_owner && i == type_idx) {
                print "- Owner: " owner
                injected = 1
                changed = 1
            }
        }

        if (!has_owner && !injected) {
            print "- Owner: " owner
            changed = 1
        }

        reset_block()
    }

    BEGIN {
        changed = 0
        block_count = 0
    }

    /^### Task [0-9]+: .* \[(PENDING|IN_PROGRESS|COMPLETED|FAILED)\][[:space:]]*$/ {
        flush_block()
        block_count++
        block[block_count] = $0
        next
    }

    {
        if (block_count > 0) {
            block_count++
            block[block_count] = $0
        } else {
            print $0
        }
    }

    END {
        flush_block()
    }
    ' "$task_plan" > "$tmp_file"

    if cmp -s "$task_plan" "$tmp_file"; then
        rm -f "$tmp_file" 2>/dev/null || true
    else
        mv "$tmp_file" "$task_plan"
    fi
}

default_role_for_phase() {
    local phase_upper="$1"

    case "$phase_upper" in
        UNDERSTAND|INITIALIZE|ANALYZE|DECOMPOSE)
            echo "planner"
            ;;
        VERIFY|REVIEW)
            echo "reviewer"
            ;;
        EXECUTE|COMMIT|DELIVER)
            echo "coder"
            ;;
        *)
            echo "coder"
            ;;
    esac
}

normalize_role() {
    local role_raw="$1"
    local role
    role=$(printf '%s' "$role_raw" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')

    case "$role" in
        planner|coder|reviewer)
            echo "$role"
            ;;
        *)
            echo ""
            ;;
    esac
}

backend_for_role() {
    local role="$1"

    case "$role" in
        planner|reviewer)
            echo "codex"
            ;;
        coder)
            echo "claude"
            ;;
        *)
            echo ""
            ;;
    esac
}

default_backend_for_phase() {
    local phase_upper="$1"
    local default_backend="$2"

    case "$phase_upper" in
        UNDERSTAND|INITIALIZE|ANALYZE|DECOMPOSE|VERIFY|REVIEW)
            echo "codex"
            ;;
        EXECUTE|COMMIT|DELIVER)
            echo "claude"
            ;;
        *)
            echo "$default_backend"
            ;;
    esac
}

render_prompt() {
    local phase="$1"
    local goal="$2"
    local role="$3"
    local task_plan=""
    local role_mandate=""
    [ -f "$FUSION_DIR/task_plan.md" ] && task_plan=$(cat "$FUSION_DIR/task_plan.md")

    case "$role" in
        planner)
            role_mandate="Focus on planning/decomposition, priorities, and execution handoff."
            ;;
        reviewer)
            role_mandate="Focus on review quality, risks, regressions, and acceptance criteria."
            ;;
        coder)
            role_mandate="Focus on implementation with tests, task completion, and progress updates."
            ;;
        *)
            role_mandate="Continue current workflow tasks and keep plan/progress in sync."
            ;;
    esac

    cat <<PROMPT_EOF
[Fusion Runner]
Role: $role
Phase: $phase
Goal: $goal

Role mandate:
$role_mandate

请在当前仓库执行下一步工作，并更新：
1) .fusion/task_plan.md
2) .fusion/progress.md

当前 task_plan 内容：
$task_plan
PROMPT_EOF
}

run_backend() {
    local backend="$1"
    local prompt="$2"
    local session_id="$3"

    if [ -z "${CODEAGENT_WRAPPER_BIN:-}" ] || [ ! -x "$CODEAGENT_WRAPPER_BIN" ]; then
        echo "[fusion] codeagent-wrapper not resolved" >&2
        return 127
    fi

    local timeout_sec="${FUSION_CODEAGENT_TIMEOUT_SEC:-}"
    local timeout_bin=""
    if [ -n "$timeout_sec" ]; then
        if [[ "$timeout_sec" =~ ^[0-9]+$ ]] && [ "$timeout_sec" -gt 0 ]; then
            timeout_bin="$(command -v timeout 2>/dev/null || command -v gtimeout 2>/dev/null || true)"
            if [ -z "$timeout_bin" ]; then
                echo "[fusion] FUSION_CODEAGENT_TIMEOUT_SEC is set but no timeout binary found (timeout/gtimeout)" >&2
                timeout_sec=""
            fi
        else
            echo "[fusion] invalid FUSION_CODEAGENT_TIMEOUT_SEC: $timeout_sec (expected positive integer seconds)" >&2
            timeout_sec=""
        fi
    fi

    local output=""
    local rc=0
    if [ -n "$session_id" ]; then
        if [ -n "$timeout_sec" ] && [ -n "$timeout_bin" ]; then
            if output=$(
                "$timeout_bin" "$timeout_sec" "$CODEAGENT_WRAPPER_BIN" --backend "$backend" resume "$session_id" - "$PWD" <<BACKEND_PROMPT
$prompt
BACKEND_PROMPT
            ); then
                rc=0
            else
                rc=$?
            fi
        else
            if output=$(
                "$CODEAGENT_WRAPPER_BIN" --backend "$backend" resume "$session_id" - "$PWD" <<BACKEND_PROMPT
$prompt
BACKEND_PROMPT
            ); then
                rc=0
            else
                rc=$?
            fi
        fi
    else
        if [ -n "$timeout_sec" ] && [ -n "$timeout_bin" ]; then
            if output=$(
                "$timeout_bin" "$timeout_sec" "$CODEAGENT_WRAPPER_BIN" --backend "$backend" - "$PWD" <<BACKEND_PROMPT
$prompt
BACKEND_PROMPT
            ); then
                rc=0
            else
                rc=$?
            fi
        else
            if output=$(
                "$CODEAGENT_WRAPPER_BIN" --backend "$backend" - "$PWD" <<BACKEND_PROMPT
$prompt
BACKEND_PROMPT
            ); then
                rc=0
            else
                rc=$?
            fi
        fi
    fi

    echo "$output"
    return "$rc"
}

_session_key_for_backend() {
    local backend="$1"
    case "$backend" in
        claude) echo "claude_session" ;;
        *) echo "codex_session" ;;
    esac
}

_session_key_for_backend_role() {
    local backend="$1"
    local role="$2"

    case "$backend" in
        claude|codex)
            ;;
        *)
            backend="codex"
            ;;
    esac

    role=$(normalize_role "$role")
    if [ -z "$role" ]; then
        echo "$(_session_key_for_backend "$backend")"
        return 0
    fi

    echo "${role}_${backend}_session"
}

extract_session_id() {
    local text="$1"
    # Prefer explicit marker from codeagent-wrapper to avoid scraping unrelated numbers
    # (e.g. PIDs / timestamps) from logs.
    local sid=""
    sid=$(printf '%s\n' "$text" | grep -Eo 'SESSION_ID:[[:space:]]*[^[:space:]]+' | head -1 | sed -E 's/^SESSION_ID:[[:space:]]*//') || true
    [ -n "$sid" ] && printf '%s\n' "$sid"
}

main() {
    local phase=""
    local explicit_prompt=""

    case "${1:-}" in
        -h|--help)
            usage
            return 0
            ;;
        --)
            shift || true
            ;;
        -*)
            echo "Unknown option: $1" >&2
            usage >&2
            return 1
            ;;
        "")
            ;;
        *)
            phase="$1"
            shift || true
            ;;
    esac

    explicit_prompt="${*:-}"

    ensure_fusion
    normalize_task_plan_owners "$FUSION_DIR/task_plan.md"

    if PYTHON_CMD="$(resolve_python_cmd 2>/dev/null)"; then
        :
    else
        PYTHON_CMD=""
    fi

    if ! CODEAGENT_WRAPPER_BIN="$(resolve_codeagent_wrapper_bin)"; then
        # Avoid stale/conflicting status: if dependency is missing, backend failure context is no longer actionable.
        rm -f "$FUSION_DIR/backend_failure_report.json" 2>/dev/null || true
        write_dependency_report "codeagent-wrapper" "Missing executable for backend orchestration"
        echo "[fusion][deps] Missing dependency: codeagent-wrapper" >&2
        echo "[fusion][deps] Report written: $FUSION_DIR/dependency_report.json" >&2
        return 127
    fi

    # 依赖恢复后清理上次阻塞报告
    rm -f "$FUSION_DIR/dependency_report.json" 2>/dev/null || true

    local sessions="$FUSION_DIR/sessions.json"
    local goal
    goal=$(json_get "$sessions" "goal")

    if [ -z "$phase" ]; then
        phase=$(json_get "$sessions" "current_phase")
    fi
    [ -n "$phase" ] || phase="EXECUTE"
    phase=$(printf '%s' "$phase" | tr '[:lower:]' '[:upper:]')

    local primary="codex"
    local fallback="claude"
    local route_reason="default"

    local next_task_type=""
    next_task_type=$(extract_next_task_type "$FUSION_DIR/task_plan.md")

    local next_task_owner=""
    next_task_owner=$(extract_next_task_owner "$FUSION_DIR/task_plan.md")

    local role=""
    local role_is_explicit=false
    local role_source="phase_default"
    role=$(normalize_role "${FUSION_AGENT_ROLE:-}")

    if [ -n "$role" ]; then
        role_is_explicit=true
        role_source="env"
    else
        if [ "$phase" = "EXECUTE" ]; then
            role=$(normalize_role "$next_task_owner")
            if [ -n "$role" ]; then
                role_is_explicit=true
                role_source="task_owner"
            fi
        fi

        if [ -z "$role" ]; then
            role=$(default_role_for_phase "$phase")
            role_is_explicit=false
            role_source="phase_default"
        fi
    fi

    if [ -f "$FUSION_DIR/config.yaml" ] && [ -n "$PYTHON_CMD" ]; then
        local routed
        routed=$(FUSION_PHASE="$phase" FUSION_TASK_TYPE="$next_task_type"             PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" "$PYTHON_CMD" - <<'PYEOF' 2>/dev/null || true
import os
from runtime.config import load_fusion_config

cfg = load_fusion_config('.fusion')
phase = os.environ.get('FUSION_PHASE', 'EXECUTE').upper()
task_type = os.environ.get('FUSION_TASK_TYPE', '').lower()
primary = str(cfg.get('backend_primary', 'codex')).lower()
fallback = str(cfg.get('backend_fallback', 'claude')).lower()
phase_map = cfg.get('backend_phase_routing') or {}
task_map = cfg.get('backend_task_type_routing') or {}

selected = str(phase_map.get(phase) or '').lower()
reason = ''

if selected in ('codex', 'claude'):
    reason = f'phase:{phase}'
elif phase == 'EXECUTE' and task_type:
    selected = str(task_map.get(task_type) or '').lower()
    if selected in ('codex', 'claude'):
        reason = f'task_type:{task_type}'

if selected not in ('codex', 'claude'):
    selected = primary if primary in ('codex', 'claude') else 'codex'
    reason = f'default:{selected}'

if fallback not in ('codex', 'claude') or fallback == selected:
    if primary in ('codex', 'claude') and primary != selected:
        fallback = primary
    else:
        fallback = 'claude' if selected == 'codex' else 'codex'

print(f"{selected}|{fallback}|{reason}")
PYEOF
)

        if [ -n "$routed" ] && [[ "$routed" == *"|"* ]]; then
            IFS='|' read -r routed_primary routed_fallback routed_reason <<< "$routed"
            [ -n "$routed_primary" ] && primary="$routed_primary"
            [ -n "$routed_fallback" ] && fallback="$routed_fallback"
            [ -n "$routed_reason" ] && route_reason="$routed_reason"
        fi
    else
        primary=$(default_backend_for_phase "$phase" "$primary")
        if [ "$primary" = "codex" ]; then
            fallback="claude"
        else
            fallback="codex"
        fi
        route_reason="phase_fallback:$phase"
    fi

    local role_backend=""
    role_backend=$(backend_for_role "$role")
    if [ "$role_is_explicit" = true ] && [ -n "$role_backend" ]; then
        primary="$role_backend"
        if [ "$primary" = "codex" ]; then
            fallback="claude"
        else
            fallback="codex"
        fi
        route_reason="role:$role"
    fi

    if [ "$primary" = "$fallback" ]; then
        if [ "$primary" = "codex" ]; then
            fallback="claude"
        else
            fallback="codex"
        fi
    fi

    echo "[fusion] route: role=${role:-unknown} role_source=$role_source phase=$phase task_type=${next_task_type:-unknown} owner=${next_task_owner:-none} -> $primary (fallback=$fallback, reason=$route_reason)" >&2

    local session_role=""
    if [ "$role_is_explicit" = true ]; then
        session_role="$role"
    fi

    local primary_session_key
    primary_session_key=$(_session_key_for_backend_role "$primary" "$session_role")
    local primary_session
    primary_session=$(json_get "$sessions" "$primary_session_key")
    if [ -z "$primary_session" ] && [ -n "$session_role" ]; then
        local legacy_session_key
        legacy_session_key=$(_session_key_for_backend "$primary")
        primary_session=$(json_get "$sessions" "$legacy_session_key")
    fi

    local prompt="$explicit_prompt"
    if [ -z "$prompt" ]; then
        prompt=$(render_prompt "$phase" "$goal" "$role")
    fi

    local output=""
    local used_backend="$primary"
    local primary_ok=false
    local primary_error=""
    local fallback_error=""
    local primary_output=""
    local fallback_output=""

    if primary_output=$(run_backend "$primary" "$prompt" "$primary_session" 2>&1); then
        output="$primary_output"
        primary_ok=true
    else
        primary_error="$primary_output"
        if [ -n "$primary_session" ]; then
            echo "[fusion] primary resume failed, retry without resume on $primary" >&2
            if primary_output=$(run_backend "$primary" "$prompt" "" 2>&1); then
                output="$primary_output"
                primary_ok=true
                used_backend="$primary"
                primary_error=""
            else
                primary_error="${primary_error}"$'\n'"$primary_output"
                echo "[fusion] primary backend failed, fallback to $fallback" >&2
                used_backend="$fallback"
                if fallback_output=$(run_backend "$fallback" "$prompt" "" 2>&1); then
                    output="$fallback_output"
                    primary_ok=true
                else
                    fallback_error="$fallback_output"
                    output="$fallback_output"
                    primary_ok=false
                fi
            fi
        else
            echo "[fusion] primary backend failed, fallback to $fallback" >&2
            used_backend="$fallback"
            if fallback_output=$(run_backend "$fallback" "$prompt" "" 2>&1); then
                output="$fallback_output"
                primary_ok=true
            else
                fallback_error="$fallback_output"
                output="$fallback_output"
                primary_ok=false
            fi
        fi
    fi

    if [ "$primary_ok" = true ]; then
        rm -f "$FUSION_DIR/backend_failure_report.json" 2>/dev/null || true

        local sid
        sid=$(extract_session_id "$output")
        if [ -n "$sid" ]; then
            local session_key
            session_key=$(_session_key_for_backend_role "$used_backend" "$session_role")
            json_set "$sessions" "$session_key" "$sid" || true

            if [ -n "$session_role" ]; then
                local legacy_session_key
                legacy_session_key=$(_session_key_for_backend "$used_backend")
                json_set "$sessions" "$legacy_session_key" "$sid" || true
            fi
        fi
    else
        write_backend_failure_report "$primary" "$fallback" "$primary_error" "$fallback_error"
        echo "[fusion][deps] Backend failure report written: $FUSION_DIR/backend_failure_report.json" >&2
    fi

    echo "$output"
    if [ "$primary_ok" != true ]; then
        return 1
    fi
}

main "$@"
