# Repo Gap Priority Round 21 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 强化机器模式输出可读性与自动化消费能力：release-audit 增加 pretty JSON，regression_runner 支持 `--list-suites --json`，并将新能力同步文档契约。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先新增失败测试，再最小实现，最后 targeted + full 回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: R21-001 release-audit `--json-pretty`

**Files:**
- Modify: `scripts/release-contract-audit.sh`
- Modify: `scripts/runtime/tests/test_release_contract_audit_script`

**Step 1: RED**
- 新增测试：`--dry-run --json --json-pretty --fast --skip-rust` 应返回多行缩进 JSON。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 新增 `--json-pretty` 参数。
- JSON 输出支持 pretty 模式（缩进）。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 2: R21-002 regression_runner `--list-suites --json`

**Files:**
- Modify: `scripts/runtime/regression_runner`
- Modify: `scripts/runtime/tests/test_regression_runner_contract_suite`

**Step 1: RED**
- 新增测试：`--list-suites --json` 返回 JSON payload（包含 suites + default）。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 新增 runner `--json` 参数并在 list-suites 场景输出 JSON。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 3: R21-003 文档契约同步

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/HOOKS_SETUP.md`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`

**Step 1: RED**
- 新增 docs freshness 断言：文档出现 `--json-pretty`、`regression_runner --list-suites --json`。
- 运行 docs 测试，预期 FAIL。

**Step 2: GREEN**
- 补齐文档说明与命令示例。

**Step 3: VERIFY**
- docs freshness 测试 PASS。

---

## Batch Verification (Round 21 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite scripts/runtime/tests/test_docs_freshness`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_stop_guard_script scripts/runtime/tests/test_hook_shell_runtime_path scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_ci_contract_gates scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite`
- 全量验证记录
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

