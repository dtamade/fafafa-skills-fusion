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

# 解析参数
GOAL=""
for arg in "$@"; do
    case "$arg" in
        --force|--yolo)
            FORCE_MODE=true
            ;;
        *)
            if [ -z "$GOAL" ]; then
                GOAL="$arg"
            fi
            ;;
    esac
done

if [ -z "$GOAL" ]; then
    echo "Usage: fusion-start.sh \"<goal>\" [--force]"
    echo "       --force: Skip UNDERSTAND phase"
    exit 1
fi

# 1. 初始化 .fusion 目录
bash "$SCRIPT_DIR/fusion-init.sh"

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
