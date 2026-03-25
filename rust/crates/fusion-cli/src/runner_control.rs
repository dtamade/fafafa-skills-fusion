use anyhow::Result;
use fusion_runtime_io::{
    ensure_fusion_dir, json_get_string, json_set_string, read_json, read_text, utc_now_iso,
    write_json_pretty, write_text,
};
use std::path::Path;

use crate::models::RunOptions;
use crate::render::{
    live_next_action_for_phase, no_resume_needed_next_action, read_task_counts,
    render_current_state, render_next_action,
};
use crate::runner::cmd_run;

fn render_live_next_action(fusion_dir: &Path, current_phase: &str) -> Result<String> {
    live_next_action_for_phase(
        fusion_dir,
        current_phase,
        "Inspect .fusion/task_plan.md and continue from the next live step",
    )
}

pub(crate) fn cmd_pause(fusion_dir: &Path) -> Result<()> {
    ensure_fusion_dir(fusion_dir)?;

    let sessions_path = fusion_dir.join("sessions.json");
    let mut sessions = read_json(&sessions_path)?;
    let status = json_get_string(&sessions, &["status"]).unwrap_or_else(|| "unknown".to_string());

    if status != "in_progress" {
        anyhow::bail!("⚠️ Workflow is not in progress (current status: {status})");
    }

    json_set_string(&mut sessions, "status", "paused");
    json_set_string(&mut sessions, "last_checkpoint", &utc_now_iso());
    write_json_pretty(&sessions_path, &sessions)?;

    println!("⏸️ Workflow paused");
    println!();
    println!("Current progress saved. Use '/fusion resume' to continue.");
    Ok(())
}

pub(crate) fn cmd_cancel(fusion_dir: &Path) -> Result<()> {
    ensure_fusion_dir(fusion_dir)?;

    let sessions_path = fusion_dir.join("sessions.json");
    let mut sessions = read_json(&sessions_path)?;
    let status = json_get_string(&sessions, &["status"]).unwrap_or_else(|| "unknown".to_string());
    let goal = json_get_string(&sessions, &["goal"]).unwrap_or_else(|| "unknown".to_string());

    match status.as_str() {
        "cancelled" => {
            println!("⚠️ Workflow is already cancelled");
            return Ok(());
        }
        "completed" => {
            println!("⚠️ Workflow is already completed");
            return Ok(());
        }
        _ => {}
    }

    json_set_string(&mut sessions, "status", "cancelled");
    json_set_string(&mut sessions, "last_checkpoint", &utc_now_iso());
    write_json_pretty(&sessions_path, &sessions)?;

    println!("❌ Workflow cancelled");
    println!();
    println!("Goal: {goal}");
    println!();
    println!("Options:");
    println!("  - Start fresh: /fusion \"<new goal>\"");
    println!("  - Clean up: rm -rf .fusion/");
    Ok(())
}

pub(crate) fn cmd_continue(fusion_dir: &Path) -> Result<()> {
    if !fusion_dir.is_dir() {
        return Ok(());
    }

    let sessions_path = fusion_dir.join("sessions.json");
    if !sessions_path.is_file() {
        return Ok(());
    }

    let sessions = read_json(&sessions_path)?;
    let status = json_get_string(&sessions, &["status"]).unwrap_or_default();
    if status != "in_progress" {
        return Ok(());
    }

    let current_phase =
        json_get_string(&sessions, &["current_phase"]).unwrap_or_else(|| "unknown".to_string());
    let pending_count = read_task_counts(fusion_dir)
        .map(|counts| counts.pending_like().to_string())
        .unwrap_or_else(|_| "?".to_string());
    let next_action = render_live_next_action(fusion_dir, &current_phase)?;

    let progress_path = fusion_dir.join("progress.md");
    if progress_path.is_file() {
        let existing = read_text(&progress_path).unwrap_or_default();
        let last_line = existing.lines().last().unwrap_or_default();
        if !last_line.contains("[CONTINUE]") {
            let mut updated = existing;
            if !updated.is_empty() && !updated.ends_with('\n') {
                updated.push('\n');
            }
            updated.push('\n');
            updated.push_str(&format!(
                "<!-- [CONTINUE] Phase: {current_phase} | Pending: {pending_count} | Check task_plan.md and continue -->\n"
            ));
            write_text(&progress_path, &updated)?;
        }
    }

    println!(
        "[fusion] {}",
        render_current_state("in_progress", &current_phase)
    );
    println!("[fusion] {}", render_next_action(&next_action));

    let hook_debug = matches!(
        std::env::var("FUSION_HOOK_DEBUG")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    ) || fusion_dir.join(".hook_debug").is_file();

    if hook_debug {
        println!("[fusion][hooks] Hook debug: ON (stderr + .fusion/hook-debug.log)");
    } else {
        println!("[fusion][hooks] Hook debug: OFF (enable: touch .fusion/.hook_debug)");
    }

    Ok(())
}

pub(crate) fn cmd_resume(fusion_dir: &Path, options: RunOptions) -> Result<()> {
    ensure_fusion_dir(fusion_dir)?;

    let sessions_path = fusion_dir.join("sessions.json");
    let mut sessions = read_json(&sessions_path)?;
    let status = json_get_string(&sessions, &["status"]).unwrap_or_else(|| "unknown".to_string());
    let phase =
        json_get_string(&sessions, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());

    match status.as_str() {
        "paused" => {
            let next_action = render_live_next_action(fusion_dir, &phase)?;
            println!("[fusion] {}", render_current_state("paused", &phase));
            println!("[fusion] {}", render_next_action(&next_action));
            json_set_string(&mut sessions, "status", "in_progress");
            if json_get_string(&sessions, &["current_phase"]).is_none() {
                json_set_string(&mut sessions, "current_phase", "EXECUTE");
            }
            write_json_pretty(&sessions_path, &sessions)?;
            println!("[fusion] Workflow resumed from paused state");
        }
        "stuck" => {
            let next_action = render_live_next_action(fusion_dir, &phase)?;
            println!("[fusion] {}", render_current_state("stuck", &phase));
            println!("[fusion] {}", render_next_action(&next_action));
            println!("⚠️ Workflow is stuck. Please investigate:");
            println!("   - Check .fusion/progress.md for errors");
            println!("   - Fix the issue and run /fusion resume again");
            println!("   - Or cancel with: ./scripts/fusion-cancel.sh");

            json_set_string(&mut sessions, "status", "in_progress");
            if json_get_string(&sessions, &["current_phase"]).is_none() {
                json_set_string(&mut sessions, "current_phase", "EXECUTE");
            }
            write_json_pretty(&sessions_path, &sessions)?;
            println!("[fusion] Status has been set to 'in_progress'. Continuing.");
        }
        "in_progress" => {
            let next_action = render_live_next_action(fusion_dir, &phase)?;
            println!("[fusion] {}", render_current_state("in_progress", &phase));
            println!("[fusion] {}", render_next_action(&next_action));
            println!("[fusion] Workflow already in progress, continuing");
        }
        "completed" => {
            println!("[fusion] {}", render_current_state("completed", &phase));
            println!(
                "[fusion] {}",
                render_next_action(no_resume_needed_next_action())
            );
            println!("✅ Workflow already completed. Nothing to resume.");
            return Ok(());
        }
        "cancelled" => {
            anyhow::bail!(
                "❌ Workflow was cancelled. Start a new workflow with:\n   /fusion \"<new goal>\""
            );
        }
        _ => {
            anyhow::bail!("Cannot resume workflow with status: {status}");
        }
    }

    cmd_run(fusion_dir, options)
}
