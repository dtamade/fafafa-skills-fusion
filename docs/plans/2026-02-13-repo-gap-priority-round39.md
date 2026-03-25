# Round39 Backend Report Consistency Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 对齐 backend/dependency 阻塞报告的行为与文档契约，避免陈旧 `backend_failure_report.json` 误导，并让 SKILL/CLI 合约文档明确该报告与 status JSON 字段。

**Architecture:** 用归档记录中的测试命令驱动 shell 行为与文档同步。优先修复 `fusion-codeagent.sh` 的“缺依赖时清理 stale backend report”行为，再用 docs freshness 守卫把 `SKILL.md` 与 `docs/CLI_CONTRACT_MATRIX.md` 同步到新增的 backend failure 报告契约。

**Tech Stack:** Bash, Markdown docs.

---

### Task 1: codeagent 缺依赖时清理 stale backend failure report

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_codeagent_script`
- Modify: `scripts/fusion-codeagent.sh`

**Step 1: Write the failing test**

新增测试：当 `codeagent-wrapper` 缺失、脚本写入 `.fusion/dependency_report.json` 并返回 `127` 时，必须删除陈旧的 `.fusion/backend_failure_report.json`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_missing_wrapper_clears_stale_backend_failure_report`
Expected: FAIL（backend failure report 未被清理）

**Step 3: Write minimal implementation**

在 `scripts/fusion-codeagent.sh` 的缺依赖分支：
- 写 dependency report 前（或后）删除 `$FUSION_DIR/backend_failure_report.json`。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_missing_wrapper_clears_stale_backend_failure_report`
Expected: PASS

**Step 5: Verify task scope regression**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script`
Expected: PASS

---

### Task 2: SKILL.md 补齐 backend failure report 文档契约

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `SKILL.md`

**Step 1: Write the failing test**

新增 docs freshness 守卫：`SKILL.md` 必须提及 `.fusion/backend_failure_report.json`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_skill_md_mentions_backend_failure_report`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `SKILL.md`：
- `.fusion/` 文件树中新增 `backend_failure_report.json`。
- “依赖与自动修复”段落补充双后端失败会写入该报告，并在 `/fusion status` 显示 `Backend Failure Report`。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_skill_md_mentions_backend_failure_report`
Expected: PASS

**Step 5: Verify task scope regression**

测试记录： `scripts/runtime/tests/test_docs_freshness`
Expected: PASS

---

### Task 3: CLI_CONTRACT_MATRIX 补齐 status backend_* JSON 字段契约

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`

**Step 1: Write the failing test**

新增 docs freshness 守卫：`docs/CLI_CONTRACT_MATRIX.md` 必须提及：
- `.fusion/backend_failure_report.json`
- `backend_status`
- `backend_primary`
- `backend_fallback`

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_cli_contract_matrix_mentions_backend_failure_report`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `docs/CLI_CONTRACT_MATRIX.md` 的 `fusion-status.sh` 行（或 Notes）补充：
- 当存在 `.fusion/backend_failure_report.json` 时，`--json` 摘要包含 `backend_status/backend_primary/backend_fallback`。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_cli_contract_matrix_mentions_backend_failure_report`
Expected: PASS

**Step 5: Verify task scope regression**

测试记录： `scripts/runtime/tests/test_docs_freshness`
Expected: PASS

---

### Final Verification Bundle

Run:
- `bash -n scripts/*.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_docs_freshness`
- 全量验证记录
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

Expected:
- all commands pass with no regressions

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
