use anyhow::Result;
use fusion_runtime_io::{json_get_string, load_flat_config, read_json, write_text};
use std::path::Path;

use crate::posttool_progress::{
    build_no_progress_lines, build_progress_delta_lines, bump_unchanged_count, counts_snapshot,
    parse_previous_progress, print_lines, read_previous_snapshot,
};
use crate::posttool_runtime::dispatch_runtime_posttool_events;
use crate::render::read_task_counts;
use crate::safe_backlog::{
    reset_safe_backlog_backoff, try_inject_safe_backlog, SafeBacklogTrigger,
};

pub(crate) fn cmd_hook_posttool(fusion_dir: &Path) -> Result<()> {
    let sessions_path = fusion_dir.join("sessions.json");
    if !sessions_path.is_file() {
        return Ok(());
    }

    let mut snapshot = read_json(&sessions_path)?;
    if json_get_string(&snapshot, &["status"]).as_deref() != Some("in_progress") {
        return Ok(());
    }

    let cfg = load_flat_config(fusion_dir);
    let runtime_enabled = cfg.runtime_enabled;
    let counts = read_task_counts(fusion_dir)?;
    let total = counts.total();
    let pending_like = counts.pending_like();
    let current_snap = counts_snapshot(counts);

    let snap_file = fusion_dir.join(".progress_snapshot");
    let unchanged_file = fusion_dir.join(".snapshot_unchanged_count");

    if runtime_enabled
        && cfg.safe_backlog_enabled
        && cfg.safe_backlog_inject_on_task_exhausted
        && total > 0
        && pending_like == 0
    {
        if let Some(lines) = try_inject_safe_backlog(
            fusion_dir,
            &snapshot,
            &cfg,
            SafeBacklogTrigger {
                counts: &counts,
                pending_like,
                current_snap: &current_snap,
                reason: "task_exhausted",
                no_progress_rounds: 0,
                snap_file: &snap_file,
            },
        )? {
            let _ = write_text(&unchanged_file, "0");
            print_lines(&lines);
            return Ok(());
        }
    }

    let prev_snap = read_previous_snapshot(&snap_file);
    let _ = write_text(&snap_file, &current_snap);

    if current_snap == prev_snap {
        let unchanged = bump_unchanged_count(&unchanged_file);
        let trigger_rounds = cfg.safe_backlog_trigger_no_progress_rounds.max(1);
        let advisory_lines = build_no_progress_lines(
            fusion_dir,
            &snapshot,
            &cfg,
            counts,
            total,
            pending_like,
            &current_snap,
            &snap_file,
            unchanged,
            runtime_enabled,
            trigger_rounds,
        )?;
        print_lines(&advisory_lines);
        return Ok(());
    }

    if runtime_enabled {
        reset_safe_backlog_backoff(fusion_dir)?;
    }
    let _ = write_text(&unchanged_file, "0");

    let (prev_completed, prev_failed) = parse_previous_progress(&prev_snap);
    let completed_delta = counts.completed - prev_completed;
    let failed_delta = counts.failed - prev_failed;

    if runtime_enabled {
        dispatch_runtime_posttool_events(
            fusion_dir,
            &mut snapshot,
            &current_snap,
            counts,
            completed_delta,
        )?;
    }

    let lines = build_progress_delta_lines(
        fusion_dir,
        &snapshot,
        counts,
        total,
        completed_delta,
        failed_delta,
        runtime_enabled,
    )?;
    print_lines(&lines);

    Ok(())
}
