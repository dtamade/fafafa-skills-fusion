# Round35 Repo Gap Priority Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在保持门禁全绿前提下，继续收敛 machine JSON 文档契约，补齐版本约束、分母语义和 CI artifact 使用说明。

**Architecture:** 继续以 `scripts/runtime/tests/test_docs_freshness.py` 做文档契约守卫，按任务逐项新增失败断言（RED），再做最小文档变更（GREEN），最后做 targeted/full/shell/rust 回归（VERIFY）。

**Tech Stack:** Python unittest (pytest), Markdown docs, Bash verification

---

### Task 1: HOOKS_SETUP 补齐 basis 分母语义文案

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `docs/HOOKS_SETUP.md`
- Test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: Write the failing test**

```python
def test_hooks_setup_explains_basis_denominators(self):
    content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
    self.assertIn("step_rate_basis=total_steps", content)
    self.assertIn("command_rate_basis=total_commands", content)
```

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_explains_basis_denominators`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `docs/HOOKS_SETUP.md` 的 `Machine JSON key highlights` 段落新增分母语义行：
- `step_rate_basis=total_steps`
- `command_rate_basis=total_commands`
- `rate_basis=total_scenarios`

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_explains_basis_denominators`
Expected: PASS

**Step 5: Checkpoint**

Run: `bash -n scripts/*.sh`
Expected: `shell-syntax:OK`

### Task 2: CLI Contract Matrix 增加 `schema_version=v1` 约束说明

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`
- Test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: Write the failing test**

```python
def test_cli_contract_matrix_mentions_schema_version_v1(self):
    content = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")
    self.assertIn("schema_version=v1", content)
```

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_mentions_schema_version_v1`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `docs/CLI_CONTRACT_MATRIX.md` 的 `Required machine JSON keys (minimum)` 段落后补一行：
- `Current schema contract: schema_version=v1`

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_mentions_schema_version_v1`
Expected: PASS

**Step 5: Checkpoint**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py`
Expected: PASS

### Task 3: README EN/ZH 增补 CI machine artifact 文件说明

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: Write the failing test**

```python
def test_readme_en_zh_mention_ci_machine_artifacts(self):
    readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
    readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
    for marker in ("/tmp/release-audit-dry-run.json", "/tmp/runner-contract.json"):
        self.assertIn(marker, readme_en)
        self.assertIn(marker, readme_zh)
```

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_ci_machine_artifacts`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `README.md` 与 `README.zh-CN.md` 的 CI & Release Contract Gates 章节增加 artifact 文件说明，至少包含：
- `/tmp/release-audit-dry-run.json`
- `/tmp/runner-contract.json`

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_ci_machine_artifacts`
Expected: PASS

**Step 5: Verify task scope**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
Expected: PASS

### Final Verification Bundle

Run:
- `bash -n scripts/*.sh`
- `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

Expected:
- all checks pass with no regressions
