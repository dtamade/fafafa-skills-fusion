# Repo Gap Priority Round 1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复全仓扫描发现的高优先级缺口（文档陈旧测试结果、doctor 自动化能力不足、status 排行榜可控性不足），并用严格 TDD 完成首批实现。

**Architecture:** 采用“测试先行”的增量策略：每个缺口先用失败测试锁定期望行为，再做最小实现，最后跑目标测试与全量回归。脚本改动保持向后兼容，新增行为通过可选参数或环境变量触发。

**Tech Stack:** Bash, Markdown docs。

---

### Task 1: 文档测试结果防陈旧（README.zh-CN）

**Priority:** P1  
**Files:**
- Create: `scripts/runtime/tests/test_docs_freshness`
- Modify: `README.zh-CN.md`

**Step 1: Write the failing test**

新增测试断言：`README.zh-CN.md` 不应包含硬编码的“全量测试：`<number> passed`”文本，避免后续过时。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_docs_freshness`  
Expected: FAIL，指出 `README.zh-CN.md` 仍包含硬编码通过数。

**Step 3: Write minimal implementation**

将 `README.zh-CN.md` 对应段落改为动态描述（例如“全量测试：查看当时记录的全量测试结果”），避免固定数字。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_docs_freshness`  
Expected: PASS。

**Step 5: Regression checkpoint**

测试记录： `scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_fusion_status_script`

---

### Task 2: `fusion-hook-doctor.sh` 增加 `--json` 自动化输出

**Priority:** P0  
**Files:**
- Create: `scripts/runtime/tests/test_fusion_hook_doctor_script`
- Modify: `scripts/fusion-hook-doctor.sh`

**Step 1: Write the failing test**

新增测试：
- `--json` 模式输出合法 JSON（包含 `ok_count`, `warn_count`, `project_root`, `result`）
- `warn_count>0` 时脚本返回非零。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_hook_doctor_script`  
Expected: FAIL（当前不支持 `--json`）。

**Step 3: Write minimal implementation**

在 `scripts/fusion-hook-doctor.sh` 增加：
- 参数解析：`--json`
- 累计结果以 JSON 输出
- 保持现有文本模式与退出码语义不变。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_hook_doctor_script`  
Expected: PASS。

**Step 5: Regression checkpoint**

Run: `bash scripts/fusion-hook-doctor.sh --json .`（人工快速检查输出结构）

---

### Task 3: `fusion-status.sh` 增加排行榜开关

**Priority:** P1  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script`
- Modify: `scripts/fusion-status.sh`

**Step 1: Write the failing test**

新增测试：当设置 `FUSION_STATUS_SHOW_LEADERBOARD=0` 时，不输出 `## Achievement Leaderboard (Top 3)`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_can_disable_leaderboard`  
Expected: FAIL（当前总是尝试输出排行榜）。

**Step 3: Write minimal implementation**

在 `scripts/fusion-status.sh` 中读取环境变量：
- 默认开启（兼容现状）
- 为 `0/false/no` 时跳过 `print_achievement_leaderboard_top3`。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_status_script::TestFusionStatusScript::test_status_can_disable_leaderboard`  
Expected: PASS。

**Step 5: Final regression**

Run:
- 测试记录： `scripts/runtime/tests/test_fusion_status_script scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_docs_freshness`
- 全量验证记录

---

## Batch Strategy (executing-plans)

- 本轮执行 Batch 1（Task 1~3 全部完成），每个 Task 严格走 RED → GREEN → REFACTOR。
- 每一步回报命令与关键输出。
- 若遇阻塞，立即停止并报告，不盲猜。

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
