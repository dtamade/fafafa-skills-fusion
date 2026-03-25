# Repo Gap Priority Round 29 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 继续增强 machine JSON 一致性：release-audit 增加成功/失败率，runner 增加 `success_rate` 显式字段，CI schema smoke 同步新字段。

**Architecture:** 严格 `RED -> GREEN -> VERIFY`。先写失败测试明确契约，再做最小实现，最后做 targeted + full + rust 门禁收口。

**Tech Stack:** Bash, GitHub Actions YAML。

---

### Task 1: R29-001 release-audit JSON 增加 `success_rate`/`failed_rate`

**Files:**
- Modify: `scripts/runtime/tests/test_release_contract_audit_script`
- Modify: `scripts/release-contract-audit.sh`

**Step 1: Write the failing test**
- run-json 场景断言：
  - `success_rate == 1.0`
  - `failed_rate == 0.0`
- force-fail 场景断言：
  - `success_rate == 0.0`
  - `failed_rate == 1.0`

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script`
- Expected: FAIL（缺 success_rate/failed_rate）

**Step 3: Write minimal implementation**
- 计算：
  - `success_rate = success_steps_count / steps_executed`（0 步时 0.0）
  - `failed_rate = failed_steps_count / steps_executed`（0 步时 0.0）

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script`
- Expected: PASS

---

### Task 2: R29-002 runner JSON 增加 `success_rate`

**Files:**
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite`
- Modify: `scripts/runtime/regression_runner`

**Step 1: Write the failing test**
- contract suite json 断言：
  - `success_rate` 字段存在
  - `success_rate == passed / total`
  - `success_rate + failed_rate == 1.0`（容差断言）

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_regression_runner_contract_suite`
- Expected: FAIL（缺 success_rate）

**Step 3: Write minimal implementation**
- payload 增加 `success_rate` 聚合字段。

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_regression_runner_contract_suite`
- Expected: PASS

---

### Task 3: R29-003 CI machine schema smoke 同步 rate 字段

**Files:**
- Modify: `scripts/runtime/tests/test_ci_contract_gates`
- Modify: `.github/workflows/ci-contract-gates.yml`

**Step 1: Write the failing test**
- workflow 测试新增断言包含：
  - `success_rate`
  - `failed_rate`（release 与 runner 两侧都应出现）

**Step 2: Run test to verify it fails**
- 测试记录： `scripts/runtime/tests/test_ci_contract_gates`
- Expected: FAIL（schema smoke required keys 未同步）

**Step 3: Write minimal implementation**
- CI schema smoke required keys补齐：
  - release: `success_rate`, `failed_rate`
  - runner-contract: `success_rate`, `failed_rate`

**Step 4: Run test to verify it passes**
- 测试记录： `scripts/runtime/tests/test_ci_contract_gates`
- Expected: PASS

---

## Batch Verification (Round 29 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite scripts/runtime/tests/test_ci_contract_gates`
- 全量验证记录
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

