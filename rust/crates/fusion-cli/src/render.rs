use crate::models::TaskCounts;
use anyhow::Result;
use fusion_runtime_io::load_flat_config;
use std::path::Path;

pub(crate) use crate::render_status::{
    epoch_to_iso, extract_status_block, guardian_status_from_metrics, last_safe_backlog,
    read_guardian_status,
};
pub(crate) use crate::render_taskplan::{
    extract_all_task_metadata, extract_next_task_metadata, extract_next_task_type,
    extract_task_metadata_by_id, normalize_task_plan_owners, render_prompt,
    task_counts_from_metadata, task_has_pending_review, task_is_effectively_completed,
    task_needs_review, ActiveTaskMetadata,
};
pub(crate) use crate::render_tasks::{
    find_first_task_with_status, find_last_task_with_status, find_next_task,
};

pub(crate) fn phase_num(phase: &str) -> &'static str {
    match phase {
        "UNDERSTAND" => "0/8",
        "INITIALIZE" => "1/8",
        "ANALYZE" => "2/8",
        "DECOMPOSE" => "3/8",
        "EXECUTE" => "4/8",
        "VERIFY" => "5/8",
        "REVIEW" => "6/8",
        "COMMIT" => "7/8",
        "DELIVER" => "8/8",
        _ => "?/8",
    }
}

fn summary_value<'a>(value: &'a str, fallback: &'static str) -> &'a str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}

pub(crate) fn render_current_state(status: &str, phase: &str) -> String {
    format!(
        "Current state: {} @ {}",
        summary_value(status, "unknown"),
        summary_value(phase, "?")
    )
}

pub(crate) fn render_next_action(action: &str) -> String {
    format!(
        "Next action: {}",
        summary_value(action, live_next_action_fallback())
    )
}

pub(crate) fn live_next_action_fallback() -> &'static str {
    "Inspect .fusion/task_plan.md and continue from the next live step"
}

pub(crate) fn review_gate_next_action(task_id: &str) -> String {
    format!(
        "reviewer approve {} before execution continues",
        summary_value(task_id, "current task")
    )
}

pub(crate) fn review_gate_guidance(task_id: &str) -> String {
    format!("Review gate: {}", review_gate_next_action(task_id))
}

pub(crate) fn continue_task_next_action_from_parts(title: &str, status: &str) -> String {
    format!(
        "Continue task: {} [{}]",
        summary_value(title, "current task"),
        summary_value(status, "pending").to_ascii_uppercase()
    )
}

pub(crate) fn continue_task_next_action(task: &ActiveTaskMetadata) -> String {
    continue_task_next_action_from_parts(&task.title, &task.status)
}

pub(crate) fn create_task_plan_next_action() -> &'static str {
    "Create task plan and run the DECOMPOSE phase"
}

pub(crate) fn initialize_workspace_next_action() -> &'static str {
    "Initialize workspace files and proceed to ANALYZE"
}

pub(crate) fn proceed_to_verify_next_action() -> &'static str {
    "Proceed to VERIFY phase"
}

pub(crate) fn no_resume_needed_next_action() -> &'static str {
    "No resume needed"
}

pub(crate) fn stop_guard_continue_instructions() -> &'static str {
    r#"Instructions:
1. Read .fusion/task_plan.md
2. Find next PENDING or IN_PROGRESS task
3. Execute based on task type:
   - implementation/verification → TDD flow (RED→GREEN→REFACTOR)
   - design/documentation/configuration/research → direct execution
4. Update task status to [COMPLETED]
5. Continue until all tasks done

Only ask user if 3-Strike exhausted."#
}

pub(crate) fn stop_guard_decompose_reason(goal: &str, next_action: &str) -> String {
    format!(
        "Continue with task decomposition for goal: {}.\n\nNext action: {}\n1. Break the goal into explicit tasks\n2. Save them to .fusion/task_plan.md",
        summary_value(goal, "(not set)"),
        summary_value(next_action, create_task_plan_next_action())
    )
}

pub(crate) fn stop_guard_review_gate_reason(
    goal: &str,
    current_phase: &str,
    total_remaining: i64,
    task: &ActiveTaskMetadata,
    next_action: &str,
) -> String {
    let review_gate_fallback = review_gate_next_action(&task.task_id);
    let review_gate_display = if next_action.trim().is_empty() {
        review_gate_fallback.as_str()
    } else {
        next_action
    };

    format!(
        "Continue executing the Fusion workflow.\n\nGoal: {}\nPhase: {}\nRemaining: {} tasks\nReview gate: {}\nTask: {} ({})\n\nReviewer instructions:\n1. Read .fusion/task_plan.md\n2. Review the task output and regressions only\n3. If approved, set `- Review-Status: approved` and mark the task [COMPLETED]\n4. If changes are required, set `- Review-Status: changes_requested` and return it to implementation\n5. Continue only after the review decision is recorded",
        summary_value(goal, "(not set)"),
        summary_value(current_phase, "?"),
        total_remaining,
        review_gate_display,
        summary_value(&task.task_id, "current task"),
        summary_value(&task.title, "current task")
    )
}

pub(crate) fn stop_guard_continue_reason(
    goal: &str,
    current_phase: &str,
    total_remaining: i64,
    next_action: &str,
) -> String {
    format!(
        "Continue executing the Fusion workflow.\n\nGoal: {}\nPhase: {}\nRemaining: {} tasks\nNext action: {}\n\n{}",
        summary_value(goal, "(not set)"),
        summary_value(current_phase, "?"),
        total_remaining,
        summary_value(next_action, live_next_action_fallback()),
        stop_guard_continue_instructions()
    )
}

pub(crate) fn stop_guard_backlog_reason(
    goal: &str,
    current_phase: &str,
    total_remaining: i64,
) -> String {
    format!(
        "Continue executing the Fusion workflow.\n\nGoal: {}\nPhase: {}\nRemaining: {} tasks (safe_backlog tasks injected)\n\nSafe backlog tasks have been automatically added to maintain continuous development.\nThese are low-risk quality/documentation/optimization tasks.\n\n{}",
        summary_value(goal, "(not set)"),
        summary_value(current_phase, "?"),
        total_remaining,
        stop_guard_continue_instructions()
    )
}

pub(crate) fn stop_guard_phase_correction_note(current_phase: &str) -> String {
    format!(
        "\n\nNote: Phase auto-corrected to {} based on task states.",
        summary_value(current_phase, "?")
    )
}

pub(crate) fn posttool_next_action_line(action: &str) -> String {
    format!("[fusion] {}", render_next_action(action))
}

pub(crate) fn posttool_next_action_with_mode(action: &str, mode: &str) -> String {
    let next_line = posttool_next_action_line(action);
    let trimmed_mode = mode.trim();
    if trimmed_mode.is_empty() {
        next_line
    } else {
        format!("{next_line} | Mode: {trimmed_mode}")
    }
}

pub(crate) fn stop_guard_system_message(
    phase: &str,
    total_remaining: Option<i64>,
    next_action: &str,
) -> String {
    match total_remaining {
        Some(remaining) => format!(
            "🔄 Fusion | Phase: {} | Remaining: {} | Next: {}",
            summary_value(phase, "?"),
            remaining,
            summary_value(next_action, live_next_action_fallback())
        ),
        None => format!(
            "🔄 Fusion | Phase: {} | Next: {}",
            summary_value(phase, "?"),
            summary_value(next_action, live_next_action_fallback())
        ),
    }
}

pub(crate) fn truncate_chars(input: String, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}

pub(crate) fn read_task_counts(fusion_dir: &Path) -> Result<TaskCounts> {
    let cfg = load_flat_config(fusion_dir);
    let tasks = extract_all_task_metadata(fusion_dir)?;
    if tasks.is_empty() {
        return Ok(TaskCounts::default());
    }
    Ok(task_counts_from_metadata(&tasks, &cfg.agent_review_policy))
}

pub(crate) fn next_review_gate_task(fusion_dir: &Path) -> Result<Option<ActiveTaskMetadata>> {
    let cfg = load_flat_config(fusion_dir);
    Ok(extract_all_task_metadata(fusion_dir)?
        .into_iter()
        .find(|task| task_has_pending_review(task, &cfg.agent_review_policy)))
}

pub(crate) fn live_next_action_for_phase(
    fusion_dir: &Path,
    current_phase: &str,
    fallback: &str,
) -> Result<String> {
    if let Some(task) = next_review_gate_task(fusion_dir)? {
        return Ok(review_gate_next_action(&task.task_id));
    }

    if let Some(task) = extract_next_task_metadata(fusion_dir)? {
        return Ok(continue_task_next_action(&task));
    }

    let counts = read_task_counts(fusion_dir)?;
    if counts.total() == 0 && matches!(current_phase, "INITIALIZE" | "ANALYZE" | "DECOMPOSE") {
        return Ok(create_task_plan_next_action().to_string());
    }

    if counts.completed > 0 && counts.pending == 0 && counts.in_progress == 0 && counts.failed == 0
    {
        return Ok(proceed_to_verify_next_action().to_string());
    }

    Ok(summary_value(fallback, live_next_action_fallback()).to_string())
}

pub(crate) fn guidance_for_task(phase: &str, task_name: &str, task_type: &str) -> String {
    match task_type.trim().to_ascii_lowercase().as_str() {
        "implementation" | "verification" => "TDD flow: RED → GREEN → REFACTOR".to_string(),
        "design" | "documentation" | "configuration" | "research" => "Direct execution".to_string(),
        _ if phase == "EXECUTE" && !task_name.trim().is_empty() && task_name != "unknown" => {
            "Check task type in task_plan.md".to_string()
        }
        _ => String::new(),
    }
}

pub(crate) fn execution_mode_for_task_type(task_type: &str) -> &'static str {
    match task_type.trim().to_ascii_lowercase().as_str() {
        "implementation" | "verification" => "TDD",
        _ => "Direct",
    }
}

pub(crate) fn default_role_for_phase(phase: &str) -> &'static str {
    match phase {
        "UNDERSTAND" | "INITIALIZE" | "ANALYZE" | "DECOMPOSE" => "planner",
        "VERIFY" | "REVIEW" => "reviewer",
        "EXECUTE" | "COMMIT" | "DELIVER" => "coder",
        _ => "coder",
    }
}

pub(crate) fn normalize_role(role_raw: &str) -> Option<String> {
    match role_raw.trim().to_ascii_lowercase().as_str() {
        "planner" => Some("planner".to_string()),
        "coder" => Some("coder".to_string()),
        "reviewer" => Some("reviewer".to_string()),
        _ => None,
    }
}

pub(crate) fn backend_for_role(role: &str) -> Option<&'static str> {
    match role {
        "planner" | "reviewer" => Some("codex"),
        "coder" => Some("claude"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        continue_task_next_action, continue_task_next_action_from_parts,
        initialize_workspace_next_action, no_resume_needed_next_action, posttool_next_action_line,
        posttool_next_action_with_mode, render_current_state, render_next_action,
        review_gate_guidance, review_gate_next_action, stop_guard_backlog_reason,
        stop_guard_continue_reason, stop_guard_decompose_reason, stop_guard_phase_correction_note,
        stop_guard_review_gate_reason, stop_guard_system_message, ActiveTaskMetadata,
    };

    #[test]
    fn render_current_state_uses_fallbacks_for_empty_values() {
        assert_eq!(render_current_state("", ""), "Current state: unknown @ ?");
    }

    #[test]
    fn render_next_action_uses_fallback_for_empty_value() {
        assert!(render_next_action("").contains("Inspect .fusion/task_plan.md"));
    }

    #[test]
    fn review_gate_next_action_renders_task_id() {
        assert_eq!(
            review_gate_next_action("task_2"),
            "reviewer approve task_2 before execution continues"
        );
    }

    #[test]
    fn review_gate_guidance_wraps_reviewer_action() {
        assert_eq!(
            review_gate_guidance("task_2"),
            "Review gate: reviewer approve task_2 before execution continues"
        );
    }

    #[test]
    fn continue_task_next_action_uppercases_status() {
        assert_eq!(
            continue_task_next_action_from_parts("Build API", "pending"),
            "Continue task: Build API [PENDING]"
        );
    }

    #[test]
    fn continue_task_next_action_delegates_to_parts_helper() {
        let task = ActiveTaskMetadata {
            task_id: "task_1".to_string(),
            title: "Build API".to_string(),
            status: "pending".to_string(),
            task_type: "implementation".to_string(),
            owner: "coder".to_string(),
            risk: "low".to_string(),
            review: "auto".to_string(),
            review_status: "none".to_string(),
            writes: "[]".to_string(),
            dependencies: "[]".to_string(),
        };
        assert_eq!(
            continue_task_next_action(&task),
            "Continue task: Build API [PENDING]"
        );
    }

    #[test]
    fn posttool_next_action_line_wraps_rendered_next_action() {
        assert_eq!(
            posttool_next_action_line("Proceed to VERIFY phase"),
            "[fusion] Next action: Proceed to VERIFY phase"
        );
    }

    #[test]
    fn posttool_next_action_with_mode_appends_mode() {
        assert_eq!(
            posttool_next_action_with_mode("Continue task: Build API [PENDING]", "TDD"),
            "[fusion] Next action: Continue task: Build API [PENDING] | Mode: TDD"
        );
    }

    #[test]
    fn initialize_workspace_next_action_matches_live_surface() {
        assert_eq!(
            initialize_workspace_next_action(),
            "Initialize workspace files and proceed to ANALYZE"
        );
    }

    #[test]
    fn no_resume_needed_next_action_matches_live_surface() {
        assert_eq!(no_resume_needed_next_action(), "No resume needed");
    }

    #[test]
    fn stop_guard_decompose_reason_uses_fallback_next_action() {
        assert!(stop_guard_decompose_reason("拆任务", "")
            .contains("Next action: Create task plan and run the DECOMPOSE phase"));
    }

    #[test]
    fn stop_guard_review_gate_reason_uses_review_gate_fallback() {
        let task = ActiveTaskMetadata {
            task_id: "task_2".to_string(),
            title: "Build API".to_string(),
            status: "in_progress".to_string(),
            task_type: "implementation".to_string(),
            owner: "coder".to_string(),
            risk: "high".to_string(),
            review: "required".to_string(),
            review_status: "pending".to_string(),
            writes: "[]".to_string(),
            dependencies: "[]".to_string(),
        };

        let reason = stop_guard_review_gate_reason("review gate", "EXECUTE", 1, &task, "");
        assert!(reason.contains("Review gate: reviewer approve task_2 before execution continues"));
        assert!(reason.contains("Task: task_2 (Build API)"));
    }

    #[test]
    fn stop_guard_continue_reason_includes_next_action_and_instructions() {
        let reason =
            stop_guard_continue_reason("继续执行", "EXECUTE", 1, "Continue task: A [PENDING]");
        assert!(reason.contains("Next action: Continue task: A [PENDING]"));
        assert!(reason.contains("Only ask user if 3-Strike exhausted."));
    }

    #[test]
    fn stop_guard_backlog_reason_mentions_safe_backlog() {
        let reason = stop_guard_backlog_reason("继续执行", "VERIFY", 2);
        assert!(reason.contains("Remaining: 2 tasks (safe_backlog tasks injected)"));
        assert!(reason.contains("Only ask user if 3-Strike exhausted."));
    }

    #[test]
    fn stop_guard_phase_correction_note_matches_live_surface() {
        assert_eq!(
            stop_guard_phase_correction_note("VERIFY"),
            "\n\nNote: Phase auto-corrected to VERIFY based on task states."
        );
    }

    #[test]
    fn stop_guard_system_message_renders_remaining_and_next_action() {
        assert_eq!(
            stop_guard_system_message("EXECUTE", Some(2), "Continue task: Build API [PENDING]"),
            "🔄 Fusion | Phase: EXECUTE | Remaining: 2 | Next: Continue task: Build API [PENDING]"
        );
    }

    #[test]
    fn stop_guard_system_message_without_remaining_renders_decompose_surface() {
        assert_eq!(
            stop_guard_system_message(
                "INITIALIZE",
                None,
                "Create task plan and run the DECOMPOSE phase"
            ),
            "🔄 Fusion | Phase: INITIALIZE | Next: Create task plan and run the DECOMPOSE phase"
        );
    }
}
