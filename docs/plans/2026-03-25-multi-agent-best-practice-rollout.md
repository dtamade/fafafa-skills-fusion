# Multi-Agent Best-Practice Rollout Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Land a post-GA multi-agent roadmap that keeps Fusion's current single-runner reliability, clarifies the role of `role_handoff`, and introduces true parallel workers only after worker isolation, lease control, and merge gates exist.

**Architecture:** Treat the current agent spine as a control-plane foundation, not as proof of true concurrent execution. Promote multi-agent in layers: document and test the current boundary first, then add worker leases and isolated workspaces, then add a real orchestrated parallel worker executor, and only then consider default promotion.

**Tech Stack:** Rust (`fusion-cli`, `fusion-runtime-io`), Bash thin wrappers, Markdown docs, contract tests, release-oriented CI.

---

### Task 1: Freeze the live documentation boundary

**Files:**
- Create: `docs/MULTI_AGENT_EXECUTION_ROADMAP.md`
- Modify: `docs/V3_GA_EXECUTION_ROADMAP.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `PARALLEL_EXECUTION.md`

**Intent:**
- Define three operating modes: `single_runner`, `role_handoff`, and post-GA `parallel_workers`
- State clearly that current batch planning is not the same as true parallel workers
- Keep GA and post-GA scope separate

**Verification:**
- Read the affected docs together and confirm they all describe the same boundary

### Task 2: Add repo contract coverage for the new doc truth

**Files:**
- Modify: `rust/crates/fusion-cli/tests/repo_contract.rs`

**Intent:**
- Fail if README or parallel docs claim that current `single_orchestrator` behavior already equals true parallel workers
- Fail if the new roadmap reference disappears from live docs

**Verification:**
- Run: `cd rust && cargo test --release -p fusion-cli --test repo_contract`

### Task 3: Design worker lease and isolation state

**Files:**
- Modify: `rust/crates/fusion-cli/src/runner.rs`
- Modify: `rust/crates/fusion-cli/src/status.rs`
- Modify: `rust/crates/fusion-cli/src/status_runtime.rs`
- Modify: `rust/crates/fusion-runtime-io/src/lib.rs`
- Test: `rust/crates/fusion-cli/tests/cli_smoke.rs`

**Intent:**
- Add `_runtime.agents.workers.*` state for lease id, task id, workspace path, role, backend, status, and lease timeout
- Keep the orchestrator as the only assignment authority
- Expose worker state on the existing status surfaces

**Verification:**
- Run: `cd rust && cargo test --release -p fusion-cli --test cli_smoke`

### Task 4: Add isolated workspace provisioning

**Files:**
- Modify: `rust/crates/fusion-cli/src/runner.rs`
- Modify: `rust/crates/fusion-cli/src/git.rs` or supporting workspace helper modules
- Modify: `rust/crates/fusion-cli/src/reporting.rs` if worker workspace paths need surfaced output
- Test: `rust/crates/fusion-cli/tests/cli_smoke.rs`

**Intent:**
- Provision one isolated workspace per worker
- Prevent shared-workspace parallel writes from becoming the default implementation path
- Make worker cleanup deterministic

**Verification:**
- Run: `cd rust && cargo test --release -p fusion-cli --test cli_smoke parallel`

### Task 5: Promote batch planning into a true parallel worker executor

**Files:**
- Modify: `rust/crates/fusion-cli/src/agent_orchestrator.rs`
- Modify: `rust/crates/fusion-cli/src/runner.rs`
- Modify: `rust/crates/fusion-cli/src/runner_route.rs`
- Test: `rust/crates/fusion-cli/tests/cli_smoke.rs`

**Intent:**
- Turn selected batch tasks into active worker leases
- Run independent tasks concurrently only when dependencies are satisfied and writes do not conflict
- Preserve one-task-per-worker discipline

**Verification:**
- Run: `cd rust && cargo test --release -p fusion-cli --test cli_smoke`
- Run: `cd rust && cargo test --release -p fusion-cli --test shell_contract`

### Task 6: Add merge and review gates for worker outputs

**Files:**
- Modify: `rust/crates/fusion-cli/src/agent_handoff.rs`
- Modify: `rust/crates/fusion-cli/src/runner.rs`
- Modify: `rust/crates/fusion-cli/src/status.rs`
- Test: `rust/crates/fusion-cli/tests/cli_smoke.rs`

**Intent:**
- Ensure worker completion does not equal repository integration
- Centralize merge approval and reviewer gating under orchestrator control
- Expose merge queue and pending review state on status surfaces

**Verification:**
- Run: `cd rust && cargo test --release -p fusion-cli --test cli_smoke role_handoff`

### Task 7: Define promotion criteria and operator defaults

**Files:**
- Modify: `templates/config.yaml`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/MULTI_AGENT_EXECUTION_ROADMAP.md`

**Intent:**
- Keep `agents.enabled: false` as the safe default until worker evidence is strong enough
- Define when `role_handoff` should be recommended
- Define when `parallel_workers` may be enabled and when it must stay off

**Verification:**
- Read the config comments and README guidance together

### Task 8: Run the release-oriented verification bundle

**Files:**
- No code changes; verification only

**Intent:**
- Prove the multi-agent roadmap and any landed implementation do not break the current release contract

**Verification:**
- Run:
  - `cd rust && cargo build --release -p fusion-cli --bin fusion-bridge`
  - `cd rust && cargo clippy --release --workspace --all-targets -- -D warnings`
  - `cd rust && cargo test --release`
  - `cd rust && cargo fmt --all -- --check`
  - `bash scripts/ci-machine-mode-smoke.sh`
  - `bash scripts/ci-cross-platform-smoke.sh`

## Delivery notes

- Batch A: complete Tasks 1-2 before any executor changes
- Batch B: complete Tasks 3-4 before claiming true parallel worker safety
- Batch C: complete Tasks 5-6 before exposing `parallel_workers` as a live option
- Batch D: complete Tasks 7-8 before any promotion discussion
