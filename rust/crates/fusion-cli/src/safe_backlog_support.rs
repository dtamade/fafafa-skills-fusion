use anyhow::Result;
use fusion_runtime_io::{read_text, write_text};
use sha1::{Digest, Sha1};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::models::{SafeBacklogState, SafeTask, TaskCounts};

pub(crate) fn project_root_from_fusion_dir(fusion_dir: &Path) -> PathBuf {
    fusion_dir
        .canonicalize()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub(crate) fn counts_snapshot(counts: TaskCounts) -> String {
    format!(
        "{}:{}:{}:{}",
        counts.completed, counts.pending, counts.in_progress, counts.failed
    )
}

pub(crate) fn parse_allowed_categories(raw: &str) -> HashSet<String> {
    raw.split(',')
        .map(|item| item.trim().to_lowercase())
        .filter(|item| !item.is_empty())
        .collect()
}

pub(crate) fn fingerprint(task: &SafeTask) -> String {
    let source = format!("{}|{}|{}", task.title, task.category, task.output);
    let mut hasher = Sha1::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn priority_score(
    task: &SafeTask,
    last_category: &str,
    category_counts: &HashMap<String, i64>,
) -> f64 {
    let base = match task.category.as_str() {
        "quality" => 0.82,
        "optimization" => 0.79,
        "documentation" => 0.72,
        _ => 0.65,
    };

    let rotation_bonus = if !task.category.is_empty() && task.category != last_category {
        0.08
    } else {
        0.0
    };
    let usage_count = *category_counts.get(&task.category).unwrap_or(&0) as f64;
    let repetition_penalty = (usage_count * 0.03).min(0.25);

    (base + rotation_bonus - repetition_penalty).clamp(0.1, 0.99)
}

pub(crate) fn candidate_tasks(project_root: &Path) -> Vec<SafeTask> {
    let mut candidates: Vec<SafeTask> = Vec::new();

    if project_root.join("README.md").exists() {
        candidates.push(SafeTask {
            title: "更新 README 快速开始说明".to_string(),
            category: "documentation".to_string(),
            task_type: "documentation".to_string(),
            execution: "Direct".to_string(),
            output: "README.md".to_string(),
            priority_score: None,
        });
    }

    if project_root.join("rust/crates/fusion-cli/tests").exists() {
        candidates.push(SafeTask {
            title: "补充 Rust 契约测试清单".to_string(),
            category: "quality".to_string(),
            task_type: "verification".to_string(),
            execution: "TDD".to_string(),
            output: "rust/crates/fusion-cli/tests".to_string(),
            priority_score: None,
        });
    }

    if project_root.join("rust/crates/fusion-cli/src").exists() {
        candidates.push(SafeTask {
            title: "优化 Rust 控制面热路径扫描开销".to_string(),
            category: "optimization".to_string(),
            task_type: "configuration".to_string(),
            execution: "Direct".to_string(),
            output: "rust/crates/fusion-cli/src".to_string(),
            priority_score: None,
        });
    }

    if candidates.is_empty() {
        candidates.push(SafeTask {
            title: "整理实现说明与限制".to_string(),
            category: "documentation".to_string(),
            task_type: "documentation".to_string(),
            execution: "Direct".to_string(),
            output: "docs".to_string(),
            priority_score: None,
        });
    }

    candidates
}

pub(crate) fn load_safe_backlog_state(path: &Path) -> SafeBacklogState {
    if !path.is_file() {
        return SafeBacklogState::default();
    }

    read_text(path)
        .ok()
        .and_then(|text| serde_json::from_str::<SafeBacklogState>(&text).ok())
        .unwrap_or_default()
}

pub(crate) fn persist_safe_backlog_state(path: &Path, state: &SafeBacklogState) {
    if let Ok(text) = serde_json::to_string_pretty(state) {
        let _ = write_text(path, &text);
    }
}

pub(crate) fn append_task_plan(task_plan_path: &Path, tasks: &[SafeTask]) -> Result<()> {
    let original = read_text(task_plan_path)?;

    let mut existing_numbers: Vec<i64> = Vec::new();
    for line in original.lines() {
        if !line.starts_with("### Task ") {
            continue;
        }
        let prefix = line.split(':').next().unwrap_or("");
        let number = prefix.replace("### Task", "").trim().parse::<i64>().ok();
        if let Some(number) = number {
            existing_numbers.push(number);
        }
    }

    let mut next_index = existing_numbers.into_iter().max().unwrap_or(0) + 1;

    let mut chunks: Vec<String> = vec![original.trim_end_matches('\n').to_string()];
    if !chunks[0].is_empty() {
        chunks.push(String::new());
    }

    for task in tasks {
        chunks.push(format!(
            "### Task {next_index}: {} [PENDING] [SAFE_BACKLOG]",
            task.title
        ));
        chunks.push(format!("- Type: {}", task.task_type));
        chunks.push(format!("- Execution: {}", task.execution));
        chunks.push("- Dependencies: []".to_string());
        chunks.push(format!("- Category: {}", task.category));
        chunks.push(format!("- Output: {}", task.output));
        chunks.push(String::new());
        next_index += 1;
    }

    let merged = format!("{}\n", chunks.join("\n").trim_end_matches('\n'));
    write_text(task_plan_path, &merged)?;
    Ok(())
}
