# Repo Convergence Change Groups

This file groups the current repository convergence work into reviewable batches.

## Group 1: Rust CLI modularization

Focus:

- Split oversized Rust control-plane modules into smaller orchestration + helper modules
- Preserve CLI behavior and public call sites
- Keep release-oriented verification green

Representative files:

- `rust/crates/fusion-cli/src/main.rs`
- `rust/crates/fusion-cli/src/render.rs`
- `rust/crates/fusion-cli/src/render_status.rs`
- `rust/crates/fusion-cli/src/render_taskplan.rs`
- `rust/crates/fusion-cli/src/render_tasks.rs`
- `rust/crates/fusion-cli/src/posttool.rs`
- `rust/crates/fusion-cli/src/posttool_progress.rs`
- `rust/crates/fusion-cli/src/posttool_runtime.rs`
- `rust/crates/fusion-cli/src/bootstrap.rs`
- `rust/crates/fusion-cli/src/bootstrap_config.rs`
- `rust/crates/fusion-cli/src/safe_backlog.rs`
- `rust/crates/fusion-cli/src/safe_backlog_core.rs`
- `rust/crates/fusion-cli/src/safe_backlog_support.rs`
- `rust/crates/fusion-cli/src/catchup.rs`
- `rust/crates/fusion-cli/src/catchup_render.rs`
- `rust/crates/fusion-cli/src/catchup_session.rs`
- `rust/crates/fusion-cli/src/catchup_taskplan.rs`
- `rust/crates/fusion-cli/src/status.rs`
- `rust/crates/fusion-cli/src/status_artifacts.rs`
- `rust/crates/fusion-cli/src/status_cmd.rs`
- `rust/crates/fusion-cli/src/status_owner.rs`
- `rust/crates/fusion-cli/src/status_render.rs`
- `rust/crates/fusion-cli/src/status_reports.rs`
- `rust/crates/fusion-cli/src/status_runtime.rs`
- `rust/crates/fusion-cli/src/runner.rs`
- `rust/crates/fusion-cli/src/runner_backend.rs`
- `rust/crates/fusion-cli/src/runner_control.rs`
- `rust/crates/fusion-cli/src/runner_route.rs`
- `rust/crates/fusion-cli/src/achievements.rs`

Suggested verification:

```bash
cd rust
cargo test --release --test cli_smoke status
cargo test --release --test cli_smoke catchup
cargo test --release --test cli_smoke hook_posttool
cargo test --release --test cli_smoke stop_guard
```

## Group 2: Shell thin-wrapper and hook contract convergence

Focus:

- Keep shell scripts as thin wrappers / hook entry adapters
- Standardize project hook wiring on `${CLAUDE_PROJECT_DIR:-.}`
- Remove ambiguity around fallback behavior

Representative files:

- `scripts/fusion-start.sh`
- `scripts/fusion-status.sh`
- `scripts/fusion-achievements.sh`
- `scripts/fusion-pause.sh`
- `scripts/fusion-resume.sh`
- `scripts/fusion-cancel.sh`
- `scripts/fusion-continue.sh`
- `scripts/fusion-logs.sh`
- `scripts/fusion-codeagent.sh`
- `scripts/fusion-pretool.sh`
- `scripts/fusion-posttool.sh`
- `scripts/fusion-stop-guard.sh`
- `scripts/fusion-hook-doctor.sh`
- `scripts/fusion-hook-selfcheck.sh`
- `scripts/lib/fusion-bridge.sh`
- `scripts/lib/fusion-hook-adapter.sh`
- `scripts/lib/fusion-hook-common.sh`
- `scripts/lib/fusion-hook-doctor-core.sh`
- `scripts/lib/fusion-hook-doctor-behavior.sh`
- `scripts/lib/fusion-hook-doctor-summary.sh`
- `scripts/lib/fusion-hook-selfcheck-core.sh`
- `scripts/lib/fusion-posttool-fallback.sh`
- `scripts/lib/fusion-pretool-fallback.sh`
- `scripts/lib/fusion-stop-guard-common.sh`
- `scripts/lib/fusion-stop-guard-fallback.sh`
- `scripts/lib/fusion-task-plan.sh`
- `scripts/fusion-catchup.sh`

Suggested verification:

```text
Recorded checks:
- scripts/runtime/tests/test_fusion_start_script
- scripts/runtime/tests/test_fusion_hook_doctor_script
- scripts/runtime/tests/test_fusion_hook_selfcheck_script
- scripts/runtime/tests/test_hook_shell_runtime_path
```

## Group 3: Removed-runtime compatibility and reference boundary

Focus:

- Treat the now-removed runtime as a compatibility, regression, and reference surface rather than a peer primary control plane
- Review remaining direct state/session helpers and replay fixtures explicitly
- Make the live-path versus reference-only boundary readable for future maintainers

Representative files:

- `scripts/runtime/compat_v2`
- `scripts/runtime/kernel`
- `scripts/runtime/_session_store`
- `scripts/runtime/regression_runner`
- `scripts/runtime/tests/test_compat_v2`
- `scripts/runtime/tests/test_resume_replay`
- `scripts/runtime/tests/test_session_store`
- `docs/RUST_FUSION_BRIDGE_ROADMAP.md`
- `docs/RUNTIME_KERNEL_DESIGN.md`
- `docs/UPGRADE_v2_COMPAT.md`

Suggested verification:

```text
Recorded checks:
- scripts/runtime/tests/test_compat_v2
- scripts/runtime/tests/test_resume_replay
- scripts/runtime/tests/test_session_store
```

## Group 4: Documentation and repository hygiene

Focus:

- Make Rust-first architecture explicit
- Document shell and removed-runtime roles precisely
- Document runtime artifact placement and root cleanup policy
- Keep release CI guidance aligned with the real workflow

Representative files:

- `README.md`
- `README.zh-CN.md`
- `rust/README.md`
- `docs/HOOKS_SETUP.md`
- `docs/CLI_CONTRACT_MATRIX.md`
- `docs/REPO_HYGIENE.md`
- `docs/REPO_CONVERGENCE_SUMMARY_2026-03.md`
- `CHANGELOG.md`
- `.gitignore`
- `examples/root-session/README.md`

Suggested verification:

```text
Recorded checks:
- scripts/runtime/tests/test_docs_freshness
- scripts/runtime/tests/test_ci_contract_gates
- scripts/runtime/tests/test_repo_hygiene
```

## Group 5: Tests and CI contract updates

Focus:

- Keep CI release-oriented
- Update tests for hook path contract and repo hygiene
- Keep smoke coverage aligned with the converged Rust control plane

Representative files:

- `.github/workflows/ci-contract-gates.yml`
- `scripts/runtime/tests/test_ci_contract_gates`
- `scripts/runtime/tests/test_docs_freshness`
- `scripts/runtime/tests/test_repo_hygiene`
- `scripts/runtime/tests/test_fusion_status_script`
- `scripts/runtime/tests/test_fusion_codeagent_script`
- `scripts/runtime/tests/test_fusion_control_script_validation`
- `scripts/runtime/tests/test_fusion_hook_doctor_script`
- `scripts/runtime/tests/test_fusion_hook_selfcheck_script`
- `scripts/runtime/tests/test_fusion_start_script`
- `scripts/runtime/tests/test_hook_shell_runtime_path`
- `rust/crates/fusion-cli/tests/cli_smoke.rs`

Suggested verification:

```text
Recorded full verification batch
Rust release bundle:
cd rust && cargo test --release
```

## Review order

Recommended review / commit order:

1. Group 1: Rust CLI modularization
2. Group 2: Shell thin-wrapper and hook contract convergence
3. Group 3: Removed-runtime compatibility and reference boundary
4. Group 4: Documentation and repository hygiene
5. Group 5: Tests and CI contract updates

## Notes

- `rust/.cargo-codex/` is environment cache and is now ignored; it should not be reviewed as source.
- The current worktree still contains many modified files from earlier repository evolution; reviewers should focus on the grouped intent above rather than raw diff size alone.

> Archive note: this review keeps its historical context. For current behavior, use the Rust and shell contracts.
