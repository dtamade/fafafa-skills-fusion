#!/usr/bin/env bash
set -euo pipefail

DEFAULT_SUMMARY_JSON="/tmp/cross-platform-smoke-summary.json"

usage() {
    cat <<'USAGE'
Usage: ci-cross-platform-json-smoke.sh [cross_platform_summary_json]

Validate the machine-readable cross-platform smoke artifact produced by:
- ci-cross-platform-smoke.sh --artifacts-dir <path> --platform-label <label>

Defaults:
- /tmp/cross-platform-smoke-summary.json
USAGE
}

log() {
    echo "[ci-cross-platform-json-smoke] $*"
}

fail() {
    echo "[ci-cross-platform-json-smoke] $*" >&2
    exit 1
}

require_jq() {
    command -v jq >/dev/null 2>&1 || \
        fail "jq is required to validate cross-platform smoke JSON artifacts"
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
        SUMMARY_JSON="$DEFAULT_SUMMARY_JSON"
        ;;
    *)
        [ "$#" -eq 1 ] || fail "expected either 0 args or 1 summary path"
        SUMMARY_JSON="$1"
        ;;
esac

require_jq
assert_file "$SUMMARY_JSON"

assert_has_keys \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    "schema_version" \
    "result" \
    "platform_label" \
    "commands" \
    "commands_count" \
    "completed_commands" \
    "completed_commands_count" \
    "runtime_engine" \
    "selfcheck_result" \
    "selfcheck_contract_regression_skipped" \
    "project_artifacts" \
    "failure_reason"

assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    '.schema_version == "v1"' \
    "schema_version must equal v1"
assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    '.result == "ok" or .result == "error"' \
    "result must be ok or error"
assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    '.commands_count == (.commands | length)' \
    "commands_count mismatch commands length"
assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    '.completed_commands_count == (.completed_commands | length)' \
    "completed_commands_count mismatch completed_commands length"
assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    '.completed_commands_count <= .commands_count' \
    "completed_commands_count exceeds commands_count"
assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    '(.platform_label | type) == "string" and (.platform_label | length) > 0' \
    "platform_label must be a non-empty string"
assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    '(.runtime_engine | type) == "string"' \
    "runtime_engine must be a string"
assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    '(.project_artifacts | type) == "array"
      and (.project_artifacts | index(".fusion/config.yaml") != null)
      and (.project_artifacts | index(".fusion/sessions.json") != null)
      and (.project_artifacts | index(".claude/settings.local.json") != null)' \
    "project_artifacts missing required entries"
assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    '(.failure_reason == null) or ((.failure_reason | type) == "string")' \
    "failure_reason must be null or string"
assert_filter \
    "$SUMMARY_JSON" \
    "cross-platform-smoke-summary.json" \
    'if .result == "ok"
     then .completed_commands_count == .commands_count
       and .runtime_engine == "rust"
       and .selfcheck_result == "ok"
       and .selfcheck_contract_regression_skipped == true
       and .failure_reason == null
     else true
     end' \
    "ok payload must report full completion and rust/selfcheck success"

log "cross-platform JSON smoke passed"
