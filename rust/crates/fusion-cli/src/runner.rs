use anyhow::{Context, Result};
use fusion_runtime_io::{
    append_event, ensure_fusion_dir, json_get_string, load_flat_config, read_json,
    remove_dependency_report_if_exists, write_json_pretty, FlatConfig,
};
use serde_json::{json, Map, Value};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::agent_handoff::{plan_role_handoff_turn, AgentCollaborationTurn};
use crate::agent_orchestrator::{plan_agent_batch, AgentBatchPlan};
use crate::hooks::{cmd_hook_posttool, cmd_hook_pretool, evaluate_stop_guard};
use crate::models::{CodeagentExecution, RunOptions};
use crate::render::{
    extract_next_task_metadata, extract_task_metadata_by_id, live_next_action_for_phase,
    normalize_task_plan_owners, read_task_counts, render_next_action, render_prompt,
    task_has_pending_review, task_is_effectively_completed, task_needs_review, ActiveTaskMetadata,
};
use crate::runner_backend::{
    clear_backend_failure_report, execute_backend_with_fallback, persist_backend_success,
    resolve_wrapper_or_dependency_error, write_backend_failure_report,
};
use crate::runner_route::{lookup_primary_session, resolve_codeagent_route};

fn ensure_object(value: &mut Value) -> &mut Map<String, Value> {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
    value
        .as_object_mut()
        .expect("value should be object after normalization")
}

fn ensure_child_object<'a>(
    parent: &'a mut Map<String, Value>,
    key: &str,
) -> &'a mut Map<String, Value> {
    let child = parent
        .entry(key.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !child.is_object() {
        *child = Value::Object(Map::new());
    }
    child
        .as_object_mut()
        .expect("child should be object after normalization")
}

fn codeagent_report_source() -> String {
    std::env::var("FUSION_CODEAGENT_REPORT_SOURCE").unwrap_or_else(|_| "fusion-bridge".to_string())
}

fn codeagent_rerun_command(source: &str) -> &'static str {
    if source == "fusion-codeagent.sh" {
        "bash scripts/fusion-codeagent.sh EXECUTE"
    } else {
        "fusion-bridge codeagent EXECUTE"
    }
}

fn codeagent_timeout_sec() -> Option<u64> {
    let raw = std::env::var("FUSION_CODEAGENT_TIMEOUT_SEC").ok()?;
    match raw.trim().parse::<u64>() {
        Ok(seconds) if seconds > 0 => Some(seconds),
        _ => None,
    }
}

fn synthesize_active_task(role: &str) -> ActiveTaskMetadata {
    let task_type = if role == "planner" {
        "research"
    } else {
        "implementation"
    };
    ActiveTaskMetadata {
        task_id: "task_0".to_string(),
        title: String::new(),
        status: "pending".to_string(),
        task_type: task_type.to_string(),
        owner: role.to_string(),
        risk: "low".to_string(),
        review: "auto".to_string(),
        review_status: "none".to_string(),
        writes: "[]".to_string(),
        dependencies: "[]".to_string(),
    }
}

fn build_collaboration_runtime_payload(turn: &AgentCollaborationTurn) -> Value {
    json!({
        "mode": turn.mode.clone(),
        "turn_role": turn.role.clone(),
        "turn_task_id": turn.task.task_id.clone(),
        "turn_kind": turn.turn_kind.clone(),
        "pending_reviews": turn.pending_reviews.clone(),
        "blocked_handoff_reason": turn.blocked_handoff_reason.clone(),
    })
}

fn build_agent_turn_payload(turn: &AgentCollaborationTurn, batch_id: i64) -> Value {
    json!({
        "batch_id": batch_id,
        "mode": turn.mode.clone(),
        "role": turn.role.clone(),
        "task_id": turn.task.task_id.clone(),
        "title": turn.task.title.clone(),
        "turn_kind": turn.turn_kind.clone(),
        "pending_reviews": turn.pending_reviews.clone(),
        "blocked_handoff_reason": turn.blocked_handoff_reason.clone(),
        "decision_reason": turn.decision_reason.clone(),
    })
}

fn render_collaboration_prompt_suffix(
    turn: Option<&AgentCollaborationTurn>,
    review_policy: &str,
) -> String {
    let Some(turn) = turn else {
        return String::new();
    };

    if turn.turn_kind == "review_gate" {
        return format!(
            "\n\n[Role handoff]\nCurrent collaboration mode: role_handoff\nCurrent turn: reviewer gate for {}\nPending reviews: {}\n\nReviewer instructions:\n- Review the task output and regressions only.\n- If approved, update the task header to [COMPLETED] and set `- Review-Status: approved`.\n- If changes are required, update the task header to [PENDING] and set `- Review-Status: changes_requested`.\n- Do not implement unrelated code while acting as reviewer.\n",
            turn.task.task_id,
            if turn.pending_reviews.is_empty() {
                "(none)".to_string()
            } else {
                turn.pending_reviews.join(", ")
            }
        );
    }

    if task_needs_review(&turn.task, review_policy) {
        return format!(
            "\n\n[Role handoff]\nCurrent collaboration mode: role_handoff\nCurrent turn: {} for {}\n\nThis task requires reviewer approval before it is truly complete.\nWhen the implementation is ready:\n- keep the task header as [IN_PROGRESS]\n- set `- Review-Status: pending`\n- leave completion for the reviewer gate\n",
            turn.role, turn.task.task_id
        );
    }

    format!(
        "\n\n[Role handoff]\nCurrent collaboration mode: role_handoff\nCurrent turn: {} for {}\nComplete only the current turn before handing off to the next role.\n",
        turn.role, turn.task.task_id
    )
}

#[derive(Clone, Copy)]
struct AgentRuntimeSummaryContext<'a> {
    cfg: &'a FlatConfig,
    active_task: &'a ActiveTaskMetadata,
    batch_plan: Option<&'a AgentBatchPlan>,
    collaboration_turn: Option<&'a AgentCollaborationTurn>,
    role: &'a str,
    route_reason: &'a str,
}

#[derive(Clone, Copy)]
struct AgentEventContext<'a> {
    role: &'a str,
    route_reason: &'a str,
    batch_id: i64,
    primary_backend: &'a str,
    fallback_backend: &'a str,
    used_backend: Option<&'a str>,
    turn_kind: Option<&'a str>,
}

fn persist_agent_runtime_summary(
    sessions_path: &Path,
    sessions: &mut Value,
    context: &AgentRuntimeSummaryContext<'_>,
) -> Result<i64> {
    let current_batch_id = sessions
        .get("_runtime")
        .and_then(|value| value.get("agents"))
        .and_then(|value| value.get("current_batch_id"))
        .and_then(|value| value.as_i64())
        .unwrap_or(0)
        + 1;

    let root = ensure_object(sessions);
    let runtime = ensure_child_object(root, "_runtime");
    let current_batch_tasks: Vec<String> = context
        .batch_plan
        .map(|plan| {
            plan.current_batch_tasks
                .iter()
                .map(|task| task.task_id.clone())
                .collect()
        })
        .unwrap_or_else(|| vec![context.active_task.task_id.clone()]);
    let active_roles: Vec<String> = context
        .batch_plan
        .map(|plan| plan.active_roles.clone())
        .unwrap_or_else(|| vec![context.role.to_string()]);
    let blocked_tasks: Vec<String> = context
        .batch_plan
        .map(|plan| plan.blocked_tasks.clone())
        .unwrap_or_default();
    let review_queue: Vec<String> = context
        .batch_plan
        .map(|plan| plan.review_queue.clone())
        .unwrap_or_default();
    let policy = context
        .batch_plan
        .and_then(|plan| build_agent_policy_payload(plan, &context.cfg.agent_explain_level));
    let collaboration = context
        .collaboration_turn
        .map(build_collaboration_runtime_payload)
        .unwrap_or(Value::Null);

    runtime.insert("version".to_string(), Value::String("2.6.3".to_string()));
    runtime.insert(
        "agents".to_string(),
        json!({
            "enabled": true,
            "mode": context.cfg.agent_mode,
            "review_policy": context.cfg.agent_review_policy,
            "explain_level": context.cfg.agent_explain_level,
            "current_batch_id": current_batch_id,
            "active_roles": active_roles,
            "current_batch_tasks": current_batch_tasks,
            "blocked_tasks": blocked_tasks,
            "review_queue": review_queue,
            "review_queue_size": context
                .batch_plan
                .map(|plan| plan.review_queue.len())
                .unwrap_or(0),
            "last_decision_reason": context.route_reason,
            "policy": policy,
            "collaboration": collaboration,
        }),
    );
    runtime.insert(
        "scheduler".to_string(),
        json!({
            "enabled": context.cfg.scheduler_enabled,
            "current_batch_id": current_batch_id,
            "parallel_tasks": context
                .batch_plan
                .map(|plan| plan.parallel_tasks)
                .unwrap_or(1),
        }),
    );

    write_json_pretty(sessions_path, sessions)?;
    Ok(current_batch_id)
}

fn build_agent_policy_payload(batch_plan: &AgentBatchPlan, explain_level: &str) -> Option<Value> {
    let mut policy = Map::new();
    policy.insert(
        "batch_reason".to_string(),
        Value::String(batch_plan.batch_reason.clone()),
    );

    if explain_level != "off" {
        if !batch_plan.selected_reasons.is_empty() {
            policy.insert(
                "selected_reasons".to_string(),
                json!(batch_plan.selected_reasons),
            );
        }
        if !batch_plan.blocked_reasons.is_empty() {
            policy.insert(
                "blocked_reasons".to_string(),
                json!(batch_plan.blocked_reasons),
            );
        }
        if !batch_plan.review_reasons.is_empty() {
            policy.insert(
                "review_reasons".to_string(),
                json!(batch_plan.review_reasons),
            );
        }
    }

    (!policy.is_empty()).then_some(Value::Object(policy))
}

fn build_agent_batch_payload(
    batch_plan: &AgentBatchPlan,
    batch_id: i64,
    primary_backend: &str,
    fallback_backend: &str,
    explain_level: &str,
) -> Value {
    let mut payload = json!({
        "batch_id": batch_id,
        "selected_tasks": batch_plan
            .current_batch_tasks
            .iter()
            .map(|task| task.task_id.clone())
            .collect::<Vec<_>>(),
        "blocked_tasks": batch_plan.blocked_tasks.clone(),
        "active_roles": batch_plan.active_roles.clone(),
        "review_queue": batch_plan.review_queue.clone(),
        "parallel_tasks": batch_plan.parallel_tasks,
        "batch_reason": batch_plan.batch_reason.clone(),
        "primary_backend": primary_backend,
        "fallback_backend": fallback_backend,
    });
    if let Some(policy) = build_agent_policy_payload(batch_plan, explain_level).filter(|_| {
        !batch_plan.selected_reasons.is_empty()
            || !batch_plan.blocked_reasons.is_empty()
            || !batch_plan.review_reasons.is_empty()
            || explain_level == "off"
    }) {
        payload
            .as_object_mut()
            .expect("agent batch payload should be object")
            .insert("policy".to_string(), policy);
    }
    payload
}

fn build_agent_event_payload(task: &ActiveTaskMetadata, context: &AgentEventContext<'_>) -> Value {
    let mut payload = json!({
        "task_id": task.task_id,
        "title": task.title,
        "status": task.status,
        "task_type": task.task_type,
        "owner": task.owner,
        "role": context.role,
        "risk": task.risk,
        "review": task.review,
        "review_status": task.review_status,
        "writes": task.writes,
        "dependencies": task.dependencies,
        "decision_reason": context.route_reason,
        "batch_id": context.batch_id,
        "primary_backend": context.primary_backend,
        "fallback_backend": context.fallback_backend,
    });
    if let Some(turn_kind) = context.turn_kind {
        payload
            .as_object_mut()
            .expect("payload should be object")
            .insert(
                "turn_kind".to_string(),
                Value::String(turn_kind.to_string()),
            );
    }
    if let Some(used_backend) = context.used_backend {
        payload
            .as_object_mut()
            .expect("payload should be object")
            .insert(
                "used_backend".to_string(),
                Value::String(used_backend.to_string()),
            );
    }
    payload
}

fn execute_codeagent(
    fusion_dir: &Path,
    phase: &str,
    prompt_args: &[String],
) -> Result<CodeagentExecution> {
    ensure_fusion_dir(fusion_dir)?;
    normalize_task_plan_owners(fusion_dir)?;

    let cwd = std::env::current_dir().context("failed reading cwd")?;
    let explicit_bin = std::env::var("CODEAGENT_WRAPPER_BIN").ok();
    let report_source = codeagent_report_source();

    let wrapper = match resolve_wrapper_or_dependency_error(
        fusion_dir,
        &cwd,
        explicit_bin.as_deref(),
        &report_source,
        codeagent_rerun_command(&report_source),
        clear_backend_failure_report,
    )? {
        Ok(resolved) => resolved,
        Err((output, exit_code)) => {
            return Ok(CodeagentExecution { output, exit_code });
        }
    };

    remove_dependency_report_if_exists(fusion_dir)?;

    let sessions_path = fusion_dir.join("sessions.json");
    let mut sessions = read_json(&sessions_path)?;
    let goal = json_get_string(&sessions, &["goal"]).unwrap_or_default();
    let phase = phase.trim().to_ascii_uppercase();
    let cfg = load_flat_config(fusion_dir);
    let batch_plan = if cfg.agent_enabled {
        plan_agent_batch(fusion_dir, &cfg)?
    } else {
        None
    };
    let collaboration_turn = if cfg.agent_enabled && cfg.agent_mode == "role_handoff" {
        batch_plan
            .as_ref()
            .and_then(|plan| plan_role_handoff_turn(plan, &cfg.agent_review_policy))
    } else {
        None
    };
    let next_task = collaboration_turn
        .as_ref()
        .map(|turn| turn.task.clone())
        .or_else(|| {
            batch_plan
                .as_ref()
                .and_then(|plan| plan.current_batch_tasks.first().cloned())
        })
        .or(extract_next_task_metadata(fusion_dir)?);
    let route_owner = collaboration_turn
        .as_ref()
        .map(|turn| turn.role.clone())
        .unwrap_or_else(|| {
            next_task
                .as_ref()
                .map(|task| task.owner.clone())
                .unwrap_or_default()
        });
    let next_task_type = next_task
        .as_ref()
        .map(|task| task.task_type.clone())
        .unwrap_or_default();
    let route = resolve_codeagent_route(&cfg, &phase, &next_task_type, &route_owner);
    let role = route.role;
    let role_source = route.role_source;
    let role_is_explicit = route.role_is_explicit;
    let primary = route.primary;
    let fallback = route.fallback;
    let route_reason = route.route_reason;
    let active_task = collaboration_turn
        .as_ref()
        .map(|turn| turn.task.clone())
        .or(next_task)
        .unwrap_or_else(|| synthesize_active_task(&role));

    eprintln!(
        "[fusion] route: role={} role_source={} phase={} task_type={} owner={} -> {} (fallback={}, reason={})",
        if role.is_empty() { "unknown" } else { &role },
        role_source,
        phase,
        if next_task_type.is_empty() {
            "unknown"
        } else {
            &next_task_type
        },
        if route_owner.is_empty() {
            "none"
        } else {
            &route_owner
        },
        primary,
        fallback,
        route_reason,
    );

    let session_role = role_is_explicit.then_some(role.as_str());
    let primary_session = lookup_primary_session(&sessions, &primary, session_role);

    let agent_batch_id = if cfg.agent_enabled {
        let summary_context = AgentRuntimeSummaryContext {
            cfg: &cfg,
            active_task: &active_task,
            batch_plan: batch_plan.as_ref(),
            collaboration_turn: collaboration_turn.as_ref(),
            role: &role,
            route_reason: &route_reason,
        };
        Some(persist_agent_runtime_summary(
            &sessions_path,
            &mut sessions,
            &summary_context,
        )?)
    } else {
        None
    };
    if let Some(batch_id) = agent_batch_id {
        if let Some(batch_plan) = batch_plan.as_ref() {
            let _ = append_event(
                fusion_dir,
                "AGENT_BATCH_PLANNED",
                &phase,
                &phase,
                build_agent_batch_payload(
                    batch_plan,
                    batch_id,
                    &primary,
                    &fallback,
                    &cfg.agent_explain_level,
                ),
                &format!("agent:{batch_id}:batch"),
            );
        }
        if let Some(turn) = collaboration_turn.as_ref() {
            let _ = append_event(
                fusion_dir,
                "AGENT_HANDOFF_PLANNED",
                &phase,
                &phase,
                build_agent_turn_payload(turn, batch_id),
                &format!("agent:{batch_id}:handoff:{}", turn.task.task_id),
            );
        }
        let base_event_context = AgentEventContext {
            role: &role,
            route_reason: &route_reason,
            batch_id,
            primary_backend: &primary,
            fallback_backend: &fallback,
            used_backend: None,
            turn_kind: collaboration_turn
                .as_ref()
                .map(|turn| turn.turn_kind.as_str()),
        };
        let base_payload = build_agent_event_payload(&active_task, &base_event_context);
        let _ = append_event(
            fusion_dir,
            "AGENT_TASK_ASSIGNED",
            &phase,
            &phase,
            base_payload.clone(),
            &format!("agent:{batch_id}:assigned:{}", active_task.task_id),
        );
        if collaboration_turn.is_some() {
            let _ = append_event(
                fusion_dir,
                "AGENT_ROLE_TURN_STARTED",
                &phase,
                &phase,
                base_payload.clone(),
                &format!("agent:{batch_id}:turn-started:{}", active_task.task_id),
            );
        }
        let _ = append_event(
            fusion_dir,
            "AGENT_TASK_STARTED",
            &phase,
            &phase,
            base_payload,
            &format!("agent:{batch_id}:started:{}", active_task.task_id),
        );
        sessions = read_json(&sessions_path)?;
    }

    let mut prompt = if prompt_args.is_empty() {
        render_prompt(fusion_dir, &phase, &goal, &role)?
    } else {
        prompt_args.join(" ")
    };
    prompt.push_str(&render_collaboration_prompt_suffix(
        collaboration_turn.as_ref(),
        &cfg.agent_review_policy,
    ));

    let backend_outcome = execute_backend_with_fallback(
        &wrapper,
        &primary,
        &fallback,
        &prompt,
        primary_session.as_deref(),
        &cwd,
        codeagent_timeout_sec(),
    )?;
    let output = backend_outcome.output;
    let exit_code = backend_outcome.exit_code;
    let used_backend = backend_outcome.used_backend;
    let primary_ok = backend_outcome.primary_ok;
    let primary_error = backend_outcome.primary_error;
    let fallback_error = backend_outcome.fallback_error;

    if primary_ok {
        persist_backend_success(
            fusion_dir,
            &sessions_path,
            &mut sessions,
            &output,
            &used_backend,
            session_role,
        )?;
    } else {
        write_backend_failure_report(
            fusion_dir,
            &report_source,
            &primary,
            &fallback,
            &primary_error,
            &fallback_error,
        )?;
        eprintln!(
            "[fusion][deps] Backend failure report written: {}",
            fusion_dir.join("backend_failure_report.json").display()
        );
    }

    let updated_task = if exit_code == 0 {
        extract_task_metadata_by_id(fusion_dir, &active_task.task_id)?
    } else {
        None
    };

    if let Some(batch_id) = agent_batch_id {
        if used_backend != primary {
            let fallback_event_context = AgentEventContext {
                role: &role,
                route_reason: &route_reason,
                batch_id,
                primary_backend: &primary,
                fallback_backend: &fallback,
                used_backend: Some(&used_backend),
                turn_kind: collaboration_turn
                    .as_ref()
                    .map(|turn| turn.turn_kind.as_str()),
            };
            let _ = append_event(
                fusion_dir,
                "AGENT_FALLBACK_USED",
                &phase,
                &phase,
                build_agent_event_payload(&active_task, &fallback_event_context),
                &format!(
                    "agent:{batch_id}:fallback:{}:{used_backend}",
                    active_task.task_id
                ),
            );
        }
        if exit_code == 0 {
            let completed_payload_task = updated_task.as_ref().unwrap_or(&active_task);
            let completed_event_context = AgentEventContext {
                role: &role,
                route_reason: &route_reason,
                batch_id,
                primary_backend: &primary,
                fallback_backend: &fallback,
                used_backend: Some(&used_backend),
                turn_kind: collaboration_turn
                    .as_ref()
                    .map(|turn| turn.turn_kind.as_str()),
            };
            if collaboration_turn.is_some() {
                let _ = append_event(
                    fusion_dir,
                    "AGENT_ROLE_TURN_COMPLETED",
                    &phase,
                    &phase,
                    build_agent_event_payload(completed_payload_task, &completed_event_context),
                    &format!(
                        "agent:{batch_id}:turn-completed:{}:{used_backend}",
                        active_task.task_id
                    ),
                );
            }
            if let Some(turn) = collaboration_turn.as_ref() {
                if turn.turn_kind == "review_gate" {
                    let review_gate_context = AgentEventContext {
                        turn_kind: Some("review_gate"),
                        ..completed_event_context
                    };
                    match completed_payload_task.review_status.as_str() {
                        "approved" => {
                            let _ = append_event(
                                fusion_dir,
                                "AGENT_REVIEW_APPROVED",
                                &phase,
                                &phase,
                                build_agent_event_payload(
                                    completed_payload_task,
                                    &review_gate_context,
                                ),
                                &format!(
                                    "agent:{batch_id}:review-approved:{}:{used_backend}",
                                    active_task.task_id
                                ),
                            );
                        }
                        "changes_requested" => {
                            let _ = append_event(
                                fusion_dir,
                                "AGENT_REVIEW_CHANGES_REQUESTED",
                                &phase,
                                &phase,
                                build_agent_event_payload(
                                    completed_payload_task,
                                    &review_gate_context,
                                ),
                                &format!(
                                    "agent:{batch_id}:review-changes:{}:{used_backend}",
                                    active_task.task_id
                                ),
                            );
                        }
                        _ => {}
                    }
                } else if task_has_pending_review(completed_payload_task, &cfg.agent_review_policy)
                {
                    let _ = append_event(
                        fusion_dir,
                        "AGENT_REVIEW_REQUESTED",
                        &phase,
                        &phase,
                        build_agent_event_payload(
                            completed_payload_task,
                            &AgentEventContext {
                                turn_kind: Some("task"),
                                ..completed_event_context
                            },
                        ),
                        &format!(
                            "agent:{batch_id}:review-requested:{}:{used_backend}",
                            active_task.task_id
                        ),
                    );
                }
            }
            if task_is_effectively_completed(completed_payload_task, &cfg.agent_review_policy) {
                let _ = append_event(
                    fusion_dir,
                    "AGENT_TASK_COMPLETED",
                    &phase,
                    &phase,
                    build_agent_event_payload(completed_payload_task, &completed_event_context),
                    &format!(
                        "agent:{batch_id}:completed:{}:{used_backend}",
                        active_task.task_id
                    ),
                );
            }
        }
    }

    Ok(CodeagentExecution { output, exit_code })
}

pub(crate) fn cmd_codeagent(fusion_dir: &Path, phase: &str, prompt_args: &[String]) -> Result<()> {
    let run = execute_codeagent(fusion_dir, phase, prompt_args)?;

    if run.exit_code != 0 {
        eprint!("{}", run.output);
        std::process::exit(run.exit_code.max(1));
    }

    print!("{}", run.output);
    Ok(())
}

pub(crate) fn cmd_run(fusion_dir: &Path, options: RunOptions) -> Result<()> {
    ensure_fusion_dir(fusion_dir)?;

    let max_iterations = options.max_iterations.max(1);
    let max_no_progress_rounds = options.max_no_progress_rounds.max(1);
    let mut no_progress_rounds: i64 = 0;
    let mut backoff_ms = options.initial_backoff_ms.max(1);

    let mut last_counts = read_task_counts(fusion_dir)?;

    for iteration in 1..=max_iterations {
        let guard = evaluate_stop_guard(fusion_dir)?;
        if guard.decision == "allow" || !guard.should_block {
            println!("[fusion][run] Loop completed after {iteration} iteration(s)");
            return Ok(());
        }

        println!(
            "[fusion][run] Iteration {}/{} | {}",
            iteration, max_iterations, guard.system_message
        );
        let sessions = read_json(&fusion_dir.join("sessions.json"))?;
        let current_phase =
            json_get_string(&sessions, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());
        let next_action = live_next_action_for_phase(
            fusion_dir,
            &current_phase,
            "Inspect .fusion/task_plan.md and continue from the next live step",
        )?;
        println!("[fusion] {}", render_next_action(&next_action));

        cmd_hook_pretool(fusion_dir)?;

        let sessions = read_json(&fusion_dir.join("sessions.json"))?;
        let phase =
            json_get_string(&sessions, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());

        let step = execute_codeagent(fusion_dir, &phase, &[])?;
        if !step.output.is_empty() {
            print!("{}", step.output);
        }
        if step.exit_code != 0 {
            eprintln!(
                "[fusion][run] codeagent step failed with exit code {}",
                step.exit_code
            );
            std::process::exit(step.exit_code.max(1));
        }

        cmd_hook_posttool(fusion_dir)?;

        let current_counts = read_task_counts(fusion_dir)?;
        let changed = current_counts.completed != last_counts.completed
            || current_counts.pending != last_counts.pending
            || current_counts.in_progress != last_counts.in_progress
            || current_counts.failed != last_counts.failed;

        if changed {
            no_progress_rounds = 0;
            backoff_ms = options.initial_backoff_ms.max(1);
            last_counts = current_counts;
            continue;
        }

        no_progress_rounds += 1;
        if no_progress_rounds >= max_no_progress_rounds {
            eprintln!(
                "[fusion][run] No progress rounds limit reached ({}/{})",
                no_progress_rounds, max_no_progress_rounds
            );
            std::process::exit(2);
        }

        let bounded_max_backoff = options
            .max_backoff_ms
            .max(options.initial_backoff_ms.max(1));
        let current_sleep = backoff_ms.min(bounded_max_backoff);
        println!(
            "[fusion][run] No progress detected ({}/{}), backoff {}ms",
            no_progress_rounds, max_no_progress_rounds, current_sleep
        );
        thread::sleep(Duration::from_millis(current_sleep));
        backoff_ms = backoff_ms.saturating_mul(2).min(bounded_max_backoff);
    }

    eprintln!(
        "[fusion][run] Max iterations reached ({}) before completion",
        max_iterations
    );
    std::process::exit(3);
}
