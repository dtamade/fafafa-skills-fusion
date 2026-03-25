use crate::models::TaskCounts;
use anyhow::Result;
use fusion_runtime_io::{read_text, write_text};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActiveTaskMetadata {
    pub(crate) task_id: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) task_type: String,
    pub(crate) owner: String,
    pub(crate) risk: String,
    pub(crate) review: String,
    pub(crate) review_status: String,
    pub(crate) writes: String,
    pub(crate) dependencies: String,
}

fn owner_for_task_type(task_type_raw: &str) -> &'static str {
    match task_type_raw.trim().to_ascii_lowercase().as_str() {
        "verification" => "reviewer",
        "design" | "research" => "planner",
        "implementation" | "documentation" | "configuration" => "coder",
        _ => "coder",
    }
}

fn is_task_header(line: &str) -> bool {
    line.starts_with("### Task ")
        && ["[PENDING]", "[IN_PROGRESS]", "[COMPLETED]", "[FAILED]"]
            .iter()
            .any(|tag| line.contains(tag))
}

fn is_type_line(line: &str) -> bool {
    line.trim_start().starts_with("- Type:")
}

fn is_owner_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- Owner:") || trimmed.starts_with("- Role:")
}

fn is_risk_line(line: &str) -> bool {
    line.trim_start().starts_with("- Risk:")
}

fn is_review_line(line: &str) -> bool {
    line.trim_start().starts_with("- Review:")
}

fn is_writes_line(line: &str) -> bool {
    line.trim_start().starts_with("- Writes:")
}

fn is_review_status_line(line: &str) -> bool {
    line.trim_start().starts_with("- Review-Status:")
}

fn is_dependencies_line(line: &str) -> bool {
    line.trim_start().starts_with("- Dependencies:")
}

fn normalize_review_status(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" | "pending" | "approved" | "changes_requested" => value.trim().to_ascii_lowercase(),
        _ => "none".to_string(),
    }
}

fn parse_field_raw(line: &str, labels: &[&str]) -> Option<String> {
    let trimmed = line.trim_start();
    for label in labels {
        let prefix = format!("- {label}:");
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let value = rest.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn parse_field(line: &str, labels: &[&str]) -> Option<String> {
    parse_field_raw(line, labels).map(|rest| {
        rest.chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>()
            .to_ascii_lowercase()
    })
}

fn parse_task_header(line: &str) -> Option<(String, String, String)> {
    if !is_task_header(line) {
        return None;
    }

    let rest = line.strip_prefix("### Task ")?;
    let task_number: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if task_number.is_empty() {
        return None;
    }

    let status = ["PENDING", "IN_PROGRESS", "COMPLETED", "FAILED"]
        .iter()
        .find_map(|status| {
            line.contains(&format!("[{status}]"))
                .then(|| status.to_ascii_lowercase())
        })?;
    let title = rest
        .split_once(':')
        .map(|(_, suffix)| suffix)
        .unwrap_or(rest)
        .split('[')
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();

    Some((format!("task_{task_number}"), title, status))
}

fn parse_task_block(block: &[String]) -> Option<ActiveTaskMetadata> {
    let header = block.first()?;
    let (task_id, title, status) = parse_task_header(header)?;
    let task_type = block
        .iter()
        .find_map(|line| parse_field(line, &["Type"]))
        .unwrap_or_else(|| "implementation".to_string());
    let owner = block
        .iter()
        .find_map(|line| parse_field(line, &["Owner", "Role"]))
        .unwrap_or_else(|| owner_for_task_type(&task_type).to_string());
    let risk = block
        .iter()
        .find_map(|line| parse_field(line, &["Risk"]))
        .unwrap_or_else(|| "low".to_string());
    let review = block
        .iter()
        .find_map(|line| parse_field(line, &["Review"]))
        .unwrap_or_else(|| "auto".to_string());
    let review_status = block
        .iter()
        .find_map(|line| parse_field(line, &["Review-Status"]))
        .map(|value| normalize_review_status(&value))
        .unwrap_or_else(|| "none".to_string());
    let writes = block
        .iter()
        .find_map(|line| parse_field_raw(line, &["Writes"]))
        .unwrap_or_else(|| "[]".to_string());
    let dependencies = block
        .iter()
        .find_map(|line| parse_field_raw(line, &["Dependencies"]))
        .unwrap_or_else(|| "[]".to_string());

    Some(ActiveTaskMetadata {
        task_id,
        title,
        status,
        task_type,
        owner,
        risk,
        review,
        review_status,
        writes,
        dependencies,
    })
}

fn extract_next_task_metadata_from_content(content: &str) -> Option<ActiveTaskMetadata> {
    let mut block: Vec<String> = Vec::new();

    for line in content.lines() {
        if is_task_header(line) {
            if !block.is_empty() {
                break;
            }
            if line.contains("[IN_PROGRESS]") || line.contains("[PENDING]") {
                block.push(line.to_string());
            }
            continue;
        }

        if !block.is_empty() {
            block.push(line.to_string());
        }
    }

    parse_task_block(&block)
}

fn extract_all_task_metadata_from_content(content: &str) -> Vec<ActiveTaskMetadata> {
    let mut tasks = Vec::new();
    let mut block: Vec<String> = Vec::new();

    for line in content.lines() {
        if is_task_header(line) {
            if let Some(task) = parse_task_block(&block) {
                tasks.push(task);
            }
            block.clear();
            block.push(line.to_string());
            continue;
        }

        if !block.is_empty() {
            block.push(line.to_string());
        }
    }

    if let Some(task) = parse_task_block(&block) {
        tasks.push(task);
    }

    tasks
}

fn flush_task_block(block: &mut Vec<String>, output: &mut Vec<String>) {
    if block.is_empty() {
        return;
    }

    let has_owner = block.iter().any(|line| is_owner_line(line));
    let has_risk = block.iter().any(|line| is_risk_line(line));
    let has_review = block.iter().any(|line| is_review_line(line));
    let has_review_status = block.iter().any(|line| is_review_status_line(line));
    let has_writes = block.iter().any(|line| is_writes_line(line));
    let has_dependencies = block.iter().any(|line| is_dependencies_line(line));
    let type_idx = block.iter().position(|line| is_type_line(line));
    let task_type = type_idx
        .and_then(|idx| parse_field(&block[idx], &["Type"]))
        .unwrap_or_else(|| "implementation".to_string());
    let mut injected_lines = Vec::new();
    if !has_owner {
        injected_lines.push(format!("- Owner: {}", owner_for_task_type(&task_type)));
    }
    if !has_risk {
        injected_lines.push("- Risk: low".to_string());
    }
    if !has_review {
        injected_lines.push("- Review: auto".to_string());
    }
    if !has_review_status {
        injected_lines.push("- Review-Status: none".to_string());
    }
    if !has_writes {
        injected_lines.push("- Writes: []".to_string());
    }
    if !has_dependencies {
        injected_lines.push("- Dependencies: []".to_string());
    }

    let mut injected = false;
    for (idx, line) in block.iter().enumerate() {
        output.push(line.clone());
        if type_idx == Some(idx) && !injected_lines.is_empty() {
            for injected_line in &injected_lines {
                output.push(injected_line.clone());
            }
            injected = true;
        }
    }

    if !injected && !injected_lines.is_empty() {
        output.extend(injected_lines);
    }

    block.clear();
}

pub(crate) fn normalize_task_plan_owners(fusion_dir: &Path) -> Result<()> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok(());
    }

    let original = read_text(&task_plan)?;
    let mut output: Vec<String> = Vec::new();
    let mut block: Vec<String> = Vec::new();

    for line in original.lines() {
        if is_task_header(line) {
            flush_task_block(&mut block, &mut output);
            block.push(line.to_string());
            continue;
        }

        if block.is_empty() {
            output.push(line.to_string());
        } else {
            block.push(line.to_string());
        }
    }
    flush_task_block(&mut block, &mut output);

    let mut normalized = output.join("\n");
    if original.ends_with('\n') {
        normalized.push('\n');
    }

    if normalized != original {
        write_text(&task_plan, &normalized)?;
    }

    Ok(())
}

pub(crate) fn extract_next_task_metadata(fusion_dir: &Path) -> Result<Option<ActiveTaskMetadata>> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok(None);
    }
    Ok(extract_next_task_metadata_from_content(&read_text(
        &task_plan,
    )?))
}

pub(crate) fn extract_all_task_metadata(fusion_dir: &Path) -> Result<Vec<ActiveTaskMetadata>> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok(Vec::new());
    }
    Ok(extract_all_task_metadata_from_content(&read_text(
        &task_plan,
    )?))
}

pub(crate) fn extract_task_metadata_by_id(
    fusion_dir: &Path,
    task_id: &str,
) -> Result<Option<ActiveTaskMetadata>> {
    Ok(extract_all_task_metadata(fusion_dir)?
        .into_iter()
        .find(|task| task.task_id == task_id))
}

pub(crate) fn task_needs_review(task: &ActiveTaskMetadata, review_policy: &str) -> bool {
    match review_policy {
        "always" => true,
        "never" => false,
        _ => {
            matches!(task.risk.as_str(), "high" | "critical")
                || matches!(task.review.as_str(), "required" | "human")
        }
    }
}

pub(crate) fn task_has_pending_review(task: &ActiveTaskMetadata, review_policy: &str) -> bool {
    task_needs_review(task, review_policy) && task.review_status == "pending"
}

pub(crate) fn task_is_effectively_completed(
    task: &ActiveTaskMetadata,
    review_policy: &str,
) -> bool {
    task.status == "completed"
        && (!task_needs_review(task, review_policy) || task.review_status == "approved")
}

pub(crate) fn task_counts_from_metadata(
    tasks: &[ActiveTaskMetadata],
    review_policy: &str,
) -> TaskCounts {
    let mut counts = TaskCounts::default();
    for task in tasks {
        if task.review_status == "changes_requested" && task.status != "pending" {
            counts.pending += 1;
            continue;
        }

        match task.status.as_str() {
            "completed" if task_is_effectively_completed(task, review_policy) => {
                counts.completed += 1
            }
            "completed" => counts.in_progress += 1,
            "pending" => counts.pending += 1,
            "in_progress" => counts.in_progress += 1,
            "failed" => counts.failed += 1,
            _ => counts.pending += 1,
        }
    }
    counts
}

pub(crate) fn extract_next_task_type(fusion_dir: &Path) -> Result<String> {
    Ok(extract_next_task_metadata(fusion_dir)?
        .map(|task| task.task_type)
        .unwrap_or_default())
}

pub(crate) fn render_prompt(
    fusion_dir: &Path,
    phase: &str,
    goal: &str,
    role: &str,
) -> Result<String> {
    let task_plan_path = fusion_dir.join("task_plan.md");
    let task_plan = if task_plan_path.is_file() {
        read_text(&task_plan_path)?
    } else {
        String::new()
    };

    let role_mandate = match role {
        "planner" => "Focus on planning/decomposition, priorities, and execution handoff.",
        "reviewer" => "Focus on review quality, risks, regressions, and acceptance criteria.",
        "coder" => "Focus on implementation with tests, task completion, and progress updates.",
        _ => "Continue current workflow tasks and keep plan/progress in sync.",
    };

    Ok(format!(
        "[Fusion Runner]\nRole: {role}\nPhase: {phase}\nGoal: {goal}\n\nRole mandate:\n{role_mandate}\n\n请在当前仓库执行下一步工作，并更新：\n1) .fusion/task_plan.md\n2) .fusion/progress.md\n\n当前 task_plan 内容：\n{task_plan}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_render_prompt_contains_task_plan() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(dir.path().join("task_plan.md"), "### Task 1: A [PENDING]\n")
            .expect("write task plan");

        let prompt =
            render_prompt(dir.path(), "EXECUTE", "my goal", "coder").expect("render prompt");
        assert!(prompt.contains("Role: coder"));
        assert!(prompt.contains("Phase: EXECUTE"));
        assert!(prompt.contains("my goal"));
        assert!(prompt.contains("Task 1"));
    }

    #[test]
    fn test_normalize_task_plan_owners_injects_owner_after_type() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("task_plan.md"),
            "### Task 1: Explore [PENDING]\n- Type: research\n",
        )
        .expect("write task plan");

        normalize_task_plan_owners(dir.path()).expect("normalize task plan");
        let content = std::fs::read_to_string(dir.path().join("task_plan.md")).expect("content");
        assert!(content.contains(
            "- Type: research\n- Owner: planner\n- Risk: low\n- Review: auto\n- Review-Status: none\n- Writes: []\n- Dependencies: []"
        ));
    }
}
