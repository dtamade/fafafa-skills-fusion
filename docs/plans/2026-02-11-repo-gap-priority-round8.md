# Repo Gap Priority Round 8 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 achievements CLI 的关键易错点（错误路径横幅污染 + `--top=<n>` + `--root=<path>`），并用测试固化行为。

**Architecture:** 严格按 `RED -> GREEN -> REFACTOR` 执行。先用失败测试锁定目标行为，再做最小改动；每个任务完成后执行针对性验证，最后做 targeted + full 回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: R8-001 错误参数时不输出成功横幅

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_achievements_script`
- Modify: `scripts/fusion-achievements.sh`

**Step 1: Write the failing test**

在 `test_rejects_non_numeric_top_value` 追加断言：
- `stdout` 不包含 `=== Fusion Achievements ===`

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_rejects_non_numeric_top_value`
Expected: FAIL（当前错误路径仍输出横幅）。

**Step 3: Write minimal implementation**

在 `scripts/fusion-achievements.sh` 中将横幅输出移到参数校验之后，确保任何参数校验失败都先退出，不产生成功态横幅。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_rejects_non_numeric_top_value`
Expected: PASS。

---

### Task 2: R8-002 支持 `--top=<n>`

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_achievements_script`
- Modify: `scripts/fusion-achievements.sh`

**Step 1: Write the failing test**

新增测试 `test_supports_top_equals_syntax`：
- 构造最小 leaderboard 数据。
- 执行 `--leaderboard-only --root <root> --top=1`。
- 断言返回 0 且输出 leaderboard。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_supports_top_equals_syntax`
Expected: FAIL（当前 `Unknown option: --top=1`）。

**Step 3: Write minimal implementation**

在参数解析增加 `--top=*` 分支，提取 `TOP_N="${1#--top=}"`；空值走缺失值错误。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_supports_top_equals_syntax`
Expected: PASS。

---

### Task 3: R8-003 支持 `--root=<path>`

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_achievements_script`
- Modify: `scripts/fusion-achievements.sh`

**Step 1: Write the failing test**

新增测试 `test_supports_root_equals_syntax`：
- 构造 leaderboard 根目录。
- 执行 `--leaderboard-only --root=<root> --top 1`。
- 断言返回 0 且输出 leaderboard。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_supports_root_equals_syntax`
Expected: FAIL（当前 `Unknown option: --root=<path>`）。

**Step 3: Write minimal implementation**

在参数解析增加 `--root=*` 分支，提取 `LEADERBOARD_ROOT="${1#--root=}"`；空值走缺失值错误。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_supports_root_equals_syntax`
Expected: PASS。

---

## Batch Verification (Round 8 / Batch1)

Run:
- `bash -n scripts/fusion-achievements.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_achievements_script`
- 测试记录： `scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_status_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

