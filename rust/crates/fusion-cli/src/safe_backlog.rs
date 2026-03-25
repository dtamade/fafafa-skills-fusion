use anyhow::Result;
use fusion_runtime_io::{append_event, json_get_string, write_text, FlatConfig};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::Path;

use crate::models::{SafeBacklogResult, SafeBackoffState, TaskCounts};
use crate::render::read_task_counts;
use crate::safe_backlog_core::{
    apply_success_state, backoff_blocks_attempt, backoff_settings, mark_backoff_failure,
    score_and_rotate_candidates, select_candidates,
};
use crate::safe_backlog_support::{
    append_task_plan, candidate_tasks, counts_snapshot, load_safe_backlog_state,
    parse_allowed_categories, persist_safe_backlog_state, project_root_from_fusion_dir,
};

pub(crate) struct SafeBacklogTrigger<'a> {
    pub(crate) counts: &'a TaskCounts,
    pub(crate) pending_like: i64,
    pub(crate) current_snap: &'a str,
    pub(crate) reason: &'a str,
    pub(crate) no_progress_rounds: i64,
    pub(crate) snap_file: &'a Path,
}

pub(crate) fn try_inject_safe_backlog(
    fusion_dir: &Path,
    snapshot: &Value,
    cfg: &FlatConfig,
    trigger: SafeBacklogTrigger<'_>,
) -> Result<Option<Vec<String>>> {
    let counts = trigger.counts;
    let pending_like = trigger.pending_like;
    let current_snap = trigger.current_snap;
    let reason = trigger.reason;
    let no_progress_rounds = trigger.no_progress_rounds;
    let snap_file = trigger.snap_file;

    let project_root = project_root_from_fusion_dir(fusion_dir);
    let backlog_result = generate_safe_backlog(fusion_dir, &project_root, cfg)?;
    if backlog_result.blocked_by_backoff || backlog_result.added <= 0 {
        return Ok(None);
    }

    let current_phase =
        json_get_string(snapshot, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());
    let stall_score = compute_stall_score(no_progress_rounds, pending_like, counts.failed, reason);
    let payload = json!({
        "reason": reason,
        "stall_score": stall_score,
        "added": backlog_result.added,
        "tasks": backlog_result.tasks,
    });
    let key = format!(
        "safe_backlog:{reason}:{current_snap}:{}",
        backlog_result.added
    );

    let _ = append_event(
        fusion_dir,
        "SAFE_BACKLOG_INJECTED",
        &current_phase,
        &current_phase,
        payload,
        &key,
    );

    let latest_counts = read_task_counts(fusion_dir)?;
    let _ = write_text(snap_file, &counts_snapshot(latest_counts));

    Ok(Some(vec![format!(
        "[fusion] Safe backlog injected: +{} task(s)",
        backlog_result.added
    )]))
}

pub(crate) fn try_inject_safe_backlog_for_stop_guard(
    fusion_dir: &Path,
    snapshot: &Value,
    cfg: &FlatConfig,
    counts: TaskCounts,
    current_snap: &str,
    record_runtime_event: bool,
) -> Result<Option<i64>> {
    let project_root = project_root_from_fusion_dir(fusion_dir);
    let backlog_result = generate_safe_backlog(fusion_dir, &project_root, cfg)?;
    if backlog_result.blocked_by_backoff || backlog_result.added <= 0 {
        return Ok(None);
    }

    if record_runtime_event {
        let current_phase =
            json_get_string(snapshot, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());
        let payload = json!({
            "reason": "task_exhausted",
            "stall_score": compute_stall_score(0, 0, counts.failed, "task_exhausted"),
            "added": backlog_result.added,
            "tasks": backlog_result.tasks,
        });
        let key = format!(
            "safe_backlog:task_exhausted:{current_snap}:{}",
            backlog_result.added
        );
        let _ = append_event(
            fusion_dir,
            "SAFE_BACKLOG_INJECTED",
            &current_phase,
            &current_phase,
            payload,
            &key,
        );

        let latest_counts = read_task_counts(fusion_dir)?;
        let _ = write_text(
            &fusion_dir.join(".progress_snapshot"),
            &counts_snapshot(latest_counts),
        );
    }

    Ok(Some(backlog_result.added))
}

pub(crate) fn reset_safe_backlog_backoff(fusion_dir: &Path) -> Result<()> {
    let state_path = fusion_dir.join("safe_backlog.json");
    let mut state = load_safe_backlog_state(&state_path);
    state.backoff.consecutive_failures = 0;
    state.backoff.consecutive_injections = 0;
    state.backoff.cooldown_until_round = 0;
    persist_safe_backlog_state(&state_path, &state);
    Ok(())
}

fn compute_stall_score(
    no_progress_rounds: i64,
    pending_like: i64,
    failed_tasks: i64,
    reason: &str,
) -> f64 {
    let mut score = 0.2;

    if reason == "task_exhausted" {
        score += 0.45;
    }
    if reason == "no_progress" {
        score += (no_progress_rounds as f64 * 0.12).min(0.4);
    }

    if pending_like == 0 {
        score += 0.2;
    }
    if failed_tasks > 0 {
        score += (failed_tasks as f64 * 0.05).min(0.15);
    }

    score.clamp(0.0, 1.0)
}

fn generate_safe_backlog(
    fusion_dir: &Path,
    project_root: &Path,
    cfg: &FlatConfig,
) -> Result<SafeBacklogResult> {
    let mut result = SafeBacklogResult {
        enabled: cfg.safe_backlog_enabled,
        added: 0,
        tasks: vec![],
        blocked_by_backoff: false,
        backoff_state: SafeBackoffState::default(),
    };

    let task_plan_path = fusion_dir.join("task_plan.md");
    let state_path = fusion_dir.join("safe_backlog.json");

    if !cfg.safe_backlog_enabled || !task_plan_path.is_file() {
        return Ok(result);
    }

    let allowed = parse_allowed_categories(&cfg.safe_backlog_allowed_categories);
    let limit = cfg.safe_backlog_max_tasks_per_run.max(1) as usize;
    let novelty_window = cfg.safe_backlog_novelty_window.max(1) as usize;

    let mut state = load_safe_backlog_state(&state_path);
    let seen_slice_start = state.fingerprints.len().saturating_sub(novelty_window);
    let seen: HashSet<String> = state.fingerprints[seen_slice_start..]
        .iter()
        .cloned()
        .collect();

    let mut backoff = state.backoff.clone();
    let backoff_settings = backoff_settings(cfg);

    backoff.attempt_round += 1;

    if backoff_blocks_attempt(&backoff, &backoff_settings) {
        state.backoff = backoff.clone();
        persist_safe_backlog_state(&state_path, &state);
        result.blocked_by_backoff = true;
        result.backoff_state = backoff;
        return Ok(result);
    }

    let candidates = score_and_rotate_candidates(
        candidate_tasks(project_root),
        &state,
        cfg.safe_backlog_diversity_rotation,
    );
    let (selected, added_fingerprints) = select_candidates(candidates, &allowed, &seen, limit);

    if selected.is_empty() {
        mark_backoff_failure(&mut backoff, &backoff_settings);

        state.backoff = backoff.clone();
        persist_safe_backlog_state(&state_path, &state);
        result.backoff_state = backoff;
        return Ok(result);
    }

    append_task_plan(&task_plan_path, &selected)?;

    apply_success_state(
        &mut state,
        &mut backoff,
        &selected,
        added_fingerprints,
        novelty_window,
        &backoff_settings,
    );

    state.backoff = backoff.clone();
    persist_safe_backlog_state(&state_path, &state);

    result.added = selected.len() as i64;
    result.tasks = selected;
    result.backoff_state = backoff;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_stall_score_bounds() {
        let score = compute_stall_score(10, 0, 4, "no_progress");
        assert!((0.0..=1.0).contains(&score));
    }
}
