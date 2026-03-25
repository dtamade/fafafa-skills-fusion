# Repo Gap Priority Round 19 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 Round18 基础上完成“发布门禁可操作化”收尾：增强发布审计脚本参数能力与失败汇总、补齐回归运行器 contract 套件、强化文档契约与新鲜度守卫。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先加失败测试锁定预期行为，再进行最小实现，最后 targeted + full 回归。

**Tech Stack:** Bash, Markdown。

---

### Task 1: R19-001 发布审计脚本能力增强

**Files:**
- Modify: `scripts/release-contract-audit.sh`
- Modify: `scripts/runtime/tests/test_release_contract_audit_script`

**Step 1: RED**
- 增加以下测试：
  - `--dry-run --fast` 跳过当时记录的全量测试批次
  - `--dry-run --skip-rust` 不包含 rust 命令
  - `--dry-run --skip-legacy-path` 不包含当时记录的测试命令
  - `FUSION_RELEASE_AUDIT_FORCE_FAIL_STEP` 触发失败时输出 step summary
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 为脚本增加组合参数：`--fast` / `--skip-rust` / `--skip-legacy-path`。
- 增加失败汇总输出：`failed at step N`。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 2: R19-002 regression_runner contract 套件

**Files:**
- Modify: `scripts/runtime/regression_runner`
- Add: `scripts/runtime/tests/test_regression_runner_contract_suite`

**Step 1: RED**
- 新增测试：
  - `--suite contract` 走 contract 套件分支。
  - 未知 suite 返回非 0 并提示错误。
- 运行测试，预期 FAIL。

**Step 2: GREEN**
- 在 runner 中新增 `CONTRACT_SCENARIOS`。
- 参数分支支持 `contract`；未知 suite 显式报错。

**Step 3: VERIFY**
- 新增测试 PASS。

---

### Task 3: R19-003 文档契约强化

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`
- Modify: `docs/HOOKS_SETUP.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: RED**
- 新增 docs freshness 断言：
  - matrix 包含 `help exit code` 列。
  - `HOOKS_SETUP` / `README` / `README.zh-CN` 包含 `release-contract-audit.sh`。
  - `README` / `README.zh-CN` 包含 `ci-contract-gates.yml`。
- 运行 docs 测试，预期 FAIL。

**Step 2: GREEN**
- 更新文档并补齐契约信息。

**Step 3: VERIFY**
- docs freshness 测试 PASS。

---

## Batch Verification (Round 19 / Batch1)

Run:
- `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh`
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite scripts/runtime/tests/test_docs_freshness`
- 测试记录： `scripts/runtime/tests/test_fusion_status_script scripts/runtime/tests/test_fusion_achievements_script scripts/runtime/tests/test_fusion_control_script_validation scripts/runtime/tests/test_fusion_codeagent_script scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_loop_guardian_script scripts/runtime/tests/test_fusion_stop_guard_script scripts/runtime/tests/test_hook_shell_runtime_path scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_ci_contract_gates scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_regression_runner_contract_suite`
- 全量验证记录
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
