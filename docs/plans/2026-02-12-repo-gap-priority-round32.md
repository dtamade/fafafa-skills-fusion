# Repo Gap Priority Round 32 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 继续收敛 machine JSON 契约，补齐 release-audit / runner 的 schema 与 rate basis 元信息，并在 CI machine smoke 中做一致性门禁校验。

**Architecture:** 严格 `RED -> GREEN -> VERIFY`。每个任务先加失败测试，再最小实现；最后执行 targeted + full + rust 门禁，确保无回归。

**Tech Stack:** Bash, GitHub Actions YAML。

---

### Task 1: R32-001 release-audit JSON 增加 `schema_version` / `step_rate_basis` / `command_rate_basis`

**Files:**
- Modify: `scripts/runtime/tests/test_release_contract_audit_script`
- Modify: `scripts/release-contract-audit.sh`

**Step 1: Write the failing test**
- run-json 场景断言：
  - `schema_version` 字段存在且为稳定版本值
  - `step_rate_basis == steps_executed`
  - `command_rate_basis == commands_count`
- force-fail 场景断言同样包含上述 3 个字段。

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script`
- Expected: FAIL（缺少新增 schema/basis 字段）

**Step 3: Write minimal implementation**
- `emit_json_summary` payload 增加：
  - `schema_version`
  - `step_rate_basis`
  - `command_rate_basis`

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script`
- Expected: PASS

---

### Task 2: R32-002 runner contract JSON 增加 `schema_version` / `rate_basis`

**Files:**
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite`
- Modify: `scripts/runtime/regression_runner`

**Step 1: Write the failing test**
- contract suite json 断言：
  - `schema_version` 字段存在且稳定
  - `rate_basis` 字段存在
  - `rate_basis == total_scenarios`

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_regression_runner_contract_suite`
- Expected: FAIL（缺少新增 schema/basis 字段）

**Step 3: Write minimal implementation**
- runner JSON payload 增加：
  - `schema_version`
  - `rate_basis = total`

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_regression_runner_contract_suite`
- Expected: PASS

---

### Task 3: R32-003 CI machine smoke 同步 required keys 并增加 basis 一致性校验

**Files:**
- Modify: `scripts/runtime/tests/test_ci_contract_gates`
- Modify: `.github/workflows/ci-contract-gates.yml`

**Step 1: Write the failing test**
- workflow 测试新增断言包含：
  - `schema_version`
  - `step_rate_basis`
  - `command_rate_basis`
  - `rate_basis`
  - 以及一致性校验文案（`rate_basis`, `step_rate_basis`, `command_rate_basis` 对应校验语句）

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_ci_contract_gates`
- Expected: FAIL（workflow schema smoke 未同步）

**Step 3: Write minimal implementation**
- machine smoke 旧 runtime 片段中：
  - `required_release` 增加 `schema_version`, `step_rate_basis`, `command_rate_basis`
  - `required_runner` 增加 `schema_version`, `rate_basis`
  - 新增一致性检查：
    - `runner_contract["rate_basis"] == runner_contract["total_scenarios"]`
    - `release_dry_run["step_rate_basis"] == release_dry_run["steps_executed"]`
    - `release_dry_run["command_rate_basis"] == release_dry_run["commands_count"]`

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_ci_contract_gates`
- Expected: PASS

---

## Batch Verification (Round 32 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite scripts/runtime/tests/test_ci_contract_gates`
- 全量验证记录
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
