#!/bin/bash
# fusion-pretool.sh - Attention Injection Engine
#
# PreToolUse hook: runs BEFORE every Write/Edit/Bash/Read/Glob/Grep call.
# Outputs a compact context summary to keep Claude focused on the current task.
#
# Design constraints:
#   - Must execute in < 50ms (no jq, pure grep/awk)
#   - Non-invasive: silent exit if no active Fusion workflow
#   - Fault-tolerant: all operations || true, never blocks Claude
#
# Output goes to stdout and appears in Claude's context window,
# implementing the Manus "attention manipulation through recitation" pattern.

FUSION_DIR=".fusion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

is_truthy() {
    case "$(printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]')" in
        1|true|yes|on)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

HOOK_DEBUG=false
if is_truthy "${FUSION_HOOK_DEBUG:-}" || [ -f "$FUSION_DIR/.hook_debug" ]; then
    HOOK_DEBUG=true
fi

hook_debug_log() {
    [ "$HOOK_DEBUG" = true ] || return 0
    local message="$1"
    local ts
    ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local line="[fusion][hook-debug][pretool][$ts] $message"
    echo "$line" >&2
    if [ -d "$FUSION_DIR" ]; then
        echo "$line" >> "$FUSION_DIR/hook-debug.log" 2>/dev/null || true
    fi
}

resolve_fusion_bridge_bin() {
    if [ -n "${FUSION_BRIDGE_BIN:-}" ] && [ -x "$FUSION_BRIDGE_BIN" ]; then
        echo "$FUSION_BRIDGE_BIN"
        return 0
    fi

    if command -v fusion-bridge >/dev/null 2>&1; then
        command -v fusion-bridge
        return 0
    fi

    local candidates=(
        "$SCRIPT_DIR/../rust/target/release/fusion-bridge"
        "$SCRIPT_DIR/../rust/target/debug/fusion-bridge"
    )

    local candidate
    for candidate in "${candidates[@]}"; do
        if [ -x "$candidate" ]; then
            echo "$candidate"
            return 0
        fi
    done

    return 1
}

runtime_engine_is_rust() {
    [ -f "$FUSION_DIR/config.yaml" ] || return 1
    grep -Eq 'engine:[[:space:]]*"?rust"?' "$FUSION_DIR/config.yaml" 2>/dev/null
}

runtime_enabled_in_config() {
    [ -f "$FUSION_DIR/config.yaml" ] || return 1

    awk '
    BEGIN { in_runtime = 0; found = 0 }
    /^[[:space:]]*#/ { next }
    /^[^[:space:]#][^:]*:[[:space:]]*$/ {
        key = $0
        sub(/[[:space:]]*:[[:space:]]*$/, "", key)
        in_runtime = (key == "runtime")
        next
    }
    in_runtime && /^[[:space:]]+enabled:[[:space:]]*/ {
        value = $0
        sub(/^[[:space:]]+enabled:[[:space:]]*/, "", value)
        sub(/[[:space:]]*#.*/, "", value)
        gsub(/[[:space:]\"]/, "", value)
        if (tolower(value) == "true") {
            found = 1
        }
        exit
    }
    /^[^[:space:]#]/ {
        in_runtime = 0
    }
    END { exit(found ? 0 : 1) }
    ' "$FUSION_DIR/config.yaml" 2>/dev/null
}

# Fast exit: no fusion directory → not in a workflow
if [ ! -d "$FUSION_DIR" ]; then
    hook_debug_log "skip: .fusion missing"
    exit 0
fi

# Fast exit: no sessions.json → not initialized
if [ ! -f "$FUSION_DIR/sessions.json" ]; then
    hook_debug_log "skip: sessions.json missing"
    exit 0
fi

hook_debug_log "invoked: cwd=$(pwd)"

# Read hook input from stdin (PreToolUse hook protocol)
# Input contains: {"tool_name": "...", "tool_input": {...}}
HOOK_INPUT=$(cat)

# --- Runtime adapter ---
# If runtime.enabled is true, prefer Rust bridge when runtime.engine=rust, else Python compat_v2.
if runtime_enabled_in_config; then
    hook_debug_log "runtime-adapter: enabled"
    if runtime_engine_is_rust; then
        hook_debug_log "runtime-adapter: engine=rust"
        BRIDGE_BIN=""
        if BRIDGE_BIN="$(resolve_fusion_bridge_bin 2>/dev/null)"; then
            if "$BRIDGE_BIN" hook pretool --fusion-dir "$FUSION_DIR" 2>/dev/null; then
                hook_debug_log "runtime-adapter: rust bridge ok"
                exit 0
            fi
            hook_debug_log "runtime-adapter: rust bridge failed"
        else
            hook_debug_log "runtime-adapter: rust bridge missing"
        fi
    fi

    if PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" python3 -m runtime.compat_v2 pretool "$FUSION_DIR" 2>/dev/null; then
        hook_debug_log "runtime-adapter: python compat ok"
        exit 0
    fi
    hook_debug_log "runtime-adapter: failed, fallback=shell"
    # Runtime adapter failed - fall through to Shell logic
fi

# --- Cross-platform Unicode detection ---
# Use ASCII fallback for terminals that don't support Unicode well (Windows CMD)
# Priority: WT_SESSION (Windows Terminal) > TERM_PROGRAM (VSCode/iTerm) > valid TERM
USE_UNICODE=false
if [ -n "${WT_SESSION:-}" ]; then
    USE_UNICODE=true
elif [ "${TERM_PROGRAM:-}" = "vscode" ] || [ "${TERM_PROGRAM:-}" = "iTerm.app" ]; then
    USE_UNICODE=true
elif [ -n "${TERM:-}" ] && [ "${TERM:-}" != "dumb" ]; then
    USE_UNICODE=true
fi

if [ "$USE_UNICODE" = true ]; then
    CHAR_FILLED="█"
    CHAR_EMPTY="░"
else
    # Fallback for basic terminals
    CHAR_FILLED="#"
    CHAR_EMPTY="-"
fi

# --- JSON parsing helper ---
# Use jq if available (more robust), fallback to grep (faster but fragile)
json_get() {
    local file="$1" key="$2"
    if command -v jq &>/dev/null; then
        jq -r ".$key // empty" "$file" 2>/dev/null || echo ""
    else
        grep -o "\"$key\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$file" 2>/dev/null | head -1 | cut -d'"' -f4 || echo ""
    fi
}

# Read status
STATUS=$(json_get "$FUSION_DIR/sessions.json" "status")

# Fast exit: not in_progress → workflow inactive
if [ "$STATUS" != "in_progress" ]; then
    hook_debug_log "skip: status=$STATUS"
    exit 0
fi

# --- Active workflow: build context summary ---

# Read goal (truncate to 60 chars, strip control chars for safe display)
GOAL=$(json_get "$FUSION_DIR/sessions.json" "goal")
GOAL=$(printf '%.60s' "$GOAL" | tr -d '"\\\t\n\r')

# Read current phase
PHASE=$(json_get "$FUSION_DIR/sessions.json" "current_phase")
PHASE="${PHASE:-EXECUTE}"

# Phase number mapping (9 phases: 0=UNDERSTAND to 8=DELIVER)
case "$PHASE" in
    UNDERSTAND) PHASE_NUM="0/8" ;;
    INITIALIZE) PHASE_NUM="1/8" ;;
    ANALYZE)    PHASE_NUM="2/8" ;;
    DECOMPOSE)  PHASE_NUM="3/8" ;;
    EXECUTE)    PHASE_NUM="4/8" ;;
    VERIFY)     PHASE_NUM="5/8" ;;
    REVIEW)     PHASE_NUM="6/8" ;;
    COMMIT)     PHASE_NUM="7/8" ;;
    DELIVER)    PHASE_NUM="8/8" ;;
    *)          PHASE_NUM="?/8" ;;
esac

# Count tasks from task_plan.md (if exists)
COMPLETED=0
PENDING=0
IN_PROGRESS=0
FAILED=0
TOTAL=0
CURRENT_TASK=""
CURRENT_TASK_TYPE=""

if [ -f "$FUSION_DIR/task_plan.md" ]; then
    COMPLETED=$(grep -c '\[COMPLETED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || COMPLETED=0
    PENDING=$(grep -c '\[PENDING\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || PENDING=0
    IN_PROGRESS=$(grep -c '\[IN_PROGRESS\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || IN_PROGRESS=0
    FAILED=$(grep -c '\[FAILED\]' "$FUSION_DIR/task_plan.md" 2>/dev/null) || FAILED=0
    TOTAL=$((COMPLETED + PENDING + IN_PROGRESS + FAILED))

    # Find current/next task (first IN_PROGRESS, then first PENDING)
    if [ "$IN_PROGRESS" -gt 0 ]; then
        CURRENT_TASK=$(grep '\[IN_PROGRESS\]' "$FUSION_DIR/task_plan.md" 2>/dev/null | head -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//')
    elif [ "$PENDING" -gt 0 ]; then
        CURRENT_TASK=$(grep '\[PENDING\]' "$FUSION_DIR/task_plan.md" 2>/dev/null | head -1 | sed 's/### Task [0-9]*: //' | sed 's/ \[.*//')
    fi

    # Get task type (look for "Type:" line after current task header)
    # Use -F for fixed string matching to avoid regex injection from task names
    if [ -n "$CURRENT_TASK" ]; then
        CURRENT_TASK_TYPE=$(grep -F -A5 "$CURRENT_TASK" "$FUSION_DIR/task_plan.md" 2>/dev/null | grep -o 'Type: *[a-z]*' | head -1 | sed 's/Type: *//')
    fi
fi

# Build progress bar (10 chars width)
# Cross-platform: use bash loop instead of seq (not available on Windows Git Bash)
PROGRESS_BAR=""
if [ "$TOTAL" -gt 0 ]; then
    FILLED=$(( COMPLETED * 10 / TOTAL ))
    EMPTY=$(( 10 - FILLED ))
    for ((i=0; i<FILLED; i++)); do PROGRESS_BAR+="$CHAR_FILLED"; done
    for ((i=0; i<EMPTY; i++)); do PROGRESS_BAR+="$CHAR_EMPTY"; done
    PERCENT=$(( COMPLETED * 100 / TOTAL ))
else
    for ((i=0; i<10; i++)); do PROGRESS_BAR+="$CHAR_EMPTY"; done
    PERCENT=0
fi

# Execution guidance based on task type
GUIDANCE=""
case "$CURRENT_TASK_TYPE" in
    implementation|verification)
        GUIDANCE="TDD flow: RED → GREEN → REFACTOR"
        ;;
    design|documentation|configuration|research)
        GUIDANCE="Direct execution"
        ;;
    *)
        if [ "$PHASE" = "EXECUTE" ] && [ -n "$CURRENT_TASK" ]; then
            GUIDANCE="Check task type in task_plan.md"
        fi
        ;;
esac

# Guardian status (quick check from loop_context.json if exists, pure grep)
GUARDIAN_STATUS="OK"
if [ -f "$FUSION_DIR/loop_context.json" ]; then
    NO_PROGRESS=$(grep -o '"no_progress_rounds"[[:space:]]*:[[:space:]]*[0-9]*' "$FUSION_DIR/loop_context.json" 2>/dev/null | grep -o '[0-9]*$') || true
    SAME_ACTION=$(grep -o '"same_action_count"[[:space:]]*:[[:space:]]*[0-9]*' "$FUSION_DIR/loop_context.json" 2>/dev/null | grep -o '[0-9]*$') || true

    if [ "${NO_PROGRESS:-0}" -ge 4 ] || [ "${SAME_ACTION:-0}" -ge 2 ]; then
        GUARDIAN_STATUS="⚠ BACKOFF"
    elif [ "${NO_PROGRESS:-0}" -ge 2 ]; then
        GUARDIAN_STATUS="~"
    fi
fi

# --- Output compact summary ---

hook_debug_log "active: phase=$PHASE completed=$COMPLETED pending=$PENDING in_progress=$IN_PROGRESS failed=$FAILED task=${CURRENT_TASK:-none}"

echo "[fusion] Goal: ${GOAL:-?} | Phase: $PHASE ($PHASE_NUM)"

if [ "$TOTAL" -gt 0 ] && [ -n "$CURRENT_TASK" ]; then
    TASK_STATUS="PENDING"
    [ "$IN_PROGRESS" -gt 0 ] && TASK_STATUS="IN_PROGRESS"
    TASK_INDEX=$((COMPLETED + 1))
    TYPE_DISPLAY=""
    [ -n "$CURRENT_TASK_TYPE" ] && TYPE_DISPLAY=" (type: $CURRENT_TASK_TYPE)"
    echo "[fusion] Task ${TASK_INDEX}/${TOTAL}: ${CURRENT_TASK} [${TASK_STATUS}]${TYPE_DISPLAY}"
    echo "[fusion] Progress: ${PROGRESS_BAR} ${PERCENT}% | Guardian: ${GUARDIAN_STATUS}"
fi

if [ -n "$GUIDANCE" ]; then
    echo "[fusion] → $GUIDANCE"
fi

hook_debug_log "done: emitted-summary"
exit 0
