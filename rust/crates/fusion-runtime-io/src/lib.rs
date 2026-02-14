use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyReport {
    pub status: String,
    pub source: String,
    pub timestamp: String,
    pub missing: Vec<String>,
    pub reason: String,
    pub auto_attempted: Vec<String>,
    pub next_actions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_prompt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FlatConfig {
    pub runtime_enabled: bool,
    pub runtime_compat_mode: bool,

    pub backend_primary: String,
    pub backend_fallback: String,

    pub safe_backlog_enabled: bool,
    pub safe_backlog_trigger_no_progress_rounds: i64,
    pub safe_backlog_max_tasks_per_run: i64,
    pub safe_backlog_allowed_categories: String,
    pub safe_backlog_inject_on_task_exhausted: bool,
    pub safe_backlog_diversity_rotation: bool,
    pub safe_backlog_novelty_window: i64,
    pub safe_backlog_backoff_enabled: bool,
    pub safe_backlog_backoff_base_rounds: i64,
    pub safe_backlog_backoff_max_rounds: i64,
    pub safe_backlog_backoff_jitter: f64,
    pub safe_backlog_backoff_force_probe_rounds: i64,

    pub supervisor_enabled: bool,
    pub supervisor_mode: String,
    pub supervisor_persona: String,
    pub supervisor_trigger_no_progress_rounds: i64,
    pub supervisor_cadence_rounds: i64,
    pub supervisor_force_emit_rounds: i64,
    pub supervisor_max_suggestions: i64,

    pub understand_pass_threshold: i64,
    pub understand_require_confirmation: bool,
    pub understand_max_questions: i64,
}

impl Default for FlatConfig {
    fn default() -> Self {
        Self {
            runtime_enabled: false,
            runtime_compat_mode: true,

            backend_primary: "codex".to_string(),
            backend_fallback: "claude".to_string(),

            safe_backlog_enabled: true,
            safe_backlog_trigger_no_progress_rounds: 3,
            safe_backlog_max_tasks_per_run: 2,
            safe_backlog_allowed_categories: "quality,documentation,optimization".to_string(),
            safe_backlog_inject_on_task_exhausted: true,
            safe_backlog_diversity_rotation: true,
            safe_backlog_novelty_window: 12,
            safe_backlog_backoff_enabled: true,
            safe_backlog_backoff_base_rounds: 1,
            safe_backlog_backoff_max_rounds: 32,
            safe_backlog_backoff_jitter: 0.2,
            safe_backlog_backoff_force_probe_rounds: 20,

            supervisor_enabled: false,
            supervisor_mode: "advisory".to_string(),
            supervisor_persona: "Guardian".to_string(),
            supervisor_trigger_no_progress_rounds: 2,
            supervisor_cadence_rounds: 2,
            supervisor_force_emit_rounds: 12,
            supervisor_max_suggestions: 2,

            understand_pass_threshold: 7,
            understand_require_confirmation: false,
            understand_max_questions: 2,
        }
    }
}

pub fn utc_now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn epoch_now_seconds_f64() -> f64 {
    Utc::now().timestamp_millis() as f64 / 1000.0
}

pub fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed reading file: {}", path.display()))
}

pub fn write_text(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("failed writing file: {}", path.display()))
}

pub fn read_json(path: &Path) -> Result<Value> {
    let text = read_text(path)?;
    let value: Value = serde_json::from_str(&text)
        .with_context(|| format!("failed parsing json: {}", path.display()))?;
    Ok(value)
}

pub fn write_json_pretty(path: &Path, value: &Value) -> Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    write_text(path, &text)
}

pub fn read_yaml(path: &Path) -> Result<serde_yaml::Value> {
    let text = read_text(path)?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text)
        .with_context(|| format!("failed parsing yaml: {}", path.display()))?;
    Ok(value)
}

pub fn json_get_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(ToOwned::to_owned)
}

pub fn json_get_bool(value: &Value, path: &[&str]) -> Option<bool> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_bool()
}

pub fn json_set_string(value: &mut Value, key: &str, data: &str) {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
    if let Some(map) = value.as_object_mut() {
        map.insert(key.to_string(), Value::String(data.to_string()));
    }
}

fn yaml_get<'a>(root: &'a serde_yaml::Value, path: &[&str]) -> Option<&'a serde_yaml::Value> {
    let mut current = root;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn yaml_to_bool(value: Option<&serde_yaml::Value>, default: bool) -> bool {
    match value {
        Some(serde_yaml::Value::Bool(v)) => *v,
        Some(serde_yaml::Value::Number(n)) => n.as_i64().map(|x| x != 0).unwrap_or(default),
        Some(serde_yaml::Value::String(s)) => {
            let normalized = s.trim().to_lowercase();
            match normalized.as_str() {
                "true" | "yes" | "on" | "1" => true,
                "false" | "no" | "off" | "0" => false,
                _ => default,
            }
        }
        _ => default,
    }
}

fn yaml_to_i64(value: Option<&serde_yaml::Value>, default: i64) -> i64 {
    match value {
        Some(serde_yaml::Value::Number(n)) => n.as_i64().unwrap_or(default),
        Some(serde_yaml::Value::String(s)) => s.trim().parse::<i64>().unwrap_or(default),
        _ => default,
    }
}

fn yaml_to_f64(value: Option<&serde_yaml::Value>, default: f64) -> f64 {
    match value {
        Some(serde_yaml::Value::Number(n)) => n.as_f64().unwrap_or(default),
        Some(serde_yaml::Value::String(s)) => s.trim().parse::<f64>().unwrap_or(default),
        _ => default,
    }
}

fn yaml_to_string(value: Option<&serde_yaml::Value>, default: &str) -> String {
    match value {
        Some(serde_yaml::Value::String(s)) => s.clone(),
        Some(serde_yaml::Value::Number(n)) => n.to_string(),
        Some(serde_yaml::Value::Bool(v)) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        _ => default.to_string(),
    }
}

pub fn load_flat_config(fusion_dir: &Path) -> FlatConfig {
    let cfg_path = fusion_dir.join("config.yaml");
    let mut cfg = FlatConfig::default();

    let Ok(raw) = read_yaml(&cfg_path) else {
        return cfg;
    };

    cfg.runtime_enabled =
        yaml_to_bool(yaml_get(&raw, &["runtime", "enabled"]), cfg.runtime_enabled);
    cfg.runtime_compat_mode = yaml_to_bool(
        yaml_get(&raw, &["runtime", "compat_mode"]),
        cfg.runtime_compat_mode,
    );

    cfg.backend_primary = yaml_to_string(
        yaml_get(&raw, &["backends", "primary"]),
        &cfg.backend_primary,
    );
    cfg.backend_fallback = yaml_to_string(
        yaml_get(&raw, &["backends", "fallback"]),
        &cfg.backend_fallback,
    );

    cfg.safe_backlog_enabled = yaml_to_bool(
        yaml_get(&raw, &["safe_backlog", "enabled"]),
        cfg.safe_backlog_enabled,
    );
    cfg.safe_backlog_trigger_no_progress_rounds = yaml_to_i64(
        yaml_get(&raw, &["safe_backlog", "trigger_no_progress_rounds"]),
        cfg.safe_backlog_trigger_no_progress_rounds,
    )
    .max(1);
    cfg.safe_backlog_max_tasks_per_run = yaml_to_i64(
        yaml_get(&raw, &["safe_backlog", "max_tasks_per_run"]),
        cfg.safe_backlog_max_tasks_per_run,
    )
    .max(1);
    cfg.safe_backlog_allowed_categories = yaml_to_string(
        yaml_get(&raw, &["safe_backlog", "allowed_categories"]),
        &cfg.safe_backlog_allowed_categories,
    );
    cfg.safe_backlog_inject_on_task_exhausted = yaml_to_bool(
        yaml_get(&raw, &["safe_backlog", "inject_on_task_exhausted"]),
        cfg.safe_backlog_inject_on_task_exhausted,
    );
    cfg.safe_backlog_diversity_rotation = yaml_to_bool(
        yaml_get(&raw, &["safe_backlog", "diversity_rotation"]),
        cfg.safe_backlog_diversity_rotation,
    );
    cfg.safe_backlog_novelty_window = yaml_to_i64(
        yaml_get(&raw, &["safe_backlog", "novelty_window"]),
        cfg.safe_backlog_novelty_window,
    )
    .max(1);
    cfg.safe_backlog_backoff_enabled = yaml_to_bool(
        yaml_get(&raw, &["safe_backlog", "backoff_enabled"]),
        cfg.safe_backlog_backoff_enabled,
    );
    cfg.safe_backlog_backoff_base_rounds = yaml_to_i64(
        yaml_get(&raw, &["safe_backlog", "backoff_base_rounds"]),
        cfg.safe_backlog_backoff_base_rounds,
    )
    .max(1);
    cfg.safe_backlog_backoff_max_rounds = yaml_to_i64(
        yaml_get(&raw, &["safe_backlog", "backoff_max_rounds"]),
        cfg.safe_backlog_backoff_max_rounds,
    )
    .max(cfg.safe_backlog_backoff_base_rounds);
    cfg.safe_backlog_backoff_jitter = yaml_to_f64(
        yaml_get(&raw, &["safe_backlog", "backoff_jitter"]),
        cfg.safe_backlog_backoff_jitter,
    )
    .clamp(0.0, 1.0);
    cfg.safe_backlog_backoff_force_probe_rounds = yaml_to_i64(
        yaml_get(&raw, &["safe_backlog", "backoff_force_probe_rounds"]),
        cfg.safe_backlog_backoff_force_probe_rounds,
    )
    .max(1);

    cfg.supervisor_enabled = yaml_to_bool(
        yaml_get(&raw, &["supervisor", "enabled"]),
        cfg.supervisor_enabled,
    );
    cfg.supervisor_mode = yaml_to_string(
        yaml_get(&raw, &["supervisor", "mode"]),
        &cfg.supervisor_mode,
    );
    cfg.supervisor_persona = yaml_to_string(
        yaml_get(&raw, &["supervisor", "persona"]),
        &cfg.supervisor_persona,
    );
    cfg.supervisor_trigger_no_progress_rounds = yaml_to_i64(
        yaml_get(&raw, &["supervisor", "trigger_no_progress_rounds"]),
        cfg.supervisor_trigger_no_progress_rounds,
    )
    .max(1);
    cfg.supervisor_cadence_rounds = yaml_to_i64(
        yaml_get(&raw, &["supervisor", "cadence_rounds"]),
        cfg.supervisor_cadence_rounds,
    )
    .max(1);
    cfg.supervisor_force_emit_rounds = yaml_to_i64(
        yaml_get(&raw, &["supervisor", "force_emit_rounds"]),
        cfg.supervisor_force_emit_rounds,
    )
    .max(1);
    cfg.supervisor_max_suggestions = yaml_to_i64(
        yaml_get(&raw, &["supervisor", "max_suggestions"]),
        cfg.supervisor_max_suggestions,
    )
    .max(1);

    cfg.understand_pass_threshold = yaml_to_i64(
        yaml_get(&raw, &["understand", "pass_threshold"]),
        cfg.understand_pass_threshold,
    )
    .clamp(0, 10);
    cfg.understand_require_confirmation = yaml_to_bool(
        yaml_get(&raw, &["understand", "require_confirmation"]),
        cfg.understand_require_confirmation,
    );
    cfg.understand_max_questions = yaml_to_i64(
        yaml_get(&raw, &["understand", "max_questions"]),
        cfg.understand_max_questions,
    )
    .max(1);

    cfg
}

pub fn load_backends_from_config(fusion_dir: &Path) -> (String, String) {
    let cfg = load_flat_config(fusion_dir);
    (cfg.backend_primary, cfg.backend_fallback)
}

pub fn write_dependency_report(fusion_dir: &Path, report: &DependencyReport) -> Result<PathBuf> {
    let path = fusion_dir.join("dependency_report.json");
    let text = serde_json::to_string_pretty(report)?;
    write_text(&path, &text)?;
    Ok(path)
}

pub fn remove_dependency_report_if_exists(fusion_dir: &Path) -> Result<()> {
    let path = fusion_dir.join("dependency_report.json");
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("failed removing file: {}", path.display()))?;
    }
    Ok(())
}

pub fn ensure_fusion_dir(fusion_dir: &Path) -> Result<()> {
    if !fusion_dir.is_dir() {
        anyhow::bail!("[fusion] .fusion not found");
    }
    let sessions = fusion_dir.join("sessions.json");
    if !sessions.is_file() {
        anyhow::bail!("[fusion] sessions.json not found");
    }
    Ok(())
}

pub fn append_event(
    fusion_dir: &Path,
    event_type: &str,
    from_state: &str,
    to_state: &str,
    payload: Value,
    idempotency_key: &str,
) -> Result<String> {
    let sessions_path = fusion_dir.join("sessions.json");
    let mut sessions = if sessions_path.is_file() {
        read_json(&sessions_path)?
    } else {
        Value::Object(Map::new())
    };

    if !sessions.is_object() {
        sessions = Value::Object(Map::new());
    }

    let mut counter = sessions
        .get("_runtime")
        .and_then(|v| v.get("last_event_counter"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    counter += 1;

    let event_id = format!("evt_{counter:06}");

    let root = sessions
        .as_object_mut()
        .expect("root object must exist for event update");
    let runtime = root
        .entry("_runtime")
        .or_insert_with(|| Value::Object(Map::new()));
    if !runtime.is_object() {
        *runtime = Value::Object(Map::new());
    }

    let runtime_obj = runtime
        .as_object_mut()
        .expect("runtime object must exist for event update");
    runtime_obj.insert("version".to_string(), Value::String("2.6.3".to_string()));
    runtime_obj.insert("state".to_string(), Value::String(to_state.to_string()));
    runtime_obj.insert("last_event_counter".to_string(), Value::from(counter));
    runtime_obj.insert("last_event_id".to_string(), Value::String(event_id.clone()));
    runtime_obj.insert(
        "updated_at".to_string(),
        Value::from(epoch_now_seconds_f64()),
    );

    write_json_pretty(&sessions_path, &sessions)?;

    let event = json!({
        "id": event_id,
        "idempotency_key": idempotency_key,
        "type": event_type,
        "from_state": from_state,
        "to_state": to_state,
        "payload": payload,
        "timestamp": epoch_now_seconds_f64(),
    });

    let events_path = fusion_dir.join("events.jsonl");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&events_path)
        .with_context(|| format!("failed opening file: {}", events_path.display()))?;

    let line = serde_json::to_string(&event)?;
    writeln!(file, "{line}")
        .with_context(|| format!("failed writing file: {}", events_path.display()))?;

    Ok(event["id"].as_str().unwrap_or_default().to_string())
}

pub fn parse_status_counts(s: &str) -> HashMap<&'static str, i64> {
    let mut counts = HashMap::new();
    counts.insert("completed", s.matches("[COMPLETED]").count() as i64);
    counts.insert("pending", s.matches("[PENDING]").count() as i64);
    counts.insert("in_progress", s.matches("[IN_PROGRESS]").count() as i64);
    counts.insert("failed", s.matches("[FAILED]").count() as i64);
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_backends_defaults() {
        let dir = tempdir().expect("tempdir");
        let (primary, fallback) = load_backends_from_config(dir.path());
        assert_eq!(primary, "codex");
        assert_eq!(fallback, "claude");
    }

    #[test]
    fn test_load_backends_from_yaml() {
        let dir = tempdir().expect("tempdir");
        let config = dir.path().join("config.yaml");
        write_text(&config, "backends:\n  primary: claude\n  fallback: codex\n")
            .expect("write config");

        let (primary, fallback) = load_backends_from_config(dir.path());
        assert_eq!(primary, "claude");
        assert_eq!(fallback, "codex");
    }

    #[test]
    fn test_dependency_report_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let report = DependencyReport {
            status: "blocked".into(),
            source: "fusion-bridge".into(),
            timestamp: utc_now_iso(),
            missing: vec!["codeagent-wrapper".into()],
            reason: "missing binary".into(),
            auto_attempted: vec!["codeagent-wrapper in PATH".into()],
            next_actions: vec!["install it".into()],
            agent_prompt: Some("help".into()),
        };

        let path = write_dependency_report(dir.path(), &report).expect("write report");
        let value = read_json(&path).expect("read json");
        assert_eq!(
            json_get_string(&value, &["status"]).as_deref(),
            Some("blocked")
        );
        assert_eq!(
            json_get_string(&value, &["source"]).as_deref(),
            Some("fusion-bridge")
        );
    }

    #[test]
    fn test_append_event_updates_sessions_and_events_jsonl() {
        let dir = tempdir().expect("tempdir");
        let sessions = dir.path().join("sessions.json");
        write_text(&sessions, "{}\n").expect("write sessions");

        let id = append_event(
            dir.path(),
            "SAFE_BACKLOG_INJECTED",
            "EXECUTE",
            "EXECUTE",
            json!({"added": 1}),
            "safe_backlog:test",
        )
        .expect("append event");
        assert_eq!(id, "evt_000001");

        let s = read_json(&sessions).expect("read sessions");
        assert_eq!(
            s.get("_runtime")
                .and_then(|v| v.get("last_event_counter"))
                .and_then(|v| v.as_i64()),
            Some(1)
        );

        let events_text = read_text(&dir.path().join("events.jsonl")).expect("read events");
        assert!(events_text.contains("SAFE_BACKLOG_INJECTED"));
    }

    #[test]
    fn test_load_flat_config_parses_safe_backlog() {
        let dir = tempdir().expect("tempdir");
        write_text(
            &dir.path().join("config.yaml"),
            "safe_backlog:\n  enabled: true\n  trigger_no_progress_rounds: 5\n",
        )
        .expect("write config");

        let cfg = load_flat_config(dir.path());
        assert!(cfg.safe_backlog_enabled);
        assert_eq!(cfg.safe_backlog_trigger_no_progress_rounds, 5);
    }
}
