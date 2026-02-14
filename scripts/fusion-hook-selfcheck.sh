#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

JSON_MODE=0
FIX_MODE=0
QUICK_MODE=0
PROJECT_ARG=""

usage() {
  cat <<'USAGE'
Usage: fusion-hook-selfcheck.sh [options] [project_root]

Options:
  --fix           Auto-fix project hooks before checks
  --quick         Skip pytest suite (doctor + stop simulation only)
  --json          Emit machine-readable JSON summary
  -h, --help      Show help

Checks:
  1) fusion-hook-doctor (with optional --fix)
  2) stop-hook simulation (empty stdin must return JSON block + rc=0)
  3) hook regression pytest suite (skipped when --quick)
USAGE
}

fail_validation() {
  local reason="$1"
  if [[ "$JSON_MODE" -eq 1 ]]; then
    if command -v python3 >/dev/null 2>&1; then
      python3 - "$reason" <<'PY'
import json
import sys
print(json.dumps({"result": "error", "reason": sys.argv[1]}, ensure_ascii=False))
PY
    else
      printf '{"result":"error","reason":"%s"}\n' "$reason"
    fi
  else
    echo "$reason" >&2
    usage >&2
  fi
  exit 1
}

log() {
  if [[ "$JSON_MODE" -eq 1 ]]; then
    echo "$1" >&2
  else
    echo "$1"
  fi
}

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --fix)
      FIX_MODE=1
      ;;
    --quick)
      QUICK_MODE=1
      ;;
    --json)
      JSON_MODE=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    -*)
      fail_validation "Unknown option: $1"
      ;;
    *)
      if [[ -z "$PROJECT_ARG" ]]; then
        PROJECT_ARG="$1"
      else
        fail_validation "Unexpected argument: $1"
      fi
      ;;
  esac
  shift
done

PROJECT_ROOT="${PROJECT_ARG:-$PWD}"
if [[ ! -d "$PROJECT_ROOT" ]]; then
  fail_validation "project_root not found: $PROJECT_ROOT"
fi
PROJECT_ROOT="$(cd "$PROJECT_ROOT" && pwd)"

DOCTOR_RC=1
DOCTOR_OUTPUT=""
DOCTOR_RESULT="error"
DOCTOR_WARN_COUNT="999"
DOCTOR_OK=0

log "[selfcheck] project_root: $PROJECT_ROOT"
log "[selfcheck] check 1/3: fusion-hook-doctor"

DOCTOR_CMD=("bash" "$SCRIPT_DIR/fusion-hook-doctor.sh" "--json")
if [[ "$FIX_MODE" -eq 1 ]]; then
  DOCTOR_CMD+=("--fix")
fi
DOCTOR_CMD+=("$PROJECT_ROOT")

set +e
DOCTOR_OUTPUT="$("${DOCTOR_CMD[@]}" 2>/dev/null)"
DOCTOR_RC=$?
set -e

if command -v python3 >/dev/null 2>&1; then
  PARSED_LINES="$(printf '%s' "$DOCTOR_OUTPUT" | python3 -c 'import json,sys
try:
    payload=json.load(sys.stdin)
except Exception:
    print("invalid_json")
    print("999")
    raise SystemExit(0)
print(payload.get("result", ""))
print(payload.get("warn_count", 999))' 2>/dev/null || true)"
  DOCTOR_RESULT="$(printf '%s' "$PARSED_LINES" | sed -n '1p')"
  DOCTOR_WARN_COUNT="$(printf '%s' "$PARSED_LINES" | sed -n '2p')"
else
  DOCTOR_RESULT="$(printf '%s' "$DOCTOR_OUTPUT" | grep -o '"result"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4 || echo "")"
  DOCTOR_WARN_COUNT="$(printf '%s' "$DOCTOR_OUTPUT" | grep -o '"warn_count"[[:space:]]*:[[:space:]]*[0-9]*' | head -1 | grep -o '[0-9]*' || echo "999")"
fi

if [[ "$DOCTOR_RC" -eq 0 && "$DOCTOR_RESULT" == "ok" && "$DOCTOR_WARN_COUNT" == "0" ]]; then
  DOCTOR_OK=1
  log "[selfcheck] ✅ doctor passed"
else
  DOCTOR_OK=0
  log "[selfcheck] ❌ doctor failed (rc=$DOCTOR_RC result=$DOCTOR_RESULT warn_count=$DOCTOR_WARN_COUNT)"
  if [[ -n "$DOCTOR_OUTPUT" ]]; then
    log "[selfcheck] doctor output: $DOCTOR_OUTPUT"
  fi
fi

STOP_RC=1
STOP_DECISION=""
STOP_OK=0
STOP_STDOUT=""
STOP_STDERR=""

log "[selfcheck] check 2/3: stop-hook simulation"
TMP_DIR="$(mktemp -d)"
cleanup_tmp() {
  rm -rf "$TMP_DIR"
}
trap cleanup_tmp EXIT

mkdir -p "$TMP_DIR/.fusion"
cat > "$TMP_DIR/.fusion/sessions.json" <<'JSON'
{"status":"in_progress","current_phase":"EXECUTE","goal":"hook-selfcheck"}
JSON
cat > "$TMP_DIR/.fusion/task_plan.md" <<'MD'
### Task 1: Verify Hook [PENDING]
MD

set +e
STOP_STDOUT="$(cd "$TMP_DIR" && printf '' | bash "$SCRIPT_DIR/fusion-stop-guard.sh" 2>"$TMP_DIR/stop.stderr")"
STOP_RC=$?
set -e
STOP_STDERR="$(cat "$TMP_DIR/stop.stderr" 2>/dev/null || true)"

if command -v python3 >/dev/null 2>&1; then
  STOP_DECISION="$(printf '%s' "$STOP_STDOUT" | python3 -c 'import json,sys
try:
    payload=json.load(sys.stdin)
except Exception:
    print("")
    raise SystemExit(0)
print(payload.get("decision", ""))' 2>/dev/null || true)"
else
  STOP_DECISION="$(printf '%s' "$STOP_STDOUT" | grep -o '"decision"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4 || echo "")"
fi

if [[ "$STOP_RC" -eq 0 && "$STOP_DECISION" == "block" ]]; then
  STOP_OK=1
  log "[selfcheck] ✅ stop-hook simulation passed"
else
  STOP_OK=0
  log "[selfcheck] ❌ stop-hook simulation failed (rc=$STOP_RC decision=$STOP_DECISION)"
  if [[ -n "$STOP_STDERR" ]]; then
    log "[selfcheck] stop stderr: $STOP_STDERR"
  fi
fi

PYTEST_RC=0
PYTEST_OK=1
PYTEST_SKIPPED=0
PYTEST_OUTPUT=""

if [[ "$QUICK_MODE" -eq 1 ]]; then
  PYTEST_SKIPPED=1
  log "[selfcheck] check 3/3: pytest hook suite (skipped by --quick)"
else
  log "[selfcheck] check 3/3: pytest hook suite"
  set +e
  PYTEST_OUTPUT="$(cd "$REPO_ROOT" && pytest -q scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_hook_shell_runtime_path.py scripts/runtime/tests/test_fusion_hook_doctor_script.py 2>&1)"
  PYTEST_RC=$?
  set -e

  if [[ "$PYTEST_RC" -eq 0 ]]; then
    PYTEST_OK=1
    log "[selfcheck] ✅ pytest hook suite passed"
  else
    PYTEST_OK=0
    log "[selfcheck] ❌ pytest hook suite failed (rc=$PYTEST_RC)"
    log "$PYTEST_OUTPUT"
  fi
fi

OVERALL_OK=0
if [[ "$DOCTOR_OK" -eq 1 && "$STOP_OK" -eq 1 ]]; then
  if [[ "$PYTEST_SKIPPED" -eq 1 || "$PYTEST_OK" -eq 1 ]]; then
    OVERALL_OK=1
  fi
fi

if [[ "$JSON_MODE" -eq 1 ]]; then
  RESULT_TEXT="error"
  if [[ "$OVERALL_OK" -eq 1 ]]; then
    RESULT_TEXT="ok"
  fi

  if command -v python3 >/dev/null 2>&1; then
    DOCTOR_OUTPUT="$DOCTOR_OUTPUT" \
    PYTEST_OUTPUT="$PYTEST_OUTPUT" \
    python3 - "$RESULT_TEXT" "$PROJECT_ROOT" "$FIX_MODE" "$QUICK_MODE" "$DOCTOR_OK" "$DOCTOR_RC" "$DOCTOR_RESULT" "$DOCTOR_WARN_COUNT" "$STOP_OK" "$STOP_RC" "$STOP_DECISION" "$PYTEST_OK" "$PYTEST_RC" "$PYTEST_SKIPPED" <<'PY'
import json
import os
import sys

(
    result_text,
    project_root,
    fix_mode,
    quick_mode,
    doctor_ok,
    doctor_rc,
    doctor_result,
    doctor_warn_count,
    stop_ok,
    stop_rc,
    stop_decision,
    pytest_ok,
    pytest_rc,
    pytest_skipped,
) = sys.argv[1:15]

payload = {
    "result": result_text,
    "project_root": project_root,
    "flags": {
        "fix": fix_mode == "1",
        "quick": quick_mode == "1",
        "json": True,
    },
    "checks": [
        {
            "name": "hook_doctor",
            "ok": doctor_ok == "1",
            "exit_code": int(doctor_rc),
            "result": doctor_result,
            "warn_count": int(doctor_warn_count) if doctor_warn_count.isdigit() else 999,
        },
        {
            "name": "stop_simulation",
            "ok": stop_ok == "1",
            "exit_code": int(stop_rc),
            "decision": stop_decision,
        },
        {
            "name": "pytest_hook_suite",
            "ok": pytest_ok == "1",
            "exit_code": int(pytest_rc),
            "skipped": pytest_skipped == "1",
        },
    ],
}

if result_text != "ok":
    payload["doctor_output"] = os.environ.get("DOCTOR_OUTPUT", "")
    payload["pytest_output"] = os.environ.get("PYTEST_OUTPUT", "")

print(json.dumps(payload, ensure_ascii=False))
PY
  else
    printf '{"result":"%s","project_root":"%s"}\n' "$RESULT_TEXT" "$PROJECT_ROOT"
  fi
fi

if [[ "$OVERALL_OK" -eq 1 ]]; then
  if [[ "$JSON_MODE" -eq 0 ]]; then
    log "[selfcheck] ✅ all checks passed"
  fi
  exit 0
fi

if [[ "$JSON_MODE" -eq 0 ]]; then
  log "[selfcheck] ❌ checks failed"
fi
exit 1
