use anyhow::Result;
use fusion_runtime_io::{json_get_string, read_json, read_text};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::render::read_task_counts;

#[derive(Debug, Clone, Default)]
pub(crate) struct AchievementSummary {
    pub(crate) status: String,
    pub(crate) completed_workflow: bool,
    pub(crate) completed_tasks: i64,
    pub(crate) completed_titles: Vec<String>,
    pub(crate) safe_rounds: i64,
    pub(crate) safe_total: i64,
    pub(crate) advisory_total: i64,
    pub(crate) score: i64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LeaderboardEntry {
    pub(crate) project_name: String,
    pub(crate) score: i64,
    pub(crate) workflows: i64,
    pub(crate) tasks: i64,
    pub(crate) safe_total: i64,
    pub(crate) advisory_total: i64,
}

pub(crate) fn parse_task_status_line(line: &str) -> Option<(String, String)> {
    if !line.starts_with("### Task ") {
        return None;
    }

    let (_, rest) = line.split_once(':')?;
    let content = rest.trim();
    let start = content.rfind('[')?;
    let end = content[start..].find(']')? + start;
    let status = content[start + 1..end].trim();
    if !matches!(status, "PENDING" | "IN_PROGRESS" | "COMPLETED" | "FAILED") {
        return None;
    }

    let title = content[..start].trim().to_string();
    Some((title, status.to_string()))
}

fn collect_completed_titles(fusion_dir: &Path) -> Result<Vec<String>> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok(Vec::new());
    }

    let content = read_text(&task_plan)?;
    Ok(content
        .lines()
        .filter_map(parse_task_status_line)
        .filter_map(|(title, status)| (status == "COMPLETED").then_some(title))
        .collect())
}

fn collect_fusion_dirs(root: &Path, dirs: &mut Vec<PathBuf>) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() || file_type.is_symlink() {
            continue;
        }

        let path = entry.path();
        if entry.file_name().to_string_lossy() == ".fusion" {
            dirs.push(path);
            continue;
        }

        collect_fusion_dirs(&path, dirs)?;
    }

    Ok(())
}

pub(crate) fn collect_achievement_summary(fusion_dir: &Path) -> Result<AchievementSummary> {
    let counts = read_task_counts(fusion_dir)?;
    let completed_titles = collect_completed_titles(fusion_dir)?;
    let sessions_path = fusion_dir.join("sessions.json");
    let events_path = fusion_dir.join("events.jsonl");
    let mut status = String::new();
    let mut completed_workflow = false;
    let mut safe_rounds = 0;
    let mut safe_total = 0;
    let mut advisory_total = 0;

    if sessions_path.is_file() {
        let sessions = read_json(&sessions_path)?;
        status = json_get_string(&sessions, &["status"]).unwrap_or_default();
        completed_workflow = status == "completed";
    }

    if events_path.is_file() {
        let content = read_text(&events_path)?;
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let Ok(value) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            match value.get("type").and_then(|v| v.as_str()) {
                Some("SAFE_BACKLOG_INJECTED") => {
                    safe_rounds += 1;
                    safe_total += value
                        .get("payload")
                        .and_then(|v| v.get("added"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                }
                Some("SUPERVISOR_ADVISORY") => {
                    advisory_total += 1;
                }
                _ => {}
            }
        }
    }

    let score = (if completed_workflow { 50 } else { 0 })
        + counts.completed * 10
        + safe_total * 3
        + advisory_total * 2;

    Ok(AchievementSummary {
        status,
        completed_workflow,
        completed_tasks: counts.completed,
        completed_titles,
        safe_rounds,
        safe_total,
        advisory_total,
        score,
    })
}

pub(crate) fn collect_achievement_leaderboard(
    root: &Path,
    top_n: usize,
) -> Result<Vec<LeaderboardEntry>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut fusion_dirs = Vec::new();
    collect_fusion_dirs(root, &mut fusion_dirs)?;

    let mut entries = fusion_dirs
        .into_iter()
        .filter_map(|fusion_dir| {
            let summary = collect_achievement_summary(&fusion_dir).ok()?;
            if summary.score <= 0 {
                return None;
            }

            let project_name = fusion_dir
                .parent()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| fusion_dir.display().to_string());

            Some(LeaderboardEntry {
                project_name,
                score: summary.score,
                workflows: i64::from(summary.completed_workflow),
                tasks: summary.completed_tasks,
                safe_total: summary.safe_total,
                advisory_total: summary.advisory_total,
            })
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.project_name.cmp(&right.project_name))
    });
    entries.truncate(top_n.max(1));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_collect_achievement_summary_counts_events() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("sessions.json"),
            serde_json::to_string(&serde_json::json!({"status":"completed"})).expect("json"),
        )
        .expect("sessions");
        std::fs::write(
            dir.path().join("task_plan.md"),
            "### Task 1: A [COMPLETED]\n### Task 2: B [COMPLETED]\n",
        )
        .expect("task plan");
        std::fs::write(
            dir.path().join("events.jsonl"),
            serde_json::to_string(
                &serde_json::json!({"type":"SAFE_BACKLOG_INJECTED","payload":{"added":2}}),
            )
            .expect("event")
                + "\n"
                + &serde_json::to_string(
                    &serde_json::json!({"type":"SUPERVISOR_ADVISORY","payload":{}}),
                )
                .expect("event")
                + "\n",
        )
        .expect("events");

        let summary = collect_achievement_summary(dir.path()).expect("summary");
        assert_eq!(summary.status, "completed");
        assert!(summary.completed_workflow);
        assert_eq!(summary.completed_tasks, 2);
        assert_eq!(summary.completed_titles, vec!["A", "B"]);
        assert_eq!(summary.safe_rounds, 1);
        assert_eq!(summary.safe_total, 2);
        assert_eq!(summary.advisory_total, 1);
        assert_eq!(summary.score, 78);
    }

    #[test]
    fn test_collect_achievement_leaderboard_sorts_by_score_then_name() {
        let dir = tempdir().expect("tempdir");
        let alpha = dir.path().join("alpha").join(".fusion");
        let beta = dir.path().join("beta").join(".fusion");
        let gamma = dir.path().join("gamma").join(".fusion");
        std::fs::create_dir_all(&alpha).expect("alpha");
        std::fs::create_dir_all(&beta).expect("beta");
        std::fs::create_dir_all(&gamma).expect("gamma");

        std::fs::write(alpha.join("sessions.json"), r#"{"status":"completed"}"#)
            .expect("alpha sessions");
        std::fs::write(
            alpha.join("task_plan.md"),
            "### Task 1: A [COMPLETED]\n### Task 2: B [COMPLETED]\n",
        )
        .expect("alpha task");
        std::fs::write(
            alpha.join("events.jsonl"),
            r#"{"type":"SAFE_BACKLOG_INJECTED","payload":{"added":1}}"#,
        )
        .expect("alpha events");

        std::fs::write(beta.join("sessions.json"), r#"{"status":"in_progress"}"#)
            .expect("beta sessions");
        std::fs::write(
            beta.join("task_plan.md"),
            "### Task 1: A [COMPLETED]\n### Task 2: B [COMPLETED]\n",
        )
        .expect("beta task");

        std::fs::write(gamma.join("sessions.json"), r#"{"status":"completed"}"#)
            .expect("gamma sessions");
        std::fs::write(gamma.join("task_plan.md"), "### Task 1: A [COMPLETED]\n")
            .expect("gamma task");
        std::fs::write(
            gamma.join("events.jsonl"),
            r#"{"type":"SUPERVISOR_ADVISORY","payload":{}}"#,
        )
        .expect("gamma events");

        let leaderboard = collect_achievement_leaderboard(dir.path(), 3).expect("leaderboard");
        assert_eq!(leaderboard.len(), 3);
        assert_eq!(leaderboard[0].project_name, "alpha");
        assert_eq!(leaderboard[0].score, 73);
        assert_eq!(leaderboard[1].project_name, "gamma");
        assert_eq!(leaderboard[1].score, 62);
        assert_eq!(leaderboard[2].project_name, "beta");
        assert_eq!(leaderboard[2].score, 20);
    }
}
