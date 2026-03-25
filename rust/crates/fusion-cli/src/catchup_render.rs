use fusion_runtime_io::json_get_string;
use serde_json::Value;

use crate::catchup_session::{truncate_chars, UnsyncedMessage};
use crate::catchup_taskplan::TaskPlanInfo;
use crate::render::{
    continue_task_next_action_from_parts, create_task_plan_next_action, live_next_action_fallback,
    proceed_to_verify_next_action, render_current_state, render_next_action,
    review_gate_next_action,
};

pub(crate) fn print_report(
    task_info: &TaskPlanInfo,
    session_info: &Value,
    git_diff: &str,
    warnings: &[String],
    unsynced: &[UnsyncedMessage],
    last_update_line: isize,
    last_update_file: Option<&str>,
) {
    println!();
    println!("[fusion-catchup] SESSION RECOVERY REPORT");
    println!("{}", "=".repeat(60));
    println!();

    let goal = json_get_string(session_info, &["goal"]).unwrap_or_else(|| "?".to_string());
    let phase =
        json_get_string(session_info, &["current_phase"]).unwrap_or_else(|| "?".to_string());
    let status = json_get_string(session_info, &["status"]).unwrap_or_else(|| "?".to_string());
    let codex_session = json_get_string(session_info, &["codex_session"]).unwrap_or_default();

    println!("Goal: {goal}");
    println!("{}", render_current_state(&status, &phase));
    print!(
        "Tasks: {}/{} completed",
        task_info.completed, task_info.total
    );
    if task_info.in_progress > 0 {
        print!(" | {} in progress", task_info.in_progress);
    }
    if task_info.failed > 0 {
        print!(" | {} failed", task_info.failed);
    }
    println!();

    if !codex_session.is_empty() {
        println!("Codex Session: {codex_session}");
    }

    if !warnings.is_empty() {
        println!();
        println!("--- WARNINGS ({}) ---", warnings.len());
        for warning in warnings {
            println!("  ⚠ {warning}");
        }
    }

    if !git_diff.is_empty() {
        println!();
        println!("--- UNCOMMITTED CHANGES ---");
        for line in git_diff.lines().take(10) {
            println!("  {line}");
        }
    }

    if !unsynced.is_empty() {
        println!();
        println!("--- UNSYNCED CONTEXT ({}) messages ---", unsynced.len());
        if last_update_line >= 0 {
            if let Some(file_name) = last_update_file {
                println!("Last .fusion update: {file_name} at line #{last_update_line}");
            }
        }
        for message in unsynced
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            match message.role {
                "user" => println!("  USER: {}", truncate_chars(&message.content, 200)),
                _ => {
                    if !message.content.is_empty() {
                        println!("  CLAUDE: {}", truncate_chars(&message.content, 200));
                    }
                    if !message.tools.is_empty() {
                        println!(
                            "    Tools: {}",
                            message
                                .tools
                                .iter()
                                .take(4)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                }
            }
        }
    }

    println!();
    println!("--- RECOVERY INSTRUCTIONS ---");

    let review_gate = task_info
        .tasks
        .iter()
        .find(|task| task.review_status == "pending");
    let next_task = task_info
        .tasks
        .iter()
        .find(|task| task.status == "IN_PROGRESS")
        .or_else(|| task_info.tasks.iter().find(|task| task.status == "PENDING"));

    if task_info.total == 0 {
        println!("{}", render_next_action(create_task_plan_next_action()));
        println!("  1. Read the goal and break it into explicit tasks");
        println!("  2. Save the plan into .fusion/task_plan.md");
    } else if let Some(task) = review_gate {
        println!(
            "{}",
            render_next_action(&review_gate_next_action(&task.task_id))
        );
        println!("  1. Review .fusion/task_plan.md for the pending gate");
        println!("  2. Resume the reviewer role and decide approve vs changes_requested");
    } else if let Some(task) = next_task {
        println!(
            "{}",
            render_next_action(&continue_task_next_action_from_parts(
                &task.name,
                &task.status
            ))
        );
        println!("  1. Read .fusion/task_plan.md for full context");
        println!("  2. Read .fusion/progress.md for recent history");
        if !codex_session.is_empty() {
            println!("  3. Resume Codex session: {codex_session}");
        }
    } else if task_info.pending == 0 && task_info.in_progress == 0 {
        println!("{}", render_next_action(proceed_to_verify_next_action()));
        println!("  1. Review .fusion/progress.md before VERIFY");
    } else {
        println!("{}", render_next_action(live_next_action_fallback()));
        println!("  1. Read .fusion/task_plan.md to find next action");
        println!("  2. Read .fusion/progress.md for recent history");
    }

    println!();
    println!("{}", "=".repeat(60));
}
