use anyhow::Result;
use fusion_runtime_io::{json_get_string, load_flat_config, read_json, read_text};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::path::Path;

use crate::achievements::collect_achievement_summary;
use crate::render::{
    epoch_to_iso, guardian_status_from_metrics, last_safe_backlog, read_task_counts,
};
use crate::status_artifacts::{read_backend_failure_summary, read_dependency_summary};
use crate::status_owner::collect_owner_metrics;

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct StatusSummary {
    pub(crate) result: String,
    pub(crate) status: String,
    pub(crate) phase: String,
    pub(crate) workflow_id: Option<String>,
    pub(crate) goal: Option<String>,
    pub(crate) started_at: Option<String>,
    pub(crate) last_checkpoint: Option<String>,
    pub(crate) runtime_state: Option<String>,
    pub(crate) understand_mode: Option<String>,
    pub(crate) understand_forced: Option<bool>,
    pub(crate) understand_decision: Option<String>,
    pub(crate) codex_session: Option<String>,
    pub(crate) claude_session: Option<String>,
    pub(crate) planner_codex_session: Option<String>,
    pub(crate) planner_claude_session: Option<String>,
    pub(crate) coder_codex_session: Option<String>,
    pub(crate) coder_claude_session: Option<String>,
    pub(crate) reviewer_codex_session: Option<String>,
    pub(crate) reviewer_claude_session: Option<String>,
    pub(crate) task_completed: i64,
    pub(crate) task_pending: i64,
    pub(crate) task_in_progress: i64,
    pub(crate) task_failed: i64,
    pub(crate) dependency_status: String,
    pub(crate) dependency_source: String,
    pub(crate) dependency_reason: String,
    pub(crate) dependency_missing: String,
    pub(crate) dependency_next: String,
    pub(crate) guardian_status: Option<String>,
    pub(crate) guardian_total_iterations: Option<i64>,
    pub(crate) guardian_no_progress_rounds: Option<i64>,
    pub(crate) guardian_same_action_count: Option<i64>,
    pub(crate) guardian_same_error_count: Option<i64>,
    pub(crate) guardian_max_state_visit_count: Option<i64>,
    pub(crate) guardian_wall_time_ms: Option<i64>,
    pub(crate) achievement_completed_tasks: i64,
    pub(crate) achievement_safe_total: i64,
    pub(crate) achievement_advisory_total: i64,
    pub(crate) owner_planner: i64,
    pub(crate) owner_coder: i64,
    pub(crate) owner_reviewer: i64,
    pub(crate) current_role: String,
    pub(crate) current_role_task: String,
    pub(crate) current_role_status: String,
    pub(crate) backend_status: String,
    pub(crate) backend_source: String,
    pub(crate) backend_primary: String,
    pub(crate) backend_fallback: String,
    pub(crate) backend_primary_error: String,
    pub(crate) backend_fallback_error: String,
    pub(crate) backend_next: String,
    pub(crate) safe_backlog_last_added: Option<i64>,
    pub(crate) safe_backlog_last_injected_at: Option<f64>,
    pub(crate) safe_backlog_last_injected_at_iso: Option<String>,
    pub(crate) runtime_last_event_id: Option<String>,
    pub(crate) runtime_last_event_counter: Option<i64>,
    pub(crate) runtime_scheduler_enabled: Option<bool>,
    pub(crate) runtime_scheduler_batch_id: Option<i64>,
    pub(crate) runtime_scheduler_parallel_tasks: Option<i64>,
    pub(crate) agents_enabled: bool,
    pub(crate) agent_mode: Option<String>,
    pub(crate) agent_explain_level: Option<String>,
    pub(crate) agent_current_batch_id: Option<i64>,
    pub(crate) agent_active_roles: Vec<String>,
    pub(crate) agent_current_batch_tasks: Vec<String>,
    pub(crate) agent_review_queue: Vec<String>,
    pub(crate) agent_review_queue_size: Option<i64>,
    pub(crate) agent_last_decision_reason: Option<String>,
    pub(crate) agent_batch_reason: Option<String>,
    pub(crate) agent_collaboration_mode: Option<String>,
    pub(crate) agent_turn_role: Option<String>,
    pub(crate) agent_turn_task_id: Option<String>,
    pub(crate) agent_turn_kind: Option<String>,
    pub(crate) agent_pending_reviews: Vec<String>,
    pub(crate) agent_blocked_handoff_reason: Option<String>,
    pub(crate) agent_selected_reasons: BTreeMap<String, String>,
    pub(crate) agent_blocked_reasons: BTreeMap<String, String>,
    pub(crate) agent_review_reasons: BTreeMap<String, String>,
    pub(crate) hook_debug_enabled: bool,
    pub(crate) hook_debug_flag: String,
    pub(crate) hook_debug_log: String,
    pub(crate) hook_debug_tail: Vec<String>,
    pub(crate) runtime_enabled: bool,
    pub(crate) runtime_compat_mode: bool,
    pub(crate) runtime_engine: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct HookDebugSummary {
    pub(crate) enabled: bool,
    pub(crate) flag_path: String,
    pub(crate) log_path: String,
    pub(crate) log_tail: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SafeBacklogSummary {
    pub(crate) last_added: Option<i64>,
    pub(crate) last_injected_at: Option<f64>,
    pub(crate) last_injected_at_iso: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GuardianSummary {
    pub(crate) status: Option<String>,
    pub(crate) total_iterations: Option<i64>,
    pub(crate) no_progress_rounds: Option<i64>,
    pub(crate) same_action_count: Option<i64>,
    pub(crate) same_error_count: Option<i64>,
    pub(crate) max_state_visit_count: Option<i64>,
    pub(crate) wall_time_ms: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SessionIdentitySummary {
    pub(crate) workflow_id: Option<String>,
    pub(crate) goal: Option<String>,
    pub(crate) started_at: Option<String>,
    pub(crate) last_checkpoint: Option<String>,
    pub(crate) runtime_state: Option<String>,
    pub(crate) understand_mode: Option<String>,
    pub(crate) understand_forced: Option<bool>,
    pub(crate) understand_decision: Option<String>,
    pub(crate) codex_session: Option<String>,
    pub(crate) claude_session: Option<String>,
    pub(crate) planner_codex_session: Option<String>,
    pub(crate) planner_claude_session: Option<String>,
    pub(crate) coder_codex_session: Option<String>,
    pub(crate) coder_claude_session: Option<String>,
    pub(crate) reviewer_codex_session: Option<String>,
    pub(crate) reviewer_claude_session: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeSessionSummary {
    pub(crate) last_event_id: Option<String>,
    pub(crate) last_event_counter: Option<i64>,
    pub(crate) scheduler_enabled: Option<bool>,
    pub(crate) scheduler_batch_id: Option<i64>,
    pub(crate) scheduler_parallel_tasks: Option<i64>,
    pub(crate) agent_mode: Option<String>,
    pub(crate) agent_explain_level: Option<String>,
    pub(crate) agent_current_batch_id: Option<i64>,
    pub(crate) agent_active_roles: Vec<String>,
    pub(crate) agent_current_batch_tasks: Vec<String>,
    pub(crate) agent_review_queue: Vec<String>,
    pub(crate) agent_review_queue_size: Option<i64>,
    pub(crate) agent_last_decision_reason: Option<String>,
    pub(crate) agent_batch_reason: Option<String>,
    pub(crate) agent_collaboration_mode: Option<String>,
    pub(crate) agent_turn_role: Option<String>,
    pub(crate) agent_turn_task_id: Option<String>,
    pub(crate) agent_turn_kind: Option<String>,
    pub(crate) agent_pending_reviews: Vec<String>,
    pub(crate) agent_blocked_handoff_reason: Option<String>,
    pub(crate) agent_selected_reasons: BTreeMap<String, String>,
    pub(crate) agent_blocked_reasons: BTreeMap<String, String>,
    pub(crate) agent_review_reasons: BTreeMap<String, String>,
}

fn json_string_map(value: Option<&Value>) -> BTreeMap<String, String> {
    value
        .and_then(|v| v.as_object())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|(key, value)| value.as_str().map(|s| (key.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

fn truthy_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub(crate) fn collect_hook_debug_summary(fusion_dir: &Path) -> Result<HookDebugSummary> {
    let flag_path = fusion_dir.join(".hook_debug");
    let log_path = fusion_dir.join("hook-debug.log");
    let env_enabled = env::var("FUSION_HOOK_DEBUG")
        .map(|value| truthy_flag(&value))
        .unwrap_or(false);

    let log_tail = if log_path.is_file() {
        let mut lines = read_text(&log_path)?
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        if lines.len() > 5 {
            lines = lines.split_off(lines.len() - 5);
        }
        lines
    } else {
        Vec::new()
    };

    Ok(HookDebugSummary {
        enabled: env_enabled || flag_path.is_file(),
        flag_path: if flag_path.is_file() {
            flag_path.display().to_string()
        } else {
            String::new()
        },
        log_path: if log_path.is_file() {
            log_path.display().to_string()
        } else {
            String::new()
        },
        log_tail,
    })
}

pub(crate) fn collect_safe_backlog_summary(fusion_dir: &Path) -> Result<SafeBacklogSummary> {
    let events_path = fusion_dir.join("events.jsonl");
    if !events_path.is_file() {
        return Ok(SafeBacklogSummary::default());
    }

    let Some((added, timestamp)) = last_safe_backlog(&events_path)? else {
        return Ok(SafeBacklogSummary::default());
    };

    Ok(SafeBacklogSummary {
        last_added: Some(added),
        last_injected_at: Some(timestamp),
        last_injected_at_iso: epoch_to_iso(timestamp),
    })
}

pub(crate) fn collect_guardian_summary(fusion_dir: &Path) -> Result<GuardianSummary> {
    let loop_context_path = fusion_dir.join("loop_context.json");
    if !loop_context_path.is_file() {
        return Ok(GuardianSummary::default());
    }

    let payload = read_json(&loop_context_path)?;
    let total_iterations = payload.get("total_iterations").and_then(|v| v.as_i64());
    let no_progress_rounds = payload.get("no_progress_rounds").and_then(|v| v.as_i64());
    let same_action_count = payload.get("same_action_count").and_then(|v| v.as_i64());
    let same_error_count = payload.get("same_error_count").and_then(|v| v.as_i64());
    let max_state_visit_count = payload
        .get("max_state_visit_count")
        .and_then(|v| v.as_i64());
    let wall_time_ms = payload.get("wall_time_ms").and_then(|v| v.as_i64());

    Ok(GuardianSummary {
        status: Some(guardian_status_from_metrics(
            no_progress_rounds.unwrap_or(0),
            same_action_count.unwrap_or(0),
        )),
        total_iterations,
        no_progress_rounds,
        same_action_count,
        same_error_count,
        max_state_visit_count,
        wall_time_ms,
    })
}

pub(crate) fn collect_session_identity_summary(sessions: &Value) -> SessionIdentitySummary {
    SessionIdentitySummary {
        workflow_id: json_get_string(sessions, &["workflow_id"]),
        goal: json_get_string(sessions, &["goal"]),
        started_at: json_get_string(sessions, &["started_at"]),
        last_checkpoint: json_get_string(sessions, &["last_checkpoint"]),
        runtime_state: json_get_string(sessions, &["_runtime", "state"]),
        understand_mode: json_get_string(sessions, &["_runtime", "understand", "mode"]),
        understand_forced: sessions
            .get("_runtime")
            .and_then(|v| v.get("understand"))
            .and_then(|v| v.get("forced"))
            .and_then(|v| v.as_bool()),
        understand_decision: json_get_string(sessions, &["_runtime", "understand", "decision"]),
        codex_session: json_get_string(sessions, &["codex_session"]),
        claude_session: json_get_string(sessions, &["claude_session"]),
        planner_codex_session: json_get_string(sessions, &["planner_codex_session"]),
        planner_claude_session: json_get_string(sessions, &["planner_claude_session"]),
        coder_codex_session: json_get_string(sessions, &["coder_codex_session"]),
        coder_claude_session: json_get_string(sessions, &["coder_claude_session"]),
        reviewer_codex_session: json_get_string(sessions, &["reviewer_codex_session"]),
        reviewer_claude_session: json_get_string(sessions, &["reviewer_claude_session"]),
    }
}

pub(crate) fn collect_runtime_session_summary(sessions: &Value) -> RuntimeSessionSummary {
    RuntimeSessionSummary {
        last_event_id: json_get_string(sessions, &["_runtime", "last_event_id"]),
        last_event_counter: sessions
            .get("_runtime")
            .and_then(|v| v.get("last_event_counter"))
            .and_then(|v| v.as_i64()),
        scheduler_enabled: sessions
            .get("_runtime")
            .and_then(|v| v.get("scheduler"))
            .and_then(|v| v.get("enabled"))
            .and_then(|v| v.as_bool()),
        scheduler_batch_id: sessions
            .get("_runtime")
            .and_then(|v| v.get("scheduler"))
            .and_then(|v| v.get("current_batch_id"))
            .and_then(|v| v.as_i64()),
        scheduler_parallel_tasks: sessions
            .get("_runtime")
            .and_then(|v| v.get("scheduler"))
            .and_then(|v| v.get("parallel_tasks"))
            .and_then(|v| v.as_i64()),
        agent_mode: json_get_string(sessions, &["_runtime", "agents", "mode"]),
        agent_explain_level: json_get_string(sessions, &["_runtime", "agents", "explain_level"]),
        agent_current_batch_id: sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("current_batch_id"))
            .and_then(|v| v.as_i64()),
        agent_active_roles: sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("active_roles"))
            .and_then(|v| v.as_array())
            .map(|roles| {
                roles
                    .iter()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
        agent_current_batch_tasks: sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("current_batch_tasks"))
            .and_then(|v| v.as_array())
            .map(|tasks| {
                tasks
                    .iter()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
        agent_review_queue: sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("review_queue"))
            .and_then(|v| v.as_array())
            .map(|tasks| {
                tasks
                    .iter()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
        agent_review_queue_size: sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("review_queue_size"))
            .and_then(|v| v.as_i64()),
        agent_last_decision_reason: json_get_string(
            sessions,
            &["_runtime", "agents", "last_decision_reason"],
        ),
        agent_batch_reason: json_get_string(
            sessions,
            &["_runtime", "agents", "policy", "batch_reason"],
        ),
        agent_collaboration_mode: json_get_string(
            sessions,
            &["_runtime", "agents", "collaboration", "mode"],
        ),
        agent_turn_role: json_get_string(
            sessions,
            &["_runtime", "agents", "collaboration", "turn_role"],
        ),
        agent_turn_task_id: json_get_string(
            sessions,
            &["_runtime", "agents", "collaboration", "turn_task_id"],
        ),
        agent_turn_kind: json_get_string(
            sessions,
            &["_runtime", "agents", "collaboration", "turn_kind"],
        ),
        agent_pending_reviews: sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("collaboration"))
            .and_then(|v| v.get("pending_reviews"))
            .and_then(|v| v.as_array())
            .map(|tasks| {
                tasks
                    .iter()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
        agent_blocked_handoff_reason: json_get_string(
            sessions,
            &[
                "_runtime",
                "agents",
                "collaboration",
                "blocked_handoff_reason",
            ],
        ),
        agent_selected_reasons: json_string_map(
            sessions
                .get("_runtime")
                .and_then(|v| v.get("agents"))
                .and_then(|v| v.get("policy"))
                .and_then(|v| v.get("selected_reasons")),
        ),
        agent_blocked_reasons: json_string_map(
            sessions
                .get("_runtime")
                .and_then(|v| v.get("agents"))
                .and_then(|v| v.get("policy"))
                .and_then(|v| v.get("blocked_reasons")),
        ),
        agent_review_reasons: json_string_map(
            sessions
                .get("_runtime")
                .and_then(|v| v.get("agents"))
                .and_then(|v| v.get("policy"))
                .and_then(|v| v.get("review_reasons")),
        ),
    }
}

pub(crate) fn render_understand_handoff(
    mode: Option<&str>,
    forced: Option<bool>,
    decision: Option<&str>,
) -> Option<String> {
    let mode = mode?;
    let mut details: Vec<String> = Vec::new();
    if let Some(decision) = decision {
        details.push(format!("decision={decision}"));
    }
    if let Some(forced) = forced {
        details.push(format!("forced={forced}"));
    }

    if details.is_empty() {
        Some(mode.to_string())
    } else {
        Some(format!("{mode} ({})", details.join(", ")))
    }
}

pub(crate) fn build_status_summary(fusion_dir: &Path) -> Result<StatusSummary> {
    let sessions_path = fusion_dir.join("sessions.json");
    let cfg = load_flat_config(fusion_dir);
    let task_counts = read_task_counts(fusion_dir)?;
    let owner_metrics = collect_owner_metrics(fusion_dir)?;
    let achievements = collect_achievement_summary(fusion_dir)?;

    let (status, phase, session_identity, runtime_session) = if sessions_path.is_file() {
        let sessions = read_json(&sessions_path)?;
        (
            json_get_string(&sessions, &["status"]).unwrap_or_default(),
            json_get_string(&sessions, &["current_phase"]).unwrap_or_default(),
            collect_session_identity_summary(&sessions),
            collect_runtime_session_summary(&sessions),
        )
    } else {
        (
            String::new(),
            String::new(),
            SessionIdentitySummary::default(),
            RuntimeSessionSummary::default(),
        )
    };

    let dependency = read_dependency_summary(fusion_dir)?;
    let backend = read_backend_failure_summary(fusion_dir)?;
    let guardian = collect_guardian_summary(fusion_dir)?;
    let safe_backlog = collect_safe_backlog_summary(fusion_dir)?;
    let hook_debug = collect_hook_debug_summary(fusion_dir)?;

    Ok(StatusSummary {
        result: "ok".to_string(),
        status,
        phase,
        workflow_id: session_identity.workflow_id,
        goal: session_identity.goal,
        started_at: session_identity.started_at,
        last_checkpoint: session_identity.last_checkpoint,
        runtime_state: session_identity.runtime_state,
        understand_mode: session_identity.understand_mode,
        understand_forced: session_identity.understand_forced,
        understand_decision: session_identity.understand_decision,
        codex_session: session_identity.codex_session,
        claude_session: session_identity.claude_session,
        planner_codex_session: session_identity.planner_codex_session,
        planner_claude_session: session_identity.planner_claude_session,
        coder_codex_session: session_identity.coder_codex_session,
        coder_claude_session: session_identity.coder_claude_session,
        reviewer_codex_session: session_identity.reviewer_codex_session,
        reviewer_claude_session: session_identity.reviewer_claude_session,
        task_completed: task_counts.completed,
        task_pending: task_counts.pending,
        task_in_progress: task_counts.in_progress,
        task_failed: task_counts.failed,
        dependency_status: dependency.status,
        dependency_source: dependency.source,
        dependency_reason: dependency.reason,
        dependency_missing: dependency.missing,
        dependency_next: dependency.next,
        guardian_status: guardian.status,
        guardian_total_iterations: guardian.total_iterations,
        guardian_no_progress_rounds: guardian.no_progress_rounds,
        guardian_same_action_count: guardian.same_action_count,
        guardian_same_error_count: guardian.same_error_count,
        guardian_max_state_visit_count: guardian.max_state_visit_count,
        guardian_wall_time_ms: guardian.wall_time_ms,
        achievement_completed_tasks: achievements.completed_tasks,
        achievement_safe_total: achievements.safe_total,
        achievement_advisory_total: achievements.advisory_total,
        owner_planner: owner_metrics.planner,
        owner_coder: owner_metrics.coder,
        owner_reviewer: owner_metrics.reviewer,
        current_role: owner_metrics.current_role,
        current_role_task: owner_metrics.current_task,
        current_role_status: owner_metrics.current_status,
        backend_status: backend.status,
        backend_source: backend.source,
        backend_primary: backend.primary_backend,
        backend_fallback: backend.fallback_backend,
        backend_primary_error: backend.primary_error,
        backend_fallback_error: backend.fallback_error,
        backend_next: backend.next,
        safe_backlog_last_added: safe_backlog.last_added,
        safe_backlog_last_injected_at: safe_backlog.last_injected_at,
        safe_backlog_last_injected_at_iso: safe_backlog.last_injected_at_iso,
        runtime_last_event_id: runtime_session.last_event_id,
        runtime_last_event_counter: runtime_session.last_event_counter,
        runtime_scheduler_enabled: runtime_session.scheduler_enabled,
        runtime_scheduler_batch_id: runtime_session.scheduler_batch_id,
        runtime_scheduler_parallel_tasks: runtime_session.scheduler_parallel_tasks,
        agents_enabled: cfg.agent_enabled,
        agent_mode: runtime_session
            .agent_mode
            .or_else(|| cfg.agent_enabled.then(|| cfg.agent_mode.clone())),
        agent_explain_level: runtime_session
            .agent_explain_level
            .or_else(|| cfg.agent_enabled.then(|| cfg.agent_explain_level.clone())),
        agent_current_batch_id: runtime_session.agent_current_batch_id,
        agent_active_roles: runtime_session.agent_active_roles,
        agent_current_batch_tasks: runtime_session.agent_current_batch_tasks,
        agent_review_queue: runtime_session.agent_review_queue,
        agent_review_queue_size: runtime_session.agent_review_queue_size,
        agent_last_decision_reason: runtime_session.agent_last_decision_reason,
        agent_batch_reason: runtime_session.agent_batch_reason,
        agent_collaboration_mode: runtime_session.agent_collaboration_mode,
        agent_turn_role: runtime_session.agent_turn_role,
        agent_turn_task_id: runtime_session.agent_turn_task_id,
        agent_turn_kind: runtime_session.agent_turn_kind,
        agent_pending_reviews: runtime_session.agent_pending_reviews,
        agent_blocked_handoff_reason: runtime_session.agent_blocked_handoff_reason,
        agent_selected_reasons: runtime_session.agent_selected_reasons,
        agent_blocked_reasons: runtime_session.agent_blocked_reasons,
        agent_review_reasons: runtime_session.agent_review_reasons,
        hook_debug_enabled: hook_debug.enabled,
        hook_debug_flag: hook_debug.flag_path,
        hook_debug_log: hook_debug.log_path,
        hook_debug_tail: hook_debug.log_tail,
        runtime_enabled: cfg.runtime_enabled,
        runtime_compat_mode: cfg.runtime_compat_mode,
        runtime_engine: cfg.runtime_engine,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_collect_hook_debug_summary_reads_flag_and_log_tail() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(dir.path().join(".hook_debug"), "").expect("flag");
        std::fs::write(dir.path().join("hook-debug.log"), "a\nb\nc\nd\ne\nf\n").expect("log");

        let summary = collect_hook_debug_summary(dir.path()).expect("hook debug");
        assert!(summary.enabled);
        assert!(summary.flag_path.ends_with(".hook_debug"));
        assert!(summary.log_path.ends_with("hook-debug.log"));
        assert_eq!(summary.log_tail, vec!["b", "c", "d", "e", "f"]);
    }
}
