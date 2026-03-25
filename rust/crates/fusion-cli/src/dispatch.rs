use anyhow::Result;

use crate::cli::{
    GitCommands, HookCommands, InspectCommands, LoopContextInspectCommands, TaskPlanInspectCommands,
};
use crate::git::{
    cmd_branch, cmd_changes, cmd_cleanup, cmd_commit, cmd_create_branch, cmd_diff, cmd_status,
};
use crate::hooks::{cmd_hook_posttool, cmd_hook_pretool, cmd_hook_set_goal, cmd_hook_stop_guard};
use crate::inspect::{
    cmd_json_field, cmd_loop_context_array_values, cmd_loop_context_decision_history,
    cmd_loop_context_state_visits, cmd_loop_guardian_config, cmd_runtime_config,
    cmd_task_plan_counts, cmd_task_plan_first, cmd_task_plan_last, cmd_task_plan_next,
    cmd_task_plan_task_type,
};

pub(crate) fn dispatch_inspect(command: InspectCommands) -> Result<()> {
    match command {
        InspectCommands::JsonField {
            key,
            file,
            number,
            bool,
        } => cmd_json_field(file.as_deref(), &key, number, bool),
        InspectCommands::RuntimeConfig { fusion_dir, field } => {
            cmd_runtime_config(&fusion_dir, &field)
        }
        InspectCommands::LoopContext { file, query } => match query {
            LoopContextInspectCommands::ArrayValues { key } => {
                cmd_loop_context_array_values(&file, &key)
            }
            LoopContextInspectCommands::StateVisits => cmd_loop_context_state_visits(&file),
            LoopContextInspectCommands::DecisionHistory => cmd_loop_context_decision_history(&file),
        },
        InspectCommands::LoopGuardianConfig { fusion_dir, field } => {
            cmd_loop_guardian_config(&fusion_dir, &field)
        }
        InspectCommands::TaskPlan { file, query } => match query {
            TaskPlanInspectCommands::Counts => cmd_task_plan_counts(&file),
            TaskPlanInspectCommands::First { status } => cmd_task_plan_first(&file, &status),
            TaskPlanInspectCommands::Last { status } => cmd_task_plan_last(&file, &status),
            TaskPlanInspectCommands::Next => cmd_task_plan_next(&file),
            TaskPlanInspectCommands::TaskType { title } => cmd_task_plan_task_type(&file, &title),
        },
    }
}

pub(crate) fn dispatch_git(command: GitCommands) -> Result<()> {
    match command {
        GitCommands::Status => cmd_status(),
        GitCommands::CreateBranch { goal_slug } => cmd_create_branch(&goal_slug),
        GitCommands::Commit {
            message,
            task_id: _,
        } => cmd_commit(&message),
        GitCommands::Branch => cmd_branch(),
        GitCommands::Changes => cmd_changes(),
        GitCommands::Diff => cmd_diff(),
        GitCommands::Cleanup { original_branch } => cmd_cleanup(original_branch.as_deref()),
    }
}

pub(crate) fn dispatch_hook(command: HookCommands) -> Result<()> {
    match command {
        HookCommands::Pretool { fusion_dir } => cmd_hook_pretool(&fusion_dir),
        HookCommands::Posttool { fusion_dir } => cmd_hook_posttool(&fusion_dir),
        HookCommands::StopGuard { fusion_dir } => cmd_hook_stop_guard(&fusion_dir),
        HookCommands::SetGoal { fusion_dir, goal } => cmd_hook_set_goal(&fusion_dir, &goal),
    }
}
