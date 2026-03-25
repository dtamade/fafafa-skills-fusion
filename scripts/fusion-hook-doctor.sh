#!/bin/bash
# fusion-hook-doctor.sh - Validate Fusion hook wiring and runtime behavior

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FUSION_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

source "$SCRIPT_DIR/lib/fusion-bridge.sh"
source "$SCRIPT_DIR/lib/fusion-json.sh"
source "$SCRIPT_DIR/lib/fusion-task-plan.sh"
source "$SCRIPT_DIR/lib/fusion-hook-doctor-core.sh"
source "$SCRIPT_DIR/lib/fusion-hook-doctor-behavior.sh"
source "$SCRIPT_DIR/lib/fusion-hook-doctor-summary.sh"

PRETOOL_SCRIPT="$FUSION_ROOT/scripts/fusion-pretool.sh"
POSTTOOL_SCRIPT="$FUSION_ROOT/scripts/fusion-posttool.sh"
STOP_SCRIPT="$FUSION_ROOT/scripts/fusion-stop-guard.sh"

JSON_MODE=false
FIX_MODE=false
FIX_APPLIED=false
PROJECT_ARG=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --json)
      JSON_MODE=true
      ;;
    --fix)
      FIX_MODE=true
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    -*)
      fail_with_reason "Unknown option: $1"
      ;;
    *)
      if [ -z "$PROJECT_ARG" ]; then
        PROJECT_ARG="$1"
      else
        fail_with_reason "Unexpected argument: $1"
      fi
      ;;
  esac
  shift
done

PROJECT_ROOT="${PROJECT_ARG:-$PWD}"
if [ ! -d "$PROJECT_ROOT" ]; then
  fail_with_reason "project_root not found: $PROJECT_ROOT"
fi
PROJECT_ROOT="$(cd "$PROJECT_ROOT" && pwd)"

if [ "$JSON_MODE" = true ] && ! fusion_bridge_disabled; then
  if bridge_bin="$(resolve_fusion_bridge_bin "$SCRIPT_DIR")"; then
    BRIDGE_ARGS=(doctor --json "$PROJECT_ROOT")
    if [ "$FIX_MODE" = true ]; then
      BRIDGE_ARGS=(doctor --json --fix "$PROJECT_ROOT")
    fi
    exec "$bridge_bin" "${BRIDGE_ARGS[@]}"
  fi
fi

if [ "$JSON_MODE" = true ]; then
  exec 3>&1
  exec >/dev/null
fi

PROJECT_SETTINGS_LOCAL="$PROJECT_ROOT/.claude/settings.local.json"
PROJECT_SETTINGS="$PROJECT_ROOT/.claude/settings.json"
GLOBAL_SETTINGS="$HOME/.claude/settings.json"
FUSION_DIR="$PROJECT_ROOT/.fusion"

ok_count=0
warn_count=0

section "Context"
printf 'fusion_root: %s\n' "$FUSION_ROOT"
printf 'project_root: %s\n' "$PROJECT_ROOT"
printf 'date_utc: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

section "Script Presence"
for script in "$PRETOOL_SCRIPT" "$POSTTOOL_SCRIPT" "$STOP_SCRIPT"; do
  if [ -x "$script" ]; then
    ok "executable: $script"
  elif [ -f "$script" ]; then
    warn "not executable (still runnable via bash): $script"
  else
    warn "missing script: $script"
  fi
done

if [ "$FIX_MODE" = true ]; then
  if write_project_hooks_settings; then
    FIX_APPLIED=true
    ok "auto-fixed project hooks: $PROJECT_SETTINGS_LOCAL"
  else
    warn "auto-fix failed: unable to write $PROJECT_SETTINGS_LOCAL"
  fi
fi

section "Hook Wiring"
found_project_hooks=0

if has_literal "$PROJECT_SETTINGS_LOCAL" "fusion-pretool.sh" && has_literal "$PROJECT_SETTINGS_LOCAL" "fusion-posttool.sh" && has_literal "$PROJECT_SETTINGS_LOCAL" "fusion-stop-guard.sh"; then
  found_project_hooks=1
  if has_project_dir_hook_paths "$PROJECT_SETTINGS_LOCAL"; then
    ok "project local hooks wired: $PROJECT_SETTINGS_LOCAL"
  else
    warn "project local hooks use relative command paths; run --fix to rewrite with \${CLAUDE_PROJECT_DIR:-.}"
  fi
elif has_literal "$PROJECT_SETTINGS" "fusion-pretool.sh" && has_literal "$PROJECT_SETTINGS" "fusion-posttool.sh" && has_literal "$PROJECT_SETTINGS" "fusion-stop-guard.sh"; then
  found_project_hooks=1
  if has_project_dir_hook_paths "$PROJECT_SETTINGS"; then
    ok "project hooks wired: $PROJECT_SETTINGS"
  else
    warn "project hooks use relative command paths; run --fix to rewrite with \${CLAUDE_PROJECT_DIR:-.}"
  fi
else
  warn "project settings missing full Fusion hook trio (.claude/settings*.json)"
fi

if has_literal "$GLOBAL_SETTINGS" "fusion-pretool.sh" || has_literal "$GLOBAL_SETTINGS" "fusion-stop-guard.sh"; then
  ok "global settings contains Fusion hooks: $GLOBAL_SETTINGS"
elif [ "$found_project_hooks" -eq 1 ]; then
  ok "global settings has no Fusion hooks (project hooks are active): $GLOBAL_SETTINGS"
else
  warn "global settings has no Fusion hooks: $GLOBAL_SETTINGS"
fi

section "Workflow State"
if [ -d "$FUSION_DIR" ]; then
  ok "fusion workspace exists: $FUSION_DIR"
else
  warn "fusion workspace missing: $FUSION_DIR"
fi

STATUS="$(json_get_status "$FUSION_DIR/sessions.json")"
if [ -n "$STATUS" ]; then
  printf 'sessions.status: %s\n' "$STATUS"
else
  warn "sessions.status unavailable (.fusion/sessions.json missing or invalid)"
fi

TASK_PLAN="$FUSION_DIR/task_plan.md"
RUNTIME_ENGINE="$(fusion_runtime_engine "$FUSION_DIR")"
RUNTIME_COMPAT_MODE="$(fusion_runtime_compat_mode "$FUSION_DIR")"
printf 'runtime.engine: %s
' "$RUNTIME_ENGINE"
printf 'runtime.compat_mode: %s
' "$RUNTIME_COMPAT_MODE"
PENDING_COUNT="$(count_tasks "$TASK_PLAN" '\[PENDING\]')"
INPROGRESS_COUNT="$(count_tasks "$TASK_PLAN" '\[IN_PROGRESS\]')"
COMPLETED_COUNT="$(count_tasks "$TASK_PLAN" '\[COMPLETED\]')"
printf 'task_counts: pending=%s in_progress=%s completed=%s
' "$PENDING_COUNT" "$INPROGRESS_COUNT" "$COMPLETED_COUNT"

run_doctor_behavior_checks
emit_doctor_summary
exit_doctor_with_summary
