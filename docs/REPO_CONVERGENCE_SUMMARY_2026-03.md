# Repo Convergence Summary (2026-03)

This document summarizes the repository convergence work completed in the March 2026 cleanup cycle.

## Outcome

The project now has a clearer single-path architecture:

- Rust / `fusion-bridge` is the primary control plane
- Shell remains a thin wrapper and hook wiring layer
- Former runtime/reference layer has been removed from the repository

## Contracts unified

### Hook path contract

Project hook wiring is standardized on exactly one form:

```bash
bash "${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh"
bash "${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh"
bash "${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh"
```

Older forms such as `${CLAUDE_PROJECT_DIR}` without fallback and bare `bash scripts/...` are treated as outdated wiring.

### Runtime artifact contract

- Live mutable workflow state belongs in `.fusion/`
- Checked-in seed files belong in `templates/`
- Checked-in illustrative layouts belong in `examples/`
- Root-level `task_plan.md`, `progress.md`, and `findings.md` are not canonical repository inputs
- Generated local Rust caches such as `rust/target/` and `rust/.cargo-codex/` stay ignored and are not repository evidence
- Host-local tool settings such as `.ace-tool/`, `.claude/settings.json`, and `.claude/settings.local.json` are local machine state and not part of the repository contract; `.claude/settings.example.json` remains the checked-in template

## Codebase structure improvements

Several oversized Rust entry modules were split into smaller modules with stable exports preserved:

- `render.rs`
- `posttool.rs`
- `bootstrap.rs`
- `safe_backlog.rs`
- `catchup.rs`
- `status_render.rs`
- `status.rs`
- `runner.rs`

The goal of these splits was not cosmetic line-count reduction alone; it was to separate orchestration from parsing, reporting, state transitions, and helper logic so behavior remains easier to verify.

## CI and verification alignment

Release-oriented Rust verification is now the documented default:

```bash
cd rust
cargo test --release
```

CI gate workflow:

- `.github/workflows/ci-contract-gates.yml`

Key enforced checks include:

- `bash scripts/ci-machine-mode-smoke.sh`
- `bash scripts/ci-cross-platform-smoke.sh`
- `cargo clippy --release --workspace --all-targets -- -D warnings`
- `cargo test --release`
- `cargo fmt --all -- --check`

## Documentation alignment

The following docs were aligned to the converged model:

- `README.md`
- `README.zh-CN.md`
- `docs/HOOKS_SETUP.md`
- `docs/CLI_CONTRACT_MATRIX.md`
- `docs/UPGRADE_v2_COMPAT.md`
- `docs/COMPATIBILITY.md`
- `PARALLEL_EXECUTION.md`
- `rust/README.md`
- `CONTRIBUTING.md`
- `CONTRIBUTING.zh-CN.md`
- `docs/REPO_HYGIENE.md`

These docs now consistently describe the same baseline:

- Rust / `fusion-bridge` is the primary control plane
- Shell remains thin wrapper and hook wiring glue
- Release-oriented Rust verification is the default
- `jq` is optional for machine JSON smoke or manual inspection, not a runtime requirement
- When that baseline changes, update the affected active docs together with `rust/crates/fusion-cli/tests/repo_contract.rs`

## Remaining work that is still reasonable

The highest-value follow-up work is no longer aggressive module splitting. More useful next steps are:

1. Keep README and docs index curated as architecture changes land
2. Continue pruning historical references that imply the removed runtime/reference layer is still present in active repository paths
3. Keep examples under `examples/` rather than reintroducing root runtime artifacts
4. Use release-oriented Rust verification consistently in future maintenance work

## Non-goals

This cleanup did not try to remove every compatibility path or rewrite all historical docs. The goal was convergence with behavior stability, not a risky rewrite.
