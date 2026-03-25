#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat <<'USAGE'
Usage: ci-cross-platform-smoke.sh [--artifacts-dir <path>] [--platform-label <label>]

Runs a release-bridge-backed shell smoke flow that exercises the live wrappers:
- fusion-start.sh
- fusion-status.sh --json
- fusion-achievements.sh --leaderboard-only
- fusion-hook-selfcheck.sh --json --quick --fix
- fusion-catchup.sh

Options:
- --artifacts-dir <path>  Write cross-platform-smoke-summary.json into this directory
- --platform-label <label>  Label stored in the summary JSON (default: local or FUSION_CI_PLATFORM_LABEL)

Set FUSION_KEEP_CI_SMOKE_TMP=1 to keep the temporary workspace for debugging.
USAGE
}

log() {
    echo "[ci-cross-platform-smoke] $*"
}

fail() {
    FAIL_REASON="$*"
    echo "[ci-cross-platform-smoke] $*" >&2
    exit 1
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local label="$3"
    if [[ "$haystack" != *"$needle"* ]]; then
        fail "$label missing expected marker: $needle"
    fi
}

json_escape() {
    local value="$1"
    value=${value//\\/\\\\}
    value=${value//\"/\\\"}
    value=${value//$'\n'/\\n}
    value=${value//$'\r'/\\r}
    value=${value//$'\t'/\\t}
    printf '%s' "$value"
}

json_array_from_args() {
    local first=1
    printf '['
    for value in "$@"; do
        if [[ "$first" -eq 0 ]]; then
            printf ','
        fi
        printf '"%s"' "$(json_escape "$value")"
        first=0
    done
    printf ']'
}

json_string_or_null() {
    local value="$1"
    if [[ -z "$value" ]]; then
        printf 'null'
    else
        printf '"%s"' "$(json_escape "$value")"
    fi
}

write_summary() {
    if [[ -z "$ARTIFACTS_DIR" ]]; then
        return
    fi

    mkdir -p "$ARTIFACTS_DIR"
    SUMMARY_PATH="$ARTIFACTS_DIR/cross-platform-smoke-summary.json"

    cat > "$SUMMARY_PATH" <<EOF
{"schema_version":"v1","result":"$SMOKE_RESULT","platform_label":"$(json_escape "$PLATFORM_LABEL")","commands":$(json_array_from_args "${COMMANDS[@]}"),"commands_count":${#COMMANDS[@]},"completed_commands":$(json_array_from_args "${COMPLETED_COMMANDS[@]}"),"completed_commands_count":${#COMPLETED_COMMANDS[@]},"runtime_engine":"$(json_escape "$RUNTIME_ENGINE")","selfcheck_result":"$(json_escape "$SELFCHECK_RESULT")","selfcheck_contract_regression_skipped":$SELFCHECK_CONTRACT_REGRESSION_SKIPPED,"project_artifacts":[".fusion/config.yaml",".fusion/sessions.json",".claude/settings.local.json"],"failure_reason":$(json_string_or_null "$FAIL_REASON")}
EOF

    log "wrote summary artifact: $SUMMARY_PATH"
}

cleanup() {
    if [[ -z "${TMP_ROOT:-}" ]]; then
        return
    fi

    if [[ "${FUSION_KEEP_CI_SMOKE_TMP:-0}" == "1" ]]; then
        log "keeping temporary workspace: $TMP_ROOT"
        return
    fi

    rm -rf "$TMP_ROOT"
}

on_exit() {
    local exit_code=$?
    trap - EXIT
    if [[ "$exit_code" -eq 0 ]]; then
        SMOKE_RESULT="ok"
    fi
    write_summary
    cleanup
    exit "$exit_code"
}

ARTIFACTS_DIR=""
PLATFORM_LABEL="${FUSION_CI_PLATFORM_LABEL:-local}"
TMP_ROOT=""
FAIL_REASON=""
SMOKE_RESULT="error"
SUMMARY_PATH=""
RUNTIME_ENGINE="unknown"
SELFCHECK_RESULT="unknown"
SELFCHECK_CONTRACT_REGRESSION_SKIPPED=false
COMMANDS=(
    "fusion-start.sh"
    "fusion-status.sh --json"
    "fusion-achievements.sh --leaderboard-only"
    "fusion-hook-selfcheck.sh --json --quick --fix"
    "fusion-catchup.sh"
)
COMPLETED_COMMANDS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --artifacts-dir)
            shift
            [[ $# -gt 0 ]] || fail "Missing value for --artifacts-dir"
            ARTIFACTS_DIR="$1"
            ;;
        --platform-label)
            shift
            [[ $# -gt 0 ]] || fail "Missing value for --platform-label"
            PLATFORM_LABEL="$1"
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            fail "Unknown option: $1"
            ;;
    esac
    shift
done

trap on_exit EXIT

TMP_ROOT="$(mktemp -d)"

HOME_DIR="$TMP_ROOT/home"
LEADERBOARD_ROOT="$TMP_ROOT/workspaces"
PROJECT_ROOT="$LEADERBOARD_ROOT/cross-platform-smoke"
mkdir -p "$HOME_DIR/.claude" "$PROJECT_ROOT"

export HOME="$HOME_DIR"
export FUSION_LEADERBOARD_ROOT="$LEADERBOARD_ROOT"

log "running fusion-start.sh"
start_output="$(
    cd "$PROJECT_ROOT" &&
    bash "$SCRIPT_DIR/fusion-start.sh" "cross-platform shell smoke" --force 2>&1
)"
assert_contains "$start_output" "[FUSION] Workflow initialized." "fusion-start.sh output"
[ -f "$PROJECT_ROOT/.fusion/config.yaml" ] || fail "fusion-start.sh did not create .fusion/config.yaml"
[ -f "$PROJECT_ROOT/.fusion/sessions.json" ] || fail "fusion-start.sh did not create .fusion/sessions.json"
[ -f "$PROJECT_ROOT/.claude/settings.local.json" ] || fail "fusion-start.sh did not auto-wire hooks"
COMPLETED_COMMANDS+=("${COMMANDS[0]}")

log "running fusion-status.sh --json"
status_output="$(
    cd "$PROJECT_ROOT" &&
    bash "$SCRIPT_DIR/fusion-status.sh" --json
)"
assert_contains "$status_output" '"result":"ok"' "fusion-status.sh --json output"
assert_contains "$status_output" '"runtime_engine":"rust"' "fusion-status.sh --json output"
RUNTIME_ENGINE="rust"
COMPLETED_COMMANDS+=("${COMMANDS[1]}")

log "running fusion-achievements.sh --leaderboard-only"
achievements_output="$(
    cd "$PROJECT_ROOT" &&
    bash "$SCRIPT_DIR/fusion-achievements.sh" --leaderboard-only --root "$LEADERBOARD_ROOT" --top 1
)"
assert_contains "$achievements_output" "## Achievement Leaderboard" "fusion-achievements.sh output"
COMPLETED_COMMANDS+=("${COMMANDS[2]}")

log "running fusion-hook-selfcheck.sh --json --quick --fix"
selfcheck_output="$(
    bash "$SCRIPT_DIR/fusion-hook-selfcheck.sh" --json --quick --fix "$PROJECT_ROOT"
)"
assert_contains "$selfcheck_output" '"result":"ok"' "fusion-hook-selfcheck.sh output"
assert_contains "$selfcheck_output" '"project_root":"' "fusion-hook-selfcheck.sh output"
assert_contains "$selfcheck_output" 'cross-platform-smoke' "fusion-hook-selfcheck.sh output"
assert_contains "$selfcheck_output" '"name":"hook_doctor"' "fusion-hook-selfcheck.sh output"
assert_contains "$selfcheck_output" '"name":"stop_simulation"' "fusion-hook-selfcheck.sh output"
assert_contains "$selfcheck_output" '"name":"contract_regression_suite"' "fusion-hook-selfcheck.sh output"
assert_contains "$selfcheck_output" '"skipped":true' "fusion-hook-selfcheck.sh output"
SELFCHECK_RESULT="ok"
SELFCHECK_CONTRACT_REGRESSION_SKIPPED=true
COMPLETED_COMMANDS+=("${COMMANDS[3]}")

log "running fusion-catchup.sh"
catchup_output="$(
    bash "$SCRIPT_DIR/fusion-catchup.sh" --project-path "$PROJECT_ROOT" 2>&1
)"
if [[ -n "$catchup_output" ]]; then
    printf '%s\n' "$catchup_output"
fi
COMPLETED_COMMANDS+=("${COMMANDS[4]}")

log "running fallback hook shell smoke"
cat > "$PROJECT_ROOT/.fusion/sessions.json" <<'EOF'
{
  "status": "in_progress",
  "current_phase": "EXECUTE",
  "goal": "cross-platform fallback shell smoke"
}
EOF
cat > "$PROJECT_ROOT/.fusion/task_plan.md" <<'EOF'
### Task 1: A [COMPLETED]
### Task 2: B [PENDING]
- Type: implementation
EOF
cat > "$PROJECT_ROOT/.fusion/loop_context.json" <<'EOF'
{
  "no_progress_rounds": 4,
  "same_action_count": 0
}
EOF

pretool_output="$(
    cd "$PROJECT_ROOT" &&
    printf '{}\n' | FUSION_BRIDGE_DISABLE=1 bash "$SCRIPT_DIR/fusion-pretool.sh" 2>&1
)"
assert_contains "$pretool_output" "[fusion] Goal:" "fusion-pretool.sh fallback output"
assert_contains "$pretool_output" "Task 2/2: B [PENDING] (type: implementation)" "fusion-pretool.sh fallback output"
assert_contains "$pretool_output" "TDD flow: RED → GREEN → REFACTOR" "fusion-pretool.sh fallback output"

printf '0:2:0:0\n' > "$PROJECT_ROOT/.fusion/.progress_snapshot"
posttool_output="$(
    cd "$PROJECT_ROOT" &&
    printf '{}\n' | FUSION_BRIDGE_DISABLE=1 bash "$SCRIPT_DIR/fusion-posttool.sh" 2>&1
)"
assert_contains "$posttool_output" "Task A → COMPLETED" "fusion-posttool.sh fallback output"
assert_contains "$posttool_output" "Next action: Continue task: B [PENDING] | Mode: TDD" "fusion-posttool.sh fallback output"

cat > "$PROJECT_ROOT/.fusion/task_plan.md" <<'EOF'
### Task 1: A [PENDING]
EOF
stop_guard_output="$(
    cd "$PROJECT_ROOT" &&
    printf '{}\n' | FUSION_BRIDGE_DISABLE=1 bash "$SCRIPT_DIR/fusion-stop-guard.sh" 2>&1
)"
assert_contains "$stop_guard_output" '"decision":"block"' "fusion-stop-guard.sh fallback output"
assert_contains "$stop_guard_output" 'Continue task: A [PENDING]' "fusion-stop-guard.sh fallback output"
log "fallback hook shell smoke passed"

log "shell smoke passed"
