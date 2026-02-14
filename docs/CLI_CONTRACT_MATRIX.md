# CLI Contract Matrix

This matrix defines the CLI parameter and output contracts used by Fusion scripts.

| command | valid args | invalid args | help exit code | exit code | stdout/stderr/json expectations |
|---|---|---|---|---|---|
| `fusion-start.sh` | `<goal>`, `--force`, `--help` | unknown option, missing goal, extra goal tokens | `0` | success `0`; validation error `1` | Help prints usage; validation errors are human-readable; no silent fallback |
| `fusion-init.sh` | `--engine python\|rust`, `--json`, `--help` | unknown option, unsupported engine, extra args | `0` | success `0`; validation error `1` | `--json` returns machine-readable object; `--help` shows usage |
| `fusion-status.sh` | `--json`, `--help` | unknown option, unsupported combinations | `0` | success `0`; validation error `1` | `--json` prints structured payload/error object only; includes backend health fields (`backend_status`, `backend_primary`, `backend_fallback`) derived from `.fusion/backend_failure_report.json` when present; human mode prints summary |
| `fusion-logs.sh` | `[lines]`, `--help` | non-numeric lines, unknown option, too many args | `0` | success `0`; validation error `1` | Errors include usage; unknown option and argument-count errors are explicit |
| `fusion-git.sh` | `{status\|create-branch\|commit\|branch\|changes\|diff\|cleanup}`, `--help` | unknown action | `0` | success `0`; validation error `1` | Unknown action goes to `stderr` and includes usage |
| `fusion-codeagent.sh` | `[phase] [prompt...]`, `--help` | unknown option | `0` | success `0`; validation error `1` | Help exits early; unknown option rejected without routing |
| `fusion-hook-doctor.sh` | `[project_root]`, `--json`, `--fix`, `--help` | unknown option, invalid path | `0` | success `0`; validation error `1` | `--json` returns machine object including `fixed`; `--fix` failure reports `warn` + `fixed=false` |
| `fusion-hook-selfcheck.sh` | `[project_root]`, `--fix`, `--quick`, `--json`, `--help` | unknown option, invalid path, extra args | `0` | success `0`; check failure `1`; validation error `1` | default mode runs doctor+stop simulation+pytest; `--quick` skips pytest; `--json` returns per-check summary |
| `fusion-achievements.sh` | `--local-only`, `--leaderboard-only`, `--root <path>`, `--root=<path>`, `--top <n>`, `--top=<n>`, `--help` | unknown option, missing option value, non-numeric top | `0` | success `0`; validation error `1` | Usage for help/errors; invalid options avoid success banner |
| `fusion-pause.sh` | `--help` or no args | unknown option or extra args | `0` | success `0`; validation error `1` | Validation failures return explicit message |
| `fusion-resume.sh` | `--help` or no args | unknown option or extra args | `0` | success `0`; validation error `1` | Validation failures return explicit message |
| `fusion-cancel.sh` | `--help` or no args | unknown option or extra args | `0` | success `0`; validation error `1` | Validation failures return explicit message |
| `fusion-continue.sh` | `--help` or no args | unknown option or extra args | `0` | success `0`; validation error `1` | Validation failures return explicit message |
| `fusion-stop-guard.sh` | default hook invocation, `FUSION_STOP_HOOK_MODE=legacy\|structured` | malformed mode env | N/A | structured mode returns block JSON (`rc=0`); legacy lock conflict `rc=2` | structured mode emits machine block response; legacy preserves historical non-zero lock semantics |
| `release-contract-audit.sh` | `--dry-run`, `--json`, `--json-pretty`, `--fast`, `--skip-rust`, `--skip-python`, `--help` | unknown option, `--json-pretty` without `--json` | `0` | success `0`; validation error `1`; gate failure non-zero | JSON mode returns machine summary with `schema_version`, `step_rate_basis`, `command_rate_basis`, `success_command_rate`, `failed_command_rate`; text mode prints step logs and failure summaries |
| `python3 scripts/runtime/regression_runner.py` | `--suite <phase1\|phase2\|contract\|all>`, `--list-suites`, `--list-suites --json`, `--scenario resume_reliability`, `--help` | unknown suite, invalid CLI args | `0` | success `0`; validation/threshold failure non-zero | `--list-suites --json` returns machine payload with suites/default; contract JSON includes `schema_version`, `rate_basis`, `total_scenarios`; unknown suite prints explicit error |

## Required machine JSON keys (minimum)
- `release-contract-audit` payload: `schema_version`, `step_rate_basis`, `command_rate_basis`
- `regression_runner` (contract suite) payload: `schema_version`, `rate_basis`, `total_scenarios`
- Current schema contract: `schema_version=v1`

## Notes
- Contract tests live under `scripts/runtime/tests/`.
- Release gate command bundle is available at `scripts/release-contract-audit.sh`.
- CI gate workflow definition: `.github/workflows/ci-contract-gates.yml`.
- CI machine artifact examples: `/tmp/release-audit-dry-run.json`, `/tmp/runner-suites.json`, `/tmp/runner-contract.json`.
