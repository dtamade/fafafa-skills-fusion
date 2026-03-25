use anyhow::Result;
use fusion_runtime_io::read_text;
use std::path::Path;

const TASK_TAGS: [&str; 4] = ["[IN_PROGRESS]", "[PENDING]", "[COMPLETED]", "[FAILED]"];

fn extract_task_name(line: &str) -> Option<String> {
    if !line.contains("### Task") {
        return None;
    }

    let mut name = if let Some((_, right)) = line.split_once(':') {
        right.trim().to_string()
    } else {
        line.to_string()
    };

    for tag in TASK_TAGS {
        name = name.replace(tag, "").trim().to_string();
    }

    (!name.is_empty()).then_some(name)
}

pub(crate) fn find_next_task(fusion_dir: &Path) -> Result<String> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok("unknown".to_string());
    }

    let content = read_text(&task_plan)?;
    for line in content.lines() {
        if !(line.contains("[IN_PROGRESS]") || line.contains("[PENDING]")) {
            continue;
        }
        if let Some(name) = extract_task_name(line) {
            return Ok(name);
        }
    }

    Ok("unknown".to_string())
}

pub(crate) fn find_last_task_with_status(fusion_dir: &Path, status: &str) -> Result<String> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok("unknown".to_string());
    }

    let content = read_text(&task_plan)?;
    let mut found = "unknown".to_string();
    for line in content.lines() {
        if !line.contains(status) {
            continue;
        }
        if let Some(name) = extract_task_name(line) {
            found = name;
        }
    }

    Ok(found)
}

pub(crate) fn find_first_task_with_status(fusion_dir: &Path, status: &str) -> Result<String> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok(String::new());
    }

    let content = read_text(&task_plan)?;
    for line in content.lines() {
        if !line.contains(status) {
            continue;
        }
        if let Some(name) = extract_task_name(line) {
            return Ok(name);
        }
    }

    Ok(String::new())
}
