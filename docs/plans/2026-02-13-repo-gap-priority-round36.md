# Round36 Repo Gap Priority Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在全仓门禁持续全绿前提下，补齐 artifact 文档契约，确保 HOOKS/Matrix/README 对 CI machine JSON 产物说明一致。

**Architecture:** 继续以 `scripts/runtime/tests/test_docs_freshness` 做文档契约守门测试，先加失败断言（RED），再做最小文档补全（GREEN），每个任务独立 VERIFY，最后执行 targeted/full/shell/rust 回归。

**Tech Stack:** Markdown docs, Bash verification

---

### Task 1: HOOKS_SETUP 增加 CI machine artifact 文件说明

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `docs/HOOKS_SETUP.md`
- Test: `scripts/runtime/tests/test_docs_freshness`

**Step 1: Write the failing test**

```text
def test_hooks_setup_mentions_ci_machine_artifacts(self):
    content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
    self.assertIn("/tmp/release-audit-dry-run.json", content)
    self.assertIn("/tmp/runner-contract.json", content)
```

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_hooks_setup_mentions_ci_machine_artifacts`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `docs/HOOKS_SETUP.md` 的 machine-readable 段落新增 `CI machine artifact examples`，至少包含：
- `/tmp/release-audit-dry-run.json`
- `/tmp/runner-contract.json`

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_hooks_setup_mentions_ci_machine_artifacts`
Expected: PASS

**Step 5: Checkpoint**

Run: `bash -n scripts/*.sh`
Expected: `shell-syntax:OK`

### Task 2: CLI Contract Matrix Notes 增加 artifact 文件说明

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`
- Test: `scripts/runtime/tests/test_docs_freshness`

**Step 1: Write the failing test**

```text
def test_cli_contract_matrix_notes_mention_ci_machine_artifacts(self):
    content = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")
    self.assertIn("/tmp/release-audit-dry-run.json", content)
    self.assertIn("/tmp/runner-contract.json", content)
```

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_cli_contract_matrix_notes_mention_ci_machine_artifacts`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `docs/CLI_CONTRACT_MATRIX.md` 的 Notes 段落增加 CI machine artifact 文件说明，至少包含：
- `/tmp/release-audit-dry-run.json`
- `/tmp/runner-contract.json`

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_cli_contract_matrix_notes_mention_ci_machine_artifacts`
Expected: PASS

**Step 5: Checkpoint**

测试记录： `scripts/runtime/tests/test_docs_freshness`
Expected: PASS

### Task 3: README EN/ZH 增加 runner-suites artifact 文件说明

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Test: `scripts/runtime/tests/test_docs_freshness`

**Step 1: Write the failing test**

```text
def test_readme_en_zh_mention_runner_suites_artifact(self):
    readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
    readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
    self.assertIn("/tmp/runner-suites.json", readme_en)
    self.assertIn("/tmp/runner-suites.json", readme_zh)
```

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_readme_en_zh_mention_runner_suites_artifact`
Expected: FAIL

**Step 3: Write minimal implementation**

在 `README.md` 与 `README.zh-CN.md` 的 CI & Release Contract Gates 章节 artifact 列表中新增：
- `/tmp/runner-suites.json`

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_docs_freshness::TestDocsFreshness::test_readme_en_zh_mention_runner_suites_artifact`
Expected: PASS

**Step 5: Verify task scope**

测试记录： `scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite scripts/runtime/tests/test_ci_contract_gates`
Expected: PASS

### Final Verification Bundle

Run:
- `bash -n scripts/*.sh`
- 测试记录： `scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite scripts/runtime/tests/test_ci_contract_gates`
- 全量验证记录
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

Expected:
- all checks pass with zero regressions

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

