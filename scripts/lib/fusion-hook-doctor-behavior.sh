#!/bin/bash

run_doctor_behavior_checks() {
  section "Behavior Checks"

  PRE_OUT=""
  PRE_RC=0
  PRE_OUT="$(cd "$PROJECT_ROOT" && printf '{"hook_event_name":"PreToolUse","tool_name":"Read"}' | bash "$PRETOOL_SCRIPT" 2>&1)" || PRE_RC=$?
  printf 'pretool.rc: %s\n' "$PRE_RC"
  if [ -n "$PRE_OUT" ]; then
    printf 'pretool.out: %s\n' "$(printf '%s' "$PRE_OUT" | head -n 1)"
  fi
  if [ "$PRE_RC" -eq 0 ]; then
    ok "pretool executes"
  else
    warn "pretool non-zero exit"
  fi

  STOP_LEGACY_OUT=""
  STOP_LEGACY_RC=0
  STOP_LEGACY_OUT="$(cd "$PROJECT_ROOT" && FUSION_STOP_HOOK_MODE=legacy bash "$STOP_SCRIPT" </dev/null 2>&1)" || STOP_LEGACY_RC=$?
  printf 'stop.legacy.rc: %s\n' "$STOP_LEGACY_RC"
  if [ -n "$STOP_LEGACY_OUT" ]; then
    printf 'stop.legacy.out: %s\n' "$(printf '%s' "$STOP_LEGACY_OUT" | head -n 1)"
  fi
  if [ "$STATUS" = "in_progress" ] && [ "$STOP_LEGACY_RC" -eq 2 ]; then
    ok "stop hook blocks in legacy mode"
  elif [ "$STATUS" = "in_progress" ] && [ "$STOP_LEGACY_RC" -eq 0 ]; then
    warn "workflow in progress but stop allowed (check task status / state)"
  else
    ok "stop hook return is consistent with current status"
  fi

  STOP_JSON_OUT=""
  STOP_JSON_RC=0
  STOP_JSON_OUT="$(cd "$PROJECT_ROOT" && printf '{}' | FUSION_STOP_HOOK_MODE=structured bash "$STOP_SCRIPT" 2>&1)" || STOP_JSON_RC=$?
  printf 'stop.structured.rc: %s\n' "$STOP_JSON_RC"
  if [ -n "$STOP_JSON_OUT" ]; then
    printf 'stop.structured.out: %s\n' "$(printf '%s' "$STOP_JSON_OUT" | head -n 1)"
  fi
  if [ "$STOP_JSON_RC" -eq 0 ]; then
    ok "structured stop mode executes"
  else
    warn "structured stop mode non-zero"
  fi
}
