# Repo Gap Priority Round 17 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐当前方向最后一批高优契约保护：`status --json` 参数语义、`hook-doctor --fix` 失败路径、`logs` 多参数边界。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。新增失败测试先锁定行为，再做最小改动，最后执行 targeted + full 回归。

**Tech Stack:** Bash, Python `pytest`, Markdown。

---

### Task 1: R17-001/R17-002 status JSON 参数契约

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script.py`

**Step 1: RED**
- 新增测试：
  - `test_status_json_unknown_option_reports_error_object`
  - `test_status_json_help_still_shows_usage_and_exits_zero`
- Run 对应单测，预期 FAIL（若无保护）。

**Step 2: GREEN**
- 如需，修正 `fusion-status.sh` 参数解析顺序与错误对象输出。

**Step 3: VERIFY**
- 两个测试 PASS。

---

### Task 2: R17-003 hook-doctor fix 失败路径

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_hook_doctor_script.py`

**Step 1: RED**
- 新增 `test_json_mode_fix_failure_reports_warn_and_fixed_false`。
- 使用 `.claude` 文件占位制造 `mkdir -p` 失败，预期当前行为不满足则 FAIL。

**Step 2: GREEN**
- 如需，修正 `fusion-hook-doctor.sh` 失败路径 JSON 输出（`result=warn` + `fixed=false` + 非零退出）。

**Step 3: VERIFY**
- 测试 PASS。

---

### Task 3: R17-004 logs 多参数边界

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation.py`

**Step 1: RED**
- 新增 `test_logs_rejects_too_many_arguments`。
- 运行测试，预期 FAIL（若无保护）。

**Step 2: GREEN**
- 若必要，调整 `fusion-logs.sh` 多参数路径错误文案与 usage。

**Step 3: VERIFY**
- 测试 PASS。

---

## Batch Verification (Round 17 / Batch1)

Run:
- `bash -n scripts/fusion-status.sh scripts/fusion-hook-doctor.sh scripts/fusion-logs.sh`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_control_script_validation.py`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_achievements_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_codeagent_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_loop_guardian_script.py scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_hook_shell_runtime_path.py scripts/runtime/tests/test_docs_freshness.py`
- `pytest -q`
