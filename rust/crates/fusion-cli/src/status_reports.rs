use anyhow::Result;
use fusion_runtime_io::read_text;
use std::env;
use std::path::{Path, PathBuf};

use crate::achievements::{collect_achievement_leaderboard, collect_achievement_summary};
use crate::render::extract_status_block;
use crate::status::StatusSummary;
use crate::status_artifacts::{read_backend_failure_summary, read_dependency_summary};

pub(crate) fn print_task_plan(fusion_dir: &Path) -> Result<()> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok(());
    }

    println!("## Task Plan");
    let content = read_text(&task_plan)?;
    if let Some(block) = extract_status_block(&content) {
        print!("{block}");
        if !block.ends_with('\n') {
            println!();
        }
    } else {
        println!("No status found");
    }
    println!();
    Ok(())
}

pub(crate) fn print_progress(fusion_dir: &Path) -> Result<()> {
    let progress = fusion_dir.join("progress.md");
    if !progress.is_file() {
        return Ok(());
    }

    println!("## Recent Progress (last 10 entries)");
    let content = read_text(&progress)?;
    let mut rows: Vec<&str> = content
        .lines()
        .filter(|line| line.starts_with('|'))
        .collect();
    if rows.len() > 12 {
        rows = rows[rows.len() - 12..].to_vec();
    }
    for row in rows {
        println!("{row}");
    }
    println!();

    let error_lines: Vec<&str> = content
        .lines()
        .filter(|line| line.contains("ERROR") || line.contains("FAILED"))
        .collect();
    if !error_lines.is_empty() {
        println!("## Errors: {} found", error_lines.len());
        for row in error_lines.iter().rev().take(5).rev() {
            println!("{row}");
        }
    }

    Ok(())
}

pub(crate) fn print_team_roles(summary: &StatusSummary) {
    if summary.owner_planner <= 0
        && summary.owner_coder <= 0
        && summary.owner_reviewer <= 0
        && summary.current_role.is_empty()
    {
        return;
    }

    println!();
    println!("## Team Roles");
    println!("owner.planner: {}", summary.owner_planner);
    println!("owner.coder: {}", summary.owner_coder);
    println!("owner.reviewer: {}", summary.owner_reviewer);
    if !summary.current_role.is_empty() {
        println!("current_role: {}", summary.current_role);
        if !summary.current_role_task.is_empty() {
            if !summary.current_role_status.is_empty() {
                println!(
                    "current_role_task: {} [{}]",
                    summary.current_role_task, summary.current_role_status
                );
            } else {
                println!("current_role_task: {}", summary.current_role_task);
            }
        }
    }
}

pub(crate) fn print_achievements(fusion_dir: &Path) -> Result<()> {
    let achievements = collect_achievement_summary(fusion_dir)?;
    println!();
    println!("## Achievements");
    let mut achievement_found = false;
    if achievements.completed_workflow {
        println!("- 🎯 Workflow completed");
        achievement_found = true;
    }
    if achievements.completed_tasks > 0 {
        println!("- ✅ Completed tasks: {}", achievements.completed_tasks);
        for title in &achievements.completed_titles {
            println!("- 🏆 {title}");
        }
        achievement_found = true;
    }
    if achievements.safe_rounds > 0 {
        println!(
            "- 🧩 Safe backlog unlocked: +{} tasks ({} rounds)",
            achievements.safe_total, achievements.safe_rounds
        );
        achievement_found = true;
    }
    if achievements.advisory_total > 0 {
        println!(
            "- 🛡️ Supervisor advisories recorded: {}",
            achievements.advisory_total
        );
        achievement_found = true;
    }
    if !achievement_found {
        println!("- (no achievements yet)");
    }

    Ok(())
}

pub(crate) fn print_leaderboard() -> Result<()> {
    let show_leaderboard = env::var("FUSION_STATUS_SHOW_LEADERBOARD")
        .map(|value| {
            !matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            )
        })
        .unwrap_or(true);
    if !show_leaderboard {
        return Ok(());
    }

    let leaderboard_root = env::var("FUSION_LEADERBOARD_ROOT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let leaderboard = collect_achievement_leaderboard(&leaderboard_root, 3)?;
    if leaderboard.is_empty() {
        return Ok(());
    }

    println!();
    println!("## Achievement Leaderboard (Top 3)");
    for (index, entry) in leaderboard.iter().enumerate() {
        println!(
            "{}) {} | score={} | workflows={} | tasks={} | safe={} | advisory={}",
            index + 1,
            entry.project_name,
            entry.score,
            entry.workflows,
            entry.tasks,
            entry.safe_total,
            entry.advisory_total,
        );
    }

    Ok(())
}

pub(crate) fn print_dependency_report(fusion_dir: &Path) -> Result<()> {
    let dep_path = fusion_dir.join("dependency_report.json");
    if !dep_path.is_file() {
        return Ok(());
    }

    println!();
    println!("## Dependency Report");
    let dep = read_dependency_summary(fusion_dir)?;
    if !dep.status.is_empty() {
        println!("status: {}", dep.status);
    }
    if !dep.source.is_empty() {
        println!("source: {}", dep.source);
    }
    if !dep.reason.is_empty() {
        println!("reason: {}", dep.reason);
    }
    if !dep.missing.is_empty() {
        println!("missing: {}", dep.missing);
    }
    if !dep.next.is_empty() {
        println!("next: {}", dep.next);
    }

    Ok(())
}

pub(crate) fn print_backend_failure_report(fusion_dir: &Path) -> Result<()> {
    let backend_path = fusion_dir.join("backend_failure_report.json");
    if !backend_path.is_file() {
        return Ok(());
    }

    println!();
    println!("## Backend Failure Report");
    let backend = read_backend_failure_summary(fusion_dir)?;
    if !backend.status.is_empty() {
        println!("status: {}", backend.status);
    }
    if !backend.source.is_empty() {
        println!("source: {}", backend.source);
    }
    if !backend.primary_backend.is_empty() {
        println!("primary_backend: {}", backend.primary_backend);
    }
    if !backend.fallback_backend.is_empty() {
        println!("fallback_backend: {}", backend.fallback_backend);
    }
    if !backend.primary_error.is_empty() {
        println!("primary_error: {}", backend.primary_error);
    }
    if !backend.fallback_error.is_empty() {
        println!("fallback_error: {}", backend.fallback_error);
    }
    if !backend.next.is_empty() {
        println!("next: {}", backend.next);
    }

    Ok(())
}
