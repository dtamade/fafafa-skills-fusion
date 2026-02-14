# Repo Gap Priority Round 18 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完成当前方向收尾：建立 CI 契约门禁、沉淀统一 CLI 参数契约矩阵、提供发布前一键审计脚本，确保 hook/契约能力可持续。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先补失败测试锁定缺口，再最小实现，最后执行 targeted + full 回归。

**Tech Stack:** Bash, GitHub Actions, Python `pytest`, Markdown。

---

### Task 1: R18-001 CI 门禁落地

**Files:**
- Add: `.github/workflows/ci-contract-gates.yml`
- Add/Modify test: `scripts/runtime/tests/test_ci_contract_gates.py`

**Step 1: RED**
- 新增 CI 契约测试：要求 workflow 存在并包含以下门禁命令：
  - `bash -n scripts/*.sh`
  - `pytest -q`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
- 运行新测试，预期 FAIL（workflow 尚不存在）。

**Step 2: GREEN**
- 新增 workflow，触发 `push`/`pull_request`。
- 按步骤执行 shell / python / rust 门禁。

**Step 3: VERIFY**
- 新测试 PASS。

---

### Task 2: R18-002 CLI Contract Matrix 文档

**Files:**
- Add: `docs/CLI_CONTRACT_MATRIX.md`
- Modify test: `scripts/runtime/tests/test_docs_freshness.py`

**Step 1: RED**
- 新增 docs freshness 测试：要求 contract matrix 文档存在并覆盖 13+ CLI（含 `fusion-stop-guard.sh`）以及列定义（`command` / `valid args` / `invalid args` / `exit code` / `stdout/stderr/json expectations`）。
- 运行对应测试，预期 FAIL（文档尚不存在）。

**Step 2: GREEN**
- 编写矩阵文档，覆盖核心脚本参数契约、错误契约与机器模式。

**Step 3: VERIFY**
- docs freshness 新增测试 PASS。

---

### Task 3: R18-003 发布前自动审计脚本

**Files:**
- Add: `scripts/release-contract-audit.sh`
- Add test: `scripts/runtime/tests/test_release_contract_audit_script.py`

**Step 1: RED**
- 新增脚本测试，至少校验：
  - `--help` 返回 usage 且 exit 0。
  - `--dry-run` 打印完整审计命令清单。
  - 未知选项返回非 0 并提示错误。
- 运行新测试，预期 FAIL（脚本尚不存在）。

**Step 2: GREEN**
- 实现审计脚本，默认执行：
  - shell syntax
  - contract 相关 tests
  - 全量 `pytest -q`
  - rust clippy/fmt
- 支持 `--dry-run` 仅打印命令。

**Step 3: VERIFY**
- 新增脚本测试 PASS。

---

## Batch Verification (Round 18 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- `pytest -q scripts/runtime/tests/test_ci_contract_gates.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_docs_freshness.py`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_achievements_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_codeagent_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_loop_guardian_script.py scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_hook_shell_runtime_path.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_ci_contract_gates.py scripts/runtime/tests/test_release_contract_audit_script.py`
- `pytest -q`
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

