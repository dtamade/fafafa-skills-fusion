# Repo Gap Priority Round 6 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 `fusion-start.sh` 的 usage 输出重定向误解析问题，并通过测试固化 help/错误路径行为。

**Architecture:** 采用 RED→GREEN→REFACTOR：先在 `test_fusion_start_script` 增加失败测试覆盖 `--help`、未知参数、无参数路径，再最小修改 usage 输出字符串，最后回归验证。

**Tech Stack:** Bash, Markdown。

---

### Task 1: `fusion-start.sh` `--help` 输出与退出码修复

**Priority:** P0  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_start_script`
- Modify: `scripts/fusion-start.sh`

**Step 1: Write the failing test**

新增测试：`bash scripts/fusion-start.sh --help` 应退出 0，且输出包含 `Usage: fusion-start.sh <goal> [--force]`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_help_exits_zero_and_shows_usage`
Expected: FAIL（当前 exit 1 且出现 `goal: No such file or directory`）。

**Step 3: Write minimal implementation**

将 `fusion-start.sh` 中 usage 输出改为单个安全字符串（避免 `"<goal>"` 被 shell 解析为重定向）。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_help_exits_zero_and_shows_usage`
Expected: PASS。

---

### Task 2: 未知参数路径输出一致性修复

**Priority:** P0  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_start_script`
- Modify: `scripts/fusion-start.sh`

**Step 1: Write the failing test**

增强测试：`--bad` 路径应包含 usage 行且不出现 `No such file or directory`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_unknown_option_reports_usage_without_shell_redirection_error`
Expected: FAIL。

**Step 3: Write minimal implementation**

统一 unknown/no-goal/multi-goal/help 四处 usage 打印为同一安全文本。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_unknown_option_reports_usage_without_shell_redirection_error`
Expected: PASS。

---

### Task 3: 无参数路径 usage 一致性修复

**Priority:** P1  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_start_script`
- Modify: `scripts/fusion-start.sh`

**Step 1: Write the failing test**

新增测试：无参数启动应退出非 0，输出 usage 且不包含 `No such file or directory`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_missing_goal_reports_usage_without_shell_redirection_error`
Expected: FAIL。

**Step 3: Write minimal implementation**

复用 Task2 的 usage 修复结果（无需额外复杂逻辑）。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_missing_goal_reports_usage_without_shell_redirection_error`
Expected: PASS。

---

## Final Regression (Round 6)

Run:
- 测试记录： `scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_status_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

