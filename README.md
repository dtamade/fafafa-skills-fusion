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
  2. task metadata `- Owner:` (or `- Role:`) in `.fusion/task_plan.md`
  3. phase default (`planner`/`coder`/`reviewer`)
- Review gate metadata in `.fusion/task_plan.md`:
  - `- Review-Status: none|pending|approved|changes_requested`
- Role to backend mapping:
  - `planner` -> `codex`
  - `coder` -> `claude`
  - `reviewer` -> `codex`
- Session isolation uses role-aware keys, e.g. `planner_codex_session`, `coder_claude_session` (older keys remain compatible).

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

| Command                | Description                              |
| ---------------------- | ---------------------------------------- |
| `/fusion "<goal>"`     | Start autonomous workflow                |
| `/fusion status`       | Show runtime state and progress          |
| `/fusion resume`       | Resume interrupted workflow              |
| `/fusion pause`        | Pause current run                        |
| `/fusion cancel`       | Cancel current run                       |
| `/fusion logs`         | View execution logs                      |
| `/fusion achievements` | Show achievement summary and leaderboard |

### Script fallback (if slash commands are unavailable)

```bash
bash scripts/fusion-start.sh "implement user authentication"
bash scripts/fusion-status.sh
bash scripts/fusion-achievements.sh
```

These shell commands are thin wrappers. The primary control plane is `fusion-bridge` (Rust); Shell remains a compatibility entry layer, and the old runtime/reference layer has been removed from the repository.

## Repository Hygiene

- Live session state belongs in `.fusion/` only.
- Checked-in seeds live in `templates/`.
- Root-level `task_plan.md`, `progress.md`, and `findings.md` are intentionally not canonical and are gitignored to avoid committing mutable runtime artifacts.
- Local Rust caches such as `rust/target/` and `rust/.cargo-codex/` are generated machine state and should stay ignored.
- Host-local tool settings such as `.ace-tool/`, `.claude/settings.json`, and `.claude/settings.local.json` are also local machine state, not repository evidence.
- `.claude/settings.example.json` remains the checked-in hook template; only the generated `.claude/settings.json` and `.claude/settings.local.json` files are host-local config.
- For a checked-in illustrative session layout, see `examples/root-session/README.md`.
- If you need a sample checked-in session tree, place it under `examples/` rather than the repository root.

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

Use `scripts/fusion-init.sh` or `fusion-bridge init` to generate `.fusion/config.yaml` from `templates/config.yaml`, then edit the generated file as needed.

Edit `.fusion/config.yaml`:

```yaml
runtime:
  enabled: true
  compat_mode: true # Keep Shell fallback available for troubleshooting
  engine: "rust" # rust primary control plane

backends:
  primary: codex
  fallback: claude

agents:
  enabled: false
  mode: single_orchestrator # default; set role_handoff for planner -> coder -> reviewer handoff
  review_policy: high_risk
  explain_level: compact

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

execution:
  parallel: 2
  timeout: 7200000

scheduler:
  enabled: true
  max_parallel: 2
  fail_fast: false

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

`agents.enabled: false` keeps the current default single-runner path. When enabled, `single_orchestrator` still plans a ready non-conflicting batch from task metadata, constrained by `execution.parallel` and `scheduler.max_parallel`, records `_runtime.agents` batch state, and starts each run from the configured primary backend. `role_handoff` is the baton-passing mode: planner -> coder -> reviewer, with reviewer approval as a hard gate for tasks that still need review.

`agents.explain_level` controls how much policy detail is mirrored into `_runtime.agents.policy`, `fusion status --json`, and the human-readable `fusion status` output on the existing command surface; there is no separate explain command. When `role_handoff` is active, `fusion status --json` may also expose `agent_collaboration_mode`, `agent_turn_role`, `agent_turn_task_id`, `agent_turn_kind`, `agent_pending_reviews`, and `agent_blocked_handoff_reason`.

For reviewer-gated tasks, `.fusion/task_plan.md` is the canonical approval surface:

- `Review-Status: none` means no active review gate.
- `Review-Status: pending` means implementation is waiting on reviewer approval.
- `Review-Status: approved` means the reviewer accepted the task.
- `Review-Status: changes_requested` means the reviewer rejected the current revision and hands the task back for changes.

See `templates/config.yaml` for the full recommended baseline, including `scheduler.enabled: true` as the current default.

- `engine: "rust"` is the recommended default and makes hook/runtime entrypoints prefer `fusion-bridge` when available.
- Thin wrappers auto-discover only an installed `fusion-bridge` or the local `rust/target/release/` build; they do not silently fall back to `target/debug`.
- `FUSION_BRIDGE_DISABLE=1` forces supported hook/runtime entrypoints to skip the Rust bridge for troubleshooting; thin wrappers such as `fusion-status.sh`, `fusion-logs.sh`, `fusion-git.sh`, `fusion-achievements.sh`, `fusion-pause.sh`, `fusion-resume.sh`, `fusion-catchup.sh`, `fusion-cancel.sh`, and `fusion-continue.sh` now require `fusion-bridge` or an explicit `cargo --release` fallback where documented.
- No alternate runtime engine selection remains on the current control path; the old runtime/reference layer has been removed from the repository.
- `compat_mode: true` keeps live hook and helper fallback on the Shell path when the Rust bridge is skipped or unavailable, and the thin wrapper control scripts still require `fusion-bridge`.
- CI release gates run on `ubuntu-latest`; the workflow also includes a `macos-latest` smoke job and a `windows-latest` Git Bash smoke job for cross-platform evidence across shell helpers, control wrappers, real hook paths, and the catchup recovery wrapper.
- macOS and Windows (Git Bash) are still described as partially verified until fresh CI evidence upgrades that wording; see [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md).
- WSL is tracked as post-GA evidence rather than a current GA blocker.

## Dependency Auto-Heal

Fusion now attempts dependency recovery before failing hard:

- Auto-resolves `codeagent-wrapper` from:
  - `CODEAGENT_WRAPPER_BIN` (explicit path),
  - `PATH`,
  - `./node_modules/.bin/codeagent-wrapper`,
  - `~/.local/bin/codeagent-wrapper`,
  - `~/.npm-global/bin/codeagent-wrapper`.
- No interpreter auto-detection remains anywhere on the live control path.
- If unresolved, writes `.fusion/dependency_report.json` with actionable next steps for people or agents.
- If primary+fallback backend invocation both fail, writes `.fusion/backend_failure_report.json` with backend/error context.

Operational maintenance wrappers are also converging on Rust-only execution:

- `scripts/release-contract-audit.sh` now validates shell args locally, then delegates to `fusion-bridge audit`
- `scripts/fusion-hook-selfcheck.sh` now validates shell args locally, then delegates to `fusion-bridge selfcheck`
- The live audit flow is now shell/Rust-only: it runs shell syntax, machine-mode JSON smoke, wrapper smoke, and release Rust gates; selfcheck uses Rust contract regression directly

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

Hook path contract is intentionally strict: project-scoped commands should use only `bash "${CLAUDE_PROJECT_DIR:-.}/scripts/<hook>.sh"`. Older `${CLAUDE_PROJECT_DIR}` and bare relative `bash scripts/...` forms are treated as outdated wiring and should be rewritten.

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
- [`docs/REPO_CONVERGENCE_SUMMARY_2026-03.md`](docs/REPO_CONVERGENCE_SUMMARY_2026-03.md): March 2026 convergence summary
- [`docs/REPO_HYGIENE.md`](docs/REPO_HYGIENE.md): runtime artifact and root cleanup policy
- [`docs/HOOKS_SETUP.md`](docs/HOOKS_SETUP.md): host hook wiring notes
- [`docs/E2E_EXAMPLE.md`](docs/E2E_EXAMPLE.md): end-to-end workflow sample
- [`.claude/settings.example.json`](.claude/settings.example.json): checked-in hook template; copy it to your host-local `.claude/settings.json`
- [`docs/V3_GA_EXECUTION_ROADMAP.md`](docs/V3_GA_EXECUTION_ROADMAP.md): current v3 GA execution roadmap
- [`rust/README.md`](rust/README.md): Rust bridge current usage
- [`docs/RUST_FUSION_BRIDGE_ROADMAP.md`](docs/RUST_FUSION_BRIDGE_ROADMAP.md): historical Rust binary migration roadmap

## Development

Run tests:

```bash
cd rust
cargo test --release
```

Targeted contract suites:

```bash
cd rust
cargo test --release -p fusion-cli --test repo_contract
cargo test --release -p fusion-cli --test shell_contract
cargo test --release -p fusion-cli --test cli_smoke
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

- `--fast`: skip wrapper smoke and keep machine-mode smoke only
- `--skip-rust`: skip rust clippy/test/fmt gates

Machine-readable examples:

```bash
bash scripts/release-contract-audit.sh --dry-run --json --json-pretty --fast --skip-rust
bash scripts/ci-machine-mode-smoke.sh
fusion-bridge regression --list-suites --json
```

CI also uploads cross-platform smoke summaries for the remote `macos-latest` and `windows-latest` jobs. Those artifacts are written as `cross-platform-smoke-summary.json` under `/tmp/cross-platform-smoke-macos/` and `/tmp/cross-platform-smoke-windows/`.

After those changes are pushed to GitHub, use `bash scripts/ci-remote-evidence.sh --repo dtamade/fafafa-skills-fusion --branch main --json --artifacts-dir /tmp/remote-ci-evidence` to fetch the latest remote promotion evidence and write `remote-ci-evidence-summary.json`.

For the exact machine JSON fields, artifact paths, and CLI exit-code contract, see [`docs/CLI_CONTRACT_MATRIX.md`](docs/CLI_CONTRACT_MATRIX.md).

If active docs or repository/runtime contracts change, update `rust/crates/fusion-cli/tests/repo_contract.rs` together with the affected contract docs.
