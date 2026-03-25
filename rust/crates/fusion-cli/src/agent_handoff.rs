use crate::agent_orchestrator::AgentBatchPlan;
use crate::render::{task_has_pending_review, ActiveTaskMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentCollaborationTurn {
    pub(crate) mode: String,
    pub(crate) role: String,
    pub(crate) task: ActiveTaskMetadata,
    pub(crate) turn_kind: String,
    pub(crate) pending_reviews: Vec<String>,
    pub(crate) blocked_handoff_reason: String,
    pub(crate) decision_reason: String,
}

fn first_task_for_owner<'a>(
    tasks: &'a [ActiveTaskMetadata],
    owner: &str,
) -> Option<&'a ActiveTaskMetadata> {
    tasks.iter().find(|task| task.owner == owner)
}

pub(crate) fn plan_role_handoff_turn(
    batch_plan: &AgentBatchPlan,
    review_policy: &str,
) -> Option<AgentCollaborationTurn> {
    let tasks = &batch_plan.current_batch_tasks;
    if tasks.is_empty() {
        return None;
    }

    let pending_reviews: Vec<String> = tasks
        .iter()
        .filter(|task| task_has_pending_review(task, review_policy))
        .map(|task| task.task_id.clone())
        .collect();
    if let Some(task) = tasks
        .iter()
        .find(|task| task_has_pending_review(task, review_policy))
    {
        return Some(AgentCollaborationTurn {
            mode: "role_handoff".to_string(),
            role: "reviewer".to_string(),
            task: task.clone(),
            turn_kind: "review_gate".to_string(),
            pending_reviews,
            blocked_handoff_reason: format!("awaiting_review_approval:{}", task.task_id),
            decision_reason: "role_handoff:review_gate".to_string(),
        });
    }

    if let Some(task) = first_task_for_owner(tasks, "planner") {
        let blocked_handoff_reason = if tasks.iter().any(|candidate| candidate.owner != "planner") {
            "awaiting_planner_completion".to_string()
        } else {
            String::new()
        };
        return Some(AgentCollaborationTurn {
            mode: "role_handoff".to_string(),
            role: "planner".to_string(),
            task: task.clone(),
            turn_kind: "task".to_string(),
            pending_reviews,
            blocked_handoff_reason,
            decision_reason: "role_handoff:planner_stage".to_string(),
        });
    }

    if let Some(task) = first_task_for_owner(tasks, "coder") {
        let blocked_handoff_reason = first_task_for_owner(tasks, "reviewer")
            .map(|_| "awaiting_coder_completion".to_string())
            .unwrap_or_default();
        return Some(AgentCollaborationTurn {
            mode: "role_handoff".to_string(),
            role: "coder".to_string(),
            task: task.clone(),
            turn_kind: "task".to_string(),
            pending_reviews,
            blocked_handoff_reason,
            decision_reason: "role_handoff:coder_stage".to_string(),
        });
    }

    if let Some(task) = first_task_for_owner(tasks, "reviewer") {
        return Some(AgentCollaborationTurn {
            mode: "role_handoff".to_string(),
            role: "reviewer".to_string(),
            task: task.clone(),
            turn_kind: "task".to_string(),
            pending_reviews,
            blocked_handoff_reason: String::new(),
            decision_reason: "role_handoff:reviewer_stage".to_string(),
        });
    }

    let task = tasks.first()?.clone();
    Some(AgentCollaborationTurn {
        mode: "role_handoff".to_string(),
        role: task.owner.clone(),
        task,
        turn_kind: "task".to_string(),
        pending_reviews,
        blocked_handoff_reason: String::new(),
        decision_reason: "role_handoff:fallback_owner".to_string(),
    })
}
