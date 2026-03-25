use anyhow::Result;
use fusion_runtime_io::{load_flat_config, read_json, read_text, read_yaml};
use serde_json::Value;
use std::io::{self, Read};
use std::path::Path;

fn read_json_source(file: Option<&Path>) -> Result<serde_json::Value> {
    match file {
        Some(path) => read_json(path),
        None => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            Ok(serde_json::from_str(&input)?)
        }
    }
}

pub(crate) fn cmd_json_field(
    file: Option<&Path>,
    key: &str,
    number: bool,
    bool_mode: bool,
) -> Result<()> {
    let payload = read_json_source(file)?;
    let value = payload.get(key);

    if bool_mode {
        if let Some(flag) = value.and_then(|item| item.as_bool()) {
            println!("{flag}");
        }
        return Ok(());
    }

    if number {
        if let Some(number) = value.and_then(|item| item.as_i64()) {
            println!("{number}");
        }
        return Ok(());
    }

    if let Some(text) = value.and_then(|item| item.as_str()) {
        println!("{text}");
    }

    Ok(())
}

pub(crate) fn cmd_runtime_config(fusion_dir: &Path, field: &str) -> Result<()> {
    let cfg = load_flat_config(fusion_dir);
    match field {
        "enabled" => println!("{}", if cfg.runtime_enabled { "true" } else { "false" }),
        "engine" => println!("{}", cfg.runtime_engine),
        "compat_mode" => println!(
            "{}",
            if cfg.runtime_compat_mode {
                "true"
            } else {
                "false"
            }
        ),
        _ => {}
    }
    Ok(())
}

fn yaml_field<'a>(root: &'a serde_yaml::Value, path: &[&str]) -> Option<&'a serde_yaml::Value> {
    let mut current = root;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

pub(crate) fn cmd_loop_guardian_config(fusion_dir: &Path, field: &str) -> Result<()> {
    let config_path = fusion_dir.join("config.yaml");
    if !config_path.is_file() {
        return Ok(());
    }

    let raw = read_yaml(&config_path)?;
    let value = match field {
        "max_iterations" => yaml_field(&raw, &["loop_guardian", "max_iterations"]),
        "max_no_progress" => yaml_field(&raw, &["loop_guardian", "max_no_progress"]),
        "max_same_action" => yaml_field(&raw, &["loop_guardian", "max_same_action"]),
        "max_same_error" => yaml_field(&raw, &["loop_guardian", "max_same_error"]),
        "max_state_visits" => yaml_field(&raw, &["loop_guardian", "max_state_visits"]),
        "max_wall_time_ms" => yaml_field(&raw, &["loop_guardian", "max_wall_time_ms"]),
        "backoff_threshold" => yaml_field(&raw, &["loop_guardian", "backoff_threshold"]),
        _ => None,
    };

    if let Some(value) = value {
        match value {
            serde_yaml::Value::Number(number) => println!("{number}"),
            serde_yaml::Value::String(text) => println!("{text}"),
            serde_yaml::Value::Bool(flag) => println!("{flag}"),
            _ => {}
        }
    }

    Ok(())
}

fn read_json_file(path: &Path) -> Result<Value> {
    if !path.is_file() {
        return Ok(Value::Null);
    }
    read_json(path)
}

pub(crate) fn cmd_loop_context_array_values(path: &Path, key: &str) -> Result<()> {
    let payload = read_json_file(path)?;
    if let Some(items) = payload.get(key).and_then(|value| value.as_array()) {
        for item in items {
            match item {
                Value::String(text) => println!("{text}"),
                Value::Number(number) => println!("{number}"),
                Value::Bool(flag) => println!("{flag}"),
                _ => {}
            }
        }
    }
    Ok(())
}

pub(crate) fn cmd_loop_context_state_visits(path: &Path) -> Result<()> {
    let payload = read_json_file(path)?;
    if let Some(entries) = payload
        .get("state_visits")
        .and_then(|value| value.as_object())
    {
        for (key, value) in entries {
            if let Some(number) = value.as_i64() {
                println!("{key}={number}");
            }
        }
    }
    Ok(())
}

pub(crate) fn cmd_loop_context_decision_history(path: &Path) -> Result<()> {
    let payload = read_json_file(path)?;
    if let Some(items) = payload
        .get("decision_history")
        .and_then(|value| value.as_array())
    {
        for item in items {
            println!("{}", serde_json::to_string(item)?);
        }
    }
    Ok(())
}

fn read_task_plan(path: &Path) -> Result<String> {
    if !path.is_file() {
        return Ok(String::new());
    }
    read_text(path)
}

fn strip_task_name(line: &str) -> String {
    let mut name = if let Some((_, right)) = line.split_once(':') {
        right.trim().to_string()
    } else {
        line.trim().to_string()
    };

    for tag in ["[IN_PROGRESS]", "[PENDING]", "[COMPLETED]", "[FAILED]"] {
        name = name.replace(tag, "").trim().to_string();
    }

    name
}

pub(crate) fn cmd_task_plan_counts(path: &Path) -> Result<()> {
    let content = read_task_plan(path)?;
    let completed = content.matches("[COMPLETED]").count();
    let pending = content.matches("[PENDING]").count();
    let in_progress = content.matches("[IN_PROGRESS]").count();
    let failed = content.matches("[FAILED]").count();
    println!("{completed}:{pending}:{in_progress}:{failed}");
    Ok(())
}

pub(crate) fn cmd_task_plan_first(path: &Path, status: &str) -> Result<()> {
    let content = read_task_plan(path)?;
    for line in content.lines() {
        if line.starts_with("### Task ") && line.contains(status) {
            let name = strip_task_name(line);
            if !name.is_empty() {
                println!("{name}");
                break;
            }
        }
    }
    Ok(())
}

pub(crate) fn cmd_task_plan_last(path: &Path, status: &str) -> Result<()> {
    let content = read_task_plan(path)?;
    let mut found = String::new();
    for line in content.lines() {
        if line.starts_with("### Task ") && line.contains(status) {
            let name = strip_task_name(line);
            if !name.is_empty() {
                found = name;
            }
        }
    }
    if !found.is_empty() {
        println!("{found}");
    }
    Ok(())
}

pub(crate) fn cmd_task_plan_next(path: &Path) -> Result<()> {
    let content = read_task_plan(path)?;
    for line in content.lines() {
        if !line.starts_with("### Task ") {
            continue;
        }
        if !(line.contains("[IN_PROGRESS]") || line.contains("[PENDING]")) {
            continue;
        }
        let name = strip_task_name(line);
        if !name.is_empty() {
            println!("{name}");
            break;
        }
    }
    Ok(())
}

pub(crate) fn cmd_task_plan_task_type(path: &Path, title: &str) -> Result<()> {
    let content = read_task_plan(path)?;
    let mut active = false;

    for line in content.lines() {
        if line.starts_with("### Task ") {
            active = strip_task_name(line) == title;
            continue;
        }
        if !active {
            continue;
        }
        let trimmed = line.trim_start();
        if let Some(value) = trimmed.strip_prefix("- Type:") {
            let normalized: String = value.chars().filter(|ch| !ch.is_whitespace()).collect();
            if !normalized.is_empty() {
                println!("{}", normalized.to_ascii_lowercase());
            }
            break;
        }
    }
    Ok(())
}
