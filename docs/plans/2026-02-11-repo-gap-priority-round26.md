# Repo Gap Priority Round 26 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 继续强化 machine JSON 契约：release-audit 输出 failed command 聚合、runner 输出结果分布统计、CI machine smoke 扩展多文件 schema 校验。

**Architecture:** 严格 `RED -> GREEN -> VERIFY`。先新增失败测试锁定契约缺口，再做最小实现，最后执行 targeted + full + rust 门禁闭环。

**Tech Stack:** Bash, Python (`pytest`), GitHub Actions YAML。

---

### Task 1: R26-001 release-audit JSON 增加 `failed_commands`

**Files:**
- Modify: `scripts/runtime/tests/test_release_contract_audit_script.py`
- Modify: `scripts/release-contract-audit.sh`

**Step 1: Write the failing test**
- run-json 成功场景断言：`failed_commands` 是空列表。
- force-fail 场景断言：`failed_commands == ["bash -n scripts/*.sh"]`。

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: FAIL（缺 `failed_commands`）

**Step 3: Write minimal implementation**
- 基于 `step_results` 聚合错误命令列表，输出 `failed_commands`。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py`
- Expected: PASS

---

### Task 2: R26-002 runner JSON 增加 `scenario_count_by_result`

**Files:**
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Modify: `scripts/runtime/regression_runner.py`

**Step 1: Write the failing test**
- `--suite contract --json` 断言新增字段：
  - `scenario_count_by_result.passed`
  - `scenario_count_by_result.failed`
- 并断言 `passed == payload["passed"]`、`failed == len(failed_scenarios)`。

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: FAIL（缺 `scenario_count_by_result`）

**Step 3: Write minimal implementation**
- 在 payload 中聚合并输出 `scenario_count_by_result`。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py`
- Expected: PASS

---

### Task 3: R26-003 CI machine smoke 增加 release/suites schema 校验

**Files:**
- Modify: `scripts/runtime/tests/test_ci_contract_gates.py`
- Modify: `.github/workflows/ci-contract-gates.yml`

**Step 1: Write the failing test**
- workflow 测试断言 schema smoke 涵盖：
  - `/tmp/release-audit-dry-run.json`
  - `/tmp/runner-suites.json`
  - 关键字段：`failed_commands`、`scenario_count_by_result`、`default_suite`

**Step 2: Run test to verify it fails**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: FAIL（缺新的 schema 校验片段）

**Step 3: Write minimal implementation**
- 在 machine mode 的 python 校验片段中：
  - 校验 runner-contract 包含 `scenario_count_by_result`。
  - 校验 release-audit-dry-run 包含 `failed_commands`。
  - 校验 runner-suites 包含 `default_suite` 与 `suites`。

**Step 4: Run test to verify it passes**
- Run: `pytest -q scripts/runtime/tests/test_ci_contract_gates.py`
- Expected: PASS

---

## Batch Verification (Round 26 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`
