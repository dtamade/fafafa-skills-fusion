# Contributing to Fusion Skill

Thanks for contributing.

## Before You Start

- Read [`README.md`](README.md) and [`SKILL.md`](SKILL.md).
- Open an issue first for major features or behavior changes.
- Keep changes focused and small when possible.

## Development Setup

1. Clone the repository.
2. Install your local tooling (`bash`, Rust stable toolchain, and optional `jq` for machine JSON smoke or manual JSON inspection).
3. Fusion runtime state lives in `.fusion/`; keep `templates/` as the checked-in source of truth and avoid committing live session artifacts at repo root.
4. Treat `rust/target/` and `rust/.cargo-codex/` as generated local Rust caches; keep them ignored and do not use them as repository evidence.
5. Treat `.ace-tool/`, `.claude/settings.json`, and `.claude/settings.local.json` as host-local tool state. Keep those ignored as well; only `.claude/settings.example.json` remains the checked-in template.
6. If you need an illustrative checked-in layout, document it under `examples/` (for example `examples/root-session/README.md`) instead of reintroducing mutable root files.
7. Run tests:

```bash
cd rust
cargo test --release
```

## Coding Guidelines

- Follow existing style and structure.
- Prefer fixing root causes over surface patches.
- Keep runtime hooks fault-safe (do not break the base workflow on exceptions).
- Add or update tests for behavior changes.
- Avoid unrelated refactors in the same PR.

## Pull Request Checklist

- [ ] Clear title and summary
- [ ] Tests added/updated
- [ ] `cd rust && cargo test --release` passes locally
- [ ] `cd rust && cargo test --release` passes locally when Rust code changes
- [ ] Docs updated (`README`, `CHANGELOG`, or relevant docs)
- [ ] If active docs or repository/runtime contracts changed, update `rust/crates/fusion-cli/tests/repo_contract.rs` too
- [ ] No sensitive data in commits

## Commit Messages

Use concise, descriptive messages. Conventional commit style is recommended:

- `feat: ...`
- `fix: ...`
- `docs: ...`
- `test: ...`
- `refactor: ...`

## Reporting Bugs

Please include:

- Environment (OS, shell, `fusion-bridge --version` or build source)
- Reproduction steps
- Expected behavior vs actual behavior
- Relevant logs or `.fusion/events.jsonl` snippets

## Security Issues

Do not open public issues for security vulnerabilities.
Please follow [`SECURITY.md`](SECURITY.md).
