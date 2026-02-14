# Repo Gap Priority Round 24 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 继续提升机器模式可消费性与 CI 可追踪性：release-audit step exit code、runner longest scenario 摘要、CI machine JSON 产物上传。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先新增失败测试，再做最小实现，最后 targeted + full + rust 回归。

**Tech Stack:** Bash, Python `pytest`, GitHub Actions YAML。

---

### Task 1: R24-001 release-audit step-level `exit_code`

**Files:**
- Modify: `scripts/release-contract-audit.sh`
- Modify: `scripts/runtime/tests/test_release_contract_audit_script.py`

**Step 1: RED**
- 新增测试：run-mode JSON 的 `step_results` 每项应包含 `exit_code`。
- 新增测试：forced-fail JSON 模式下 `step_results[0].exit_code=1`。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 扩展 step row schema，记录 step exit_code。
- 修复 JSON 分支失败时 exit_code 采集逻辑。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 2: R24-002 regression_runner JSON 增加 `longest_scenario`

**Files:**
- Modify: `scripts/runtime/regression_runner.py`
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite.py`

**Step 1: RED**
- 新增测试：`--suite contract --json` 输出包含 `longest_scenario`（含 name/duration_ms）。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 在 JSON payload 增加 longest_scenario 聚合字段。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 3: R24-003 CI 上传 machine JSON artifacts

**Files:**
- Modify: `.github/workflows/ci-contract-gates.yml`
- Modify: `scripts/runtime/tests/test_ci_contract_gates.py`

**Step 1: RED**
- 新增测试：workflow 包含 `actions/upload-artifact@v4` 且上传 machine JSON 文件。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 在 machine mode smoke step 保存 JSON 到 `/tmp/*.json`。
- 新增 artifact upload step。

**Step 3: VERIFY**
- 新增测试 PASS。

---

## Batch Verification (Round 24 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_achievements_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_codeagent_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_loop_guardian_script.py scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_hook_shell_runtime_path.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_ci_contract_gates.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`
