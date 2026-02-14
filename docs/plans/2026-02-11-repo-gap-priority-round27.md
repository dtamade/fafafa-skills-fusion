# Repo Gap Priority Round 27 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 继续强化 machine JSON 契约完整性：release-audit 增加 failed command 计数、runner 增加耗时统计、CI machine schema smoke 同步新字段。

**Architecture:** 严格执行 `RED -> GREEN -> VERIFY`。先用失败测试锁住契约，再做最小实现，最后跑 targeted + full + rust 门禁闭环。

**Tech Stack:** Bash, Python (`pytest`), GitHub Actions YAML。

---

### Task 1: R27-001 release-audit JSON 增加 `failed_commands_count`

**Files:**
- Modify: `scripts/runtime/tests/test_release_contract_audit_script.py`
- Modify: `scripts/release-contract-audit.sh`

**Step 1: Write the failing test**
- run-json 场景断言：`failed_commands_count == 0`。
- force-fail 场景断言：`failed_commands_count == 1`。

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: FAIL（缺 `failed_commands_count`）

**Step 3: Write minimal implementation**
- 在 release-audit JSON payload 中新增 `failed_commands_count = len(failed_commands)`。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: PASS

---

### Task 2: R27-002 runner JSON 增加 `duration_stats`

**Files:**
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Modify: `scripts/runtime/regression_runner.py`

**Step 1: Write the failing test**
- contract suite json 断言新增 `duration_stats`：
  - `min_duration_ms`
  - `max_duration_ms`
  - `avg_duration_ms`
- 并断言 `max >= min` 且 `avg` 在区间内。

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: FAIL（缺 `duration_stats`）

**Step 3: Write minimal implementation**
- 基于 `scenario_results` 计算最小/最大/平均耗时并输出 `duration_stats`。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: PASS

---

### Task 3: R27-003 CI machine schema smoke 同步新字段

**Files:**
- Modify: `scripts/runtime/tests/test_ci_contract_gates.py`
- Modify: `.github/workflows/ci-contract-gates.yml`

**Step 1: Write the failing test**
- workflow 测试新增断言包含：
  - `failed_commands_count`
  - `duration_stats`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: FAIL（schema smoke required keys 未同步）

**Step 3: Write minimal implementation**
- CI schema smoke required key 列表补齐：
  - release: `failed_commands_count`
  - runner-contract: `duration_stats`

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: PASS

---

## Batch Verification (Round 27 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`
