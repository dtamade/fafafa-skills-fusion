#!/bin/bash
# fusion-start.sh - /fusion 命令启动入口
#
# 用法: fusion-start.sh "目标描述" [--force]
#
# 功能:
# 1. 调用 fusion-init.sh 初始化 .fusion
# 2. 委托 Rust fusion-bridge start 执行启动
# 3. 输出 Hook 自检与恢复提示

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FUSION_DIR=".fusion"
FORCE_MODE=false
USAGE="Usage: fusion-start.sh <goal> [--force]"
HOOK_RESTART_REQUIRED=false

source "$SCRIPT_DIR/lib/fusion-bridge.sh"
source "$SCRIPT_DIR/lib/fusion-json.sh"

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

run_bridge_start() {
    local bridge_bin="$1"
    local bridge_args=(start --fusion-dir "$FUSION_DIR" --templates-dir "$(dirname "$SCRIPT_DIR")/templates" "$GOAL")
    if [ "$FORCE_MODE" = true ]; then
        bridge_args+=(--force)
    fi

    "$bridge_bin" "${bridge_args[@]}"
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
    if [ "$(json_get_bool_from_text "$fix_output" "fixed")" = "true" ]; then
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

# 1. 初始化 .fusion 目录（由 Rust bridge init 执行）
bash "$SCRIPT_DIR/fusion-init.sh"

# 1.1 自动检测并修复 Hook 接线（首次启动友好）
ensure_hook_wiring "$(pwd)"

if fusion_bridge_disabled; then
    echo "[fusion][deps] fusion-start.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release" >&2
    exit 127
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || {
    echo "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release" >&2
    exit 127
}

run_bridge_start "$bridge_bin"

# 2. 输出额外提示
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
