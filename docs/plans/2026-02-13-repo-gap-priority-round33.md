# Repo Gap Priority Round 33 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完成 machine JSON 契约文档收敛：补齐 CLI 契约矩阵与中英文 README 对 `schema_version` 与 basis 字段说明，并加 docs freshness 守卫测试。

**Architecture:** 严格 `RED -> GREEN -> VERIFY`。每个任务先写失败测试，再做最小文档实现，最后做 targeted + full + rust 门禁回归。

**Tech Stack:** Markdown docs, Bash gates。

---

### Task 1: R33-001 为 CLI_CONTRACT_MATRIX 增加 schema/basis 文档守卫并补齐说明

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`

**Step 1: Write the failing test**
- 在 docs freshness 中新增断言：`docs/CLI_CONTRACT_MATRIX.md` 必须出现
  - `schema_version`
  - `step_rate_basis`
  - `command_rate_basis`
  - `rate_basis`

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_cli_contract_matrix_mentions_schema_and_basis_fields`
- Expected: FAIL（当前矩阵未写这些字段）

**Step 3: Write minimal implementation**
- 在 `release-contract-audit.sh` 行的 expectations 中补充 schema/basis 字段。
- 在 `regression_runner` 行的 expectations 中补充 `schema_version/rate_basis`。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_cli_contract_matrix_mentions_schema_and_basis_fields`
- Expected: PASS

---

### Task 2: R33-002 为 README(EN) 增加 machine JSON schema/basis 文档守卫并补齐示例

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `README.md`

**Step 1: Write the failing test**
- 新增断言：`README.md` 需明确提及 machine JSON 字段：
  - `schema_version`
  - `step_rate_basis`
  - `command_rate_basis`
  - `rate_basis`

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_readme_mentions_machine_schema_and_basis_fields`
- Expected: FAIL（当前 README 未提及这些字段）

**Step 3: Write minimal implementation**
- 在 CI & Release Contract Gates 章节下补充 machine JSON key highlights（release/runner）。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_readme_mentions_machine_schema_and_basis_fields`
- Expected: PASS

---

### Task 3: R33-003 为 README(ZH) 增加 machine JSON schema/basis 文档守卫并补齐示例

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `README.zh-CN.md`

**Step 1: Write the failing test**
- 新增断言：`README.zh-CN.md` 需明确提及 machine JSON 字段：
  - `schema_version`
  - `step_rate_basis`
  - `command_rate_basis`
  - `rate_basis`

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_readme_zh_cn_mentions_machine_schema_and_basis_fields`
- Expected: FAIL（当前 README.zh-CN 未提及这些字段）

**Step 3: Write minimal implementation**
- 在“CI 与发布契约门禁”章节补充 machine JSON 字段说明（中文）。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_readme_zh_cn_mentions_machine_schema_and_basis_fields`
- Expected: PASS

---

## Batch Verification (Round 33 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- 测试记录： `scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite scripts/runtime/tests/test_ci_contract_gates`
- 全量验证记录
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
