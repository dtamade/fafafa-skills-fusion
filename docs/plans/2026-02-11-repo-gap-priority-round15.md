# Repo Gap Priority Round 15 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完整补齐 stop-hook 稳定性与可观测性：锁竞争场景不再导致 structured 会话硬退出，并新增专属测试与 README 快速恢复路径。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先补 3 个失败测试（lock 竞争、stop-guard 契约、README 新鲜度），再修复脚本和文档，最后做 targeted + full 回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: R15-001 stop-guard 锁竞争 structured 阻断

**Files:**
- Create: `scripts/runtime/tests/test_fusion_stop_guard_script`
- Modify: `scripts/fusion-stop-guard.sh`

**Step 1: RED**
- 新增 `test_structured_lock_contention_returns_json_block`。
- 测试记录： `scripts/runtime/tests/test_fusion_stop_guard_script::TestFusionStopGuardScript::test_structured_lock_contention_returns_json_block`
- Expected: FAIL（当前 lock 竞争返回 rc=2 且无 JSON）。

**Step 2: GREEN**
- `fusion-stop-guard.sh` 在 lock 竞争时调用 `emit_block_response`，structured 输出 JSON block，legacy 维持 exit 2 兼容。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

### Task 2: R15-002 stop-guard 专项脚本测试集

**Files:**
- Create: `scripts/runtime/tests/test_fusion_stop_guard_script`

**Step 1: RED**
- 新增 stop-guard 合同测试：
  - structured pending 阻断 JSON
  - legacy pending 阻断 exit2
  - 非 in_progress 放行
  - 无 `.fusion` 放行
- 测试记录： `scripts/runtime/tests/test_fusion_stop_guard_script`
- Expected: 至少 1 项 FAIL（lock 竞争/行为契约缺口）。

**Step 2: GREEN**
- 对齐脚本行为，保证四类场景稳定。

**Step 3: VERIFY**
- 运行测试文件应 PASS。

---

### Task 3: R15-003 README 快速恢复路径

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: RED**
- 增加文档新鲜度测试：README / README.zh-CN 需包含 `fusion-hook-doctor.sh --json --fix`。
- 测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_readme_mentions_hook_doctor_fix scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_readme_zh_cn_mentions_hook_doctor_fix`
- Expected: FAIL（当前未明确给出该命令）。

**Step 2: GREEN**
- 在中英文 README 增加 hook quick-fix 小节并给出命令。

**Step 3: VERIFY**
- 运行同一测试应 PASS。

---

## Batch Verification (Round 15 / Batch1)

Run:
- `bash -n scripts/fusion-stop-guard.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_stop_guard_script scripts/runtime/tests/test_docs_freshness`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_stop_guard_script scripts/runtime/tests/test_docs_freshness`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

