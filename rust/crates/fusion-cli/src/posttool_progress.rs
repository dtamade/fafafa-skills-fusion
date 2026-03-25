use anyhow::Result;
use fusion_runtime_io::{append_event, json_get_string, read_text, write_text};
use serde_json::Value;
use std::path::Path;

use crate::models::TaskCounts;
use crate::render::{
    continue_task_next_action, execution_mode_for_task_type, extract_next_task_metadata,
    find_first_task_with_status, find_last_task_with_status, next_review_gate_task,
    posttool_next_action_line, posttool_next_action_with_mode, proceed_to_verify_next_action,
    review_gate_next_action,
};
use crate::safe_backlog::{try_inject_safe_backlog, SafeBacklogTrigger};
use crate::supervisor::generate_supervisor_advice;

pub(crate) fn read_previous_snapshot(snap_file: &Path) -> String {
    if snap_file.is_file() {
        read_text(snap_file).unwrap_or_default().trim().to_string()
    } else {
        String::new()
    }
}

pub(crate) fn bump_unchanged_count(unchanged_file: &Path) -> i64 {
    let mut unchanged = if unchanged_file.is_file() {
        read_text(unchanged_file)
            .ok()
            .and_then(|text| text.trim().parse::<i64>().ok())
            .unwrap_or(0)
    } else {
        0
    };
    unchanged += 1;
    let _ = write_text(unchanged_file, &unchanged.to_string());
    unchanged
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_no_progress_lines(
    fusion_dir: &Path,
    snapshot: &Value,
    cfg: &fusion_runtime_io::FlatConfig,
    counts: TaskCounts,
    total: i64,
    pending_like: i64,
    current_snap: &str,
    snap_file: &Path,
    unchanged: i64,
    runtime_enabled: bool,
    trigger_rounds: i64,
) -> Result<Vec<String>> {
    let mut advisory_lines: Vec<String> = Vec::new();

    if runtime_enabled {
        if let Some(advice) =
            generate_supervisor_advice(fusion_dir, cfg, unchanged, counts, pending_like)?
        {
            advisory_lines.push(advice.line.clone());
            let current_phase = json_get_string(snapshot, &["current_phase"])
                .unwrap_or_else(|| "EXECUTE".to_string());
            let key = format!(
                "supervisor:{current_snap}:{unchanged}:{}",
                (advice.risk_score.clamp(0.0, 1.0) * 1000.0).round() as i64
            );
            let _ = append_event(
                fusion_dir,
                "SUPERVISOR_ADVISORY",
                &current_phase,
                &current_phase,
                advice.payload,
                &key,
            );
        }
    }

    if runtime_enabled && cfg.safe_backlog_enabled && unchanged >= trigger_rounds {
        if let Some(lines) = try_inject_safe_backlog(
            fusion_dir,
            snapshot,
            cfg,
            SafeBacklogTrigger {
                counts: &counts,
                pending_like,
                current_snap,
                reason: "no_progress",
                no_progress_rounds: unchanged,
                snap_file,
            },
        )? {
            advisory_lines.extend(lines);
        }
    }

    if unchanged >= 5 && total > 0 {
        advisory_lines.push(format!(
            "[fusion] Info: {unchanged} file edits since last task status change."
        ));
        let current_task = find_first_task_with_status(fusion_dir, "[IN_PROGRESS]")?;
        if !current_task.is_empty() {
            advisory_lines.push(format!(
                "[fusion] Current: {current_task} [IN_PROGRESS] | When done, mark [COMPLETED] in task_plan.md"
            ));
        }
    }

    Ok(advisory_lines)
}

pub(crate) fn parse_previous_progress(prev_snap: &str) -> (i64, i64) {
    let prev_parts: Vec<&str> = if prev_snap.is_empty() {
        vec!["0", "0", "0", "0"]
    } else {
        prev_snap.split(':').collect()
    };

    let prev_completed = prev_parts
        .first()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    let prev_failed = prev_parts
        .get(3)
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);

    (prev_completed, prev_failed)
}

pub(crate) fn build_progress_delta_lines(
    fusion_dir: &Path,
    snapshot: &Value,
    counts: TaskCounts,
    total: i64,
    completed_delta: i64,
    failed_delta: i64,
    runtime_enabled: bool,
) -> Result<Vec<String>> {
    let mut lines: Vec<String> = Vec::new();

    if completed_delta > 0 {
        let just_completed = find_last_task_with_status(fusion_dir, "[COMPLETED]")?;
        lines.push(format!(
            "[fusion] Task {} → COMPLETED ({}/{total} done)",
            if just_completed.is_empty() {
                "?"
            } else {
                &just_completed
            },
            counts.completed
        ));
        if counts.pending_like() > 0 {
            if let Some(review_gate_task) = next_review_gate_task(fusion_dir)? {
                lines.push(posttool_next_action_line(&review_gate_next_action(
                    &review_gate_task.task_id,
                )));
            } else if let Some(next_task) = extract_next_task_metadata(fusion_dir)? {
                let guidance = execution_mode_for_task_type(&next_task.task_type);
                lines.push(posttool_next_action_with_mode(
                    &continue_task_next_action(&next_task),
                    guidance,
                ));
            }
        } else {
            lines.push(posttool_next_action_line(proceed_to_verify_next_action()));
        }
    }

    if failed_delta > 0 {
        let just_failed = find_last_task_with_status(fusion_dir, "[FAILED]")?;
        lines.push(format!(
            "[fusion] Task {} → FAILED. Apply 3-Strike protocol.",
            if just_failed.is_empty() {
                "?"
            } else {
                &just_failed
            }
        ));
    }

    let sched = snapshot
        .get("_runtime")
        .and_then(|value| value.get("scheduler"));
    if runtime_enabled
        && sched
            .and_then(|value| value.get("enabled"))
            .and_then(|value| value.as_bool())
            == Some(true)
    {
        let batch_id = sched
            .and_then(|value| value.get("current_batch_id"))
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        if batch_id > 0 && completed_delta > 0 {
            lines.push(format!(
                "[fusion] Batch {batch_id} progress: +{completed_delta} tasks completed"
            ));
        }
    }

    Ok(lines)
}

pub(crate) fn counts_snapshot(counts: TaskCounts) -> String {
    format!(
        "{}:{}:{}:{}",
        counts.completed, counts.pending, counts.in_progress, counts.failed
    )
}

pub(crate) fn print_lines(lines: &[String]) {
    for line in lines {
        println!("{line}");
    }
}
