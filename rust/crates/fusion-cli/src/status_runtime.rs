use anyhow::Result;
use fusion_runtime_io::{json_get_bool, json_get_string, read_json, read_text};
use std::collections::BTreeMap;
use std::path::Path;

use crate::render::{epoch_to_iso, last_safe_backlog};
use crate::status::{collect_hook_debug_summary, render_understand_handoff, StatusSummary};

fn format_reason_map(entries: &BTreeMap<String, String>) -> String {
    entries
        .iter()
        .map(|(task_id, reason)| format!("{task_id}={reason}"))
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn print_runtime(fusion_dir: &Path, summary: &StatusSummary) -> Result<()> {
    let sessions_path = fusion_dir.join("sessions.json");
    if !sessions_path.is_file() {
        return Ok(());
    }

    println!("## Active Sessions");
    let sessions_text = read_text(&sessions_path)?;
    for line in sessions_text.lines().take(5) {
        println!("{line}");
    }

    println!();
    println!("## Runtime");
    let sessions = read_json(&sessions_path)?;
    if let Some(status) = json_get_string(&sessions, &["status"]) {
        println!("status: {status}");
    }
    println!("runtime.enabled: {}", summary.runtime_enabled);
    println!("runtime.compat_mode: {}", summary.runtime_compat_mode);
    println!("runtime.engine: {}", summary.runtime_engine);
    if let Some(phase) = json_get_string(&sessions, &["current_phase"]) {
        println!("phase: {phase}");
    }
    if let Some(handoff) = render_understand_handoff(
        summary.understand_mode.as_deref(),
        summary.understand_forced,
        summary.understand_decision.as_deref(),
    ) {
        println!("understand: {handoff}");
    }
    if let Some(last_event_id) = json_get_string(&sessions, &["_runtime", "last_event_id"]) {
        println!("last_event_id: {last_event_id}");
    }
    if let Some(counter) = sessions
        .get("_runtime")
        .and_then(|v| v.get("last_event_counter"))
        .and_then(|v| v.as_i64())
    {
        println!("event_counter: {counter}");
    }
    if let Some(enabled) = json_get_bool(&sessions, &["_runtime", "scheduler", "enabled"]) {
        println!("scheduler.enabled: {enabled}");
        if let Some(batch_id) = sessions
            .get("_runtime")
            .and_then(|v| v.get("scheduler"))
            .and_then(|v| v.get("current_batch_id"))
            .and_then(|v| v.as_i64())
        {
            println!("scheduler.batch_id: {batch_id}");
        }
        if let Some(parallel_tasks) = sessions
            .get("_runtime")
            .and_then(|v| v.get("scheduler"))
            .and_then(|v| v.get("parallel_tasks"))
            .and_then(|v| v.as_i64())
        {
            println!("scheduler.parallel_tasks: {parallel_tasks}");
        }
    }
    if summary.agents_enabled {
        println!("agents.enabled: {}", summary.agents_enabled);
        if let Some(mode) = summary.agent_mode.as_deref() {
            println!("agents.mode: {mode}");
        }
        if let Some(explain_level) = summary.agent_explain_level.as_deref() {
            println!("agents.explain_level: {explain_level}");
        }
        if let Some(batch_id) = summary.agent_current_batch_id {
            println!("agents.batch_id: {batch_id}");
        }
        if !summary.agent_active_roles.is_empty() {
            println!(
                "agents.active_roles: {}",
                summary.agent_active_roles.join(", ")
            );
        }
        if !summary.agent_current_batch_tasks.is_empty() {
            println!(
                "agents.current_batch_tasks: {}",
                summary.agent_current_batch_tasks.join(", ")
            );
        }
        if let Some(review_queue_size) = summary.agent_review_queue_size {
            println!("agents.review_queue_size: {review_queue_size}");
        }
        if !summary.agent_review_queue.is_empty() {
            println!(
                "agents.review_queue: {}",
                summary.agent_review_queue.join(", ")
            );
        }
        if let Some(batch_reason) = summary.agent_batch_reason.as_deref() {
            println!("agents.policy.batch_reason: {batch_reason}");
        }
        if let Some(collaboration_mode) = summary.agent_collaboration_mode.as_deref() {
            println!("agents.collaboration_mode: {collaboration_mode}");
        }
        if let Some(turn_role) = summary.agent_turn_role.as_deref() {
            let turn_task_id = summary.agent_turn_task_id.as_deref().unwrap_or("unknown");
            let turn_kind = summary.agent_turn_kind.as_deref().unwrap_or("task");
            println!("agents.turn: {turn_role} -> {turn_task_id} ({turn_kind})");
        }
        if !summary.agent_pending_reviews.is_empty() {
            println!(
                "agents.pending_reviews: {}",
                summary.agent_pending_reviews.join(", ")
            );
        }
        if let Some(blocked_reason) = summary.agent_blocked_handoff_reason.as_deref() {
            if !blocked_reason.is_empty() {
                println!("agents.blocked_handoff_reason: {blocked_reason}");
            }
        }
        if !summary.agent_selected_reasons.is_empty() {
            println!(
                "agents.policy.selected: {}",
                format_reason_map(&summary.agent_selected_reasons)
            );
        }
        if !summary.agent_blocked_reasons.is_empty() {
            println!(
                "agents.policy.blocked: {}",
                format_reason_map(&summary.agent_blocked_reasons)
            );
        }
        if !summary.agent_review_reasons.is_empty() {
            println!(
                "agents.policy.review: {}",
                format_reason_map(&summary.agent_review_reasons)
            );
        }
    }

    let events = fusion_dir.join("events.jsonl");
    if events.is_file() {
        if let Some((added, timestamp)) = last_safe_backlog(&events)? {
            println!("safe_backlog.last_added: {added}");
            println!("safe_backlog.last_injected_at: {timestamp}");
            if let Some(iso) = epoch_to_iso(timestamp) {
                println!("safe_backlog.last_injected_at_iso: {iso}");
            }
        }
    }

    let hook_debug = collect_hook_debug_summary(fusion_dir)?;
    println!();
    println!("## Hook Debug");
    println!("hook_debug.enabled: {}", hook_debug.enabled);
    if !hook_debug.flag_path.is_empty() {
        println!("hook_debug.flag: {}", hook_debug.flag_path);
    }
    if !hook_debug.log_path.is_empty() {
        println!("hook_debug.log: {}", hook_debug.log_path);
        println!("hook_debug.tail:");
        for line in hook_debug.log_tail {
            println!("  {line}");
        }
    } else {
        println!("hook_debug.log: (none yet)");
    }

    Ok(())
}
