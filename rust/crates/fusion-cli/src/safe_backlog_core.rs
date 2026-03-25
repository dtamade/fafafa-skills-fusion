use rand::Rng;
use std::collections::HashSet;

use fusion_runtime_io::FlatConfig;

use crate::models::{SafeBacklogState, SafeBackoffState, SafeTask};
use crate::safe_backlog_support::{fingerprint, priority_score};

pub(crate) struct BackoffSettings {
    pub(crate) base_rounds: i64,
    pub(crate) max_rounds: i64,
    pub(crate) jitter: f64,
    pub(crate) force_probe_rounds: i64,
    pub(crate) enabled: bool,
}

pub(crate) fn backoff_settings(cfg: &FlatConfig) -> BackoffSettings {
    BackoffSettings {
        base_rounds: cfg.safe_backlog_backoff_base_rounds.max(1),
        max_rounds: cfg
            .safe_backlog_backoff_max_rounds
            .max(cfg.safe_backlog_backoff_base_rounds.max(1)),
        jitter: cfg.safe_backlog_backoff_jitter.clamp(0.0, 1.0),
        force_probe_rounds: cfg.safe_backlog_backoff_force_probe_rounds.max(1),
        enabled: cfg.safe_backlog_backoff_enabled,
    }
}

pub(crate) fn backoff_blocks_attempt(
    backoff: &SafeBackoffState,
    settings: &BackoffSettings,
) -> bool {
    settings.enabled
        && backoff.attempt_round <= backoff.cooldown_until_round
        && (backoff.attempt_round % settings.force_probe_rounds != 0)
}

pub(crate) fn score_and_rotate_candidates(
    mut candidates: Vec<SafeTask>,
    state: &SafeBacklogState,
    rotate_categories: bool,
) -> Vec<SafeTask> {
    for candidate in &mut candidates {
        let score = priority_score(
            candidate,
            &state.last_category,
            &state.stats.category_counts,
        );
        candidate.priority_score = Some((score * 10000.0).round() / 10000.0);
    }

    if rotate_categories && !state.last_category.is_empty() {
        let mut rotated: Vec<SafeTask> = candidates
            .iter()
            .filter(|task| task.category != state.last_category)
            .cloned()
            .collect();
        if !rotated.is_empty() {
            rotated.extend(
                candidates
                    .iter()
                    .filter(|task| task.category == state.last_category)
                    .cloned(),
            );
            candidates = rotated;
        }
    }

    candidates.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates
}

pub(crate) fn select_candidates(
    candidates: Vec<SafeTask>,
    allowed: &HashSet<String>,
    seen: &HashSet<String>,
    limit: usize,
) -> (Vec<SafeTask>, Vec<String>) {
    let mut selected: Vec<SafeTask> = Vec::new();
    let mut added_fingerprints: Vec<String> = Vec::new();

    for candidate in candidates {
        if !allowed.is_empty() && !allowed.contains(&candidate.category.to_lowercase()) {
            continue;
        }
        let fp = fingerprint(&candidate);
        if seen.contains(&fp) {
            continue;
        }

        selected.push(candidate);
        added_fingerprints.push(fp);
        if selected.len() >= limit {
            break;
        }
    }

    (selected, added_fingerprints)
}

pub(crate) fn mark_backoff_failure(backoff: &mut SafeBackoffState, settings: &BackoffSettings) {
    if !settings.enabled {
        return;
    }
    backoff.consecutive_failures += 1;
    backoff.consecutive_injections = 0;
    let mut cooldown = (settings.base_rounds
        * (2_i64.pow((backoff.consecutive_failures - 1).max(0) as u32)))
    .min(settings.max_rounds);
    if settings.jitter > 0.0 {
        let factor =
            rand::thread_rng().gen_range((1.0 - settings.jitter)..=(1.0 + settings.jitter));
        cooldown = ((cooldown as f64) * factor).round().max(1.0) as i64;
    }
    backoff.cooldown_until_round = backoff.attempt_round + cooldown;
}

pub(crate) fn apply_success_state(
    state: &mut SafeBacklogState,
    backoff: &mut SafeBackoffState,
    selected: &[SafeTask],
    added_fingerprints: Vec<String>,
    novelty_window: usize,
    settings: &BackoffSettings,
) {
    state.fingerprints.extend(added_fingerprints);
    if state.fingerprints.len() > novelty_window {
        let start = state.fingerprints.len() - novelty_window;
        state.fingerprints = state.fingerprints[start..].to_vec();
    }

    state.last_category = selected
        .last()
        .map(|task| task.category.clone())
        .unwrap_or_default();

    state.stats.total_injections += selected.len() as i64;
    for task in selected {
        *state
            .stats
            .category_counts
            .entry(task.category.clone())
            .or_insert(0) += 1;
    }

    if settings.enabled {
        backoff.consecutive_failures = 0;
        backoff.consecutive_injections += 1;
        let mut cooldown = (settings.base_rounds
            * (2_i64.pow((backoff.consecutive_injections - 1).max(0) as u32)))
        .min(settings.max_rounds);
        if settings.jitter > 0.0 {
            let factor =
                rand::thread_rng().gen_range((1.0 - settings.jitter)..=(1.0 + settings.jitter));
            cooldown = ((cooldown as f64) * factor).round().max(1.0) as i64;
        }
        backoff.cooldown_until_round = backoff.attempt_round + cooldown;
    }
}
