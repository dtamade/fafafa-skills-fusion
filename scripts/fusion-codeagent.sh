#!/bin/bash
# fusion-codeagent.sh - 统一 codeagent-wrapper 适配层

set -euo pipefail

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ensure_fusion() {
    [ -d "$FUSION_DIR" ] || { echo "[fusion] .fusion not found" >&2; exit 1; }
    [ -f "$FUSION_DIR/sessions.json" ] || { echo "[fusion] sessions.json not found" >&2; exit 1; }
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
    python3 <<PYEOF
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

render_prompt() {
    local phase="$1"
    local goal="$2"
    local task_plan=""
    [ -f "$FUSION_DIR/task_plan.md" ] && task_plan=$(cat "$FUSION_DIR/task_plan.md")

    cat <<EOF
[Fusion Runner]
Phase: $phase
Goal: $goal

请在当前仓库执行下一步工作，并更新：
1) .fusion/task_plan.md
2) .fusion/progress.md

当前 task_plan 内容：
$task_plan
EOF
}

run_backend() {
    local backend="$1"
    local prompt="$2"
    local session_id="$3"

    if ! command -v codeagent-wrapper >/dev/null 2>&1; then
        echo "[fusion] codeagent-wrapper not found" >&2
        return 127
    fi

    local output=""
    local rc=0
    if [ -n "$session_id" ]; then
        if output=$(codeagent-wrapper --backend "$backend" resume "$session_id" - "$PWD" <<EOF
$prompt
EOF
); then
            rc=0
        else
            rc=$?
        fi
    else
        if output=$(codeagent-wrapper --backend "$backend" - "$PWD" <<EOF
$prompt
EOF
); then
            rc=0
        else
            rc=$?
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

extract_session_id() {
    local text="$1"
    printf '%s\n' "$text" | grep -Eo '[0-9]{6,}[A-Za-z0-9_-]*' | head -1 || true
}

main() {
    ensure_fusion

    local phase="${1:-EXECUTE}"
    shift || true
    local explicit_prompt="${*:-}"

    local sessions="$FUSION_DIR/sessions.json"
    local goal
    goal=$(json_get "$sessions" "goal")
    local codex_session
    codex_session=$(json_get "$sessions" "codex_session")

    local primary="codex"
    local fallback="claude"

    if [ -f "$FUSION_DIR/config.yaml" ]; then
        primary=$(PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" python3 - <<'PYEOF' 2>/dev/null || echo "codex"
from runtime.config import load_fusion_config
cfg = load_fusion_config('.fusion')
print(cfg.get('backend_primary','codex'))
PYEOF
)
        fallback=$(PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" python3 - <<'PYEOF' 2>/dev/null || echo "claude"
from runtime.config import load_fusion_config
cfg = load_fusion_config('.fusion')
print(cfg.get('backend_fallback','claude'))
PYEOF
)
    fi

    local primary_session_key
    primary_session_key=$(_session_key_for_backend "$primary")
    local primary_session
    primary_session=$(json_get "$sessions" "$primary_session_key")

    local prompt="$explicit_prompt"
    if [ -z "$prompt" ]; then
        prompt=$(render_prompt "$phase" "$goal")
    fi

    local output
    local used_backend="$primary"
    local primary_ok=false
    if output=$(run_backend "$primary" "$prompt" "$primary_session" 2>&1); then
        primary_ok=true
    else
        echo "[fusion] primary backend failed, fallback to $fallback" >&2
        used_backend="$fallback"
        if output=$(run_backend "$fallback" "$prompt" "" 2>&1); then
            primary_ok=true
        else
            primary_ok=false
        fi
    fi

    local sid
    sid=$(extract_session_id "$output")
    if [ -n "$sid" ]; then
        local session_key
        session_key=$(_session_key_for_backend "$used_backend")
        json_set "$sessions" "$session_key" "$sid" || true
    fi

    echo "$output"
    if [ "$primary_ok" != true ]; then
        return 1
    fi
}

main "$@"
