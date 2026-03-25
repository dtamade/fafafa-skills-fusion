# PR Summary: Repo Convergence Cleanup

## What changed

This cleanup converges the repository onto a clearer operating model:

- Rust / `fusion-bridge` is the primary control plane
- Shell remains a thin wrapper and hook wiring layer
- At the time of this archived review, the now-removed runtime still existed for compatibility, regression fixtures, and tests

## Main outcomes

### 1. Rust control-plane modules were split into focused units

Large orchestration files were reduced by moving parsing, reporting, state-transition, and helper logic into smaller modules while preserving CLI behavior and call sites.

Notable areas:

- render/status/task-plan helpers
- posttool progress/runtime helpers
- bootstrap config helpers
- safe backlog support/core helpers
- catchup render/task-plan/session helpers
- status runtime/report helpers
- runner route/control/backend helpers

### 2. Hook wiring contract was unified

Project hook commands now standardize on:

```bash
bash "${CLAUDE_PROJECT_DIR:-.}/scripts/<hook>.sh"
```

Older forms such as `${CLAUDE_PROJECT_DIR}` without fallback and bare `bash scripts/...` are now treated as outdated wiring.

### 3. Repository runtime artifact policy was clarified

- Live mutable workflow state belongs in `.fusion/`
- Checked-in seed files belong in `templates/`
- Illustrative examples belong in `examples/`
- Root runtime files such as `task_plan.md`, `progress.md`, and `findings.md` are not canonical repository inputs

### 4. Documentation and CI guidance were aligned

- Rust README consistently uses release-oriented commands
- Hook setup docs now describe the single hook path contract
- Repository hygiene and convergence summary docs were added
- Changelog now records this cleanup in `Unreleased`

## New/updated maintainer docs

- `docs/REPO_HYGIENE.md`
- `docs/REPO_CONVERGENCE_SUMMARY_2026-03.md`
- `reviews/repo-convergence-change-groups.md`

## Verification run

Recorded docs/repo checks:

```text
Recorded checks:
- scripts/runtime/tests/test_docs_freshness
- scripts/runtime/tests/test_ci_contract_gates
- scripts/runtime/tests/test_repo_hygiene
```

Rust release smoke checks:

```bash
cd rust
cargo test --release --test cli_smoke status
cargo test --release --test cli_smoke catchup
cargo test --release --test cli_smoke hook_posttool
cargo test --release --test cli_smoke stop_guard
```

## Reviewer guidance

Recommended review order:

1. Rust CLI modularization
2. Shell thin-wrapper / hook contract convergence
3. Compatibility / reference boundary for the now-removed runtime
4. Documentation and repository hygiene
5. Tests and CI contract updates

Supporting grouping file:

- `reviews/repo-convergence-change-groups.md`

## Notes

- `rust/.cargo-codex/` is environment cache and is intentionally ignored
- The cleanup prioritizes convergence with behavior stability; it is not a full historical rewrite

> Archive note: this review keeps its historical context. For current behavior, use the Rust and shell contracts.
