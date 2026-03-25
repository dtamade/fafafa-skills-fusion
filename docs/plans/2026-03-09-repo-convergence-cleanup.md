# Repo Convergence Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Unify hook path contracts, prefer the Rust bridge as the main execution path, clean repository runtime artifacts, and tighten CI/docs around the converged workflow.

**Architecture:** Keep the current behavior stable while converging on a single preferred path. Standardize generated hook wiring on one exact placeholder form, centralize shell-to-Rust bridge delegation in a shared helper, and move mutable planning artifacts out of the repository root into `.fusion/` templates/examples so the repo tracks only canonical inputs.

**Tech Stack:** Bash, Rust workspace (`clap`, `serde`), GitHub Actions.

---

### Task 1: Unify hook path contract

**Files:**
- Modify: `scripts/fusion-hook-doctor.sh`
- Modify: `scripts/fusion-start.sh`
- Modify: `docs/HOOKS_SETUP.md`
- Modify: `scripts/runtime/tests/test_fusion_start_script`
- Modify: `scripts/runtime/tests/test_fusion_hook_doctor_script`

**Step 1: Write/adjust failing tests**
- Make the start/doctor tests assert the same canonical hook command string.

**Step 2: Run tests to verify failure**
- 测试记录： `scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_fusion_hook_doctor_script`

**Step 3: Minimal implementation**
- Standardize generated hook commands and related wording to `${CLAUDE_PROJECT_DIR:-.}/scripts/...`.
- Keep backward-compatible detection for legacy `${CLAUDE_PROJECT_DIR}/...` entries so existing users are not broken.

**Step 4: Run tests to green**
- Re-run the focused test command recorded for this task.

### Task 2: Prefer Rust bridge from shell entrypoints

**Files:**
- Create: `scripts/lib/fusion-bridge.sh`
- Modify: `scripts/fusion-init.sh`
- Modify: `scripts/fusion-start.sh`
- Modify: `scripts/fusion-status.sh`
- Modify: `scripts/fusion-resume.sh`
- Modify: `scripts/fusion-codeagent.sh`
- Modify: `scripts/fusion-pretool.sh`
- Modify: `scripts/fusion-posttool.sh`
- Modify: `scripts/fusion-stop-guard.sh`
- Test: `scripts/runtime/tests/`

**Step 1: Write failing tests**
- Add shell-script tests that prove the scripts prefer `fusion-bridge` when available and can opt out via env.

**Step 2: Run targeted tests to verify red**
- Run the focused test command recorded for the new delegation behavior.

**Step 3: Minimal implementation**
- Add a shared helper to locate `fusion-bridge` (release binary or `cargo run --release` fallback).
- Keep shell-specific pre/post work that Rust does not own yet, but delegate the core command behavior to Rust by default.

**Step 4: Run tests to green**
- Re-run the focused shell delegation tests.

### Task 3: Remove mutable root runtime artifacts

**Files:**
- Delete: `findings.md`
- Delete: `progress.md`
- Delete: `task_plan.md`
- Create: `examples/root-session/README.md`
- Modify: `.gitignore`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `CONTRIBUTING.md`
- Modify: `CONTRIBUTING.zh-CN.md`

**Step 1: Write/adjust tests/docs checks**
- Add or extend docs tests to assert root runtime artifacts are not tracked as canonical working files.

**Step 2: Run focused tests to verify red**
- Run docs freshness and any new repo hygiene test.

**Step 3: Minimal implementation**
- Remove tracked mutable runtime files from root.
- Document `.fusion/` as the runtime workspace and `templates/` as canonical checked-in seeds.

**Step 4: Run tests to green**
- Re-run docs/repo hygiene tests.

### Task 4: Tighten CI and release docs

**Files:**
- Modify: `.github/workflows/ci-contract-gates.yml`
- Modify: `rust/README.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: Write failing tests/checks**
- Add docs freshness checks for release-oriented Rust commands and CI coverage mention if missing.

**Step 2: Run focused tests to verify red**
- Run the specific docs freshness tests.

**Step 3: Minimal implementation**
- Add `cargo test --release` to CI.
- Update Rust README commands to use `--release` consistently.

**Step 4: Run tests to green**
- Re-run docs freshness tests.

### Task 5: Reduce duplicated parsing/oversized entrypoints

**Files:**
- Modify: `scripts/fusion-status.sh`
- Modify: `scripts/fusion-pretool.sh`
- Modify: `scripts/fusion-posttool.sh`
- Modify: `scripts/fusion-resume.sh`
- Modify: `scripts/fusion-stop-guard.sh`
- Modify: `rust/crates/fusion-cli/src/main.rs`

**Step 1: Write failing tests**
- Add targeted tests around shared parsing helpers / bridge delegation where behavior is currently duplicated.

**Step 2: Run focused tests to verify red**
- Run only the tests covering the extracted helpers.

**Step 3: Minimal implementation**
- Replace repeated shell JSON parsing with shared helper functions.
- Split Rust `main.rs` into small modules without changing CLI behavior.

**Step 4: Run tests to green**
- Re-run focused tests, then broader old-runtime/Rust suites.

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
