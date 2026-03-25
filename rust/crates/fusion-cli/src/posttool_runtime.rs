use anyhow::Result;
use fusion_runtime_io::{
    append_event, json_get_string, json_set_string, read_json, write_json_pretty,
};
use serde_json::{json, Value};
use std::path::Path;

use crate::models::TaskCounts;

pub(crate) fn dispatch_runtime_posttool_events(
    fusion_dir: &Path,
    snapshot: &mut Value,
    current_snap: &str,
    counts: TaskCounts,
    completed_delta: i64,
) -> Result<()> {
    if completed_delta <= 0 {
        return Ok(());
    }

    let current_phase =
        json_get_string(snapshot, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());
    if current_phase != "EXECUTE" {
        return Ok(());
    }

    let total_remaining = counts.pending + counts.in_progress + counts.failed;
    let payload = json!({
        "pending_tasks": total_remaining,
        "completed_tasks": counts.completed,
        "failed_tasks": counts.failed,
    });
    let sessions_path = fusion_dir.join("sessions.json");

    if total_remaining == 0 && counts.completed > 0 {
        json_set_string(snapshot, "current_phase", "VERIFY");
        write_json_pretty(&sessions_path, snapshot)?;
        let _ = append_event(
            fusion_dir,
            "ALL_TASKS_DONE",
            "EXECUTE",
            "VERIFY",
            payload,
            &format!("posttool:all_tasks_done:{current_snap}"),
        );
    } else if total_remaining > 0 {
        let _ = append_event(
            fusion_dir,
            "TASK_DONE",
            "EXECUTE",
            "EXECUTE",
            payload,
            &format!("posttool:task_done:{current_snap}:{}", counts.completed),
        );
    }

    if let Ok(updated) = read_json(&sessions_path) {
        *snapshot = updated;
    }

    Ok(())
}
