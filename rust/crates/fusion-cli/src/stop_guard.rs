use anyhow::Result;
use fusion_runtime_io::{
    append_event, json_get_string, json_set_string, load_flat_config, read_json, read_text,
    utc_now_iso, write_json_pretty, write_text,
};
use serde_json::{json, Value};
use std::path::Path;

use crate::models::StopGuardOutput;
use crate::render::{
    live_next_action_fallback, live_next_action_for_phase, next_review_gate_task, read_task_counts,
    stop_guard_backlog_reason, stop_guard_continue_reason, stop_guard_decompose_reason,
    stop_guard_phase_correction_note, stop_guard_review_gate_reason, stop_guard_system_message,
};
use crate::safe_backlog::try_inject_safe_backlog_for_stop_guard;

pub(crate) fn evaluate_stop_guard(fusion_dir: &Path) -> Result<StopGuardOutput> {
    let sessions_path = fusion_dir.join("sessions.json");
    if !sessions_path.is_file() {
        return Ok(allow_output());
    }

    let mut snapshot = read_json(&sessions_path)?;
    let status = json_get_string(&snapshot, &["status"]).unwrap_or_default();
    if status != "in_progress" {
        return Ok(allow_output());
    }

    let cfg = load_flat_config(fusion_dir);
    let runtime_enabled = cfg.runtime_enabled;
    let counts = read_task_counts(fusion_dir)?;
    let total_remaining = counts.pending + counts.in_progress + counts.failed;
    let total = counts.total();
    let current_snap = format!(
        "{}:{}:{}:{}",
        counts.completed, counts.pending, counts.in_progress, counts.failed
    );

    let mut current_phase =
        json_get_string(&snapshot, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());
    let mut phase_corrected = false;
    let mut events_dispatched: Vec<String> = Vec::new();
    let mut correction_events: Vec<(String, String, String, Value, String)> = Vec::new();

    if current_phase == "EXECUTE" && total_remaining == 0 && counts.completed > 0 {
        json_set_string(&mut snapshot, "current_phase", "VERIFY");
        current_phase = "VERIFY".to_string();
        phase_corrected = true;
        events_dispatched.push("ALL_TASKS_DONE".to_string());
        if runtime_enabled {
            correction_events.push((
                "ALL_TASKS_DONE".to_string(),
                "EXECUTE".to_string(),
                "VERIFY".to_string(),
                json!({}),
                format!("stop_guard:all_tasks_done:{current_snap}"),
            ));
        }
    } else if matches!(
        current_phase.as_str(),
        "VERIFY" | "REVIEW" | "COMMIT" | "DELIVER"
    ) && counts.pending > 0
    {
        let previous_phase = current_phase.clone();
        json_set_string(&mut snapshot, "current_phase", "EXECUTE");
        current_phase = "EXECUTE".to_string();
        phase_corrected = true;

        if previous_phase == "VERIFY" {
            events_dispatched.push("VERIFY_FAIL".to_string());
            if runtime_enabled {
                correction_events.push((
                    "VERIFY_FAIL".to_string(),
                    "VERIFY".to_string(),
                    "EXECUTE".to_string(),
                    json!({}),
                    format!("stop_guard:verify_fail:{current_snap}"),
                ));
            }
        } else if previous_phase == "REVIEW" {
            events_dispatched.push("REVIEW_FAIL".to_string());
            if runtime_enabled {
                correction_events.push((
                    "REVIEW_FAIL".to_string(),
                    "REVIEW".to_string(),
                    "EXECUTE".to_string(),
                    json!({}),
                    format!("stop_guard:review_fail:{current_snap}"),
                ));
            }
        } else {
            events_dispatched.push("ERROR_OCCURRED".to_string());
            events_dispatched.push("RECOVER".to_string());
            if runtime_enabled {
                correction_events.push((
                    "ERROR_OCCURRED".to_string(),
                    previous_phase.clone(),
                    "ERROR".to_string(),
                    json!({"error": "pending tasks found"}),
                    format!("stop_guard:error:{current_snap}"),
                ));
                correction_events.push((
                    "RECOVER".to_string(),
                    "ERROR".to_string(),
                    "EXECUTE".to_string(),
                    json!({}),
                    format!("stop_guard:recover:{current_snap}"),
                ));
            }
        }
    }

    if phase_corrected {
        write_json_pretty(&sessions_path, &snapshot)?;
        if runtime_enabled {
            for (event_type, from_state, to_state, payload, key) in correction_events {
                let _ = append_event(
                    fusion_dir,
                    &event_type,
                    &from_state,
                    &to_state,
                    payload,
                    &key,
                );
            }
            if let Ok(updated) = read_json(&sessions_path) {
                snapshot = updated;
            }
        }
    }

    if total_remaining == 0 && counts.completed > 0 {
        if cfg.safe_backlog_enabled && cfg.safe_backlog_inject_on_task_exhausted {
            if let Some(_added) = try_inject_safe_backlog_for_stop_guard(
                fusion_dir,
                &snapshot,
                &cfg,
                counts,
                &current_snap,
                runtime_enabled,
            )? {
                let refreshed_counts = read_task_counts(fusion_dir)?;
                let refreshed_remaining = refreshed_counts.pending
                    + refreshed_counts.in_progress
                    + refreshed_counts.failed;
                let goal = display_goal(&snapshot);
                return Ok(StopGuardOutput {
                    decision: "block".to_string(),
                    should_block: true,
                    reason: stop_guard_backlog_reason(&goal, &current_phase, refreshed_remaining),
                    system_message: format!(
                        "🔄 Fusion (safe_backlog injected) | Phase: {} | Remaining: {}",
                        current_phase, refreshed_remaining
                    ),
                    phase_corrected,
                    events_dispatched,
                });
            }
        }

        if !runtime_enabled {
            json_set_string(&mut snapshot, "status", "completed");
            write_json_pretty(&sessions_path, &snapshot)?;
            let _ = append_progress_completion_entry(fusion_dir);
        }

        return Ok(allow_output_with_meta(phase_corrected, events_dispatched));
    }

    let live_next_action =
        live_next_action_for_phase(fusion_dir, &current_phase, live_next_action_fallback())?;
    if total == 0
        && matches!(
            current_phase.as_str(),
            "INITIALIZE" | "ANALYZE" | "DECOMPOSE"
        )
    {
        let goal = display_goal(&snapshot);
        return Ok(StopGuardOutput {
            decision: "block".to_string(),
            should_block: true,
            reason: stop_guard_decompose_reason(&goal, &live_next_action),
            system_message: stop_guard_system_message(&current_phase, None, &live_next_action),
            phase_corrected,
            events_dispatched,
        });
    }

    let goal = display_goal(&snapshot);
    if let Some(review_gate_task) = next_review_gate_task(fusion_dir)? {
        let mut reason = stop_guard_review_gate_reason(
            &goal,
            &current_phase,
            total_remaining,
            &review_gate_task,
            &live_next_action,
        );
        if phase_corrected {
            reason.push_str(&stop_guard_phase_correction_note(&current_phase));
        }

        return Ok(StopGuardOutput {
            decision: "block".to_string(),
            should_block: true,
            reason,
            system_message: stop_guard_system_message(
                &current_phase,
                Some(total_remaining),
                &live_next_action,
            ),
            phase_corrected,
            events_dispatched,
        });
    }

    let mut reason =
        stop_guard_continue_reason(&goal, &current_phase, total_remaining, &live_next_action);
    if phase_corrected {
        reason.push_str(&stop_guard_phase_correction_note(&current_phase));
    }

    Ok(StopGuardOutput {
        decision: "block".to_string(),
        should_block: true,
        reason,
        system_message: stop_guard_system_message(
            &current_phase,
            Some(total_remaining),
            &live_next_action,
        ),
        phase_corrected,
        events_dispatched,
    })
}

fn allow_output() -> StopGuardOutput {
    allow_output_with_meta(false, vec![])
}

fn allow_output_with_meta(
    phase_corrected: bool,
    events_dispatched: Vec<String>,
) -> StopGuardOutput {
    StopGuardOutput {
        decision: "allow".to_string(),
        should_block: false,
        reason: String::new(),
        system_message: String::new(),
        phase_corrected,
        events_dispatched,
    }
}

fn display_goal(snapshot: &Value) -> String {
    let goal = json_get_string(snapshot, &["goal"]).unwrap_or_else(|| "(not set)".to_string());
    if goal.is_empty() {
        "(not set)".to_string()
    } else {
        goal
    }
}

fn append_progress_completion_entry(fusion_dir: &Path) -> Result<()> {
    let progress_path = fusion_dir.join("progress.md");
    if !progress_path.is_file() {
        return Ok(());
    }

    let existing = read_text(&progress_path).unwrap_or_default();
    let line = format!(
        "| {} | COMPLETE | Workflow finished | OK | All tasks done |",
        utc_now_iso()
    );
    let updated = if existing.is_empty() {
        format!("{line}\n")
    } else if existing.ends_with('\n') {
        format!("{existing}{line}\n")
    } else {
        format!("{existing}\n{line}\n")
    };

    write_text(&progress_path, &updated)
}
