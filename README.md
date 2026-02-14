# Fusion Skill

English | [简体中文](README.zh-CN.md)

Fusion is an autonomous development workflow skill for Codex/Claude-style agents.
Give it a goal, and it plans, executes, verifies, and reports with minimal interruptions.

## Why Fusion

- **Autonomous execution**: keeps moving without constant human confirmation.
- **TDD-first loop**: enforces `RED -> GREEN -> REFACTOR` for implementation work.
- **Resilience built in**: 3-strike recovery and backend fallback.
- **Runtime observability**: state is persisted in `.fusion/` and can be resumed.
- **Safe fallback mode**: injects low-risk quality/docs/optimization work when the main loop stalls.

## Workflow

```text
UNDERSTAND -> INITIALIZE -> ANALYZE -> DECOMPOSE -> EXECUTE -> VERIFY -> REVIEW -> COMMIT -> DELIVER
```

## Default Backend Routing

Fusion now uses role-based routing by default:

- Planning/analysis/review phases -> `codex`
- Execution/commit/delivery phases -> `claude`
- During `EXECUTE`, task type overrides:
  - `implementation`, `verification` -> `claude`
  - `design`, `research` -> `codex`
  - `documentation`, `configuration` -> `claude`

- Team role source priority:
  1. `FUSION_AGENT_ROLE` env override
  2. task metadata `- Owner:` (or `- Role:`) in `task_plan.md`
  3. phase default (`planner`/`coder`/`reviewer`)
- Role to backend mapping:
  - `planner` -> `codex`
  - `coder` -> `claude`
  - `reviewer` -> `codex`
- Session isolation uses role-aware keys, e.g. `planner_codex_session`, `coder_claude_session` (legacy keys remain compatible).

## Quick Start

### Start a workflow

```bash
/fusion "implement user authentication"
```

Fusion will:

1. Analyze repository context.
2. Decompose the goal into atomic tasks.
3. Execute tasks with TDD or direct mode (by task type).
4. Persist progress to `.fusion/`.
5. Deliver a final summary.

### Useful commands

| Command | Description |
| --- | --- |
| `/fusion "<goal>"` | Start autonomous workflow |
| `/fusion status` | Show runtime state and progress |
| `/fusion resume` | Resume interrupted workflow |
| `/fusion pause` | Pause current run |
| `/fusion cancel` | Cancel current run |
| `/fusion logs` | View execution logs |
| `/fusion achievements` | Show achievement summary and leaderboard |

### Script fallback (if slash commands are unavailable)

```bash
bash scripts/fusion-start.sh "implement user authentication"
bash scripts/fusion-status.sh
bash scripts/fusion-achievements.sh
```

## Safe Backlog (Anti-Stall)

Fusion includes a long-running fallback system to prevent dead loops:

- Triggers on **no progress rounds** or **task exhaustion**.
- Discovers low-risk tasks in three categories:
  - `quality`
  - `documentation`
  - `optimization`
- Uses anti-mechanical orchestration:
  - category rotation,
  - novelty window,
  - priority scoring.
- Uses exponential backoff controls:
  - `backoff_base_rounds`,
  - `backoff_max_rounds`,
  - `backoff_jitter`,
  - `backoff_force_probe_rounds`.

Observe fallback events via:

- `/fusion status` (`safe_backlog.last_*`)
- `.fusion/events.jsonl` (`SAFE_BACKLOG_INJECTED` with `reason` and `stall_score`)

## Virtual Supervisor (Optional)

Fusion also supports an additive virtual supervisor (default `disabled`) for human-like guidance without taking over execution:

- Advisory only (`mode: advisory`) in current release.
- Emits suggestions on repeated no-progress rounds.
- Writes `SUPERVISOR_ADVISORY` events to `.fusion/events.jsonl`.
- Never mutates task state directly; safe backlog remains the execution fallback.

## Configuration

Edit `.fusion/config.yaml`:

```yaml
runtime:
  enabled: true
  compat_mode: true
  engine: "python"  # python | rust

backends:
  primary: codex
  fallback: claude

backend_routing:
  phase_routing:
    EXECUTE: claude
    REVIEW: codex
  task_type_routing:
    implementation: claude
    verification: claude
    design: codex
    documentation: claude

understand:
  pass_threshold: 7
  require_confirmation: false
  max_questions: 2

safe_backlog:
  enabled: true
  trigger_no_progress_rounds: 3
  inject_on_task_exhausted: true
  max_tasks_per_run: 2
  allowed_categories: "quality,documentation,optimization"
  diversity_rotation: true
  novelty_window: 12
  backoff_enabled: true
  backoff_base_rounds: 1
  backoff_max_rounds: 32
  backoff_jitter: 0.2
  backoff_force_probe_rounds: 20

supervisor:
  enabled: false
  mode: "advisory"
  persona: "Guardian"
  trigger_no_progress_rounds: 2
  cadence_rounds: 2
  force_emit_rounds: 12
  max_suggestions: 2
```

See `templates/config.yaml` for the full recommended baseline.

## Dependency Auto-Heal

Fusion now attempts dependency recovery before failing hard:

- Auto-resolves `codeagent-wrapper` from:
  - `CODEAGENT_WRAPPER_BIN` (explicit path),
  - `PATH`,
  - `./node_modules/.bin/codeagent-wrapper`,
  - `~/.local/bin/codeagent-wrapper`,
  - `~/.npm-global/bin/codeagent-wrapper`.
- Auto-detects Python runtime from `python3` or `python`.
- If unresolved, writes `.fusion/dependency_report.json` with actionable next steps for people or agents.
- If primary+fallback backend invocation both fail, writes `.fusion/backend_failure_report.json` with backend/error context.

You can inspect unresolved dependency state with:

```bash
/fusion status
```

Look for the `## Dependency Report` section in the output.
If backend invocation fails twice, also check `## Backend Failure Report`.


## Hook Doctor Quick Fix

If hooks look unhealthy or the session exits unexpectedly, run:

```bash
bash scripts/fusion-hook-doctor.sh --json --fix .
```

Then re-check health:

```bash
bash scripts/fusion-hook-doctor.sh --json .
```

`result=ok` and `warn_count=0` indicate hook wiring is healthy.


After first-time auto-fix, open `/hooks` to approve changes, then restart the Claude Code session once.

## Hook Debug Visibility

When you want to verify hook triggering directly in Claude Code output:

```bash
# Enable hook debug (persistent for this project)
touch .fusion/.hook_debug

# Disable hook debug
rm -f .fusion/.hook_debug
```

With debug enabled, hooks emit stderr lines like:

- `[fusion][hook-debug][pretool] ...`
- `[fusion][hook-debug][posttool] ...`
- `[fusion][hook-debug][stop] ...`

You can inspect recent hook debug logs via:

```bash
/fusion status
# or
tail -n 50 .fusion/hook-debug.log
```

## Project Docs

- [`SKILL.md`](SKILL.md): skill spec and execution protocol
- [`EXECUTION_PROTOCOL.md`](EXECUTION_PROTOCOL.md): detailed phase rules
- [`PARALLEL_EXECUTION.md`](PARALLEL_EXECUTION.md): parallel scheduling strategy
- [`SESSION_RECOVERY.md`](SESSION_RECOVERY.md): resume and recovery behavior
- [`CHANGELOG.md`](CHANGELOG.md): release history
- [`docs/HOOKS_SETUP.md`](docs/HOOKS_SETUP.md): host hook wiring notes
- [`docs/E2E_EXAMPLE.md`](docs/E2E_EXAMPLE.md): end-to-end workflow sample
- [`.claude/settings.example.json`](.claude/settings.example.json): standard hook template
- [`rust/README.md`](rust/README.md): Rust bridge MVP usage
- [`docs/RUST_FUSION_BRIDGE_ROADMAP.md`](docs/RUST_FUSION_BRIDGE_ROADMAP.md): Rust binary migration roadmap

## Development

Run tests:

```bash
pytest -q
```

Targeted runtime regression set:

```bash
pytest -q \
  scripts/runtime/tests/test_safe_backlog.py \
  scripts/runtime/tests/test_compat_v2.py \
  scripts/runtime/tests/test_hook_shell_runtime_path.py
```

## Contributing

Please read:

- [Contributing Guide (EN)](CONTRIBUTING.md)
- [贡献指南 (ZH)](CONTRIBUTING.zh-CN.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md)

## Maintainer & Community

- Maintainer: **dtamade**
- Studio: **fafafa studio**
- Email: `dtamade@gmail.com`
- QQ Group (CN): `685403987`

## License

This project is licensed under the [MIT License](LICENSE).

## CI & Release Contract Gates

Fusion includes a CI gate workflow at `.github/workflows/ci-contract-gates.yml`.

Run release contract audit locally before merge:

```bash
bash scripts/release-contract-audit.sh --dry-run
bash scripts/release-contract-audit.sh
```

Useful flags:
- `--fast`: skip full `pytest -q`
- `--skip-rust`: skip rust clippy/fmt
- `--skip-python`: skip pytest gates

Machine-readable examples:

```bash
bash scripts/release-contract-audit.sh --dry-run --json --json-pretty --fast --skip-rust
python3 scripts/runtime/regression_runner.py --list-suites --json
```

Machine JSON key highlights:
- release audit payload: `schema_version`, `step_rate_basis`, `command_rate_basis`
- runner contract payload: `schema_version`, `rate_basis` (equals `total_scenarios`)
- denominator semantics: `step_rate_basis=total_steps`, `command_rate_basis=total_commands`, `rate_basis=total_scenarios`
- current schema contract: `schema_version=v1`

CI machine artifact examples:
- `/tmp/release-audit-dry-run.json`
- `/tmp/runner-suites.json`
- `/tmp/runner-contract.json`
