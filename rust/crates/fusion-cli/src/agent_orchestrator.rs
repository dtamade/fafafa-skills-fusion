use anyhow::Result;
use fusion_runtime_io::FlatConfig;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use crate::render::{extract_all_task_metadata, task_needs_review, ActiveTaskMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentBatchPlan {
    pub(crate) current_batch_tasks: Vec<ActiveTaskMetadata>,
    pub(crate) blocked_tasks: Vec<String>,
    pub(crate) active_roles: Vec<String>,
    pub(crate) review_queue: Vec<String>,
    pub(crate) parallel_tasks: i64,
    pub(crate) batch_reason: String,
    pub(crate) selected_reasons: BTreeMap<String, String>,
    pub(crate) blocked_reasons: BTreeMap<String, String>,
    pub(crate) review_reasons: BTreeMap<String, String>,
}

fn parse_list(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Vec::new();
    }

    let inner = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(trimmed);

    inner
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn batch_limit(cfg: &FlatConfig) -> usize {
    if !cfg.parallel_enabled || !cfg.scheduler_enabled {
        return 1;
    }

    cfg.execution_parallel
        .min(cfg.scheduler_max_parallel)
        .max(1) as usize
}

fn unresolved_dependencies(
    task: &ActiveTaskMetadata,
    completed_ids: &HashSet<String>,
) -> Vec<String> {
    if task.status == "in_progress" {
        return Vec::new();
    }
    parse_list(&task.dependencies)
        .into_iter()
        .filter(|dep| !completed_ids.contains(dep))
        .collect()
}

fn conflicting_writes(task: &ActiveTaskMetadata, selected: &[ActiveTaskMetadata]) -> Vec<String> {
    let writes = parse_list(&task.writes);
    if writes.is_empty() {
        return Vec::new();
    }

    let selected_writes: HashSet<String> = selected
        .iter()
        .flat_map(|item| parse_list(&item.writes))
        .collect();

    writes
        .into_iter()
        .filter(|path| selected_writes.contains(path))
        .collect()
}

fn dedupe_roles(tasks: &[ActiveTaskMetadata]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut roles = Vec::new();

    for task in tasks {
        if seen.insert(task.owner.clone()) {
            roles.push(task.owner.clone());
        }
    }

    roles
}

fn explain_is_off(explain_level: &str) -> bool {
    explain_level == "off"
}

fn format_joined(values: &[String]) -> String {
    values.join("+")
}

fn batch_reason(limit: usize, explain_level: &str) -> String {
    match (limit > 1, explain_level) {
        (true, "verbose") => format!("ready_non_conflicting_parallel:max={limit}"),
        (false, "verbose") => format!("serial_fallback:max={limit}"),
        (true, _) => "ready_non_conflicting_parallel".to_string(),
        (false, _) => "serial_fallback".to_string(),
    }
}

fn selected_reason(task: &ActiveTaskMetadata, explain_level: &str) -> Option<String> {
    match explain_level {
        "off" => None,
        "compact" => Some("ready".to_string()),
        "verbose" => {
            if task.status == "in_progress" {
                Some("ready:already_in_progress".to_string())
            } else {
                let deps = parse_list(&task.dependencies);
                if deps.is_empty() {
                    Some("ready:no_dependencies".to_string())
                } else {
                    Some(format!(
                        "ready:dependencies_satisfied:{}",
                        format_joined(&deps)
                    ))
                }
            }
        }
        _ => Some("ready".to_string()),
    }
}

fn blocked_dependency_reason(missing: &[String], explain_level: &str) -> Option<String> {
    match explain_level {
        "off" => None,
        "compact" => Some("waiting_for_dependencies".to_string()),
        "verbose" => Some(format!(
            "waiting_for_dependencies:{}",
            format_joined(missing)
        )),
        _ => Some("waiting_for_dependencies".to_string()),
    }
}

fn blocked_parallel_limit_reason(limit: usize, explain_level: &str) -> Option<String> {
    match explain_level {
        "off" => None,
        "compact" => Some("parallel_limit".to_string()),
        "verbose" => Some(format!("parallel_limit:max={limit}")),
        _ => Some("parallel_limit".to_string()),
    }
}

fn blocked_conflict_reason(conflicts: &[String], explain_level: &str) -> Option<String> {
    match explain_level {
        "off" => None,
        "compact" => Some("write_conflict".to_string()),
        "verbose" => Some(format!("write_conflict:{}", format_joined(conflicts))),
        _ => Some("write_conflict".to_string()),
    }
}

fn review_reason(
    task: &ActiveTaskMetadata,
    review_policy: &str,
    explain_level: &str,
) -> Option<String> {
    match explain_level {
        "off" => None,
        "compact" => Some("review_required".to_string()),
        "verbose" => {
            if review_policy == "always" {
                Some("review_required:policy=always".to_string())
            } else {
                Some(format!(
                    "review_required:risk={}+flag={}",
                    task.risk, task.review
                ))
            }
        }
        _ => Some("review_required".to_string()),
    }
}

pub(crate) fn plan_agent_batch(
    fusion_dir: &Path,
    cfg: &FlatConfig,
) -> Result<Option<AgentBatchPlan>> {
    let tasks = extract_all_task_metadata(fusion_dir)?;
    if tasks.is_empty() {
        return Ok(None);
    }

    let completed_ids: HashSet<String> = tasks
        .iter()
        .filter(|task| task.status == "completed")
        .map(|task| task.task_id.clone())
        .collect();

    let ordered_live_tasks: Vec<ActiveTaskMetadata> = tasks
        .iter()
        .filter(|task| task.status == "in_progress")
        .cloned()
        .chain(
            tasks
                .iter()
                .filter(|task| task.status == "pending")
                .cloned(),
        )
        .collect();
    if ordered_live_tasks.is_empty() {
        return Ok(None);
    }

    let limit = batch_limit(cfg);
    let mut current_batch_tasks = Vec::new();
    let mut blocked_tasks = Vec::new();
    let mut selected_reasons = BTreeMap::new();
    let mut blocked_reasons = BTreeMap::new();

    for task in ordered_live_tasks {
        let missing_dependencies = unresolved_dependencies(&task, &completed_ids);
        if !missing_dependencies.is_empty() {
            blocked_tasks.push(task.task_id.clone());
            if let Some(reason) =
                blocked_dependency_reason(&missing_dependencies, &cfg.agent_explain_level)
            {
                blocked_reasons.insert(task.task_id.clone(), reason);
            }
            continue;
        }
        let conflicts = conflicting_writes(&task, &current_batch_tasks);
        if cfg.parallel_conflict_check && !conflicts.is_empty() {
            blocked_tasks.push(task.task_id.clone());
            if let Some(reason) = blocked_conflict_reason(&conflicts, &cfg.agent_explain_level) {
                blocked_reasons.insert(task.task_id.clone(), reason);
            }
            continue;
        }
        if current_batch_tasks.len() >= limit {
            blocked_tasks.push(task.task_id.clone());
            if let Some(reason) = blocked_parallel_limit_reason(limit, &cfg.agent_explain_level) {
                blocked_reasons.insert(task.task_id.clone(), reason);
            }
            continue;
        }
        if let Some(reason) = selected_reason(&task, &cfg.agent_explain_level) {
            selected_reasons.insert(task.task_id.clone(), reason);
        }
        current_batch_tasks.push(task);
    }

    if current_batch_tasks.is_empty() {
        return Ok(None);
    }

    let active_roles = dedupe_roles(&current_batch_tasks);
    let review_queue: Vec<String> = current_batch_tasks
        .iter()
        .filter(|task| task_needs_review(task, &cfg.agent_review_policy))
        .map(|task| task.task_id.clone())
        .collect();
    let review_reasons = if explain_is_off(&cfg.agent_explain_level) {
        BTreeMap::new()
    } else {
        current_batch_tasks
            .iter()
            .filter(|task| task_needs_review(task, &cfg.agent_review_policy))
            .filter_map(|task| {
                review_reason(task, &cfg.agent_review_policy, &cfg.agent_explain_level)
                    .map(|reason| (task.task_id.clone(), reason))
            })
            .collect()
    };

    Ok(Some(AgentBatchPlan {
        parallel_tasks: current_batch_tasks.len() as i64,
        blocked_tasks,
        active_roles,
        review_queue,
        batch_reason: batch_reason(limit, &cfg.agent_explain_level),
        current_batch_tasks,
        selected_reasons,
        blocked_reasons,
        review_reasons,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_list_supports_bracketed_values() {
        assert_eq!(
            parse_list("[task_1, task_2, docs/a.md]"),
            vec!["task_1", "task_2", "docs/a.md"]
        );
        assert!(parse_list("[]").is_empty());
    }
}
