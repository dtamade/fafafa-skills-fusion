# Repo Gap Priority Round 14 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 打通 hooks 的“诊断 + 自动修复 + 文档指引”闭环，确保 `fusion-hook-doctor` 可以一键修复并输出机器可读修复状态。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先补 3 个失败测试，再在 `fusion-hook-doctor.sh` 落地 `--fix` 与 `fixed` 字段，最后补文档并回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: R14-001 hook-doctor 增加 `--fix`

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_hook_doctor_script`
- Modify: `scripts/fusion-hook-doctor.sh`

**Step 1: RED**
- 新增 `test_json_mode_fix_writes_project_settings`。
- 测试记录： `scripts/runtime/tests/test_fusion_hook_doctor_script::TestFusionHookDoctorScript::test_json_mode_fix_writes_project_settings`
- Expected: FAIL（当前 `--fix` 为 Unknown option）。

**Step 2: GREEN**
- 在 hook-doctor 增加 `--fix`，自动写入项目 `.claude/settings.local.json`（完整 pre/post/stop hooks）。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

### Task 2: R14-002 hook-doctor JSON 增加 `fixed`

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_hook_doctor_script`
- Modify: `scripts/fusion-hook-doctor.sh`

**Step 1: RED**
- 新增 `test_json_mode_reports_fixed_flag`。
- 测试记录： `scripts/runtime/tests/test_fusion_hook_doctor_script::TestFusionHookDoctorScript::test_json_mode_reports_fixed_flag`
- Expected: FAIL（当前无 `fixed` 字段）。

**Step 2: GREEN**
- JSON summary 输出新增 `fixed`（bool）：本次是否执行了自动修复。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

### Task 3: R14-003 文档补齐 `--fix` 流程

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `docs/HOOKS_SETUP.md`

**Step 1: RED**
- 在 docs freshness 测试中新增 `HOOKS_SETUP` 需包含 `--fix` 指引。
- 测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_hooks_setup_mentions_fix_flow`
- Expected: FAIL（当前文档无该指引）。

**Step 2: GREEN**
- 在 `docs/HOOKS_SETUP.md` 增补 `fusion-hook-doctor --json --fix <project_root>` 推荐流程。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

## Batch Verification (Round 14 / Batch1)

Run:
- `bash -n scripts/fusion-hook-doctor.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_docs_freshness`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_docs_freshness`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

