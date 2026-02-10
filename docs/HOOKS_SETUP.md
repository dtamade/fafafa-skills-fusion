# Hook Wiring Notes

`SKILL.md` includes `allowed-tools` and `hooks` in frontmatter for portability and readability.

These keys are **project metadata**, not guaranteed runtime wiring by themselves in every host.

## What is already wired in this repo

- Hook logic scripts live in `scripts/`:
  - `scripts/fusion-pretool.sh`
  - `scripts/fusion-posttool.sh`
  - `scripts/fusion-stop-guard.sh`
- Runtime bridge lives in `scripts/runtime/compat_v2.py`.

## What host environments may still require

Depending on your host (Codex CLI, Claude Code, custom orchestrator), you may need to map lifecycle hooks to those scripts in host-specific settings.

Recommended mapping:

- PreToolUse → `bash scripts/fusion-pretool.sh`
- PostToolUse → `bash scripts/fusion-posttool.sh`
- Stop → `bash scripts/fusion-stop-guard.sh`

## Verify wiring

Run:

```bash
pytest -q \
  scripts/runtime/tests/test_hook_shell_runtime_path.py \
  scripts/runtime/tests/test_compat_v2.py
```

If these pass, shell↔runtime bridge and path wiring are healthy in this repository.
