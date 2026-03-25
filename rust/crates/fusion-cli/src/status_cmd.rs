use anyhow::Result;
use serde_json::json;
use std::path::Path;

use crate::status::build_status_summary;
use crate::status_render::{
    print_achievements, print_backend_failure_report, print_dependency_report, print_leaderboard,
    print_progress, print_runtime, print_task_plan, print_team_roles,
};

pub(crate) fn cmd_status(fusion_dir: &Path, json_mode: bool) -> Result<()> {
    if !fusion_dir.is_dir() {
        let message = "[fusion] No .fusion directory found. Run /fusion to start.";
        if json_mode {
            println!(
                "{}",
                serde_json::to_string(&json!({
                    "result": "error",
                    "status": "",
                    "phase": "",
                    "reason": message,
                }))?
            );
        } else {
            println!("{message}");
        }
        std::process::exit(1);
    }

    let summary = build_status_summary(fusion_dir)?;
    if json_mode {
        println!("{}", serde_json::to_string(&summary)?);
        return Ok(());
    }

    println!("=== Fusion Status ===");
    println!();

    print_task_plan(fusion_dir)?;
    print_progress(fusion_dir)?;
    print_runtime(fusion_dir, &summary)?;
    print_team_roles(&summary);
    print_achievements(fusion_dir)?;
    print_leaderboard()?;
    print_dependency_report(fusion_dir)?;
    print_backend_failure_report(fusion_dir)?;

    Ok(())
}
