#!/bin/bash
# fusion-start.sh - /fusion 命令启动入口
#
# 用法: fusion-start.sh "目标描述" [--force]
#
# 功能:
# 1. 初始化 .fusion 目录
# 2. 写入 goal 和触发 START 事件
# 3. 输出引导 Claude 进入执行循环

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FUSION_DIR=".fusion"
FORCE_MODE=false
PYTHON_CMD=""
USAGE="Usage: fusion-start.sh <goal> [--force]"
HOOK_RESTART_REQUIRED=false

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

    cat > "$report_file" <<REPORT_EOF
{
  "status": "blocked",
  "source": "fusion-start.sh",
  "timestamp": "$ts",
  "missing": ["$missing"],
  "reason": "$reason",
  "auto_attempted": [
    "python3",
    "python"
  ],
  "next_actions": [
    "Install Python 3.8+ and ensure it is available as python3 or python.",
    "Re-run: bash scripts/fusion-start.sh \"<goal>\"",
    "If automation is available, let the agent resolve dependencies and retry."
  ]
}
REPORT_EOF
}

has_literal() {
    local file="$1"
    local needle="$2"
    [ -f "$file" ] || return 1
    grep -Fq "$needle" "$file"
}

is_truthy() {
    case "$(printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]')" in
        1|true|yes|on)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

hook_debug_enabled() {
    if is_truthy "${FUSION_HOOK_DEBUG:-}"; then
        return 0
    fi

    [ -f "$FUSION_DIR/.hook_debug" ]
}

project_hooks_wired() {
    local project_root="$1"
    local settings_local="$project_root/.claude/settings.local.json"
    local settings_project="$project_root/.claude/settings.json"

    if has_literal "$settings_local" "fusion-pretool.sh" \
        && has_literal "$settings_local" "fusion-posttool.sh" \
        && has_literal "$settings_local" "fusion-stop-guard.sh"; then
        return 0
    fi

    if has_literal "$settings_project" "fusion-pretool.sh" \
        && has_literal "$settings_project" "fusion-posttool.sh" \
        && has_literal "$settings_project" "fusion-stop-guard.sh"; then
        return 0
    fi

    return 1
}

ensure_hook_wiring() {
    local project_root="$1"
    local doctor_script="$SCRIPT_DIR/fusion-hook-doctor.sh"
    local had_project_hooks=false

    if project_hooks_wired "$project_root"; then
        had_project_hooks=true
    fi

    [ -f "$doctor_script" ] || return 0

    if bash "$doctor_script" --json "$project_root" >/dev/null 2>&1; then
        return 0
    fi

    echo "[fusion][hooks] Detected hook wiring gaps. Auto-fixing..."

    local fix_output=""
    local fix_rc=0
    local fix_changed=false

    fix_output=$(bash "$doctor_script" --json --fix "$project_root" 2>/dev/null) || fix_rc=$?
    if printf '%s' "$fix_output" | grep -Eq '"fixed"[[:space:]]*:[[:space:]]*true'; then
        fix_changed=true
    fi

    if [ "$fix_rc" -eq 0 ] && project_hooks_wired "$project_root"; then
        if [ "$had_project_hooks" = false ] || [ "$fix_changed" = true ]; then
            HOOK_RESTART_REQUIRED=true
        fi
        echo "[fusion][hooks] Hook auto-fix complete."
        return 0
    fi

    echo "[fusion][hooks] Auto-fix incomplete. Run: bash scripts/fusion-hook-doctor.sh --json --fix ." >&2
    return 0
}


# 解析参数
GOAL=""
EXTRA_GOAL=""
SHOW_HELP=false

for arg in "$@"; do
    case "$arg" in
        --force|--yolo)
            FORCE_MODE=true
            ;;
        -h|--help)
            SHOW_HELP=true
            ;;
        -*)
            echo "Unknown option: $arg" >&2
            echo "$USAGE" >&2
            echo "       --force: Skip UNDERSTAND phase" >&2
            exit 1
            ;;
        *)
            if [ -z "$GOAL" ]; then
                GOAL="$arg"
            elif [ -z "$EXTRA_GOAL" ]; then
                EXTRA_GOAL="$arg"
            else
                EXTRA_GOAL="$EXTRA_GOAL $arg"
            fi
            ;;
    esac
done

if [ "$SHOW_HELP" = true ]; then
    echo "$USAGE"
    echo "       --force: Skip UNDERSTAND phase"
    exit 0
fi

if [ -n "$EXTRA_GOAL" ]; then
    echo "Error: only one goal is supported." >&2
    echo "$USAGE" >&2
    exit 1
fi

if [ -z "$GOAL" ]; then
    echo "$USAGE"
    echo "       --force: Skip UNDERSTAND phase"
    exit 1
fi

# 1. 初始化 .fusion 目录
bash "$SCRIPT_DIR/fusion-init.sh"

# 1.1 自动检测并修复 Hook 接线（首次启动友好）
ensure_hook_wiring "$(pwd)"

if ! PYTHON_CMD="$(resolve_python_cmd)"; then
    write_dependency_report "python3" "Python runtime is required to bootstrap Fusion workflow"
    echo "[fusion][deps] Missing Python runtime (python3/python)."
    echo "[fusion][deps] Report written: $FUSION_DIR/dependency_report.json"
    exit 1
fi

# 2. 写入 goal 到 sessions.json
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
WORKFLOW_ID="fusion_$(date +%s)"

if command -v jq &>/dev/null; then
    # 使用 jq 安全地写入（防止注入）
    jq --arg goal "$GOAL" \
       --arg ts "$TIMESTAMP" \
       --arg wid "$WORKFLOW_ID" \
       '.goal = $goal | .started_at = $ts | .workflow_id = $wid | .status = "in_progress"' \
       "$FUSION_DIR/sessions.json" > "$FUSION_DIR/sessions.json.tmp"
    mv "$FUSION_DIR/sessions.json.tmp" "$FUSION_DIR/sessions.json"
else
    # Fallback: Python 写入
    "$PYTHON_CMD" << PYEOF
import json
with open("$FUSION_DIR/sessions.json", "r", encoding="utf-8") as f:
    data = json.load(f)
data["goal"] = """$GOAL"""
data["started_at"] = "$TIMESTAMP"
data["workflow_id"] = "$WORKFLOW_ID"
data["status"] = "in_progress"
with open("$FUSION_DIR/sessions.json", "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
PYEOF
fi

# 3. 触发状态机事件
ORIGINAL_DIR="$(pwd)"
cd "$SCRIPT_DIR"

if [ "$FORCE_MODE" = true ]; then
    # --force: 跳过 UNDERSTAND，直接到 INITIALIZE
    "$PYTHON_CMD" << PYEOF
import sys
sys.path.insert(0, ".")
from runtime.kernel import create_kernel
from runtime.state_machine import Event

k = create_kernel("$ORIGINAL_DIR/.fusion")
result = k.dispatch(Event.SKIP_UNDERSTAND)
if result.success:
    print(f"[fusion] ⚠️ Skipped UNDERSTAND (--force)")
    print(f"[fusion] State: {result.to_state.name}")
else:
    print(f"[fusion] Error: {result.error}", file=sys.stderr)
    sys.exit(1)
PYEOF

    echo ""
    echo "[FUSION] Workflow initialized (--force mode). Begin Phase 1: INITIALIZE."
    echo ""
    echo "Next steps:"
    echo "1. Analyze the codebase context"
    echo "2. Create task decomposition"
    echo "3. Execute tasks with TDD flow"
else
    # 正常流程: START -> UNDERSTAND
    "$PYTHON_CMD" << PYEOF
import sys
sys.path.insert(0, ".")
from runtime.kernel import create_kernel
from runtime.state_machine import Event

k = create_kernel("$ORIGINAL_DIR/.fusion")
result = k.dispatch(Event.START)
if result.success:
    print(f"[fusion] State: {result.to_state.name}")
else:
    print(f"[fusion] Error: {result.error}", file=sys.stderr)
    sys.exit(1)
PYEOF

    # 执行 UNDERSTAND 阶段：评分 + 写 findings + 条件推进
    if PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" "$PYTHON_CMD" -m runtime.understand "$GOAL" \
        --fusion-dir "$ORIGINAL_DIR/.fusion" \
        --project-root "$ORIGINAL_DIR"; then
        :
    else
        rc=$?
        if [ "$rc" -eq 20 ]; then
            echo ""
            echo "[FUSION] UNDERSTAND requires clarification and strict mode is enabled."
            echo "[FUSION] Update goal details and retry, or use --force to bypass UNDERSTAND."
            exit 2
        fi
        echo "[fusion] ⚠️ UNDERSTAND runner failed (rc=$rc). Continue with existing session state." >&2
    fi

    echo ""
    echo "[FUSION] Workflow initialized. UNDERSTAND completed."
    echo ""
    echo "Next steps:"
    echo "1. Phase 1: INITIALIZE"
    echo "2. Phase 2: ANALYZE"
    echo "3. Phase 3: DECOMPOSE"
    echo "4. Phase 4: EXECUTE"
fi

# 4. 输出 goal 摘要
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Goal: $GOAL"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Hook selfcheck (recommended):"
echo "bash scripts/fusion-hook-selfcheck.sh --fix ."

if hook_debug_enabled; then
    echo "[fusion][hooks] Hook debug: ON (stderr + .fusion/hook-debug.log)"
else
    echo "[fusion][hooks] Hook debug: OFF (enable: touch .fusion/.hook_debug)"
fi

if [ "$HOOK_RESTART_REQUIRED" = true ]; then
    echo ""
    echo "[fusion][hooks] Hooks were activated in this run."
    echo "[fusion][hooks] Open /hooks and approve the project hook changes."
    echo "[fusion][hooks] Then restart this Claude Code session and run /fusion again."
fi
