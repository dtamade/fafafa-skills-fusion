# Repo Gap Priority Round 25 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 提升 machine JSON 契约可消费性：release-audit 聚合失败摘要、runner 输出 fastest_scenario、CI 增加 runner JSON schema smoke。

**Architecture:** 采用严格 `RED -> GREEN -> VERIFY`。先写失败测试锁定缺口，再最小实现，最后执行 targeted + full + rust 门禁，避免回归。

**Tech Stack:** Bash, GitHub Actions YAML。

---

### Task 1: R25-001 release-audit JSON 增加 `failed_steps` 聚合

**Files:**
- Modify: `scripts/runtime/tests/test_release_contract_audit_script`
- Modify: `scripts/release-contract-audit.sh`

**Step 1: Write the failing test**
- 在 run-json 场景断言 payload 包含：
  - `failed_steps`（list）
  - `failed_steps_count`（int）
- 在 forced-fail 场景断言：
  - `failed_steps == [1]`
  - `failed_steps_count == 1`

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script`
- Expected: FAIL（缺 `failed_steps` / `failed_steps_count`）

**Step 3: Write minimal implementation**
- 在 `emit_json_summary` 的旧 runtime payload 中从 `step_results` 计算：
  - `failed_steps`
  - `failed_steps_count`

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script`
- Expected: PASS

---

### Task 2: R25-002 regression_runner JSON 增加 `fastest_scenario`

**Files:**
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite`
- Modify: `scripts/runtime/regression_runner`

**Step 1: Write the failing test**
- 在 contract suite json 测试断言新增：
  - `fastest_scenario` 字段存在
  - 含 `name` / `duration_ms`
  - `fastest_scenario.duration_ms <= longest_scenario.duration_ms`

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_regression_runner_contract_suite`
- Expected: FAIL（缺 `fastest_scenario`）

**Step 3: Write minimal implementation**
- 在 JSON payload 聚合 `fastest_scenario`（基于 `scenario_results` 的最小 `duration_ms`）

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_regression_runner_contract_suite`
- Expected: PASS

---

### Task 3: R25-003 CI machine mode 增加 runner JSON schema smoke

**Files:**
- Modify: `scripts/runtime/tests/test_ci_contract_gates`
- Modify: `.github/workflows/ci-contract-gates.yml`

**Step 1: Write the failing test**
- workflow 测试新增断言：
  - machine smoke step 包含当时记录下来的 runner 命令片段 `- <<'PY'`
  - schema smoke 读取 `/tmp/runner-contract.json`
  - 校验 key: `longest_scenario` / `fastest_scenario`

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_ci_contract_gates`
- Expected: FAIL（缺 schema smoke）

**Step 3: Write minimal implementation**
- 在 workflow machine smoke step 中加入旧 runtime schema smoke 片段，失败即退出非0。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_ci_contract_gates`
- Expected: PASS

---

## Batch Verification (Round 25 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite scripts/runtime/tests/test_ci_contract_gates`
- 全量验证记录
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
