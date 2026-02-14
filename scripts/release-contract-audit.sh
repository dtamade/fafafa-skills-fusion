#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage: release-contract-audit.sh [options]

Options:
  --dry-run       Print the gate command plan only
  --json          Emit machine-readable summary JSON
  --json-pretty   Pretty-print JSON output (requires --json)
  --fast          Skip full pytest suite and run contract suite only
  --skip-rust     Skip rust clippy/fmt gates
  --skip-python   Skip pytest gates
  -h, --help      Show help

Run release contract gates:
  1) shell syntax
  2) contract-focused pytest suite
  3) full pytest suite (unless --fast)
  4) rust clippy gate (unless --skip-rust)
  5) rust fmt gate (unless --skip-rust)
USAGE
}

die() {
  echo "Unknown option: $1" >&2
  usage >&2
  exit 1
}

log() {
  if [[ "$JSON_MODE" -eq 1 ]]; then
    echo "$1" >&2
  else
    echo "$1"
  fi
}

now_ms() {
  date +%s%3N
}

emit_json_summary() {
  local result="$1"
  local mode="$2"
  local failed_step="${3:-}"
  local failed_command="${4:-}"
  local exit_code="${5:-0}"
  local total_duration_ms="${6:-0}"

  local command_rows
  command_rows="$(printf '%s\n' "${COMMANDS[@]}")"

  local step_rows=""
  if [[ ${#STEP_RESULTS[@]} -gt 0 ]]; then
    step_rows="$(printf '%s\n' "${STEP_RESULTS[@]}")"
  fi

  FUSION_AUDIT_COMMANDS="$command_rows" \
  FUSION_AUDIT_STEPS="$step_rows" \
  python3 - "$result" "$mode" "$FAST" "$SKIP_RUST" "$SKIP_PYTHON" "$JSON_MODE" "$JSON_PRETTY" "$failed_step" "$failed_command" "$exit_code" "$total_duration_ms" <<'PY'
import json
import os
import sys

(
    result,
    mode,
    fast,
    skip_rust,
    skip_python,
    json_mode,
    json_pretty,
    failed_step,
    failed_command,
    exit_code,
    total_duration_ms,
) = sys.argv[1:12]

commands = [line for line in os.environ.get("FUSION_AUDIT_COMMANDS", "").splitlines() if line]
steps_raw = [line for line in os.environ.get("FUSION_AUDIT_STEPS", "").splitlines() if line]

step_results = []
for row in steps_raw:
    status, duration_ms, step_no, started_at_ms, finished_at_ms, step_exit_code, command = row.split("|||", 6)
    step_results.append(
        {
            "status": status,
            "duration_ms": int(duration_ms),
            "step": int(step_no),
            "started_at_ms": int(started_at_ms),
            "finished_at_ms": int(finished_at_ms),
            "exit_code": int(step_exit_code),
            "command": command,
        }
    )

failed_steps = [item["step"] for item in step_results if item["status"] == "error"]
failed_commands = [item["command"] for item in step_results if item["status"] == "error"]
success_steps_count = len(step_results) - len(failed_steps)
commands_count = len(commands)
steps_executed = len(step_results)
success_rate = (success_steps_count / steps_executed) if steps_executed > 0 else 0.0
failed_rate = (len(failed_steps) / steps_executed) if steps_executed > 0 else 0.0
success_command_rate = (success_steps_count / commands_count) if commands_count > 0 else 0.0
failed_command_rate = (len(failed_commands) / commands_count) if commands_count > 0 else 0.0
step_rate_basis = steps_executed
command_rate_basis = commands_count

payload = {
    "schema_version": "v1",
    "result": result,
    "dry_run": mode == "dry_run",
    "flags": {
        "json": json_mode == "1",
        "json_pretty": json_pretty == "1",
        "fast": fast == "1",
        "skip_rust": skip_rust == "1",
        "skip_python": skip_python == "1",
    },
    "commands": commands,
    "exit_code": int(exit_code),
    "steps_executed": len(step_results),
    "step_results": step_results,
    "failed_steps": failed_steps,
    "failed_steps_count": len(failed_steps),
    "error_step_count": len(failed_steps),
    "failed_commands": failed_commands,
    "failed_commands_count": len(failed_commands),
    "success_steps_count": success_steps_count,
    "commands_count": commands_count,
    "step_rate_basis": step_rate_basis,
    "command_rate_basis": command_rate_basis,
    "success_rate": success_rate,
    "failed_rate": failed_rate,
    "success_command_rate": success_command_rate,
    "failed_command_rate": failed_command_rate,
    "total_duration_ms": int(total_duration_ms),
}

if failed_step:
    payload["failed_step"] = int(failed_step)
if failed_command:
    payload["failed_command"] = failed_command

if json_pretty == "1":
    print(json.dumps(payload, ensure_ascii=False, indent=2))
else:
    print(json.dumps(payload, ensure_ascii=False))
PY
}

DRY_RUN=0
JSON_MODE=0
JSON_PRETTY=0
FAST=0
SKIP_RUST=0
SKIP_PYTHON=0
STEP_RESULTS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      ;;
    --json)
      JSON_MODE=1
      ;;
    --json-pretty)
      JSON_PRETTY=1
      ;;
    --fast)
      FAST=1
      ;;
    --skip-rust)
      SKIP_RUST=1
      ;;
    --skip-python)
      SKIP_PYTHON=1
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "$1"
      ;;
  esac
  shift
done

if [[ "$JSON_PRETTY" -eq 1 && "$JSON_MODE" -eq 0 ]]; then
  echo "--json-pretty requires --json" >&2
  exit 1
fi

COMMANDS=("bash -n scripts/*.sh")

if [[ "$SKIP_PYTHON" -eq 0 ]]; then
  COMMANDS+=(
    "pytest -q scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_achievements_script.py scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_hook_shell_runtime_path.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_ci_contract_gates.py scripts/runtime/tests/test_release_contract_audit_script.py"
  )

  if [[ "$FAST" -eq 0 ]]; then
    COMMANDS+=("pytest -q")
  fi
fi

if [[ "$SKIP_RUST" -eq 0 ]]; then
  COMMANDS+=(
    "cd rust && cargo clippy --workspace --all-targets -- -D warnings"
    "cd rust && cargo fmt --all -- --check"
  )
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  if [[ "$JSON_MODE" -eq 1 ]]; then
    emit_json_summary "ok" "dry_run" "" "" "0" "0"
  else
    echo "[release-contract-audit] dry-run command plan"
    for cmd in "${COMMANDS[@]}"; do
      echo "$cmd"
    done
  fi
  exit 0
fi

cd "$REPO_ROOT"

log "[release-contract-audit] running release gates"

step=0
total_start_ms="$(now_ms)"

for cmd in "${COMMANDS[@]}"; do
  step=$((step + 1))
  log "[release-contract-audit] $cmd"
  step_start_ms="$(now_ms)"

  if [[ "${FUSION_RELEASE_AUDIT_FORCE_FAIL_STEP:-}" == "$step" ]]; then
    step_end_ms="$(now_ms)"
    step_duration_ms=$((step_end_ms - step_start_ms))
    STEP_RESULTS+=("error|||$step_duration_ms|||$step|||$step_start_ms|||$step_end_ms|||1|||$cmd")
    total_duration_ms=$((step_end_ms - total_start_ms))

    if [[ "$JSON_MODE" -eq 1 ]]; then
      emit_json_summary "error" "run" "$step" "$cmd" "1" "$total_duration_ms"
    else
      echo "[release-contract-audit] failed at step $step: $cmd (forced)" >&2
    fi
    exit 1
  fi

  set +e
  if [[ "$JSON_MODE" -eq 1 ]]; then
    bash -lc "$cmd" >&2
  else
    bash -lc "$cmd"
  fi
  rc=$?
  set -e

  step_end_ms="$(now_ms)"
  step_duration_ms=$((step_end_ms - step_start_ms))

  if [[ "$rc" -ne 0 ]]; then
    STEP_RESULTS+=("error|||$step_duration_ms|||$step|||$step_start_ms|||$step_end_ms|||$rc|||$cmd")
    total_duration_ms=$((step_end_ms - total_start_ms))
    if [[ "$JSON_MODE" -eq 1 ]]; then
      emit_json_summary "error" "run" "$step" "$cmd" "$rc" "$total_duration_ms"
    else
      echo "[release-contract-audit] failed at step $step: $cmd (exit=$rc)" >&2
    fi
    exit "$rc"
  fi

  STEP_RESULTS+=("ok|||$step_duration_ms|||$step|||$step_start_ms|||$step_end_ms|||0|||$cmd")
done

if [[ "$JSON_MODE" -eq 1 ]]; then
  total_end_ms="$(now_ms)"
  total_duration_ms=$((total_end_ms - total_start_ms))
  emit_json_summary "ok" "run" "" "" "0" "$total_duration_ms"
else
  echo "[release-contract-audit] all gates passed"
fi
