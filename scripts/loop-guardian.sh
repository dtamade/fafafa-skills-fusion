#!/bin/bash
# loop-guardian.sh - LoopGuardian: Intelligent anti-deadloop protection
#
# Usage:
#   source scripts/loop-guardian.sh
#   if guardian_init; then
#       guardian_record_iteration "$phase" "$task" "$error"
#       decision=$(guardian_evaluate)
#   fi
#
# Decisions:
#   CONTINUE      - Normal execution, proceed
#   BACKOFF       - Slow down, add delay before next iteration
#   ESCALATE      - Ask user for guidance
#   ABORT_STUCK   - Mark as stuck and stop
#
# IMPORTANT: Requires jq. If jq is not available, GUARDIAN_JQ_AVAILABLE=false
#            and caller should use fallback (simple block count).

set -euo pipefail

FUSION_DIR="${FUSION_DIR:-.fusion}"
LOOP_CONTEXT_FILE="${FUSION_DIR}/loop_context.json"

# Check if jq is available - REQUIRED for LoopGuardian
if command -v jq &>/dev/null; then
    GUARDIAN_JQ_AVAILABLE=true
else
    GUARDIAN_JQ_AVAILABLE=false
fi

# Configuration defaults
GUARDIAN_MAX_ITERATIONS="${GUARDIAN_MAX_ITERATIONS:-50}"
GUARDIAN_MAX_NO_PROGRESS="${GUARDIAN_MAX_NO_PROGRESS:-6}"
GUARDIAN_MAX_SAME_ACTION="${GUARDIAN_MAX_SAME_ACTION:-3}"
GUARDIAN_MAX_SAME_ERROR="${GUARDIAN_MAX_SAME_ERROR:-3}"
GUARDIAN_MAX_STATE_VISITS="${GUARDIAN_MAX_STATE_VISITS:-8}"
GUARDIAN_MAX_WALL_TIME_MS="${GUARDIAN_MAX_WALL_TIME_MS:-7200000}"  # 2 hours
GUARDIAN_BACKOFF_THRESHOLD="${GUARDIAN_BACKOFF_THRESHOLD:-3}"

# Load config from config.yaml if exists
guardian_load_config() {
    local config_file="${FUSION_DIR}/config.yaml"
    [ -f "$config_file" ] || return 0

    # Parse YAML using simple grep - handle comments and whitespace
    # Pattern: extract value, strip comments (#...) and spaces
    # Use [[:space:]] instead of \s for POSIX compatibility
    local val
    val=$(grep -E '^[[:space:]]*max_iterations:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_ITERATIONS="$val"

    val=$(grep -E '^[[:space:]]*max_no_progress:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_NO_PROGRESS="$val"

    val=$(grep -E '^[[:space:]]*max_same_action:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_SAME_ACTION="$val"

    val=$(grep -E '^[[:space:]]*max_same_error:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_SAME_ERROR="$val"

    val=$(grep -E '^[[:space:]]*max_state_visits:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_STATE_VISITS="$val"

    val=$(grep -E '^[[:space:]]*max_wall_time_ms:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_MAX_WALL_TIME_MS="$val"

    val=$(grep -E '^[[:space:]]*backoff_threshold:' "$config_file" 2>/dev/null | head -1 | sed 's/#.*//' | sed 's/.*: *//' | tr -d ' ' || true)
    [ -n "$val" ] && [[ "$val" =~ ^[0-9]+$ ]] && GUARDIAN_BACKOFF_THRESHOLD="$val"

    return 0
}

# Load config on source
guardian_load_config

# Cross-platform milliseconds timestamp
# BSD date doesn't support %N, fallback to seconds * 1000
get_timestamp_ms() {
    local ts
    ts=$(date +%s%3N 2>/dev/null)
    # Validate it's a number (BSD may return literal "%3N")
    if [[ "$ts" =~ ^[0-9]+$ ]]; then
        echo "$ts"
    else
        echo "$(date +%s)000"
    fi
}

# Cross-platform MD5 helper (md5sum on GNU, md5 on macOS)
# Use printf instead of echo to handle inputs like "-n" correctly
compute_md5() {
    local input="$1"
    if command -v md5sum &>/dev/null; then
        printf '%s' "$input" | md5sum | cut -d' ' -f1
    elif command -v md5 &>/dev/null; then
        printf '%s' "$input" | md5 -q
    else
        printf '%s' "$input"  # fallback: return original string
    fi
}

# Initialize loop context (returns 1 if jq not available)
guardian_init() {
    [ "$GUARDIAN_JQ_AVAILABLE" = true ] || return 1

    if [ ! -f "$LOOP_CONTEXT_FILE" ]; then
        local now_ms
        now_ms=$(get_timestamp_ms)

        jq -n \
            --argjson now "$now_ms" \
            '{
                iteration: 0,
                last_task_snapshot: null,
                completed_count_history: [],
                action_signatures: [],
                error_fingerprints: [],
                state_visits: {},
                started_at: $now,
                last_progress_at: $now,
                metrics: {
                    total_iterations: 0,
                    no_progress_rounds: 0,
                    same_action_count: 0,
                    same_error_count: 0,
                    wall_time_ms: 0,
                    max_state_visit_count: 0
                },
                decision_history: []
            }' > "$LOOP_CONTEXT_FILE"
    fi
    return 0
}

# Get value from loop context
guardian_get() {
    local key="$1"
    [ "$GUARDIAN_JQ_AVAILABLE" = true ] && [ -f "$LOOP_CONTEXT_FILE" ] || { echo ""; return; }
    jq -r "$key // empty" "$LOOP_CONTEXT_FILE" 2>/dev/null || echo ""
}

# Compute task snapshot hash (for progress detection)
compute_task_snapshot() {
    if [ -f "$FUSION_DIR/task_plan.md" ]; then
        local completed pending in_progress
        completed=$(grep -c "\[COMPLETED\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || completed=0
        pending=$(grep -c "\[PENDING\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || pending=0
        in_progress=$(grep -c "\[IN_PROGRESS\]" "$FUSION_DIR/task_plan.md" 2>/dev/null) || in_progress=0
        echo "${completed}:${pending}:${in_progress}"
    else
        echo "0:0:0"
    fi
}

# Compute action signature (for same-action detection)
compute_action_signature() {
    local phase="$1"
    local next_task="$2"
    compute_md5 "${phase}:${next_task}"
}

# Record iteration and update metrics
guardian_record_iteration() {
    local phase="${1:-EXECUTE}"
    local next_task="${2:-unknown}"
    local error_msg="${3:-}"

    [ "$GUARDIAN_JQ_AVAILABLE" = true ] || return 1
    [ -f "$LOOP_CONTEXT_FILE" ] || guardian_init || return 1

    local now_ms
    now_ms=$(get_timestamp_ms)

    # Current snapshot
    local snapshot
    snapshot=$(compute_task_snapshot)

    # Action signature
    local action_sig
    action_sig=$(compute_action_signature "$phase" "$next_task")

    # Error fingerprint (if any)
    local error_fp=""
    if [ -n "$error_msg" ]; then
        error_fp=$(compute_md5 "$error_msg")
    fi

    local tmp_file
    tmp_file=$(mktemp "${FUSION_DIR}/.tmp.XXXXXX")

    # Complex update via jq - TRUE tail run-length algorithm
    # Counts consecutive identical elements from the END, stopping at first different
    jq --arg snapshot "$snapshot" \
       --arg action_sig "$action_sig" \
       --arg error_fp "$error_fp" \
       --arg phase "$phase" \
       --argjson now "$now_ms" \
       '
       .iteration += 1 |
       .metrics.total_iterations += 1 |

       # Update wall time
       .metrics.wall_time_ms = ($now - .started_at) |

       # Track completed count for progress detection
       ($snapshot | split(":")[0] | tonumber) as $completed |
       .completed_count_history += [$completed] |
       .completed_count_history = .completed_count_history[-10:] |

       # Check progress (completed count increased?)
       (if (.completed_count_history | length) >= 2 then
           (.completed_count_history[-1] > .completed_count_history[-2])
       else true end) as $made_progress |

       (if $made_progress then
           .metrics.no_progress_rounds = 0 |
           .last_progress_at = $now
       else
           .metrics.no_progress_rounds += 1
       end) |

       # Track action signatures (keep last 5)
       .action_signatures += [$action_sig] |
       .action_signatures = .action_signatures[-5:] |

       # Count TRUE tail consecutive same actions (correct jq syntax)
       (.action_signatures | reverse |
        if length == 0 then 0
        else
            . as $arr | .[0] as $first |
            reduce range(0; $arr | length) as $i (
                {count: 0, done: false};
                if .done then .
                elif ($arr[$i] == $first) then .count += 1
                else .done = true
                end
            ) | .count
        end
       ) as $same_action_count |
       .metrics.same_action_count = $same_action_count |

       # Track error fingerprints - reset count if no error this round
       (if $error_fp != "" then
           # Error provided: add to history and count tail run-length
           (.error_fingerprints + [$error_fp])[-5:] as $new_fps |
           ($new_fps | reverse |
            if length == 0 then 0
            else
                . as $arr | .[0] as $first |
                reduce range(0; $arr | length) as $i (
                    {count: 0, done: false};
                    if .done then .
                    elif ($arr[$i] == $first) then .count += 1
                    else .done = true
                    end
                ) | .count
            end
           ) as $err_count |
           {fps: $new_fps, count: $err_count}
       else
           # No error: clear the consecutive count (but keep history)
           {fps: .error_fingerprints, count: 0}
       end) as $error_result |
       .error_fingerprints = $error_result.fps |
       .metrics.same_error_count = $error_result.count |

       # Track state visits and max
       .state_visits[$phase] = ((.state_visits[$phase] // 0) + 1) |
       (.state_visits | to_entries | map(.value) | max // 0) as $max_visits |
       .metrics.max_state_visit_count = $max_visits |

       # Update snapshot
       .last_task_snapshot = $snapshot
       ' "$LOOP_CONTEXT_FILE" > "$tmp_file" 2>/dev/null && mv "$tmp_file" "$LOOP_CONTEXT_FILE" || { rm -f "$tmp_file"; return 1; }
}

# Evaluate current state and return decision
guardian_evaluate() {
    [ "$GUARDIAN_JQ_AVAILABLE" = true ] || { echo "CONTINUE"; return; }
    [ -f "$LOOP_CONTEXT_FILE" ] || { echo "CONTINUE"; return; }

    # Read metrics from loop context
    local metrics
    metrics=$(jq -r '
        "\(.metrics.total_iterations):\(.metrics.no_progress_rounds):\(.metrics.same_action_count):\(.metrics.same_error_count):\(.metrics.wall_time_ms):\(.metrics.max_state_visit_count)"
    ' "$LOOP_CONTEXT_FILE" 2>/dev/null) || { echo "CONTINUE"; return; }

    local iteration no_progress same_action same_error wall_time max_state_visits
    IFS=':' read -r iteration no_progress same_action same_error wall_time max_state_visits <<< "$metrics"

    # Ensure numeric values
    [[ "$iteration" =~ ^[0-9]+$ ]] || iteration=0
    [[ "$no_progress" =~ ^[0-9]+$ ]] || no_progress=0
    [[ "$same_action" =~ ^[0-9]+$ ]] || same_action=0
    [[ "$same_error" =~ ^[0-9]+$ ]] || same_error=0
    [[ "$wall_time" =~ ^[0-9]+$ ]] || wall_time=0
    [[ "$max_state_visits" =~ ^[0-9]+$ ]] || max_state_visits=0

    local decision="CONTINUE"
    local reason=""

    # Check abort conditions (most severe first)
    if [ "$iteration" -ge "$GUARDIAN_MAX_ITERATIONS" ]; then
        decision="ABORT_STUCK"
        reason="Max iterations ($GUARDIAN_MAX_ITERATIONS) reached"
    elif [ "$wall_time" -ge "$GUARDIAN_MAX_WALL_TIME_MS" ]; then
        decision="ABORT_STUCK"
        reason="Max wall time (${GUARDIAN_MAX_WALL_TIME_MS}ms) exceeded"
    elif [ "$no_progress" -ge "$GUARDIAN_MAX_NO_PROGRESS" ]; then
        decision="ABORT_STUCK"
        reason="No progress for $no_progress rounds (max: $GUARDIAN_MAX_NO_PROGRESS)"
    elif [ "$same_action" -ge "$GUARDIAN_MAX_SAME_ACTION" ]; then
        decision="ESCALATE"
        reason="Same action repeated $same_action times (max: $GUARDIAN_MAX_SAME_ACTION)"
    elif [ "$same_error" -ge "$GUARDIAN_MAX_SAME_ERROR" ]; then
        decision="ESCALATE"
        reason="Same error repeated $same_error times (max: $GUARDIAN_MAX_SAME_ERROR)"
    elif [ "$max_state_visits" -ge "$GUARDIAN_MAX_STATE_VISITS" ]; then
        decision="ESCALATE"
        reason="State visited $max_state_visits times (max: $GUARDIAN_MAX_STATE_VISITS)"
    # Check backoff conditions
    elif [ "$no_progress" -ge "$GUARDIAN_BACKOFF_THRESHOLD" ]; then
        decision="BACKOFF"
        reason="No progress for $no_progress rounds, slowing down"
    elif [ "$same_action" -ge 2 ]; then
        decision="BACKOFF"
        reason="Same action repeated $same_action times, slowing down"
    fi

    # Record decision in history
    if [ -n "$reason" ]; then
        local tmp_file
        tmp_file=$(mktemp "${FUSION_DIR}/.tmp.XXXXXX")
        jq --arg decision "$decision" \
           --arg reason "$reason" \
           '.decision_history += [{decision: $decision, reason: $reason, timestamp: now}] |
            .decision_history = .decision_history[-20:]' \
           "$LOOP_CONTEXT_FILE" > "$tmp_file" 2>/dev/null && mv "$tmp_file" "$LOOP_CONTEXT_FILE" || rm -f "$tmp_file"
    fi

    echo "$decision"
}

# Get human-readable status summary
guardian_status() {
    [ "$GUARDIAN_JQ_AVAILABLE" = true ] || { echo "LoopGuardian: jq not available"; return; }
    [ -f "$LOOP_CONTEXT_FILE" ] || { echo "LoopGuardian: not initialized"; return; }

    jq -r '
        "LoopGuardian Status:",
        "  Iterations: \(.metrics.total_iterations)/\(env.GUARDIAN_MAX_ITERATIONS // 50)",
        "  No-Progress Rounds: \(.metrics.no_progress_rounds)/\(env.GUARDIAN_MAX_NO_PROGRESS // 6)",
        "  Same Action Count: \(.metrics.same_action_count)/\(env.GUARDIAN_MAX_SAME_ACTION // 3)",
        "  Same Error Count: \(.metrics.same_error_count)/\(env.GUARDIAN_MAX_SAME_ERROR // 3)",
        "  Wall Time: \((.metrics.wall_time_ms / 1000 | floor))s"
    ' "$LOOP_CONTEXT_FILE" 2>/dev/null || echo "LoopGuardian: error reading status"
}

# Reset loop context (call when workflow restarts or user intervenes)
guardian_reset() {
    rm -f "$LOOP_CONTEXT_FILE" 2>/dev/null || true
    guardian_init
}
