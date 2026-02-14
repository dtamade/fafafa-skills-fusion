# Repo Gap Priority Round 10 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 Rust 质量门禁（clippy + fmt）并统一 `fusion-status.sh --help` 行为，保证 CLI 与质量基线一致。

**Architecture:** 严格 `RED -> GREEN -> REFACTOR`。先用失败命令/测试确认缺口，再做最小改动；任务完成后执行 Rust + Python 回归与全量验证。

**Tech Stack:** Rust (`cargo clippy/fmt/test`), Bash, Python `pytest`, Markdown。

---

### Task 1: A10 修复 `clippy::too_many_arguments`

**Files:**
- Modify: `rust/crates/fusion-cli/src/main.rs`

**Step 1: RED**
- Run: `cd rust && cargo clippy --workspace --all-targets -- -D warnings`
- Expected: FAIL，`try_inject_safe_backlog` 参数数量超限（9/7）。

**Step 2: GREEN**
- 引入 `SafeBacklogTrigger` 参数结构体，收敛 `try_inject_safe_backlog` 参数数量。
- 更新两个调用点为结构体传参。

**Step 3: VERIFY**
- Run: `cd rust && cargo clippy --workspace --all-targets -- -D warnings`
- Expected: PASS。

---

### Task 2: B10 修复 Rust `fmt --check`

**Files:**
- Modify: `rust/crates/fusion-runtime-io/src/lib.rs`
- Modify: `rust/crates/fusion-cli/src/main.rs`（随改动自动格式化）

**Step 1: RED**
- Run: `cd rust && cargo fmt --all -- --check`
- Expected: FAIL（`fusion-runtime-io/src/lib.rs` 存在格式差异）。

**Step 2: GREEN**
- Run: `cd rust && cargo fmt --all`

**Step 3: VERIFY**
- Run: `cd rust && cargo fmt --all -- --check`
- Expected: PASS。

---

### Task 3: C10 `fusion-status.sh` 支持 `--help`

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_status_script.py`
- Modify: `scripts/fusion-status.sh`

**Step 1: RED**
- 新增测试 `test_status_help_exits_zero_without_fusion_dir`。
- Run: `pytest -q scripts/runtime/tests/test_fusion_status_script.py::TestFusionStatusScript::test_status_help_exits_zero_without_fusion_dir`
- Expected: FAIL（当前无 `.fusion` 时返回 1）。

**Step 2: GREEN**
- 增加 `usage()` 与 `-h|--help` 早返回分支。

**Step 3: VERIFY**
- Run: `bash -n scripts/fusion-status.sh && pytest -q scripts/runtime/tests/test_fusion_status_script.py::TestFusionStatusScript::test_status_help_exits_zero_without_fusion_dir`
- Expected: PASS。

---

## Batch Verification (Round 10 / Batch1)

Run:
- `bash -n scripts/fusion-status.sh scripts/fusion-logs.sh scripts/fusion-git.sh scripts/fusion-codeagent.sh`
- `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_codeagent_script.py`
- `cd rust && cargo test -q && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all -- --check`
- `pytest -q`
