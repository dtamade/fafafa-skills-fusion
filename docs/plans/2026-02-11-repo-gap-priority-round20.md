# Repo Gap Priority Round 20 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 继续收口发布契约方向的 P1 能力：release-audit 机器可读 summary、regression_runner 套件可发现性、CI 缓存门禁加速。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`，新增失败测试先锁定行为，再做最小实现，最后 targeted + full 回归。

**Tech Stack:** Bash, Python `pytest`, GitHub Actions YAML。

---

### Task 1: R20-001 release-audit `--json` summary

**Files:**
- Modify: `scripts/release-contract-audit.sh`
- Modify: `scripts/runtime/tests/test_release_contract_audit_script.py`

**Step 1: RED**
- 新增测试：`--dry-run --json` 返回合法 JSON，包含命令列表与 flags。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 新增 `--json` 开关；dry-run 模式输出 JSON summary。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 2: R20-002 regression_runner `--list-suites`

**Files:**
- Modify: `scripts/runtime/regression_runner.py`
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite.py`

**Step 1: RED**
- 新增测试：`--list-suites` 返回可用套件清单（phase1/phase2/contract/all）并 exit 0。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 添加 `--list-suites` 参数并输出 suite 清单。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 3: R20-003 CI workflow 缓存门禁

**Files:**
- Modify: `.github/workflows/ci-contract-gates.yml`
- Modify: `scripts/runtime/tests/test_ci_contract_gates.py`

**Step 1: RED**
- 新增测试：workflow 必须包含 pip cache 与 cargo cache 相关步骤。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 在 workflow 增加 pip cache 与 rust cache。

**Step 3: VERIFY**
- 新增测试 PASS。

---

## Batch Verification (Round 20 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_achievements_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_codeagent_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_loop_guardian_script.py scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_hook_shell_runtime_path.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_ci_contract_gates.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`
