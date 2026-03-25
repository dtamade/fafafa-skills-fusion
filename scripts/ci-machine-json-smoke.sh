#!/usr/bin/env bash
set -euo pipefail

DEFAULT_RELEASE_AUDIT_JSON="/tmp/release-audit-dry-run.json"
DEFAULT_RUNNER_SUITES_JSON="/tmp/runner-suites.json"
DEFAULT_RUNNER_CONTRACT_JSON="/tmp/runner-contract.json"

usage() {
    cat <<'USAGE'
Usage: ci-machine-json-smoke.sh [release_audit_json runner_suites_json runner_contract_json]

Validate the machine-readable CI smoke artifacts produced by:
- release-contract-audit.sh --dry-run --json --fast --skip-rust
- fusion-bridge regression --list-suites --json
- fusion-bridge regression --suite contract --json

Defaults:
- /tmp/release-audit-dry-run.json
- /tmp/runner-suites.json
- /tmp/runner-contract.json
USAGE
}

log() {
    echo "[ci-machine-json-smoke] $*"
}

fail() {
    echo "[ci-machine-json-smoke] $*" >&2
    exit 1
}

require_jq() {
    command -v jq >/dev/null 2>&1 || \
        fail "jq is required to validate machine JSON artifacts"
}

assert_file() {
    local path="$1"
    [ -f "$path" ] || fail "artifact not found: $path"
}

assert_has_keys() {
    local path="$1"
    local label="$2"
    shift 2

    local key
    for key in "$@"; do
        jq -e --arg key "$key" 'has($key)' "$path" >/dev/null || \
            fail "$label missing key: $key"
    done
}

assert_filter() {
    local path="$1"
    local label="$2"
    local filter="$3"
    local message="$4"

    jq -e "$filter" "$path" >/dev/null || fail "$label $message"
}

case "${1:-}" in
    -h|--help)
        usage
        exit 0
        ;;
    "")
        RELEASE_AUDIT_JSON="$DEFAULT_RELEASE_AUDIT_JSON"
        RUNNER_SUITES_JSON="$DEFAULT_RUNNER_SUITES_JSON"
        RUNNER_CONTRACT_JSON="$DEFAULT_RUNNER_CONTRACT_JSON"
        ;;
    *)
        [ "$#" -eq 3 ] || fail "expected either 0 args or 3 artifact paths"
        RELEASE_AUDIT_JSON="$1"
        RUNNER_SUITES_JSON="$2"
        RUNNER_CONTRACT_JSON="$3"
        ;;
esac

require_jq

assert_file "$RELEASE_AUDIT_JSON"
assert_file "$RUNNER_SUITES_JSON"
assert_file "$RUNNER_CONTRACT_JSON"

assert_has_keys \
    "$RUNNER_CONTRACT_JSON" \
    "runner-contract.json" \
    "schema_version" \
    "result" \
    "suite" \
    "scenario_results" \
    "longest_scenario" \
    "fastest_scenario" \
    "scenario_count_by_result" \
    "duration_stats" \
    "failed_rate" \
    "success_rate" \
    "success_count" \
    "failure_count" \
    "total_scenarios" \
    "rate_basis"
assert_filter \
    "$RUNNER_CONTRACT_JSON" \
    "runner-contract.json" \
    '.rate_basis == .total_scenarios' \
    "rate_basis mismatch total_scenarios"

assert_has_keys \
    "$RELEASE_AUDIT_JSON" \
    "release-audit-dry-run.json" \
    "schema_version" \
    "result" \
    "flags" \
    "commands" \
    "steps_executed" \
    "failed_commands" \
    "failed_commands_count" \
    "error_step_count" \
    "success_steps_count" \
    "commands_count" \
    "step_rate_basis" \
    "command_rate_basis" \
    "success_rate" \
    "failed_rate" \
    "success_command_rate" \
    "failed_command_rate"
assert_filter \
    "$RELEASE_AUDIT_JSON" \
    "release-audit-dry-run.json" \
    '.step_rate_basis == .steps_executed' \
    "step_rate_basis mismatch steps_executed"
assert_filter \
    "$RELEASE_AUDIT_JSON" \
    "release-audit-dry-run.json" \
    '.command_rate_basis == .commands_count' \
    "command_rate_basis mismatch commands_count"

assert_has_keys \
    "$RUNNER_SUITES_JSON" \
    "runner-suites.json" \
    "result" \
    "default_suite" \
    "suites"

log "machine JSON smoke passed"
