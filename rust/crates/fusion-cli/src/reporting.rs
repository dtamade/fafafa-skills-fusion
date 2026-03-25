use anyhow::Result;
use fusion_runtime_io::{json_get_bool, json_get_string, read_json, read_text};
use std::env;
use std::path::{Path, PathBuf};

use crate::achievements::{collect_achievement_leaderboard, collect_achievement_summary};
use crate::status::{collect_hook_debug_summary, render_understand_handoff};

fn count_markers(content: &str, needle: &str) -> i64 {
    content.lines().filter(|line| line.contains(needle)).count() as i64
}

fn first_in_progress_block(content: &str) -> Option<Vec<String>> {
    let lines: Vec<&str> = content.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if line.contains("[IN_PROGRESS]") {
            let start = index.saturating_sub(1);
            let end = (index + 2).min(lines.len().saturating_sub(1));
            return Some(
                lines[start..=end]
                    .iter()
                    .map(|line| (*line).to_string())
                    .collect(),
            );
        }
    }
    None
}

fn default_leaderboard_root() -> PathBuf {
    if let Ok(root) = env::var("FUSION_LEADERBOARD_ROOT") {
        if !root.trim().is_empty() {
            return PathBuf::from(root);
        }
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join("projects");
    }

    PathBuf::from("projects")
}

pub(crate) fn cmd_logs(fusion_dir: &Path, lines: usize) -> Result<()> {
    if !fusion_dir.is_dir() {
        anyhow::bail!("❌ No fusion workflow found in current directory");
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("                    FUSION WORKFLOW LOGS");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    let sessions_path = fusion_dir.join("sessions.json");
    if sessions_path.is_file() {
        println!("📋 SESSION INFO");
        println!("───────────────────────────────────────────────────────────────");
        let sessions = read_json(&sessions_path)?;
        println!(
            "Goal: {}",
            json_get_string(&sessions, &["goal"]).unwrap_or_else(|| "N/A".to_string())
        );
        println!(
            "Status: {}",
            json_get_string(&sessions, &["status"]).unwrap_or_else(|| "N/A".to_string())
        );
        println!(
            "Phase: {}",
            json_get_string(&sessions, &["current_phase"]).unwrap_or_else(|| "N/A".to_string())
        );
        let understand_mode = json_get_string(&sessions, &["_runtime", "understand", "mode"]);
        let understand_forced = json_get_bool(&sessions, &["_runtime", "understand", "forced"]);
        let understand_decision =
            json_get_string(&sessions, &["_runtime", "understand", "decision"]);
        if let Some(handoff) = render_understand_handoff(
            understand_mode.as_deref(),
            understand_forced,
            understand_decision.as_deref(),
        ) {
            println!("UNDERSTAND: {handoff}");
        }
        println!(
            "Started: {}",
            json_get_string(&sessions, &["started_at"]).unwrap_or_else(|| "N/A".to_string())
        );
        println!(
            "Last checkpoint: {}",
            json_get_string(&sessions, &["last_checkpoint"]).unwrap_or_else(|| "N/A".to_string())
        );
        println!();
    }

    println!("🪝 HOOK DEBUG");
    println!("───────────────────────────────────────────────────────────────");
    let hook_debug = collect_hook_debug_summary(fusion_dir)?;
    println!(
        "enabled: {}",
        if hook_debug.enabled { "true" } else { "false" }
    );
    if !hook_debug.flag_path.is_empty() {
        println!("flag: {}", hook_debug.flag_path);
    }
    if !hook_debug.log_path.is_empty() {
        println!("log: {}", hook_debug.log_path);
        println!("recent (last 5):");
        for line in hook_debug.log_tail {
            println!("  {line}");
        }
    } else {
        println!("log: (none)");
    }
    println!();

    let task_plan = fusion_dir.join("task_plan.md");
    if task_plan.is_file() {
        println!("📝 TASK SUMMARY");
        println!("───────────────────────────────────────────────────────────────");
        let task_plan_text = read_text(&task_plan)?;
        let counts = crate::render::read_task_counts(fusion_dir)?;
        let skipped = count_markers(&task_plan_text, "[SKIPPED]");
        println!("Total tasks: {}", counts.total() + skipped);
        println!("  ✅ Completed: {}", counts.completed);
        println!("  🔄 In Progress: {}", counts.in_progress);
        println!("  ⏳ Pending: {}", counts.pending);
        println!("  ❌ Failed: {}", counts.failed);
        println!("  ⏭️ Skipped: {}", skipped);
        println!();

        if let Some(block) = first_in_progress_block(&task_plan_text) {
            println!("Current task:");
            for line in block {
                println!("  {line}");
            }
            println!();
        }
    }

    let progress = fusion_dir.join("progress.md");
    if progress.is_file() {
        let progress_text = read_text(&progress)?;
        println!("📊 PROGRESS TIMELINE (last {lines} entries)");
        println!("───────────────────────────────────────────────────────────────");
        let rows: Vec<&str> = progress_text
            .lines()
            .filter(|line| line.starts_with('|'))
            .filter(|line| !line.starts_with("| Timestamp"))
            .filter(|line| !line.starts_with("|---"))
            .collect();
        let start = rows.len().saturating_sub(lines);
        for row in &rows[start..] {
            println!("{row}");
        }
        println!();

        let errors: Vec<&str> = progress_text
            .lines()
            .filter(|line| {
                let upper = line.to_ascii_uppercase();
                upper.contains("ERROR") || upper.contains("FAILED") || upper.contains("STRIKE")
            })
            .collect();
        if !errors.is_empty() {
            println!("⚠️ RECENT ERRORS");
            println!("───────────────────────────────────────────────────────────────");
            let start = errors.len().saturating_sub(5);
            for row in &errors[start..] {
                println!("{row}");
            }
            println!();
        }
    }

    let findings = fusion_dir.join("findings.md");
    if findings.is_file() {
        let findings_text = read_text(&findings)?;
        let headers: Vec<&str> = findings_text
            .lines()
            .filter(|line| line.starts_with("##"))
            .collect();
        if !headers.is_empty() {
            println!("🔍 FINDINGS ({} entries)", headers.len());
            println!("───────────────────────────────────────────────────────────────");
            for row in headers.iter().take(10) {
                println!("{row}");
            }
            println!();
        }
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("For full details:");
    println!(
        "  - Task plan: cat {}",
        fusion_dir.join("task_plan.md").display()
    );
    println!(
        "  - Progress: cat {}",
        fusion_dir.join("progress.md").display()
    );
    println!(
        "  - Findings: cat {}",
        fusion_dir.join("findings.md").display()
    );
    println!("═══════════════════════════════════════════════════════════════");

    Ok(())
}

pub(crate) fn cmd_achievements(
    fusion_dir: &Path,
    local_only: bool,
    leaderboard_only: bool,
    root: Option<&Path>,
    top: usize,
) -> Result<()> {
    if top == 0 {
        anyhow::bail!("--top must be a positive integer");
    }

    let mut show_local = true;
    let mut show_leaderboard = true;
    if local_only {
        show_local = true;
        show_leaderboard = false;
    }
    if leaderboard_only {
        show_local = false;
        show_leaderboard = true;
    }

    let leaderboard_root = root
        .map(Path::to_path_buf)
        .unwrap_or_else(default_leaderboard_root);

    println!("=== Fusion Achievements ===");
    println!();

    if show_local {
        println!("## Current Workspace Achievements");
        if !fusion_dir.is_dir() {
            println!("- (no .fusion workspace found)");
        } else {
            let summary = collect_achievement_summary(fusion_dir)?;
            let project_name = env::current_dir()
                .ok()
                .and_then(|cwd| {
                    cwd.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                })
                .unwrap_or_else(|| ".".to_string());
            let status = if summary.status.is_empty() {
                "unknown".to_string()
            } else {
                summary.status.clone()
            };

            println!("project: {project_name}");
            println!("status: {status}");
            println!("score={}", summary.score);

            let mut found = false;
            if summary.completed_workflow {
                println!("- 🎯 Workflow completed");
                found = true;
            }
            if summary.completed_tasks > 0 {
                println!("- ✅ Completed tasks: {}", summary.completed_tasks);
                for title in &summary.completed_titles {
                    println!("- 🏆 {title}");
                }
                found = true;
            }
            if summary.safe_rounds > 0 {
                println!(
                    "- 🧩 Safe backlog unlocked: +{} tasks ({} rounds)",
                    summary.safe_total, summary.safe_rounds
                );
                found = true;
            }
            if summary.advisory_total > 0 {
                println!(
                    "- 🛡️ Supervisor advisories recorded: {}",
                    summary.advisory_total
                );
                found = true;
            }
            if !found {
                println!("- (no achievements yet)");
            }
        }
    }

    if show_local && show_leaderboard {
        println!();
    }

    if show_leaderboard {
        println!("## Achievement Leaderboard");
        if !leaderboard_root.is_dir() {
            println!("- (root not found: {})", leaderboard_root.display());
        } else {
            let leaderboard = collect_achievement_leaderboard(&leaderboard_root, top)?;
            if leaderboard.is_empty() {
                println!(
                    "- (no achievements found under {})",
                    leaderboard_root.display()
                );
            } else {
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
            }
        }
    }

    Ok(())
}
