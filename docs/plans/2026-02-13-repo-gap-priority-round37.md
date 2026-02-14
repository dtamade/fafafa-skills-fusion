# Round37 Repo Gap Priority Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在门禁全绿前提下继续收敛文档契约，补齐 HOOKS 与 README 的 schema/version 与 artifact 一致性说明。

**Architecture:** 继续用 `scripts/runtime/tests/test_docs_freshness.py` 做文档契约回归门。每个任务先新增失败断言（RED），再最小文档修改（GREEN），随后单项和范围验证（VERIFY），最后执行全量门禁。

**Tech Stack:** Python unittest (pytest), Markdown docs, Bash verification

---

### Task 1: HOOKS_SETUP 增加 runner-suites artifact 文案

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `docs/HOOKS_SETUP.md`
- Test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: Write the failing test**

```python
def test_hooks_setup_mentions_runner_suites_artifact(self):
    content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
    self.assertIn("/tmp/runner-suites.json", content)
```

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_runner_suites_artifact`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `docs/HOOKS_SETUP.md` 的 `CI machine artifact examples` 段落新增：
- `/tmp/runner-suites.json`

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_runner_suites_artifact`
Expected: PASS

**Step 5: Checkpoint**

Run: `bash -n scripts/*.sh`
Expected: `shell-syntax:OK`

### Task 2: HOOKS_SETUP 增加 schema_version=v1 契约文案

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `docs/HOOKS_SETUP.md`
- Test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: Write the failing test**

```python
def test_hooks_setup_mentions_schema_version_v1(self):
    content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
    self.assertIn("schema_version=v1", content)
```

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_schema_version_v1`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `docs/HOOKS_SETUP.md` machine JSON 说明区新增一行：
- `Current schema contract: schema_version=v1`

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_schema_version_v1`
Expected: PASS

**Step 5: Checkpoint**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py`
Expected: PASS

### Task 3: README EN/ZH 增加 schema_version=v1 契约文案

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: Write the failing test**

```python
def test_readme_en_zh_mention_schema_version_v1(self):
    readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
    readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
    self.assertIn("schema_version=v1", readme_en)
    self.assertIn("schema_version=v1", readme_zh)
```

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_schema_version_v1`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `README.md` 与 `README.zh-CN.md` 的 CI & Release Contract Gates 章节新增：
- `Current schema contract: schema_version=v1`

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_schema_version_v1`
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
