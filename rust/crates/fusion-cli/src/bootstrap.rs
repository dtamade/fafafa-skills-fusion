use anyhow::{Context, Result};
use fusion_runtime_io::{
    json_get_string, json_set_string, read_json, read_text, utc_now_iso, write_json_pretty,
    write_text,
};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::bootstrap_config::{default_config_text, ensure_config_runtime_engine};
use crate::render::{initialize_workspace_next_action, render_current_state, render_next_action};

pub(crate) fn cmd_init(fusion_dir: &Path, templates_dir: &Path, engine: &str) -> Result<()> {
    let normalized_engine = normalize_engine(engine)?;
    init_fusion_dir(fusion_dir, templates_dir, normalized_engine)?;

    println!("[fusion] Initialized .fusion directory");
    println!("[fusion] Files created:");

    let mut names: Vec<String> = fs::read_dir(fusion_dir)
        .with_context(|| format!("failed reading dir: {}", fusion_dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    names.sort();
    for name in names {
        println!("- {name}");
    }

    Ok(())
}

pub(crate) fn cmd_start(
    fusion_dir: &Path,
    templates_dir: &Path,
    goal: &str,
    force_mode: bool,
) -> Result<()> {
    init_fusion_dir(fusion_dir, templates_dir, "rust")?;

    let sessions_path = fusion_dir.join("sessions.json");
    let mut sessions = read_json(&sessions_path)?;

    let workflow_id = format!("fusion_{}", chrono::Utc::now().timestamp());
    let timestamp = utc_now_iso();

    json_set_string(&mut sessions, "goal", goal);
    json_set_string(&mut sessions, "started_at", &timestamp);
    json_set_string(&mut sessions, "workflow_id", &workflow_id);
    json_set_string(&mut sessions, "status", "in_progress");
    json_set_string(&mut sessions, "current_phase", "INITIALIZE");
    record_understand_result(&mut sessions, force_mode);

    write_json_pretty(&sessions_path, &sessions)?;

    if force_mode {
        println!("[fusion] ⚠️ Skipped UNDERSTAND (--force)");
    } else {
        println!("[fusion] UNDERSTAND runner currently minimal; proceed to INITIALIZE");
    }

    println!(
        "[fusion] {}",
        render_current_state("in_progress", "INITIALIZE")
    );
    println!(
        "[fusion] {}",
        render_next_action(initialize_workspace_next_action())
    );
    println!();
    println!("[FUSION] Workflow initialized.");
    println!("Goal: {goal}");

    Ok(())
}

fn record_understand_result(sessions: &mut Value, force_mode: bool) {
    if !sessions.is_object() {
        *sessions = Value::Object(Map::new());
    }

    let root = sessions
        .as_object_mut()
        .expect("sessions root should be an object");
    let runtime = root
        .entry("_runtime")
        .or_insert_with(|| Value::Object(Map::new()));
    if !runtime.is_object() {
        *runtime = Value::Object(Map::new());
    }

    let runtime_obj = runtime
        .as_object_mut()
        .expect("runtime root should be an object");
    runtime_obj.insert("state".to_string(), Value::String("INITIALIZE".to_string()));

    let understand = runtime_obj
        .entry("understand")
        .or_insert_with(|| Value::Object(Map::new()));
    if !understand.is_object() {
        *understand = Value::Object(Map::new());
    }

    let understand_obj = understand
        .as_object_mut()
        .expect("understand summary should be an object");
    understand_obj.insert(
        "mode".to_string(),
        Value::String(if force_mode { "skipped" } else { "minimal" }.to_string()),
    );
    understand_obj.insert("forced".to_string(), Value::Bool(force_mode));
    understand_obj.insert(
        "decision".to_string(),
        Value::String(
            if force_mode {
                "force_skip"
            } else {
                "auto_continue"
            }
            .to_string(),
        ),
    );
}

pub(crate) fn init_fusion_dir(fusion_dir: &Path, templates_dir: &Path, engine: &str) -> Result<()> {
    let normalized_engine = normalize_engine(engine)?;

    if fusion_dir.is_symlink() {
        anyhow::bail!(
            "❌ Security: {} is a symlink, refusing to use",
            fusion_dir.display()
        );
    }

    let sessions_path = fusion_dir.join("sessions.json");
    if sessions_path.is_file() {
        let sessions = read_json(&sessions_path)?;
        let status =
            json_get_string(&sessions, &["status"]).unwrap_or_else(|| "unknown".to_string());
        if status == "in_progress" || status == "paused" {
            anyhow::bail!(
                "❌ Cannot reinitialize: workflow is {status}\n   Use /fusion cancel or /fusion resume"
            );
        }
    }

    fs::create_dir_all(fusion_dir)
        .with_context(|| format!("failed creating dir: {}", fusion_dir.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(fusion_dir)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(fusion_dir, perms)?;
    }

    copy_if_exists(
        &templates_dir.join("task_plan.md"),
        &fusion_dir.join("task_plan.md"),
    )?;
    copy_if_exists(
        &templates_dir.join("progress.md"),
        &fusion_dir.join("progress.md"),
    )?;
    copy_if_exists(
        &templates_dir.join("findings.md"),
        &fusion_dir.join("findings.md"),
    )?;

    let config_path = fusion_dir.join("config.yaml");
    let config_template = templates_dir.join("config.yaml");
    if config_template.is_file() {
        copy_if_exists(&config_template, &config_path)?;
    } else {
        write_text(&config_path, &default_config_text(normalized_engine))?;
    }
    ensure_config_runtime_engine(&config_path, normalized_engine)?;

    let sessions_template = templates_dir.join("sessions.json");
    if sessions_template.is_file() {
        copy_if_exists(&sessions_template, &sessions_path)?;
    } else if !sessions_path.exists() {
        write_text(
            &sessions_path,
            r#"{
  "workflow_id": null,
  "goal": null,
  "started_at": null,
  "status": "not_started",
  "current_phase": null,
  "codex_session": null,
  "claude_session": null,
  "tasks": {},
  "strikes": {
    "current_task": null,
    "count": 0,
    "history": []
  },
  "git": {
    "branch": null,
    "commits": []
  },
  "last_checkpoint": null
}"#,
        )?;
    }

    ensure_gitignore_has_fusion()?;

    Ok(())
}

fn normalize_engine(engine: &str) -> Result<&'static str> {
    match engine.trim().to_ascii_lowercase().as_str() {
        "rust" => Ok("rust"),
        _ => anyhow::bail!("Invalid engine: {engine} (expected: rust)"),
    }
}

fn copy_if_exists(src: &Path, dest: &Path) -> Result<()> {
    if src.is_file() {
        fs::copy(src, dest).with_context(|| {
            format!(
                "failed copying template {} -> {}",
                src.display(),
                dest.display()
            )
        })?;
    }
    Ok(())
}

fn ensure_gitignore_has_fusion() -> Result<()> {
    let gitignore = PathBuf::from(".gitignore");
    if !gitignore.is_file() {
        return Ok(());
    }

    let mut content = read_text(&gitignore)?;
    if content.lines().any(|line| line.trim() == ".fusion/") {
        return Ok(());
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n# Fusion working directory\n.fusion/\n");
    write_text(&gitignore, &content)
}
