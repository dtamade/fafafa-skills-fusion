# Repo Gap Priority Round 12 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 强化 `fusion-status --json` 机器摘要能力，补齐任务计数、依赖摘要、成就计数字段，确保自动化可直接消费。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先补 3 个失败测试，再在 `fusion-status.sh` 的 JSON 模式做最小增强，最后执行 targeted + full 回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: A12 JSON 输出任务计数

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script`
- Modify: `scripts/fusion-status.sh`

**Step 1: RED**
- 新增 `test_status_json_includes_task_counts`。
- 测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_json_includes_task_counts`
- Expected: FAIL（当前无 `task_*` 字段）。

**Step 2: GREEN**
- 在 JSON 模式汇总 `task_completed/pending/in_progress/failed`。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

### Task 2: B12 JSON 输出依赖摘要

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script`
- Modify: `scripts/fusion-status.sh`

**Step 1: RED**
- 新增 `test_status_json_includes_dependency_summary`。
- 测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_json_includes_dependency_summary`
- Expected: FAIL（当前无 `dependency_*` 字段）。

**Step 2: GREEN**
- 在 JSON 模式注入 `dependency_status/dependency_missing`。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

### Task 3: C12 JSON 输出成就计数

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script`
- Modify: `scripts/fusion-status.sh`

**Step 1: RED**
- 新增 `test_status_json_includes_achievement_counters`。
- 测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_json_includes_achievement_counters`
- Expected: FAIL（当前无 `achievement_*` 字段）。

**Step 2: GREEN**
- 在 JSON 模式计算 `achievement_completed_tasks/safe_total/advisory_total`。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

## Batch Verification (Round 12 / Batch1)

Run:
- `bash -n scripts/fusion-status.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_hook_doctor_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

