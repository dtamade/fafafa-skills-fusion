# Round34 Repo Gap Priority Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在保持全仓门禁全绿的前提下，补齐 machine JSON 文档契约缺口，并用 docs freshness 测试建立回归守卫。

**Architecture:** 以 `scripts/runtime/tests/test_docs_freshness.py` 作为文档契约守门测试，先写失败断言，再最小化补全文档（`docs/HOOKS_SETUP.md`、`docs/CLI_CONTRACT_MATRIX.md`、`README.md`、`README.zh-CN.md`）。每个任务独立 RED→GREEN→VERIFY，最后执行 targeted + full + shell + rust 回归。

**Tech Stack:** Python unittest (pytest runner), Markdown docs, Bash tooling

---

### Task 1: HOOKS_SETUP 增加 schema/basis 字段语义说明

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `docs/HOOKS_SETUP.md`
- Test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: Write the failing test**

```python
def test_hooks_setup_mentions_machine_schema_and_basis_fields(self):
    content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
    self.assertIn("schema_version", content)
    self.assertIn("step_rate_basis", content)
    self.assertIn("command_rate_basis", content)
    self.assertIn("rate_basis", content)
```

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_machine_schema_and_basis_fields`
Expected: FAIL (HOOKS_SETUP currently missing these fields)

**Step 3: Write minimal implementation**

在 `docs/HOOKS_SETUP.md` 的 machine-readable 段落补充字段要点，至少包含：
- `schema_version`
- `step_rate_basis`
- `command_rate_basis`
- `rate_basis`

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_machine_schema_and_basis_fields`
Expected: PASS

**Step 5: Checkpoint**

Run: `bash -n scripts/*.sh`
Expected: `shell-syntax:OK`

### Task 2: CLI Contract Matrix 增加 machine required keys 说明

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`
- Test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: Write the failing test**

```python
def test_cli_contract_matrix_has_machine_required_keys_note(self):
    content = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")
    self.assertIn("Required machine JSON keys", content)
    self.assertIn("release-contract-audit", content)
    self.assertIn("regression_runner", content)
```

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_has_machine_required_keys_note`
Expected: FAIL (matrix currently has no dedicated required-keys note)

**Step 3: Write minimal implementation**

在 `docs/CLI_CONTRACT_MATRIX.md` 新增 `Required machine JSON keys (minimum)` 段落，列出：
- release-contract-audit: `schema_version`, `step_rate_basis`, `command_rate_basis`
- regression_runner(contract): `schema_version`, `rate_basis`, `total_scenarios`

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_has_machine_required_keys_note`
Expected: PASS

**Step 5: Checkpoint**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py`
Expected: PASS

### Task 3: README EN/ZH 增补 basis 分母语义说明

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: Write the failing test**

```python
def test_readme_en_zh_explain_basis_denominators(self):
    readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
    readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
    self.assertIn("step_rate_basis=total_steps", readme_en)
    self.assertIn("command_rate_basis=total_commands", readme_en)
    self.assertIn("step_rate_basis=total_steps", readme_zh)
    self.assertIn("command_rate_basis=total_commands", readme_zh)
```

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_explain_basis_denominators`
Expected: FAIL (README EN/ZH currently lack explicit denominator text)

**Step 3: Write minimal implementation**

在 `README.md` 与 `README.zh-CN.md` 的 machine JSON 字段说明中新增语义行：
- `step_rate_basis=total_steps`
- `command_rate_basis=total_commands`
- 维持现有 `rate_basis=total_scenarios` 关系说明

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_explain_basis_denominators`
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
- all commands pass with zero regressions
