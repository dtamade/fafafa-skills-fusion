use anyhow::Result;
use fusion_runtime_io::read_text;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const FUSION_FILES: [&str; 4] = [
    "task_plan.md",
    "progress.md",
    "findings.md",
    "sessions.json",
];
const META_PREFIXES: [&str; 3] = ["<local-command", "<command-", "<task-notification"];

#[derive(Debug)]
pub(crate) struct SessionMessage {
    pub(crate) line_num: usize,
    pub(crate) value: Value,
}

#[derive(Debug)]
pub(crate) struct UnsyncedMessage {
    pub(crate) role: &'static str,
    pub(crate) content: String,
    pub(crate) tools: Vec<String>,
}

pub(crate) fn get_sessions_sorted(project_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut sessions = Vec::new();
    for entry in fs::read_dir(project_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("agent-"))
        {
            continue;
        }
        sessions.push(path);
    }

    sessions.sort_by(|left, right| {
        let left_modified = left.metadata().and_then(|meta| meta.modified()).ok();
        let right_modified = right.metadata().and_then(|meta| meta.modified()).ok();
        right_modified.cmp(&left_modified)
    });
    Ok(sessions)
}

pub(crate) fn select_target_session(sessions: &[PathBuf]) -> Option<&PathBuf> {
    sessions
        .iter()
        .find(|path| {
            path.metadata()
                .map(|meta| meta.len() > 5_000)
                .unwrap_or(false)
        })
        .or_else(|| sessions.first())
}

pub(crate) fn parse_session_messages(session_file: &Path) -> Result<Vec<SessionMessage>> {
    let mut messages = Vec::new();
    for (line_num, line) in read_text(session_file)?.lines().enumerate() {
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            messages.push(SessionMessage { line_num, value });
        }
    }
    Ok(messages)
}

pub(crate) fn find_last_fusion_update(messages: &[SessionMessage]) -> (isize, Option<String>) {
    let mut last_line = -1;
    let mut last_file = None;

    for message in messages {
        if message.value.get("type").and_then(|value| value.as_str()) != Some("assistant") {
            continue;
        }
        let Some(content) = message
            .value
            .get("message")
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_array())
        else {
            continue;
        };

        for item in content {
            if item.get("type").and_then(|value| value.as_str()) != Some("tool_use") {
                continue;
            }
            let tool_name = item
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            if tool_name != "Write" && tool_name != "Edit" {
                continue;
            }
            let file_path = item
                .get("input")
                .and_then(|value| value.get("file_path"))
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            if let Some(file_name) = match_fusion_file(file_path) {
                last_line = message.line_num as isize;
                last_file = Some(file_name.to_string());
            }
        }
    }

    (last_line, last_file)
}

fn match_fusion_file(file_path: &str) -> Option<&'static str> {
    let normalized = file_path.replace('\\', "/");
    FUSION_FILES.into_iter().find(|file_name| {
        normalized.contains(&format!(".fusion/{file_name}"))
            || normalized.ends_with(&format!("/.fusion/{file_name}"))
    })
}

pub(crate) fn extract_unsynced(
    messages: &[SessionMessage],
    after_line: isize,
) -> Vec<UnsyncedMessage> {
    let mut result = Vec::new();
    for message in messages {
        if message.line_num as isize <= after_line {
            continue;
        }

        let message_type = message
            .value
            .get("type")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let is_meta = message
            .value
            .get("isMeta")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);

        if message_type == "user" && !is_meta {
            let content = extract_primary_text(&message.value);
            if content.chars().count() > 20
                && !META_PREFIXES
                    .iter()
                    .any(|prefix| content.starts_with(prefix))
            {
                result.push(UnsyncedMessage {
                    role: "user",
                    content,
                    tools: Vec::new(),
                });
            }
            continue;
        }

        if message_type == "assistant" {
            let (content, tools) = extract_assistant_content(&message.value);
            if !content.is_empty() || !tools.is_empty() {
                result.push(UnsyncedMessage {
                    role: "assistant",
                    content,
                    tools,
                });
            }
        }
    }
    result
}

fn extract_primary_text(message: &Value) -> String {
    let Some(content) = message
        .get("message")
        .and_then(|value| value.get("content"))
    else {
        return String::new();
    };

    match content {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .find_map(|item| {
                (item.get("type").and_then(|value| value.as_str()) == Some("text")).then(|| {
                    item.get("text")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default()
                        .to_string()
                })
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn extract_assistant_content(message: &Value) -> (String, Vec<String>) {
    let Some(content) = message
        .get("message")
        .and_then(|value| value.get("content"))
    else {
        return (String::new(), Vec::new());
    };

    match content {
        Value::String(text) => (text.clone(), Vec::new()),
        Value::Array(items) => {
            let mut text_content = String::new();
            let mut tool_uses = Vec::new();
            for item in items {
                let item_type = item
                    .get("type")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                match item_type {
                    "text" => {
                        text_content = item
                            .get("text")
                            .and_then(|value| value.as_str())
                            .unwrap_or_default()
                            .to_string();
                    }
                    "tool_use" => {
                        let tool_name = item
                            .get("name")
                            .and_then(|value| value.as_str())
                            .unwrap_or_default();
                        let tool_input = item.get("input").unwrap_or(&Value::Null);
                        let summary = match tool_name {
                            "Edit" | "Write" => format!(
                                "{tool_name}: {}",
                                tool_input
                                    .get("file_path")
                                    .and_then(|value| value.as_str())
                                    .unwrap_or("?")
                            ),
                            "Bash" => format!(
                                "Bash: {}",
                                truncate_chars(
                                    tool_input
                                        .get("command")
                                        .and_then(|value| value.as_str())
                                        .unwrap_or_default(),
                                    80,
                                )
                            ),
                            _ => tool_name.to_string(),
                        };
                        tool_uses.push(summary);
                    }
                    _ => {}
                }
            }
            (truncate_chars(&text_content, 400), tool_uses)
        }
        _ => (String::new(), Vec::new()),
    }
}

pub(crate) fn truncate_chars(input: &str, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}
