# Repo Gap Priority Round 31 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 延续 machine JSON 收敛：release-audit 增加 command 级成功/失败率，runner 增加 `total_scenarios` 显式计数，CI schema smoke 同步新键。

**Architecture:** 严格 `RED -> GREEN -> VERIFY`。先写失败测试定义契约，再最小实现，最后执行 targeted + full + rust 门禁。

**Tech Stack:** Bash, Python (`pytest`), GitHub Actions YAML。

---

### Task 1: R31-001 release-audit JSON 增加 `success_command_rate`/`failed_command_rate`

**Files:**
- Modify: `scripts/runtime/tests/test_release_contract_audit_script.py`
- Modify: `scripts/release-contract-audit.sh`

**Step 1: Write the failing test**
- run-json 场景断言：
  - `success_command_rate == 1.0`
  - `failed_command_rate == 0.0`
- force-fail 场景断言：
  - `success_command_rate == 0.0`
  - `failed_command_rate == 1.0`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: FAIL（缺 command_rate 字段）

**Step 3: Write minimal implementation**
- 基于 `commands_count` 计算：
  - `success_command_rate = success_steps_count / commands_count`
  - `failed_command_rate = failed_commands_count / commands_count`

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: PASS

---

### Task 2: R31-002 runner JSON 增加 `total_scenarios`

**Files:**
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Modify: `scripts/runtime/regression_runner.py`

**Step 1: Write the failing test**
- contract suite json 断言：
  - `total_scenarios` 字段存在
  - `total_scenarios == total`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: FAIL（缺 `total_scenarios`）

**Step 3: Write minimal implementation**
- payload 增加 `total_scenarios = total`。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: PASS

---

### Task 3: R31-003 CI schema smoke 同步 `success_command_rate/failed_command_rate/total_scenarios`

**Files:**
- Modify: `scripts/runtime/tests/test_ci_contract_gates.py`
- Modify: `.github/workflows/ci-contract-gates.yml`

**Step 1: Write the failing test**
- workflow 测试新增断言包含：
  - `success_command_rate`
  - `failed_command_rate`
  - `total_scenarios`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: FAIL（schema smoke required keys 未同步）

**Step 3: Write minimal implementation**
- required_release 增加 command rate keys。
- required_runner 增加 total_scenarios key。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: PASS

---

## Batch Verification (Round 31 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`
