# Contributing to Fusion Skill

Thanks for contributing.

## Before You Start

- Read [`README.md`](README.md) and [`SKILL.md`](SKILL.md).
- Open an issue first for major features or behavior changes.
- Keep changes focused and small when possible.

## Development Setup

1. Clone the repository.
2. Install your local tooling (Python 3.10+ recommended).
3. Run tests:

```bash
pytest -q
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
- [ ] `pytest -q` passes locally
- [ ] Docs updated (`README`, `CHANGELOG`, or relevant docs)
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

- Environment (OS, shell, Python version)
- Reproduction steps
- Expected behavior vs actual behavior
- Relevant logs or `.fusion/events.jsonl` snippets

## Security Issues

Do not open public issues for security vulnerabilities.
Please follow [`SECURITY.md`](SECURITY.md).
