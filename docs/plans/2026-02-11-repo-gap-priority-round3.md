# Repo Gap Priority Round 3 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复控制面脚本的参数健壮性缺口，避免误操作与不可诊断错误，并用 TDD 固化。

**Architecture:** 延续“测试先行 + 最小实现”策略：先为每个缺口写失败测试（RED），再做最小脚本改动（GREEN），最后做 targeted + full 回归。保持默认行为向后兼容（仅增加参数校验和错误提示）。

**Tech Stack:** Bash, Markdown。

---

### Task 1: `fusion-resume.sh` 参数校验（拒绝未知选项）

**Priority:** P0  
**Files:**
- Create: `scripts/runtime/tests/test_fusion_control_script_validation`
- Modify: `scripts/fusion-resume.sh`

**Step 1: Write the failing test**

新增测试：当 `.fusion/sessions.json` 为 `paused` 时，执行 `bash scripts/fusion-resume.sh --bad` 应该返回非 0，并输出 `Unknown option`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionResumeValidation::test_resume_rejects_unknown_option`
Expected: FAIL（当前会忽略参数并继续恢复）。

**Step 3: Write minimal implementation**

在 `fusion-resume.sh` 顶部增加参数解析：
- `-h|--help` 输出 usage 并退出 0
- 传入任意其他参数时报错并退出 1

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionResumeValidation::test_resume_rejects_unknown_option`
Expected: PASS。

---

### Task 2: `fusion-git.sh` 未知 action 显式报错

**Priority:** P1  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation`
- Modify: `scripts/fusion-git.sh`

**Step 1: Write the failing test**

新增测试：在临时 git 仓库中执行 `bash scripts/fusion-git.sh typo-action`，应返回非 0 且输出 `Unknown action`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionGitValidation::test_git_rejects_unknown_action`
Expected: FAIL（当前会降级执行 status）。

**Step 3: Write minimal implementation**

调整 `fusion-git.sh` case 分支：
- `status` 单独处理
- `*` 分支改为错误提示 + usage + exit 1

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionGitValidation::test_git_rejects_unknown_action`
Expected: PASS。

---

### Task 3: `fusion-logs.sh` 行数参数必须为正整数

**Priority:** P1  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation`
- Modify: `scripts/fusion-logs.sh`

**Step 1: Write the failing test**

新增测试：执行 `bash scripts/fusion-logs.sh abc` 时返回非 0，并输出 `LINES must be a positive integer`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionLogsValidation::test_logs_rejects_non_numeric_lines`
Expected: FAIL（当前只会触发 tail 底层报错）。

**Step 3: Write minimal implementation**

在 `fusion-logs.sh` 读取参数后增加校验：
- `LINES` 必须匹配 `^[1-9][0-9]*$`
- 不合法时输出明确错误与 usage 并 exit 1

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation::TestFusionLogsValidation::test_logs_rejects_non_numeric_lines`
Expected: PASS。

---

## Final Regression (Round 3)

Run:
- 测试记录： `scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_status_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

