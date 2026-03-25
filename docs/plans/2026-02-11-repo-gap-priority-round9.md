# Repo Gap Priority Round 9 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 统一高频 CLI 的 `--help` 行为，消除误路由/误报错，并用测试固化（`fusion-logs.sh`、`fusion-git.sh`、`fusion-codeagent.sh`）。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。每个任务先新增失败测试，再做最小实现；最后执行 targeted + full 回归并同步 `task_plan.md`、`findings.md`、`progress.md`。

**Tech Stack:** Bash, Markdown。

---

### Task 1: A9 `fusion-logs.sh` 支持 `--help`

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation`
- Modify: `scripts/fusion-logs.sh`

**Step 1: Write the failing test**
- 新增 `test_logs_help_exits_zero_and_shows_usage`。
- 断言 `fusion-logs.sh --help` 返回码为 `0` 且输出 `Usage: fusion-logs.sh`。

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionLogsValidation::test_logs_help_exits_zero_and_shows_usage`
- Expected: FAIL（当前返回 1，报 `LINES must be a positive integer`）。

**Step 3: Write minimal implementation**
- 在脚本开头增加 `usage()`。
- 解析 `-h|--help`，直接输出 usage 并 `exit 0`。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionLogsValidation::test_logs_help_exits_zero_and_shows_usage`
- Expected: PASS。

---

### Task 2: B9 `fusion-git.sh` 支持 `--help`

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation`
- Modify: `scripts/fusion-git.sh`

**Step 1: Write the failing test**
- 新增 `test_git_help_exits_zero_and_shows_usage`。
- 断言 `fusion-git.sh --help` 返回 `0` 且输出 `Usage: fusion-git.sh`。

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionGitValidation::test_git_help_exits_zero_and_shows_usage`
- Expected: FAIL（当前被识别为 unknown action，返回 1）。

**Step 3: Write minimal implementation**
- 增加 `usage()`。
- 在 ACTION 分派前处理 `-h|--help` 并 `exit 0`。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionGitValidation::test_git_help_exits_zero_and_shows_usage`
- Expected: PASS。

---

### Task 3: C9 `fusion-codeagent.sh` 支持 `--help` 且不触发路由

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_codeagent_script`
- Modify: `scripts/fusion-codeagent.sh`

**Step 1: Write the failing test**
- 新增 `test_help_exits_zero_without_routing`。
- 断言 `fusion-codeagent.sh --help` 返回 `0`，输出 usage，且不包含 `[fusion] route:`。

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_help_exits_zero_without_routing`
- Expected: FAIL（当前会进入路由并输出 route 日志）。

**Step 3: Write minimal implementation**
- 增加 `usage()`。
- 在 `main()` 开始阶段优先处理 `-h|--help`，直接返回，不执行 `ensure_fusion` 与后端路由。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_help_exits_zero_without_routing`
- Expected: PASS。

---

## Batch Verification (Round 9 / Batch1)

Run:
- `bash -n scripts/fusion-logs.sh scripts/fusion-git.sh scripts/fusion-codeagent.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_codeagent_script`
- 测试记录： `scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_status_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

