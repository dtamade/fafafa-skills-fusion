#!/bin/bash
# fusion-hook-doctor.sh - Validate Fusion hook wiring and runtime behavior

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FUSION_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

PRETOOL_SCRIPT="$FUSION_ROOT/scripts/fusion-pretool.sh"
POSTTOOL_SCRIPT="$FUSION_ROOT/scripts/fusion-posttool.sh"
STOP_SCRIPT="$FUSION_ROOT/scripts/fusion-stop-guard.sh"

JSON_MODE=false
FIX_MODE=false
FIX_APPLIED=false
PROJECT_ARG=""

usage() {
  cat <<'USAGE'
Usage: fusion-hook-doctor.sh [--json] [--fix] [project_root]
USAGE
}

emit_json_error() {
  local reason="$1"

  if command -v jq >/dev/null 2>&1; then
    jq -nc --arg result "error" --arg reason "$reason" '{result:$result,reason:$reason}'
    return 0
  fi

  if command -v python3 >/dev/null 2>&1; then
    python3 - "$reason" <<'PYJSON'
import json
import sys

print(json.dumps({"result": "error", "reason": sys.argv[1]}, ensure_ascii=False))
PYJSON
    return 0
  fi

  printf '{"result":"error","reason":"%s"}
' "$reason"
}

fail_with_reason() {
  local reason="$1"
  if [ "$JSON_MODE" = true ]; then
    emit_json_error "$reason"
  else
    echo "$reason" >&2
  fi
  exit 1
}

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

ok() {
  printf '[OK] %s\n' "$1"
  ok_count=$((ok_count + 1))
}

warn() {
  printf '[WARN] %s\n' "$1"
  warn_count=$((warn_count + 1))
}

section() {
  printf '\n=== %s ===\n' "$1"
}

has_literal() {
  local file="$1"
  local needle="$2"
  [ -f "$file" ] || return 1
  grep -Fq "$needle" "$file"
}

has_project_dir_hook_paths() {
  local file="$1"
  (
    has_literal "$file" '${CLAUDE_PROJECT_DIR}/scripts/fusion-pretool.sh' \
      || has_literal "$file" '${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh'
  ) \
    && (
      has_literal "$file" '${CLAUDE_PROJECT_DIR}/scripts/fusion-posttool.sh' \
        || has_literal "$file" '${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh'
    ) \
    && (
      has_literal "$file" '${CLAUDE_PROJECT_DIR}/scripts/fusion-stop-guard.sh' \
        || has_literal "$file" '${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh'
    )
}


write_project_hooks_settings() {
  local settings_dir="$PROJECT_ROOT/.claude"
  mkdir -p "$settings_dir" || return 1

  cat > "$PROJECT_SETTINGS_LOCAL" <<'JSON'
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit|Bash|Read|Glob|Grep",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh\""
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh\""
          }
        ]
      }
    ]
  }
}
JSON
}

json_get_status() {
  local file="$1"
  if [ ! -f "$file" ]; then
    echo ""
    return
  fi

  if command -v jq >/dev/null 2>&1; then
    jq -r '.status // empty' "$file" 2>/dev/null || true
    return
  fi

  grep -o '"status"[[:space:]]*:[[:space:]]*"[^"]*"' "$file" 2>/dev/null | head -1 | cut -d'"' -f4
}

count_tasks() {
  local file="$1"
  local pattern="$2"
  if [ -f "$file" ]; then
    local count
    count=$(grep -c "$pattern" "$file" 2>/dev/null || true)
    [ -n "$count" ] && echo "$count" || echo "0"
  else
    echo "0"
  fi
}

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
    warn "project local hooks use relative command paths; run --fix to rewrite with \${CLAUDE_PROJECT_DIR}"
  fi
elif has_literal "$PROJECT_SETTINGS" "fusion-pretool.sh" && has_literal "$PROJECT_SETTINGS" "fusion-posttool.sh" && has_literal "$PROJECT_SETTINGS" "fusion-stop-guard.sh"; then
  found_project_hooks=1
  if has_project_dir_hook_paths "$PROJECT_SETTINGS"; then
    ok "project hooks wired: $PROJECT_SETTINGS"
  else
    warn "project hooks use relative command paths; run --fix to rewrite with \${CLAUDE_PROJECT_DIR}"
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
PENDING_COUNT="$(count_tasks "$TASK_PLAN" '\\[PENDING\\]')"
INPROGRESS_COUNT="$(count_tasks "$TASK_PLAN" '\\[IN_PROGRESS\\]')"
COMPLETED_COUNT="$(count_tasks "$TASK_PLAN" '\\[COMPLETED\\]')"
printf 'task_counts: pending=%s in_progress=%s completed=%s\n' "$PENDING_COUNT" "$INPROGRESS_COUNT" "$COMPLETED_COUNT"

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

section "Summary"
printf 'ok=%s warn=%s\n' "$ok_count" "$warn_count"

if [ "$found_project_hooks" -eq 0 ]; then
  cat <<'EOT'
next_action: add Fusion hooks into .claude/settings.local.json in this project.
EOT
fi

if [ "$JSON_MODE" = true ]; then
  result="ok"
  if [ "$warn_count" -gt 0 ]; then
    result="warn"
  fi

  fixed="false"
  if [ "$FIX_APPLIED" = true ]; then
    fixed="true"
  fi

  export DOCTOR_RESULT="$result"
  export DOCTOR_PROJECT_ROOT="$PROJECT_ROOT"
  export DOCTOR_FUSION_ROOT="$FUSION_ROOT"
  export DOCTOR_OK_COUNT="$ok_count"
  export DOCTOR_WARN_COUNT="$warn_count"
  export DOCTOR_FIXED="$fixed"

  if command -v jq >/dev/null 2>&1; then
    jq -nc       --arg result "$result"       --arg project_root "$PROJECT_ROOT"       --arg fusion_root "$FUSION_ROOT"       --argjson ok_count "$ok_count"       --argjson warn_count "$warn_count"       --argjson fixed "$fixed"       '{result:$result,project_root:$project_root,fusion_root:$fusion_root,ok_count:$ok_count,warn_count:$warn_count,fixed:$fixed}' >&3
  else
    python3 - <<'PYJSON' >&3
import json
import os
print(json.dumps({
    "result": os.environ.get("DOCTOR_RESULT", "ok"),
    "project_root": os.environ.get("DOCTOR_PROJECT_ROOT", ""),
    "fusion_root": os.environ.get("DOCTOR_FUSION_ROOT", ""),
    "ok_count": int(os.environ.get("DOCTOR_OK_COUNT", "0")),
    "warn_count": int(os.environ.get("DOCTOR_WARN_COUNT", "0")),
    "fixed": os.environ.get("DOCTOR_FIXED", "false").lower() == "true",
}, ensure_ascii=False))
PYJSON
  fi

  if [ "$warn_count" -gt 0 ]; then
    exit 1
  fi
  exit 0
fi

if [ "$warn_count" -gt 0 ]; then
  exit 1
fi

exit 0
