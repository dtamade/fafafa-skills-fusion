# Repo Gap Priority Round 4 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 `pause/cancel/continue` 控制脚本的参数健壮性与可诊断性缺口，避免误操作并补齐测试覆盖。

**Architecture:** 继续测试先行（RED→GREEN→REFACTOR）。每个脚本先加一个失败测试验证“未知参数应拒绝”，再做最小 shell 改动（`-h|--help` + unknown option error），最后执行 targeted + full 回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: `fusion-pause.sh` 参数校验

**Priority:** P0  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation`
- Modify: `scripts/fusion-pause.sh`

**Step 1: Write the failing test**

新增测试：`bash scripts/fusion-pause.sh --bad` 返回非 0，并包含 `Unknown option`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionPauseValidation::test_pause_rejects_unknown_option`  
Expected: FAIL（当前会忽略参数并执行）。

**Step 3: Write minimal implementation**

在脚本头部增加参数解析：
- `-h|--help` 输出 `Usage: fusion-pause.sh`
- 其他参数报错退出 1

**Step 4: Run test to verify it passes**

Run: `bash -n scripts/fusion-pause.sh`  
测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionPauseValidation::test_pause_rejects_unknown_option`  
Expected: PASS。

---

### Task 2: `fusion-cancel.sh` 参数校验

**Priority:** P0  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation`
- Modify: `scripts/fusion-cancel.sh`

**Step 1: Write the failing test**

新增测试：`bash scripts/fusion-cancel.sh --bad` 返回非 0，并包含 `Unknown option`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionCancelValidation::test_cancel_rejects_unknown_option`  
Expected: FAIL。

**Step 3: Write minimal implementation**

在脚本头部增加参数解析：
- `-h|--help` 输出 `Usage: fusion-cancel.sh`
- 其他参数报错退出 1

**Step 4: Run test to verify it passes**

Run: `bash -n scripts/fusion-cancel.sh`  
测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionCancelValidation::test_cancel_rejects_unknown_option`  
Expected: PASS。

---

### Task 3: `fusion-continue.sh` 参数校验

**Priority:** P1  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation`
- Modify: `scripts/fusion-continue.sh`

**Step 1: Write the failing test**

新增测试：`bash scripts/fusion-continue.sh --bad` 返回非 0，并包含 `Unknown option`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionContinueValidation::test_continue_rejects_unknown_option`  
Expected: FAIL（当前会静默忽略参数）。

**Step 3: Write minimal implementation**

在脚本头部增加参数解析：
- `-h|--help` 输出 `Usage: fusion-continue.sh`
- 其他参数报错退出 1

**Step 4: Run test to verify it passes**

Run: `bash -n scripts/fusion-continue.sh`  
测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionContinueValidation::test_continue_rejects_unknown_option`  
Expected: PASS。

---

## Final Regression (Round 4)

Run:
- 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_status_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
