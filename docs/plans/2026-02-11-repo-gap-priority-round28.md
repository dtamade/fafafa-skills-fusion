# Repo Gap Priority Round 28 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 继续完善 machine JSON 可消费性：release-audit 增加成功/命令计数，runner 增加失败率，CI schema smoke 同步新契约字段。

**Architecture:** 严格 `RED -> GREEN -> VERIFY`。先新增失败测试锁定行为，再做最小实现，最后执行 targeted + full + rust 门禁。

**Tech Stack:** Bash, Python (`pytest`), GitHub Actions YAML。

---

### Task 1: R28-001 release-audit JSON 增加 `success_steps_count` 与 `commands_count`

**Files:**
- Modify: `scripts/runtime/tests/test_release_contract_audit_script.py`
- Modify: `scripts/release-contract-audit.sh`

**Step 1: Write the failing test**
- run-json 场景断言：
  - `success_steps_count == steps_executed`
  - `commands_count == len(commands)`
- force-fail 场景断言：
  - `success_steps_count == 0`
  - `commands_count == len(commands)`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: FAIL（缺 `success_steps_count/commands_count`）

**Step 3: Write minimal implementation**
- payload 增加：
  - `success_steps_count = len(step_results) - len(failed_steps)`
  - `commands_count = len(commands)`

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: PASS

---

### Task 2: R28-002 runner JSON 增加 `failed_rate`

**Files:**
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Modify: `scripts/runtime/regression_runner.py`

**Step 1: Write the failing test**
- contract suite json 断言：
  - `failed_rate` 字段存在
  - `failed_rate == len(failed_scenarios) / total`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: FAIL（缺 `failed_rate`）

**Step 3: Write minimal implementation**
- payload 增加 `failed_rate` 聚合字段。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: PASS

---

### Task 3: R28-003 CI machine schema smoke 同步 `success_steps_count/commands_count/failed_rate`

**Files:**
- Modify: `scripts/runtime/tests/test_ci_contract_gates.py`
- Modify: `.github/workflows/ci-contract-gates.yml`

**Step 1: Write the failing test**
- workflow 测试新增断言包含：
  - `success_steps_count`
  - `commands_count`
  - `failed_rate`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: FAIL（schema smoke required keys 未同步）

**Step 3: Write minimal implementation**
- CI machine schema smoke required keys补齐：
  - release: `success_steps_count`, `commands_count`
  - runner-contract: `failed_rate`

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: PASS

---

## Batch Verification (Round 28 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`
