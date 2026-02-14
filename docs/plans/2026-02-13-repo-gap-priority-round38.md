# Round38 Backend Failure Observability Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 解决团队调度中“双后端失败不可观测”的缺口，让失败信息可机器读取、可在状态命令中展示，并同步文档。

**Architecture:** 继续沿用 shell+pytest 契约驱动。先在 `fusion-codeagent.sh` 写入 `backend_failure_report.json`，再让 `fusion-status.sh` 读取该报告暴露摘要，最后通过 docs freshness 强制 README 中英长期同步该运维路径。

**Tech Stack:** Bash, Python unittest (pytest runner), Markdown docs.

---

### Task 1: codeagent 双后端失败报告

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_codeagent_script.py`
- Modify: `scripts/fusion-codeagent.sh`

**Step 1: Write the failing test**

在 `test_fusion_codeagent_script.py` 新增：
- 双后端均失败时，返回非 0。
- 生成 `.fusion/backend_failure_report.json`。
- 报告包含 `status=blocked`、`source=fusion-codeagent.sh`、`primary_backend`、`fallback_backend`、`primary_error`、`fallback_error`。

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_fusion_codeagent_script.py::TestFusionCodeagentScript::test_double_backend_failure_writes_backend_failure_report`
Expected: FAIL（文件不存在或字段缺失）

**Step 3: Write minimal implementation**

在 `scripts/fusion-codeagent.sh`：
- 新增 `write_backend_failure_report`。
- 在 primary+fallback 全失败路径写 `.fusion/backend_failure_report.json`。
- 在任意成功路径删除旧 `backend_failure_report.json`（避免陈旧阻塞状态）。

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_fusion_codeagent_script.py::TestFusionCodeagentScript::test_double_backend_failure_writes_backend_failure_report`
Expected: PASS

**Step 5: Verify task scope regression**

Run: `pytest -q scripts/runtime/tests/test_fusion_codeagent_script.py`
Expected: PASS

---

### Task 2: status 暴露 backend failure 摘要

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script.py`
- Modify: `scripts/fusion-status.sh`

**Step 1: Write the failing tests**

新增两类断言：
- JSON 模式：当存在 `.fusion/backend_failure_report.json` 时，输出 `backend_status`、`backend_primary`、`backend_fallback`。
- 人类模式：输出 `## Backend Failure Report` 与关键字段。

**Step 2: Run tests to verify they fail**

Run:
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py::TestFusionStatusScript::test_status_json_includes_backend_failure_summary`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py::TestFusionStatusScript::test_status_prints_backend_failure_report`
Expected: FAIL（字段/区块不存在）

**Step 3: Write minimal implementation**

在 `scripts/fusion-status.sh`：
- JSON 模式读取 `backend_failure_report.json`，扩展摘要字段。
- 人类输出增加 `## Backend Failure Report` 区块。

**Step 4: Run tests to verify they pass**

Run:
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py::TestFusionStatusScript::test_status_json_includes_backend_failure_summary`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py::TestFusionStatusScript::test_status_prints_backend_failure_report`
Expected: PASS

**Step 5: Verify task scope regression**

Run: `pytest -q scripts/runtime/tests/test_fusion_status_script.py`
Expected: PASS

---

### Task 3: README 中英补齐 backend failure 指引

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness.py`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: Write the failing test**

在 `test_docs_freshness.py` 增加断言：
- `README.md` 与 `README.zh-CN.md` 都必须提及 `.fusion/backend_failure_report.json`。

**Step 2: Run test to verify it fails**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_backend_failure_report`
Expected: FAIL

**Step 3: Write minimal implementation**

在两份 README 的 Dependency Auto-Heal 段落补充：
- 缺依赖写 `dependency_report.json`
- 双后端调用失败写 `backend_failure_report.json`

**Step 4: Run test to verify it passes**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_backend_failure_report`
Expected: PASS

**Step 5: Verify task scope regression**

Run: `pytest -q scripts/runtime/tests/test_docs_freshness.py`
Expected: PASS

---

### Final Verification Bundle

Run:
- `bash -n scripts/*.sh`
- `pytest -q scripts/runtime/tests/test_fusion_codeagent_script.py scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

Expected:
- all commands pass with no regressions
