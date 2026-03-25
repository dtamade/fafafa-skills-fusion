#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARTIFACTS_DIR="/tmp"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
    cat <<'USAGE'
Usage: ci-machine-mode-smoke.sh [--artifacts-dir <path>]

Generate and validate the machine-readable CI artifacts:
- release-audit-dry-run.json
- runner-suites.json
- runner-contract.json

Defaults:
- artifacts dir: /tmp
USAGE
}

log() {
    echo "[ci-machine-mode-smoke] $*"
}

fail() {
    echo "[ci-machine-mode-smoke] $*" >&2
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --artifacts-dir)
            shift
            [[ $# -gt 0 ]] || fail "Missing value for --artifacts-dir"
            ARTIFACTS_DIR="$1"
            ;;
        --artifacts-dir=*)
            ARTIFACTS_DIR="${1#--artifacts-dir=}"
            [[ -n "$ARTIFACTS_DIR" ]] || fail "Missing value for --artifacts-dir"
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

if fusion_bridge_disabled; then
    fail "[fusion][deps] ci-machine-mode-smoke.sh requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release"
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || \
    fail "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release"

mkdir -p "$ARTIFACTS_DIR"

release_audit_json="$ARTIFACTS_DIR/release-audit-dry-run.json"
runner_suites_json="$ARTIFACTS_DIR/runner-suites.json"
runner_contract_json="$ARTIFACTS_DIR/runner-contract.json"

log "generating release audit dry-run JSON"
bash "$SCRIPT_DIR/release-contract-audit.sh" --dry-run --json --fast --skip-rust > "$release_audit_json"

log "generating regression suite JSON"
"$bridge_bin" regression --list-suites --json > "$runner_suites_json"
"$bridge_bin" regression --suite contract --json --min-pass-rate 0.99 > "$runner_contract_json"

log "validating machine JSON artifacts"
bash "$SCRIPT_DIR/ci-machine-json-smoke.sh" \
    "$release_audit_json" \
    "$runner_suites_json" \
    "$runner_contract_json"

log "machine-mode smoke passed"
