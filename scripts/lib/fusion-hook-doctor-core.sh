#!/bin/bash

usage() {
  cat <<'USAGE'
Usage: fusion-hook-doctor.sh [--json] [--fix] [project_root]
USAGE
}

emit_json_error() {
  local reason="$1"
  printf '{"result":"error","reason":"%s"}\n' "$(json_escape_string "$reason")"
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
  has_literal "$file" '${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh' \
    && has_literal "$file" '${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh' \
    && has_literal "$file" '${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh'
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

  json_get_field "$file" "status"
}

count_tasks() {
  local file="$1"
  local pattern="$2"
  [ -f "$file" ] || {
    echo "0"
    return
  }

  local completed pending in_progress failed
  IFS=':' read -r completed pending in_progress failed <<< "$(fusion_task_counts "$file")"

  case "$pattern" in
    '\[PENDING\]')
      echo "${pending:-0}"
      ;;
    '\[IN_PROGRESS\]')
      echo "${in_progress:-0}"
      ;;
    '\[COMPLETED\]')
      echo "${completed:-0}"
      ;;
    '\[FAILED\]')
      echo "${failed:-0}"
      ;;
    *)
      local count
      count=$(grep -c "$pattern" "$file" 2>/dev/null || true)
      [ -n "$count" ] && echo "$count" || echo "0"
      ;;
  esac
}
