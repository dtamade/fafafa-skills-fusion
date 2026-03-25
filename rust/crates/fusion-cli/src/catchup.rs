use anyhow::Result;
use fusion_runtime_io::read_json;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::catchup_render::print_report;
use crate::catchup_session::{
    extract_unsynced, find_last_fusion_update, get_sessions_sorted, parse_session_messages,
    select_target_session,
};
use crate::catchup_taskplan::{cross_validate, read_task_plan};

pub(crate) fn cmd_catchup(fusion_dir: &Path, project_path: Option<&Path>) -> Result<()> {
    let project_path = resolve_project_path(project_path)?;
    let fusion_dir = resolve_fusion_dir(&project_path, fusion_dir);
    if !fusion_dir.is_dir() {
        return Ok(());
    }

    let claude_project_dir = claude_project_dir(&project_path);
    let (unsynced, last_update_line, last_update_file) = if claude_project_dir.exists() {
        let sessions = get_sessions_sorted(&claude_project_dir)?;
        if let Some(target_session) = select_target_session(&sessions) {
            let messages = parse_session_messages(target_session)?;
            let (last_update_line, last_update_file) = find_last_fusion_update(&messages);
            let fallback_line = messages.len().saturating_sub(30) as isize;
            let unsynced = extract_unsynced(
                &messages,
                if last_update_line >= 0 {
                    last_update_line
                } else {
                    fallback_line
                },
            );
            (unsynced, last_update_line, last_update_file)
        } else {
            (Vec::new(), -1, None)
        }
    } else {
        (Vec::new(), -1, None)
    };

    let task_info = read_task_plan(&fusion_dir)?;
    let session_info = read_sessions_value(&fusion_dir);
    let git_diff = get_git_diff_stat(&project_path);
    let warnings = cross_validate(&task_info, &session_info, &git_diff);

    print_report(
        &task_info,
        &session_info,
        &git_diff,
        &warnings,
        &unsynced,
        last_update_line,
        last_update_file.as_deref(),
    );
    Ok(())
}

fn resolve_project_path(project_path: Option<&Path>) -> Result<PathBuf> {
    let cwd = env::current_dir()?;
    let base = match project_path {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => cwd,
    };
    Ok(fs::canonicalize(&base).unwrap_or(base))
}

fn resolve_fusion_dir(project_path: &Path, fusion_dir: &Path) -> PathBuf {
    if fusion_dir.is_absolute() {
        fusion_dir.to_path_buf()
    } else {
        project_path.join(fusion_dir)
    }
}

fn claude_project_dir(project_path: &Path) -> PathBuf {
    let mut normalized = project_path.to_string_lossy().replace('\\', "/");
    let bytes = normalized.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' {
        normalized = normalized[2..].to_string();
    }
    let mut sanitized = normalized.replace('/', "-");
    if !sanitized.starts_with('-') {
        sanitized.insert(0, '-');
    }
    sanitized = sanitized.replace('_', "-");
    PathBuf::from(env::var_os("HOME").unwrap_or_default())
        .join(".claude")
        .join("projects")
        .join(sanitized)
}

fn read_sessions_value(fusion_dir: &Path) -> Value {
    let sessions_path = fusion_dir.join("sessions.json");
    read_json(&sessions_path).unwrap_or(Value::Object(Default::default()))
}

fn get_git_diff_stat(project_path: &Path) -> String {
    let Ok(output) = Command::new("git")
        .arg("diff")
        .arg("--stat")
        .arg("HEAD")
        .current_dir(project_path)
        .output()
    else {
        return String::new();
    };

    if !output.status.success() {
        return String::new();
    }
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
