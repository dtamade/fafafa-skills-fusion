use anyhow::Result;
use fusion_runtime_io::{json_get_string, load_flat_config};
use serde_json::Value;
use std::path::Path;

use crate::render::{
    extract_all_task_metadata, task_counts_from_metadata, task_has_pending_review,
};

#[derive(Debug, Clone)]
pub(crate) struct TaskItem {
    pub(crate) task_id: String,
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) owner: String,
    pub(crate) status: String,
    pub(crate) review_status: String,
}

#[derive(Debug, Default)]
pub(crate) struct TaskPlanInfo {
    pub(crate) total: usize,
    pub(crate) completed: usize,
    pub(crate) pending: usize,
    pub(crate) in_progress: usize,
    pub(crate) failed: usize,
    pub(crate) tasks: Vec<TaskItem>,
}

pub(crate) fn read_task_plan(fusion_dir: &Path) -> Result<TaskPlanInfo> {
    let tasks = extract_all_task_metadata(fusion_dir)?;
    if tasks.is_empty() {
        return Ok(TaskPlanInfo::default());
    }

    let cfg = load_flat_config(fusion_dir);
    let counts = task_counts_from_metadata(&tasks, &cfg.agent_review_policy);
    Ok(TaskPlanInfo {
        total: counts.total() as usize,
        completed: counts.completed as usize,
        pending: counts.pending as usize,
        in_progress: counts.in_progress as usize,
        failed: counts.failed as usize,
        tasks: tasks
            .into_iter()
            .map(|task| TaskItem {
                task_id: task.task_id.clone(),
                name: task.title.clone(),
                owner: task.owner.clone(),
                status: if task_has_pending_review(&task, &cfg.agent_review_policy) {
                    "REVIEW_PENDING".to_string()
                } else {
                    task.status.to_ascii_uppercase()
                },
                review_status: task.review_status.clone(),
            })
            .collect(),
    })
}

pub(crate) fn cross_validate(
    task_info: &TaskPlanInfo,
    session_info: &Value,
    git_diff: &str,
) -> Vec<String> {
    let mut warnings = Vec::new();
    let phase = json_get_string(session_info, &["current_phase"]).unwrap_or_default();

    if phase == "EXECUTE"
        && task_info.pending == 0
        && task_info.in_progress == 0
        && task_info.completed > 0
    {
        warnings.push(
            "Phase mismatch: sessions.json says EXECUTE but all tasks completed. Should be VERIFY."
                .to_string(),
        );
    }

    if !git_diff.is_empty() && task_info.in_progress == 0 && task_info.pending > 0 {
        warnings.push(
            "Git has uncommitted changes but no task is IN_PROGRESS. A task may have been worked on without status update."
                .to_string(),
        );
    }

    if task_info.in_progress > 0 && task_info.completed == 0 && task_info.total > 3 {
        warnings.push(
            "Task marked IN_PROGRESS but no tasks completed yet. May be stuck on first task."
                .to_string(),
        );
    }

    warnings
}
