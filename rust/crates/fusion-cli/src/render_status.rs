use anyhow::Result;
use fusion_runtime_io::read_text;
use serde_json::Value;
use std::path::Path;

pub(crate) fn guardian_status_from_metrics(no_progress: i64, same_action: i64) -> String {
    if no_progress >= 4 || same_action >= 2 {
        "⚠ BACKOFF".to_string()
    } else if no_progress >= 2 {
        "~".to_string()
    } else {
        "OK".to_string()
    }
}

pub(crate) fn read_guardian_status(fusion_dir: &Path) -> String {
    let loop_context = fusion_dir.join("loop_context.json");
    if !loop_context.is_file() {
        return "OK".to_string();
    }

    let payload = read_text(&loop_context)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        .unwrap_or(Value::Null);

    let no_progress = payload
        .get("no_progress_rounds")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let same_action = payload
        .get("same_action_count")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);

    guardian_status_from_metrics(no_progress, same_action)
}

pub(crate) fn extract_status_block(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let idx = lines
        .iter()
        .position(|line| line.starts_with("## Status"))?;
    let end = usize::min(idx + 6, lines.len());
    let block = lines[idx..end].join("\n");
    Some(format!("{block}\n"))
}

pub(crate) fn last_safe_backlog(events_file: &Path) -> Result<Option<(i64, f64)>> {
    let content = read_text(events_file)?;
    let mut latest: Option<(i64, f64)> = None;

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value
            .get("type")
            .and_then(|v| v.as_str())
            .map(|kind| kind == "SAFE_BACKLOG_INJECTED")
            != Some(true)
        {
            continue;
        }

        let added = value
            .get("payload")
            .and_then(|v| v.get("added"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let timestamp = value
            .get("timestamp")
            .and_then(|v| v.as_f64())
            .unwrap_or_default();
        latest = Some((added, timestamp));
    }

    Ok(latest)
}

pub(crate) fn epoch_to_iso(timestamp: f64) -> Option<String> {
    use chrono::{DateTime, Utc};

    if !timestamp.is_finite() {
        return None;
    }
    let sec = timestamp.trunc() as i64;
    let nanos = ((timestamp.fract() * 1_000_000_000.0).round() as i64).max(0) as u32;
    let dt = DateTime::<Utc>::from_timestamp(sec, nanos)?;
    Some(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_status_block() {
        let input = "A\n## Status\n- a\n- b\n- c\n- d\n- e\n- f\n";
        let block = extract_status_block(input).expect("status block");
        assert!(block.contains("## Status"));
        assert!(!block.contains("- f"));
    }

    #[test]
    fn test_guardian_status_from_metrics() {
        assert_eq!(guardian_status_from_metrics(0, 0), "OK");
        assert_eq!(guardian_status_from_metrics(2, 0), "~");
        assert_eq!(guardian_status_from_metrics(4, 0), "⚠ BACKOFF");
        assert_eq!(guardian_status_from_metrics(0, 2), "⚠ BACKOFF");
    }
}
