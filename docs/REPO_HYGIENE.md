# Repository Hygiene

This repository treats runtime state and checked-in examples as different classes of files.

## Canonical locations

- Live mutable session state: `.fusion/`
- Checked-in seed templates: `templates/`
- Checked-in illustrative layouts: `examples/`
- Planning and review records: `docs/plans/`, `reviews/`

## Root directory policy

Do not treat repository-root runtime files as canonical inputs.

Files such as these should not live at the repository root as active tracked state:

- `task_plan.md`
- `progress.md`
- `findings.md`

If local tools generate them, move them under `.fusion/` or delete them after confirming they are not needed.

## Runtime artifact checklist

Before merge or release:

1. Confirm live workflow state is under `.fusion/`
2. Confirm no root-level `task_plan.md`, `progress.md`, or `findings.md` are reintroduced
3. Confirm `.gitignore` still covers root runtime artifacts, Rust build outputs, generated local Rust caches such as `rust/.cargo-codex/`, and host-local tool settings
4. Confirm examples stay under `examples/` and do not masquerade as active runtime state
5. Confirm docs and tests still describe `.fusion/` as the canonical runtime location
6. Confirm any repository/runtime contract change is reflected in the active docs and `rust/crates/fusion-cli/tests/repo_contract.rs`

## Generated local and host-specific state

Treat local caches and host-specific settings as disposable machine state, not repository inputs.

- `rust/target/`: local release/debug build outputs
- `rust/.cargo-codex/`: local crate cache populated by Codex-side cargo workflows
- `.ace-tool/`: local tool workspace/cache data
- `.claude/settings.json`: host-local hook wiring/configuration file
- `.claude/settings.local.json`: host-local override file written by doctor/fix flows
- `.claude/settings.example.json`: checked-in template used to generate host-local hook configuration

Only the example template is intended to stay tracked. The generated host-local variants should stay ignored and should not be used as evidence for repository structure, active source paths, or tracked artifacts.

## Historical note

This cleanup exists because earlier iterations could leave mutable workflow files at repository root. The repository now keeps only templates and examples in version control, while real runs stay inside `.fusion/`.
