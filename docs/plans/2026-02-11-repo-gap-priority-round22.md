# Repo Gap Priority Round 22 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把机器模式补齐到“可直接被自动化消费”：release-audit 运行态 JSON 指标、runner 套件执行 JSON 汇总、CI 增加机器模式 smoke gate。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`；先锁失败测试，再最小实现，最后全量回归与日志回填。

**Tech Stack:** Bash, Python `pytest`, GitHub Actions YAML。

---

### Task 1: R22-001 release-audit 运行态 JSON 指标

**Files:**
- Modify: `scripts/release-contract-audit.sh`
- Modify: `scripts/runtime/tests/test_release_contract_audit_script.py`

**Step 1: RED**
- 新增测试：`--json --fast --skip-rust` 输出包含：
  - `steps_executed`
  - `total_duration_ms`
  - `step_results`（数组，含 `status`/`duration_ms`/`command`）
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 增加运行过程计时与步骤结果收集。
- JSON 输出补齐运行指标字段。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 2: R22-002 regression_runner suite 执行 JSON 汇总

**Files:**
- Modify: `scripts/runtime/regression_runner.py`
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite.py`

**Step 1: RED**
- 新增测试：`--suite contract --json` 返回 JSON 汇总（`result/suite/passed/total/pass_rate`）。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 为 suite 执行路径增加 JSON 输出分支（保留文本模式）。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 3: R22-003 CI 增加机器模式 smoke gate

**Files:**
- Modify: `.github/workflows/ci-contract-gates.yml`
- Modify: `scripts/runtime/tests/test_ci_contract_gates.py`

**Step 1: RED**
- 新增测试：workflow 包含：
  - `release-contract-audit.sh --dry-run --json`
  - `regression_runner.py --list-suites --json`
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 在 workflow 增加 dedicated smoke step 执行上述命令。

**Step 3: VERIFY**
- 新增测试 PASS。

---

## Batch Verification (Round 22 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_achievements_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_codeagent_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_loop_guardian_script.py scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_hook_shell_runtime_path.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_ci_contract_gates.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`
