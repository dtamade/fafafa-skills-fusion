# Repo Gap Priority Round 5 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `loop-guardian.sh` 建立直接测试覆盖并修复可观测性/可初始化缺口，降低回归风险。

**Architecture:** 延续测试先行（RED→GREEN→REFACTOR）：先新增失败测试锁定 `guardian_status` 与 `guardian_init` 的缺口，再做最小 shell 实现修复，最后执行 targeted + full 回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: `guardian_status` 阈值显示与配置一致

**Priority:** P0  
**Files:**
- Create: `scripts/runtime/tests/test_loop_guardian_script`
- Modify: `scripts/loop-guardian.sh`

**Step 1: Write the failing test**

新增测试：当 `.fusion/config.yaml` 设置 `max_iterations: 7`, `max_no_progress: 2`, `max_same_action: 4`, `max_same_error: 5` 后，`guardian_status` 输出应显示 `0/7`, `0/2`, `0/4`, `0/5`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_loop_guardian_script::TestLoopGuardianStatus::test_status_uses_loaded_config_thresholds`  
Expected: FAIL（当前显示默认阈值 50/6/3/3）。

**Step 3: Write minimal implementation**

在 `guardian_status` 中改为通过 `jq --argjson` 注入当前 shell 变量阈值，不再读取 `jq env.*`。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_loop_guardian_script::TestLoopGuardianStatus::test_status_uses_loaded_config_thresholds`  
Expected: PASS。

---

### Task 2: `guardian_init` 自动创建 FUSION_DIR

**Priority:** P0  
**Files:**
- Modify: `scripts/runtime/tests/test_loop_guardian_script`
- Modify: `scripts/loop-guardian.sh`

**Step 1: Write the failing test**

新增测试：在不存在 `.fusion` 目录的临时目录中调用 `guardian_init`，应返回 0 且创建 `.fusion/loop_context.json`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_loop_guardian_script::TestLoopGuardianInit::test_init_creates_fusion_dir_when_missing`  
Expected: FAIL（当前报 `no such file or directory`）。

**Step 3: Write minimal implementation**

在 `guardian_init` 写文件前增加 `mkdir -p "$FUSION_DIR"`。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_loop_guardian_script::TestLoopGuardianInit::test_init_creates_fusion_dir_when_missing`  
Expected: PASS。

---

### Task 3: `guardian_status` 增加 state/wall-time 阈值可见性

**Priority:** P1  
**Files:**
- Modify: `scripts/runtime/tests/test_loop_guardian_script`
- Modify: `scripts/loop-guardian.sh`

**Step 1: Write the failing test**

新增测试：`guardian_status` 输出中应包含 `State Visits: 0/<max_state_visits>` 和 `Wall Time: <current>s/<max_wall_time>s`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_loop_guardian_script::TestLoopGuardianStatus::test_status_includes_state_and_walltime_thresholds`  
Expected: FAIL（当前无这两行）。

**Step 3: Write minimal implementation**

增强 `guardian_status` 输出，补充上述两行并使用当前加载阈值值。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_loop_guardian_script::TestLoopGuardianStatus::test_status_includes_state_and_walltime_thresholds`  
Expected: PASS。

---

## Final Regression (Round 5)

Run:
- 测试记录： `scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_status_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

