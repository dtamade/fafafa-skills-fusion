#!/bin/bash

emit_doctor_summary() {
  section "Summary"
  printf 'ok=%s warn=%s\n' "$ok_count" "$warn_count"

  if [ "$found_project_hooks" -eq 0 ]; then
    cat <<'EOT'
next_action: add Fusion hooks into .claude/settings.local.json in this project.
EOT
  fi
}

exit_doctor_with_summary() {
  if [ "$JSON_MODE" = true ]; then
    local result="ok"
    if [ "$warn_count" -gt 0 ]; then
      result="warn"
    fi

    local fixed="false"
    if [ "$FIX_APPLIED" = true ]; then
      fixed="true"
    fi

    printf '{"result":"%s","project_root":"%s","fusion_root":"%s","ok_count":%s,"warn_count":%s,"fixed":%s}\n' \
      "$(json_escape_string "$result")" \
      "$(json_escape_string "$PROJECT_ROOT")" \
      "$(json_escape_string "$FUSION_ROOT")" \
      "$ok_count" \
      "$warn_count" \
      "$fixed" >&3

    if [ "$warn_count" -gt 0 ]; then
      exit 1
    fi
    exit 0
  fi

  if [ "$warn_count" -gt 0 ]; then
    exit 1
  fi

  exit 0
}
