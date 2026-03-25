#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"

usage() {
  cat <<'USAGE'
Usage: release-contract-audit.sh [options]

Options:
  --dry-run       Print the gate command plan only
  --json          Emit machine-readable summary JSON
  --json-pretty   Pretty-print JSON output (requires --json)
  --fast          Skip wrapper smoke and run machine-mode smoke only
  --skip-rust     Skip rust clippy/test/fmt gates
  -h, --help      Show help

Run release contract gates:
  1) shell syntax
  2) machine-mode smoke
  3) wrapper smoke (unless --fast)
  4) rust clippy gate (unless --skip-rust)
  5) rust test gate (unless --skip-rust)
  6) rust fmt gate (unless --skip-rust)
USAGE
}

die() {
  echo "Unknown option: $1" >&2
  usage >&2
  exit 1
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

fail_wrapper() {
  if [[ "$JSON_MODE" -eq 1 ]]; then
    printf '{"result":"error","reason":"%s"}\n' "$(json_escape "$1")"
  else
    echo "$1" >&2
  fi
  exit 127
}

DRY_RUN=0
JSON_MODE=0
JSON_PRETTY=0
FAST=0
SKIP_RUST=0

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

if fusion_bridge_disabled; then
  fail_wrapper "[fusion][deps] release-contract-audit.sh now requires Rust fusion-bridge. Unset FUSION_BRIDGE_DISABLE or build with: cd rust && cargo build --release"
fi

bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")" || \
  fail_wrapper "[fusion][deps] Missing Rust fusion-bridge. Build with: cd rust && cargo build --release"

bridge_args=(audit)
if [[ "$DRY_RUN" -eq 1 ]]; then
  bridge_args+=(--dry-run)
fi
if [[ "$JSON_MODE" -eq 1 ]]; then
  bridge_args+=(--json)
fi
if [[ "$JSON_PRETTY" -eq 1 ]]; then
  bridge_args+=(--json-pretty)
fi
if [[ "$FAST" -eq 1 ]]; then
  bridge_args+=(--fast)
fi
if [[ "$SKIP_RUST" -eq 1 ]]; then
  bridge_args+=(--skip-rust)
fi

exec "$bridge_bin" "${bridge_args[@]}"
