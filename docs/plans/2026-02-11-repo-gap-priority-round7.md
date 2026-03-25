# Repo Gap Priority Round 7 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 `fusion-achievements.sh` 参数校验缺口，避免静默错误和底层命令报错泄漏，并用测试固化。

**Architecture:** 严格 RED→GREEN→REFACTOR。按参数风险优先级处理：先修 `--top` 非法数值，再修 `--root` 缺失值，最后修 `--top` 缺失值。每步都先写失败测试，再做最小实现。

**Tech Stack:** Bash, Markdown。

---

### Task 1: `--top` 非法值校验

**Priority:** P0  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_achievements_script`
- Modify: `scripts/fusion-achievements.sh`

**Step 1: Write the failing test**

新增测试：`--leaderboard-only --top abc` 应返回非 0，输出 `--top must be a positive integer`，且不出现 `head: invalid number of lines`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_rejects_non_numeric_top_value`  
Expected: FAIL（当前返回 0 并在 stderr 出现 head 错误）。

**Step 3: Write minimal implementation**

在参数解析后对 `TOP_N` 做正整数校验；非法则错误退出并打印 usage。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_rejects_non_numeric_top_value`  
Expected: PASS。

---

### Task 2: `--root` 缺失值校验

**Priority:** P0  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_achievements_script`
- Modify: `scripts/fusion-achievements.sh`

**Step 1: Write the failing test**

新增测试：`--root` 无值时应返回非 0，并输出 `Missing value for --root`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_rejects_missing_root_value`  
Expected: FAIL（当前返回 0）。

**Step 3: Write minimal implementation**

在 `--root` 分支中检测缺失值并错误退出。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_rejects_missing_root_value`  
Expected: PASS。

---

### Task 3: `--top` 缺失值校验

**Priority:** P1  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_achievements_script`
- Modify: `scripts/fusion-achievements.sh`

**Step 1: Write the failing test**

新增测试：`--top` 无值时应返回非 0，并输出 `Missing value for --top`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_rejects_missing_top_value`  
Expected: FAIL（当前默认回落到 10 并返回 0）。

**Step 3: Write minimal implementation**

在 `--top` 分支中检测缺失值并错误退出。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_achievements_script::TestFusionAchievementsScript::test_rejects_missing_top_value`  
Expected: PASS。

---

## Final Regression (Round 7)

Run:
- 测试记录： `scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_status_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

