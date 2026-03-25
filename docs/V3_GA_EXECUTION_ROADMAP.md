# V3 GA Execution Roadmap

> Status: Active
> Role: current execution source of truth for v3 GA work

This document is the current execution source of truth for v3 GA work.
Historical context remains in `ROADMAP.md` and `docs/RUST_FUSION_BRIDGE_ROADMAP.md`, but they no longer define the live implementation order by themselves.

## Current position

The repository is at **v2.6 convergence complete, before v3.0 GA**.

- Rust / `fusion-bridge` is already the primary control plane
- Shell remains a thin wrapper and hook wiring layer
- The former runtime/reference layer has been removed from the repository

## Minimum GA scope

The target is a releaseable minimum GA scope for the current converged architecture.

The current GA batch must:

- keep the existing command surface stable
- make release evidence and contract truth easy to audit
- close the remaining publish-blocking gaps around docs, contracts, and platform evidence

The current GA batch must **not** expand scope in historical directions that are not required for release:

- do not add `/fusion explain` or dual-model collaboration to the current GA batch
- do not re-open multi-engine runtime design as a live path
- do not rewrite historical roadmap documents into live truth sources

## Source of truth order

Use these sources in order when deciding current behavior:

1. `docs/CLI_CONTRACT_MATRIX.md`
2. Active behavior plus active docs such as `README.md`, `README.zh-CN.md`, `rust/README.md`, `docs/COMPATIBILITY.md`, `docs/HOOKS_SETUP.md`, and `SESSION_RECOVERY.md`
3. This roadmap
4. Historical context only: `ROADMAP.md` and `docs/RUST_FUSION_BRIDGE_ROADMAP.md`

If the live contract changes, update the affected active docs together with `rust/crates/fusion-cli/tests/repo_contract.rs` and the relevant shell/CLI contract tests.

## Execution batches

### Batch 1: Truth-source and doc alignment

- Keep this roadmap as the single live roadmap entrypoint
- Make README and docs indexes point here before historical roadmap docs
- Keep historical roadmap docs labeled as background, not current implementation truth

### Batch 2: Release evidence and platform closure

Release-blocking evidence must stay green for:

- Linux
- macOS
- Windows (Git Bash)

Current status: macOS and Windows (Git Bash) smoke jobs are already wired, but active docs should keep partial-verification wording until fresh CI evidence upgrades that status. Their remote CI runs should also keep producing auditable `cross-platform-smoke-summary.json` artifacts, and `bash scripts/ci-remote-evidence.sh --repo dtamade/fafafa-skills-fusion --branch main --json` should report `promotion_ready=true` before wording is upgraded.

WSL remains useful but is treated as post-GA evidence unless a later release decision promotes it to a blocker.
WSL remains post-GA evidence and is not a current GA blocker.

### Batch 3: Minimum explainability and diagnostics on the current command surface

Use the existing command surface instead of adding new GA scope:

- `status --json` for machine-readable workflow, backend, guardian, scheduler, and safe backlog state
- `logs` for human-readable execution context
- `catchup` for recovery summary and next action
- `doctor` and `selfcheck` for wiring and contract diagnosis

## GA acceptance gates

The GA baseline is satisfied only when the full release-oriented bundle passes on the current branch:

```bash
cd rust
cargo build --release -p fusion-cli --bin fusion-bridge
cargo clippy --release --workspace --all-targets -- -D warnings
cargo test --release
cargo fmt --all -- --check

cd ..
bash scripts/ci-machine-mode-smoke.sh
bash scripts/ci-cross-platform-smoke.sh
```

Additional checks that must stay aligned:

- `cargo test --release -p fusion-cli --test repo_contract`
- `cargo test --release -p fusion-cli --test shell_contract`
- `cargo test --release -p fusion-cli --test cli_smoke`

## Exit criteria

v3.0 GA is ready when:

- active docs agree on the same Rust-primary architecture
- release gates are green with fresh evidence
- macOS and Windows (Git Bash) no longer need wording that implies unsupported core paths
- no active doc suggests the removed runtime/reference layer or Python control path still exists
