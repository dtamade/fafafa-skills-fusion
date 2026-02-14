# Repo Gap Priority Round 23 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 继续机器化收口：release-audit JSON 增加步骤时间戳/序号，runner suite JSON 增加场景详情，CI 补 suite JSON smoke 命令。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。新增失败测试先锁定契约，再做最小实现，最后 targeted + full 回归。

**Tech Stack:** Bash, Python `pytest`, GitHub Actions YAML。

---

### Task 1: R23-001 release-audit step metrics 明细化

**Files:**
- Modify: `scripts/release-contract-audit.sh`
- Modify: `scripts/runtime/tests/test_release_contract_audit_script.py`

**Step 1: RED**
- 新增测试：run-mode JSON 的 `step_results` 每项包含：
  - `step`
  - `started_at_ms`
  - `finished_at_ms`
  - 且 `finished_at_ms >= started_at_ms`
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 在脚本步骤执行时记录 `step_start_ms/step_end_ms/step index`。
- 扩展 JSON payload 的 `step_results`。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 2: R23-002 regression_runner suite JSON 场景详情

**Files:**
- Modify: `scripts/runtime/regression_runner.py`
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite.py`

**Step 1: RED**
- 新增测试：`--suite contract --json` 输出新增字段：
  - `scenario_results`（name/passed/duration_ms/error）
  - `failed_scenarios`
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 在 JSON 模式下输出场景数组与失败列表。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 3: R23-003 CI 补 suite JSON smoke

**Files:**
- Modify: `.github/workflows/ci-contract-gates.yml`
- Modify: `scripts/runtime/tests/test_ci_contract_gates.py`

**Step 1: RED**
- 新增测试：workflow 必须包含：
  - `regression_runner.py --suite contract --json`
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 在 machine mode smoke gate 中追加 suite JSON 命令。

**Step 3: VERIFY**
- 新增测试 PASS。

---

## Batch Verification (Round 23 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_achievements_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_codeagent_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_loop_guardian_script.py scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_hook_shell_runtime_path.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_ci_contract_gates.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`
