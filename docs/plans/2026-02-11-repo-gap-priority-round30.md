# Repo Gap Priority Round 30 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 继续收敛 machine JSON 契约可读性：release-audit 增加失败步数别名，runner 增加成功/失败计数字段，CI schema smoke 同步新增字段。

**Architecture:** 严格 `RED -> GREEN -> VERIFY`。先写失败测试锁定行为，再做最小实现，最后执行 targeted + full + rust 门禁验证。

**Tech Stack:** Bash, Python (`pytest`), GitHub Actions YAML。

---

### Task 1: R30-001 release-audit JSON 增加 `error_step_count`

**Files:**
- Modify: `scripts/runtime/tests/test_release_contract_audit_script.py`
- Modify: `scripts/release-contract-audit.sh`

**Step 1: Write the failing test**
- run-json 场景断言：`error_step_count == failed_steps_count == 0`。
- force-fail 场景断言：`error_step_count == failed_steps_count == 1`。

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: FAIL（缺 `error_step_count`）

**Step 3: Write minimal implementation**
- payload 增加 `error_step_count = len(failed_steps)`。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: PASS

---

### Task 2: R30-002 runner JSON 增加 `success_count`/`failure_count`

**Files:**
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Modify: `scripts/runtime/regression_runner.py`

**Step 1: Write the failing test**
- contract suite json 断言：
  - `success_count` 字段存在且等于 `passed`
  - `failure_count` 字段存在且等于 `len(failed_scenarios)`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: FAIL（缺 `success_count/failure_count`）

**Step 3: Write minimal implementation**
- payload 增加 `success_count` 与 `failure_count` 聚合字段。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: PASS

---

### Task 3: R30-003 CI schema smoke 同步 `error_step_count/success_count/failure_count`

**Files:**
- Modify: `scripts/runtime/tests/test_ci_contract_gates.py`
- Modify: `.github/workflows/ci-contract-gates.yml`

**Step 1: Write the failing test**
- workflow 测试新增断言包含：
  - `error_step_count`
  - `success_count`
  - `failure_count`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: FAIL（schema smoke required keys 未同步）

**Step 3: Write minimal implementation**
- CI schema smoke required keys补齐：
  - release: `error_step_count`
  - runner-contract: `success_count`, `failure_count`

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: PASS

---

## Batch Verification (Round 30 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`
