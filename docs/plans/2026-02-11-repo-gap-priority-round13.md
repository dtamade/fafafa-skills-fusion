# Repo Gap Priority Round 13 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复当前最关键的 CLI 稳定性问题（`fusion-codeagent` 未知参数超时、`fusion-hook-doctor` 参数错误不可读），确保 hook 诊断链路可预测且可自动化消费。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先在脚本测试里补 3 个失败用例，再对 `fusion-codeagent.sh` 与 `fusion-hook-doctor.sh` 做最小解析增强，最后跑 targeted + full 回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: R13-001 codeagent 未知参数拒绝

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_codeagent_script`
- Modify: `scripts/fusion-codeagent.sh`

**Step 1: RED**
- 新增 `test_unknown_option_exits_nonzero_without_routing`。
- 测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_unknown_option_exits_nonzero_without_routing`
- Expected: FAIL（当前会进入 route 并可能超时）。

**Step 2: GREEN**
- 在 `fusion-codeagent.sh` 前置参数校验，拒绝未知 `-` 开头参数并输出 usage。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

### Task 2: R13-002 hook-doctor 未知参数拒绝

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_hook_doctor_script`
- Modify: `scripts/fusion-hook-doctor.sh`

**Step 1: RED**
- 新增 `test_json_mode_rejects_unknown_option`。
- 测试记录： `scripts/runtime/tests/test_fusion_hook_doctor_script::TestFusionHookDoctorScript::test_json_mode_rejects_unknown_option`
- Expected: FAIL（当前出现 `cd: --: invalid option`，错误不稳定）。

**Step 2: GREEN**
- `fusion-hook-doctor.sh` 参数解析中将未知 `-` 参数统一作为错误处理；`--json` 模式输出机器可读错误对象。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

### Task 3: R13-003 hook-doctor 无效 project_root 快返

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_hook_doctor_script`
- Modify: `scripts/fusion-hook-doctor.sh`

**Step 1: RED**
- 新增 `test_json_mode_rejects_invalid_project_root`。
- 测试记录： `scripts/runtime/tests/test_fusion_hook_doctor_script::TestFusionHookDoctorScript::test_json_mode_rejects_invalid_project_root`
- Expected: FAIL（当前对无效目录处理不可读/不稳定）。

**Step 2: GREEN**
- 增加 project_root 存在性校验；无效路径时统一返回清晰错误（JSON/文本双模式）。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

## Batch Verification (Round 13 / Batch1)

Run:
- `bash -n scripts/fusion-codeagent.sh scripts/fusion-hook-doctor.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_hook_doctor_script`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

