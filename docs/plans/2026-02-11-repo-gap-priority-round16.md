# Repo Gap Priority Round 16 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 一次性收口 CLI 契约一致性与 stop-hook 稳定性边界：`logs/git` 错误语义统一，补齐 stop-guard structured 空 stdin + runtime parity 保护测试。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先新增失败测试覆盖缺口，再最小修改 shell 脚本，最后跑 targeted + full 回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: R16-001/R16-002 logs + git 契约修复

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation`
- Modify: `scripts/fusion-logs.sh`
- Modify: `scripts/fusion-git.sh`

**Step 1: RED**
- 新增测试：
  - `logs` 对未知选项返回 `Unknown option` + usage。
  - `git` unknown action 写入 stderr 并带 usage。
- Run:
  - 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionLogsValidation::test_logs_rejects_unknown_option`
  - 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionGitValidation::test_git_unknown_action_reports_to_stderr_with_usage`
- Expected: FAIL。

**Step 2: GREEN**
- `fusion-logs.sh` 增加未知 `-` 选项分支与多参数校验。
- `fusion-git.sh` 将 `log_error` 输出改为 stderr，unknown action 显式输出 usage 到 stderr。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

### Task 2: R16-003 stop-guard structured 空 stdin 契约

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_stop_guard_script`

**Step 1: RED**
- 新增 `test_structured_blocks_with_empty_stdin`。
- 测试记录： `scripts/runtime/tests/test_fusion_stop_guard_script::TestFusionStopGuardScript::test_structured_blocks_with_empty_stdin`
- Expected: 若行为不稳定则 FAIL。

**Step 2: GREEN**
- 如需，修正 stop-guard 以确保 structured 空 stdin 仍返回 JSON block。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

### Task 3: R16-004 runtime parity（shell hook path）

**Files:**
- Modify: `scripts/runtime/tests/test_hook_shell_runtime_path`

**Step 1: RED**
- 新增 `test_stop_guard_structured_without_stdin_uses_runtime_adapter`。
- 测试记录： `scripts/runtime/tests/test_hook_shell_runtime_path::TestHookShellRuntimePath::test_stop_guard_structured_without_stdin_uses_runtime_adapter`
- Expected: FAIL（当前未覆盖该行为）。

**Step 2: GREEN**
- 调整测试辅助方法（必要时支持 env overrides），确保 runtime 模式空 stdin 行为被稳定验证。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

## Batch Verification (Round 16 / Batch1)

Run:
- `bash -n scripts/fusion-logs.sh scripts/fusion-git.sh scripts/fusion-stop-guard.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_stop_guard_script scripts/runtime/tests/test_hook_shell_runtime_path`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_stop_guard_script scripts/runtime/tests/test_hook_shell_runtime_path scripts/runtime/tests/test_docs_freshness`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

