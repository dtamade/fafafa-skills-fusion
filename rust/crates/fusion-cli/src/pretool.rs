use anyhow::Result;
use fusion_runtime_io::{json_get_string, read_json};
use serde_json::Value;
use std::path::Path;

use crate::render::{
    extract_next_task_type, find_next_task, guidance_for_task, next_review_gate_task, phase_num,
    read_guardian_status, read_task_counts, review_gate_guidance, truncate_chars,
};

pub(crate) fn cmd_hook_pretool(fusion_dir: &Path) -> Result<()> {
    let sessions_path = fusion_dir.join("sessions.json");
    if !sessions_path.is_file() {
        return Ok(());
    }

    let snapshot = read_json(&sessions_path)?;
    if json_get_string(&snapshot, &["status"]).as_deref() != Some("in_progress") {
        return Ok(());
    }

    let goal = truncate_chars(
        json_get_string(&snapshot, &["goal"]).unwrap_or_else(|| "?".to_string()),
        60,
    );
    let phase =
        json_get_string(&snapshot, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());
    let phase_num = phase_num(&phase);

    println!("[fusion] Goal: {goal} | Phase: {phase} ({phase_num})");

    let counts = read_task_counts(fusion_dir)?;
    let total = counts.total();
    let next_task = find_next_task(fusion_dir)?;
    let next_task_type = extract_next_task_type(fusion_dir)?;
    let guidance = if let Some(review_gate_task) = next_review_gate_task(fusion_dir)? {
        review_gate_guidance(&review_gate_task.task_id)
    } else {
        guidance_for_task(&phase, &next_task, &next_task_type)
    };
    let guardian_status = read_guardian_status(fusion_dir);

    if total > 0 {
        print_task_progress(
            total,
            counts.completed,
            counts.in_progress,
            &next_task,
            &next_task_type,
            &guardian_status,
        );
    }

    if !guidance.is_empty() {
        println!("[fusion] → {guidance}");
    }

    print_scheduler_summary(&snapshot);
    print_agent_summary(&snapshot);

    Ok(())
}

fn print_task_progress(
    total: i64,
    completed: i64,
    in_progress: i64,
    next_task: &str,
    next_task_type: &str,
    guardian_status: &str,
) {
    let task_index = completed + 1;
    let percent = completed * 100 / total;
    let filled = completed * 10 / total;
    let bar = format!(
        "{}{}",
        "█".repeat(filled as usize),
        "░".repeat((10 - filled) as usize)
    );
    let task_status = if in_progress > 0 {
        "IN_PROGRESS"
    } else {
        "PENDING"
    };
    let type_display = if next_task_type.is_empty() {
        String::new()
    } else {
        format!(" (type: {next_task_type})")
    };

    println!("[fusion] Task {task_index}/{total}: {next_task} [{task_status}]{type_display}");
    println!("[fusion] Progress: {bar} {percent}% | Guardian: {guardian_status}");
}

fn print_scheduler_summary(snapshot: &Value) {
    let sched = snapshot
        .get("_runtime")
        .and_then(|value| value.get("scheduler"));
    if let Some(enabled) = sched
        .and_then(|value| value.get("enabled"))
        .and_then(|value| value.as_bool())
    {
        if enabled {
            let batch_id = sched
                .and_then(|value| value.get("current_batch_id"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let parallel = sched
                .and_then(|value| value.get("parallel_tasks"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            if batch_id > 0 || parallel > 0 {
                println!("[fusion] Batch: {batch_id} | Parallel: {parallel} tasks");
            }
        }
    }
}

fn print_agent_summary(snapshot: &Value) {
    let agents = snapshot
        .get("_runtime")
        .and_then(|value| value.get("agents"));
    if agents
        .and_then(|value| value.get("enabled"))
        .and_then(|value| value.as_bool())
        != Some(true)
    {
        return;
    }

    let batch_id = agents
        .and_then(|value| value.get("current_batch_id"))
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let roles = agents
        .and_then(|value| value.get("active_roles"))
        .and_then(|value| value.as_array())
        .map(|roles| {
            roles
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let review_queue = agents
        .and_then(|value| value.get("review_queue_size"))
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let tasks = agents
        .and_then(|value| value.get("current_batch_tasks"))
        .and_then(|value| value.as_array())
        .map(|tasks| {
            tasks
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    if batch_id > 0 || !roles.is_empty() || review_queue > 0 {
        println!(
            "[fusion] Agent batch: {batch_id} | Roles: {roles} | Review queue: {review_queue}"
        );
    }
    if !tasks.is_empty() {
        println!("[fusion] Agent tasks: {tasks}");
    }

    let collaboration = agents.and_then(|value| value.get("collaboration"));
    if let Some(turn_role) = collaboration
        .and_then(|value| value.get("turn_role"))
        .and_then(|value| value.as_str())
    {
        let turn_task_id = collaboration
            .and_then(|value| value.get("turn_task_id"))
            .and_then(|value| value.as_str())
            .unwrap_or("unknown");
        let turn_kind = collaboration
            .and_then(|value| value.get("turn_kind"))
            .and_then(|value| value.as_str())
            .unwrap_or("task");
        println!("[fusion] Agent turn: {turn_role} -> {turn_task_id} ({turn_kind})");
    }
    let pending_reviews = collaboration
        .and_then(|value| value.get("pending_reviews"))
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    if !pending_reviews.is_empty() {
        println!("[fusion] Pending reviews: {pending_reviews}");
    }
}
