# Repo Gap Priority Round 11 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `fusion-status.sh` 增加稳定的 `--json` 机器可读输出，覆盖成功/失败路径并固化回归。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先补失败测试，再做最小实现；任务完成后跑 targeted + full 回归，并记录证据。

**Tech Stack:** Bash, Markdown。

---

### Task 1: A11 `fusion-status --json` 成功路径输出 JSON

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script`
- Modify: `scripts/fusion-status.sh`

**Step 1: Write the failing test**
- 新增 `test_status_json_mode_outputs_machine_readable_summary`。
- 断言 `--json` 返回 0，输出可解析 JSON，`result=ok`，含 `status`/`phase` 字段。

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_json_mode_outputs_machine_readable_summary`
- Expected: FAIL（当前输出人类文本，不是 JSON）。

**Step 3: Write minimal implementation**
- 在脚本中新增参数解析与 `emit_json_status`。
- `--json` 且 `.fusion` 存在时输出机器可读 JSON 并退出 0。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_json_mode_outputs_machine_readable_summary`
- Expected: PASS。

---

### Task 2: B11 `fusion-status --json` 缺失 `.fusion` 输出 JSON 错误对象

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script`
- Modify: `scripts/fusion-status.sh`

**Step 1: Write the failing test**
- 新增 `test_status_json_mode_reports_missing_fusion_dir`。
- 删除 `.fusion` 后执行 `--json`，断言返回非 0，输出 JSON 且 `result=error`。

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_json_mode_reports_missing_fusion_dir`
- Expected: FAIL（当前是纯文本错误）。

**Step 3: Write minimal implementation**
- 缺失 `.fusion` 且 `--json` 时，输出 JSON 错误对象再退出 1。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_json_mode_reports_missing_fusion_dir`
- Expected: PASS。

---

### Task 3: C11 `fusion-status --json` 不输出人类横幅

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script`
- Modify: `scripts/fusion-status.sh`

**Step 1: Write the failing test**
- 新增 `test_status_json_mode_omits_human_banner`。
- 断言 `--json` 输出不包含 `=== Fusion Status ===`。

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_json_mode_omits_human_banner`
- Expected: FAIL（当前始终输出横幅）。

**Step 3: Write minimal implementation**
- JSON 模式提前返回，不执行人类文本输出分支。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_json_mode_omits_human_banner`
- Expected: PASS。

---

## Batch Verification (Round 11 / Batch1)

Run:
- `bash -n scripts/fusion-status.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_hook_doctor_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

