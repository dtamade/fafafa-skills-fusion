#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="ci-remote-evidence.sh"
DEFAULT_REPO="dtamade/fafafa-skills-fusion"
DEFAULT_BRANCH="main"
DEFAULT_WORKFLOW_PATH=".github/workflows/ci-contract-gates.yml"
DEFAULT_LIMIT=10
DEFAULT_OUTPUT_NAME="remote-ci-evidence-summary.json"
GH_BIN="${GH_BIN:-gh}"

usage() {
    cat <<'USAGE'
Usage: ci-remote-evidence.sh [options]

Fetch the latest remote GitHub Actions evidence for the CI contract gates workflow
and verify whether the release-blocking jobs are green.

Options:
  --repo <owner/name>          GitHub repository (default: dtamade/fafafa-skills-fusion)
  --branch <branch>            Branch to inspect (default: main)
  --workflow-path <path>       Workflow path on GitHub (default: .github/workflows/ci-contract-gates.yml)
  --limit <n>                  How many recent runs to scan (default: 10)
  --artifacts-dir <path>       Write remote-ci-evidence-summary.json into this directory
  --json                       Emit machine-readable JSON summary
  --json-pretty                Pretty-print JSON output (requires --json)
  -h, --help                   Show help
USAGE
}

log() {
    echo "[ci-remote-evidence] $*"
}

fail() {
    echo "[ci-remote-evidence] $*" >&2
    exit 1
}

require_bin() {
    local name="$1"
    command -v "$name" >/dev/null 2>&1 || fail "required command not found: $name"
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

json_string_or_null() {
    local value="$1"
    if [[ -z "$value" ]]; then
        printf 'null'
    else
        printf '"%s"' "$(json_escape "$value")"
    fi
}

json_array_from_lines() {
    local first=1
    printf '['
    while IFS= read -r line; do
        [[ -n "$line" ]] || continue
        if [[ "$first" -eq 0 ]]; then
            printf ','
        fi
        printf '"%s"' "$(json_escape "$line")"
        first=0
    done
    printf ']'
}

emit_payload() {
    local payload="$1"
    if [[ "$JSON_MODE" -eq 1 && "$JSON_PRETTY" -eq 1 ]]; then
        printf '%s' "$payload" | jq .
    else
        printf '%s\n' "$payload"
    fi
}

write_artifact() {
    local payload="$1"
    if [[ -z "$ARTIFACTS_DIR" ]]; then
        return
    fi

    mkdir -p "$ARTIFACTS_DIR"
    local target="$ARTIFACTS_DIR/$DEFAULT_OUTPUT_NAME"
    if [[ "$JSON_PRETTY" -eq 1 ]]; then
        printf '%s' "$payload" | jq . > "$target"
    else
        printf '%s\n' "$payload" > "$target"
    fi
    if [[ "$JSON_MODE" -eq 1 ]]; then
        printf '[ci-remote-evidence] wrote summary artifact: %s\n' "$target" >&2
    else
        log "wrote summary artifact: $target"
    fi
}

build_payload() {
    local result="$1"
    local reason="$2"
    local workflow_id="$3"
    local run_id="$4"
    local run_status="$5"
    local run_conclusion="$6"
    local run_url="$7"
    local head_sha="$8"
    local created_at="$9"
    local updated_at="${10}"
    local missing_jobs="${11}"
    local failed_jobs="${12}"
    local promotion_ready="${13}"

    cat <<EOF
{"schema_version":"v1","result":"$result","reason":$(json_string_or_null "$reason"),"repo":"$(json_escape "$REPO")","branch":"$(json_escape "$BRANCH")","workflow_path":"$(json_escape "$WORKFLOW_PATH")","workflow_id":$workflow_id,"run_id":$run_id,"run_status":$(json_string_or_null "$run_status"),"run_conclusion":$(json_string_or_null "$run_conclusion"),"run_url":$(json_string_or_null "$run_url"),"head_sha":$(json_string_or_null "$head_sha"),"created_at":$(json_string_or_null "$created_at"),"updated_at":$(json_string_or_null "$updated_at"),"required_jobs":["contract-gates","cross-platform-smoke-macos","cross-platform-smoke-windows"],"missing_jobs":$missing_jobs,"failed_jobs":$failed_jobs,"promotion_ready":$promotion_ready}
EOF
}

REPO="$DEFAULT_REPO"
BRANCH="$DEFAULT_BRANCH"
WORKFLOW_PATH="$DEFAULT_WORKFLOW_PATH"
LIMIT="$DEFAULT_LIMIT"
ARTIFACTS_DIR=""
JSON_MODE=0
JSON_PRETTY=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            shift
            [[ $# -gt 0 ]] || fail "Missing value for --repo"
            REPO="$1"
            ;;
        --branch)
            shift
            [[ $# -gt 0 ]] || fail "Missing value for --branch"
            BRANCH="$1"
            ;;
        --workflow-path)
            shift
            [[ $# -gt 0 ]] || fail "Missing value for --workflow-path"
            WORKFLOW_PATH="$1"
            ;;
        --limit)
            shift
            [[ $# -gt 0 ]] || fail "Missing value for --limit"
            LIMIT="$1"
            ;;
        --artifacts-dir)
            shift
            [[ $# -gt 0 ]] || fail "Missing value for --artifacts-dir"
            ARTIFACTS_DIR="$1"
            ;;
        --json)
            JSON_MODE=1
            ;;
        --json-pretty)
            JSON_PRETTY=1
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

if [[ "$JSON_PRETTY" -eq 1 && "$JSON_MODE" -eq 0 ]]; then
    fail "--json-pretty requires --json"
fi

require_bin "$GH_BIN"
require_bin jq

workflow_json="$("$GH_BIN" api "repos/$REPO/actions/workflows")"
workflow_id="$(printf '%s' "$workflow_json" | jq -r --arg path "$WORKFLOW_PATH" '.workflows[] | select(.path == $path) | .id' | head -n 1)"

if [[ -z "$workflow_id" ]]; then
    payload="$(build_payload "error" "workflow_not_found" "null" "null" "" "" "" "" "" "" "[]" "[]" "false")"
    write_artifact "$payload"
    emit_payload "$payload"
    exit 2
fi

runs_json="$("$GH_BIN" api "repos/$REPO/actions/workflows/$workflow_id/runs?branch=$BRANCH&per_page=$LIMIT")"
run_json="$(printf '%s' "$runs_json" | jq -c '.workflow_runs[] | select(.status == "completed")' | head -n 1)"

if [[ -z "$run_json" ]]; then
    payload="$(build_payload "error" "no_completed_run" "$workflow_id" "null" "" "" "" "" "" "" "[]" "[]" "false")"
    write_artifact "$payload"
    emit_payload "$payload"
    exit 3
fi

run_id="$(printf '%s' "$run_json" | jq -r '.id')"
run_status="$(printf '%s' "$run_json" | jq -r '.status')"
run_conclusion="$(printf '%s' "$run_json" | jq -r '.conclusion')"
run_url="$(printf '%s' "$run_json" | jq -r '.html_url')"
head_sha="$(printf '%s' "$run_json" | jq -r '.head_sha')"
created_at="$(printf '%s' "$run_json" | jq -r '.created_at')"
updated_at="$(printf '%s' "$run_json" | jq -r '.updated_at')"

jobs_json="$("$GH_BIN" api "repos/$REPO/actions/runs/$run_id/jobs")"
missing_jobs="$(jq -n \
    --argjson jobs "$jobs_json" \
    '["contract-gates","cross-platform-smoke-macos","cross-platform-smoke-windows"]
      | map(select([ $jobs.jobs[].name ] | index(.) | not))')"
failed_jobs="$(jq -n \
    --argjson jobs "$jobs_json" \
    '[ $jobs.jobs[]
       | select(.name == "contract-gates" or .name == "cross-platform-smoke-macos" or .name == "cross-platform-smoke-windows")
       | select(.conclusion != "success")
       | .name ]')"

promotion_ready=false
result="error"
reason="required_jobs_not_green"
if [[ "$run_conclusion" == "success" ]] \
    && [[ "$(printf '%s' "$missing_jobs" | jq 'length')" -eq 0 ]] \
    && [[ "$(printf '%s' "$failed_jobs" | jq 'length')" -eq 0 ]]; then
    promotion_ready=true
    result="ok"
    reason=""
fi

payload="$(build_payload \
    "$result" \
    "$reason" \
    "$workflow_id" \
    "$run_id" \
    "$run_status" \
    "$run_conclusion" \
    "$run_url" \
    "$head_sha" \
    "$created_at" \
    "$updated_at" \
    "$missing_jobs" \
    "$failed_jobs" \
    "$promotion_ready")"

write_artifact "$payload"

if [[ "$JSON_MODE" -eq 1 ]]; then
    emit_payload "$payload"
    exit $([[ "$promotion_ready" == true ]] && echo 0 || echo 4)
fi

log "repo: $REPO"
log "branch: $BRANCH"
log "workflow: $WORKFLOW_PATH (#$workflow_id)"
log "run: $run_id ($run_conclusion) $run_url"
if [[ "$promotion_ready" == true ]]; then
    log "promotion ready: macOS + Windows (Git Bash) remote jobs are green"
    exit 0
fi

if [[ "$(printf '%s' "$missing_jobs" | jq 'length')" -gt 0 ]]; then
    log "missing jobs: $(printf '%s' "$missing_jobs" | jq -r 'join(", ")')"
fi
if [[ "$(printf '%s' "$failed_jobs" | jq 'length')" -gt 0 ]]; then
    log "failed jobs: $(printf '%s' "$failed_jobs" | jq -r 'join(", ")')"
fi
log "promotion not ready"
exit 4
