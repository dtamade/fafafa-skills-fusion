#!/bin/bash
# fusion-init.sh - Initialize .fusion directory for a project
set -euo pipefail

FUSION_DIR=".fusion"
STATE_LOCK="${FUSION_DIR}/.state.lock"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_DIR="$(dirname "$SCRIPT_DIR")/templates"

# Refuse to overwrite existing active workflow
if [ -f "$FUSION_DIR/sessions.json" ]; then
    if command -v jq &>/dev/null; then
        STATUS=$(jq -r '.status // "unknown"' "$FUSION_DIR/sessions.json" 2>/dev/null || echo "unknown")
    else
        STATUS=$(grep -o '"status"[[:space:]]*:[[:space:]]*"[^"]*"' "$FUSION_DIR/sessions.json" 2>/dev/null | head -1 | cut -d'"' -f4 || echo "unknown")
    fi

    if [ "$STATUS" = "in_progress" ] || [ "$STATUS" = "paused" ]; then
        echo "❌ Cannot reinitialize: workflow is $STATUS"
        echo "   Use /fusion cancel or /fusion resume"
        exit 1
    fi
fi

# Create .fusion directory
mkdir -p "$FUSION_DIR"

# Copy templates
if [ -f "$TEMPLATE_DIR/task_plan.md" ]; then
    cp "$TEMPLATE_DIR/task_plan.md" "$FUSION_DIR/task_plan.md"
fi

if [ -f "$TEMPLATE_DIR/progress.md" ]; then
    cp "$TEMPLATE_DIR/progress.md" "$FUSION_DIR/progress.md"
fi

if [ -f "$TEMPLATE_DIR/findings.md" ]; then
    cp "$TEMPLATE_DIR/findings.md" "$FUSION_DIR/findings.md"
fi

if [ -f "$TEMPLATE_DIR/config.yaml" ]; then
    cp "$TEMPLATE_DIR/config.yaml" "$FUSION_DIR/config.yaml"
fi

# Copy sessions.json template (no lock needed - this is initialization)
if [ -f "$TEMPLATE_DIR/sessions.json" ]; then
    cp "$TEMPLATE_DIR/sessions.json" "$FUSION_DIR/sessions.json"
else
    # Fallback: create basic sessions.json
    cat > "$FUSION_DIR/sessions.json" << 'SESSIONS_EOF'
{
  "workflow_id": null,
  "goal": null,
  "started_at": null,
  "status": "not_started",
  "current_phase": null,
  "codex_session": null,
  "tasks": {},
  "strikes": {
    "current_task": null,
    "count": 0,
    "history": []
  },
  "git": {
    "branch": null,
    "commits": []
  },
  "last_checkpoint": null
}
SESSIONS_EOF
fi

# Add to .gitignore if not already there
if [ -f ".gitignore" ]; then
    if ! grep -q "^\.fusion/$" .gitignore 2>/dev/null; then
        echo "" >> .gitignore
        echo "# Fusion working directory" >> .gitignore
        echo ".fusion/" >> .gitignore
    fi
fi

echo "[fusion] Initialized .fusion directory"
echo "[fusion] Files created:"
ls -la "$FUSION_DIR"
