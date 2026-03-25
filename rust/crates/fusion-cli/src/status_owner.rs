use anyhow::Result;
use fusion_runtime_io::read_text;
use std::path::Path;

use crate::achievements::parse_task_status_line;

#[derive(Debug, Clone, Default)]
pub(crate) struct OwnerMetrics {
    pub(crate) planner: i64,
    pub(crate) coder: i64,
    pub(crate) reviewer: i64,
    pub(crate) current_role: String,
    pub(crate) current_task: String,
    pub(crate) current_status: String,
}

fn parse_task_field(line: &str, names: &[&str]) -> Option<String> {
    let trimmed = line.trim_start();
    for name in names {
        let prefix = format!("- {name}:");
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let value = rest.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn normalize_owner(owner: &str) -> Option<&'static str> {
    match owner.trim().to_ascii_lowercase().as_str() {
        "planner" | "plan" | "planning" => Some("planner"),
        "coder" | "code" | "coding" | "developer" | "dev" | "implementer" => Some("coder"),
        "reviewer" | "review" | "qa" | "verifier" | "verification" => Some("reviewer"),
        _ => None,
    }
}

fn infer_owner(task_type: &str) -> &'static str {
    match task_type.trim().to_ascii_lowercase().as_str() {
        "verification" => "reviewer",
        "design" | "research" => "planner",
        _ => "coder",
    }
}

pub(crate) fn collect_owner_metrics(fusion_dir: &Path) -> Result<OwnerMetrics> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok(OwnerMetrics::default());
    }

    let content = read_text(&task_plan)?;
    let mut planner = 0;
    let mut coder = 0;
    let mut reviewer = 0;
    let mut current_role = String::new();
    let mut current_task = String::new();
    let mut current_status = String::new();
    let mut pending_role = String::new();
    let mut pending_task = String::new();
    let mut pending_status = String::new();

    let mut task_title = String::new();
    let mut task_status = String::new();
    let mut task_type = String::new();
    let mut task_owner = String::new();

    let flush_task = |planner: &mut i64,
                      coder: &mut i64,
                      reviewer: &mut i64,
                      current_role: &mut String,
                      current_task: &mut String,
                      current_status: &mut String,
                      pending_role: &mut String,
                      pending_task: &mut String,
                      pending_status: &mut String,
                      task_title: &mut String,
                      task_status: &mut String,
                      task_type: &mut String,
                      task_owner: &mut String| {
        if task_title.is_empty() {
            return;
        }

        let owner = normalize_owner(task_owner).unwrap_or_else(|| infer_owner(task_type));
        match owner {
            "planner" => *planner += 1,
            "reviewer" => *reviewer += 1,
            _ => *coder += 1,
        }

        if current_role.is_empty() && task_status.as_str() == "IN_PROGRESS" {
            *current_role = owner.to_string();
            *current_task = task_title.clone();
            *current_status = task_status.clone();
        }
        if pending_role.is_empty() && task_status.as_str() == "PENDING" {
            *pending_role = owner.to_string();
            *pending_task = task_title.clone();
            *pending_status = task_status.clone();
        }

        task_title.clear();
        task_status.clear();
        task_type.clear();
        task_owner.clear();
    };

    for line in content.lines() {
        if let Some((title, status)) = parse_task_status_line(line) {
            flush_task(
                &mut planner,
                &mut coder,
                &mut reviewer,
                &mut current_role,
                &mut current_task,
                &mut current_status,
                &mut pending_role,
                &mut pending_task,
                &mut pending_status,
                &mut task_title,
                &mut task_status,
                &mut task_type,
                &mut task_owner,
            );
            task_title = title;
            task_status = status;
            continue;
        }

        if task_title.is_empty() {
            continue;
        }

        if let Some(value) = parse_task_field(line, &["Type", "type"]) {
            task_type = value;
            continue;
        }
        if let Some(value) = parse_task_field(line, &["Owner", "owner", "Role", "role"]) {
            task_owner = value;
        }
    }

    flush_task(
        &mut planner,
        &mut coder,
        &mut reviewer,
        &mut current_role,
        &mut current_task,
        &mut current_status,
        &mut pending_role,
        &mut pending_task,
        &mut pending_status,
        &mut task_title,
        &mut task_status,
        &mut task_type,
        &mut task_owner,
    );

    if current_role.is_empty() {
        current_role = pending_role;
        current_task = pending_task;
        current_status = pending_status;
    }

    Ok(OwnerMetrics {
        planner,
        coder,
        reviewer,
        current_role,
        current_task,
        current_status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_collect_owner_metrics_infers_roles_and_current_task() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("task_plan.md"),
            "### Task 1: 方案设计 [COMPLETED]\n- Type: design\n### Task 2: 编码实现 [IN_PROGRESS]\n- Type: implementation\n### Task 3: 回归验证 [PENDING]\n- Type: verification\n",
        )
        .expect("write task plan");

        let metrics = collect_owner_metrics(dir.path()).expect("owner metrics");
        assert_eq!(metrics.planner, 1);
        assert_eq!(metrics.coder, 1);
        assert_eq!(metrics.reviewer, 1);
        assert_eq!(metrics.current_role, "coder");
        assert_eq!(metrics.current_task, "编码实现");
        assert_eq!(metrics.current_status, "IN_PROGRESS");
    }
}
