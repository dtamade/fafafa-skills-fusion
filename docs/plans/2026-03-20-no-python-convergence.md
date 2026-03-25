# Runtime Retirement Convergence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove repository dependence on the runtime being retired by migrating operational shell tooling, runtime reference paths, regression tooling, and CI/test infrastructure onto Rust implementations.

**Architecture:** Treat `fusion-bridge` as the only executable control plane and progressively collapse shell scripts into thin Rust wrappers. Retire the older runtime in layers: operational shell tools first, then runtime/reference code, then runtime-based regression/CI/test surfaces, while preserving existing CLI contracts and release verification.

**Tech Stack:** Rust (`fusion-cli`, `fusion-runtime-io`), Bash thin wrappers, GitHub Actions, Markdown docs.

---

### Phase 1: Retire the runtime from operational shell tools

**Files:**
- Modify: `scripts/release-contract-audit.sh`
- Modify: `scripts/fusion-hook-selfcheck.sh`
- Modify: `scripts/runtime/tests/test_release_contract_audit_script`
- Modify: `scripts/runtime/tests/test_fusion_hook_selfcheck_script`
- Modify: `docs/HOOKS_SETUP.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`

**Intent:**
- `release-contract-audit.sh` becomes a Rust-only wrapper around `fusion-bridge audit`
- `fusion-hook-selfcheck.sh` becomes a Rust-only wrapper around `fusion-bridge selfcheck`
- No inline historical runner stubs, shell-side JSON assembly, or shell-side historical test orchestration remains in these two scripts

### Phase 2: Retire the runtime reference layer

**Files:**
- Delete/replace: `scripts/runtime/compat_v2`
- Delete/replace: `scripts/runtime/kernel`
- Delete/replace: `scripts/runtime/state_machine`
- Delete/replace: `scripts/runtime/scheduler`
- Delete/replace: `scripts/runtime/router`
- Delete/replace: `scripts/runtime/budget_manager`
- Delete/replace: `scripts/runtime/event_bus`
- Delete/replace: `scripts/runtime/task_graph`
- Delete/replace: `scripts/runtime/conflict_detector`
- Delete/replace: `scripts/runtime/_session_store`
- Modify: Rust equivalents under `rust/crates/fusion-cli/src/`

**Intent:**
- Remove runtime behavior from that older path, not just entrypoints
- Keep any required behavior in Rust and port tests to Rust equivalents

### Phase 3: Retire runtime tooling and verification

**Files:**
- Delete/replace: `scripts/runtime/regression_runner`
- Delete/replace: `scripts/runtime/bench_hook_latency`
- Delete/replace: `scripts/runtime/bench_parallel_sim`
- Delete/replace: `scripts/runtime/understand`
- Modify: `.github/workflows/ci-contract-gates.yml`
- Modify: `scripts/lib/fusion-hook-selfcheck-core.sh` or remove it entirely
- Modify: Rust verification commands in `rust/crates/fusion-cli/src/audit.rs`

**Intent:**
- Replace historical test-command / older runtime runner expectations with Rust-native contract commands
- Remove the older runtime setup/bootstrap steps and machine-mode helpers from CI

### Phase 4: Remove older runtime references from docs and repo hygiene

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/HOOKS_SETUP.md`
- Modify: `docs/UPGRADE_v2_COMPAT.md`
- Modify: `docs/COMPATIBILITY.md`
- Modify: `rust/README.md`
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `scripts/runtime/tests/test_repo_hygiene`

**Intent:**
- Docs must stop describing the older runtime as a recommended or available dependency
- Repo hygiene should fail if tooling from that older runtime returns

### Verification Strategy

**Phase 1 verification**
- `bash -n scripts/release-contract-audit.sh scripts/fusion-hook-selfcheck.sh`
- 测试记录： `scripts/runtime/tests/test_release_contract_audit_script scripts/runtime/tests/test_fusion_hook_selfcheck_script`
- `cd rust && cargo test --release -p fusion-cli --test cli_smoke`

**End-state verification**
- No older runtime setup/bootstrap references or historical test-command wording remain in active CI/docs
- No operational shell scripts call the older runtime or outdated runner stubs
- No `scripts/runtime/*` production/reference modules remain
- Rust release gates prove the replacement behavior

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
