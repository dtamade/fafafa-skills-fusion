#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
  cat <<'USAGE'
Usage: fusion-hook-selfcheck.sh [options] [project_root]

Options:
  --fix           Auto-fix project hooks before checks
  --quick         Skip contract regression suite (doctor + stop simulation only)
  --json          Emit machine-readable JSON summary
  -h, --help      Show help

Checks:
  1) fusion-hook-doctor (with optional --fix)
  2) stop-hook simulation (empty stdin must return JSON block + rc=0)
  3) Rust contract regression suite (skipped when --quick)
USAGE
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

fail_selfcheck_validation() {
  local reason="$1"
  if [[ "$JSON_MODE" -eq 1 ]]; then
    printf '{"result":"error","reason":"%s"}\n' "$(json_escape "$reason")"
  else
    echo "$reason" >&2
    usage >&2
  fi
  exit 1
}

fail_wrapper() {
  local reason="$1"
  if [[ "$JSON_MODE" -eq 1 ]]; then
    printf '{"result":"error","reason":"%s"}\n' "$(json_escape "$reason")"
  else
    echo "$reason" >&2
  fi
  exit 127
}

JSON_MODE=0
FIX_MODE=0
QUICK_MODE=0
PROJECT_ARG=""

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
      fail_selfcheck_validation "Unknown option: $1"
      ;;
    *)
      if [[ -z "$PROJECT_ARG" ]]; then
        PROJECT_ARG="$1"
      else
        fail_selfcheck_validation "Unexpected argument: $1"
      fi
      ;;
  esac
  shift
done

PROJECT_ROOT="${PROJECT_ARG:-$PWD}"
if [[ ! -d "$PROJECT_ROOT" ]]; then
  fail_selfcheck_validation "project_root not found: $PROJECT_ROOT"
fi
PROJECT_ROOT="$(cd "$PROJECT_ROOT" && pwd)"

if fusion_bridge_disabled; then
  fail_wrapper "[fusion][deps] fusion-hook-selfcheck.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release"
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || \
  fail_wrapper "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release"

bridge_args=(selfcheck)
if [[ "$FIX_MODE" -eq 1 ]]; then
  bridge_args+=(--fix)
fi
if [[ "$QUICK_MODE" -eq 1 ]]; then
  bridge_args+=(--quick)
fi
if [[ "$JSON_MODE" -eq 1 ]]; then
  bridge_args+=(--json)
fi
bridge_args+=("$PROJECT_ROOT")

exec "$bridge_bin" "${bridge_args[@]}"
