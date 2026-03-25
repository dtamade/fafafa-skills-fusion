use anyhow::Result;
use fusion_runtime_io::{json_get_string, read_json};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub(crate) struct DependencySummary {
    pub(crate) status: String,
    pub(crate) source: String,
    pub(crate) reason: String,
    pub(crate) missing: String,
    pub(crate) next: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BackendFailureSummary {
    pub(crate) status: String,
    pub(crate) source: String,
    pub(crate) primary_backend: String,
    pub(crate) fallback_backend: String,
    pub(crate) primary_error: String,
    pub(crate) fallback_error: String,
    pub(crate) next: String,
}

pub(crate) fn read_dependency_summary(fusion_dir: &Path) -> Result<DependencySummary> {
    let path = fusion_dir.join("dependency_report.json");
    if !path.is_file() {
        return Ok(DependencySummary::default());
    }

    let value = read_json(&path)?;
    let missing = value
        .get("missing")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    Ok(DependencySummary {
        status: json_get_string(&value, &["status"]).unwrap_or_default(),
        source: json_get_string(&value, &["source"]).unwrap_or_default(),
        reason: json_get_string(&value, &["reason"]).unwrap_or_default(),
        missing,
        next: value
            .get("next_actions")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
    })
}

pub(crate) fn read_backend_failure_summary(fusion_dir: &Path) -> Result<BackendFailureSummary> {
    let path = fusion_dir.join("backend_failure_report.json");
    if !path.is_file() {
        return Ok(BackendFailureSummary::default());
    }

    let value = read_json(&path)?;
    Ok(BackendFailureSummary {
        status: json_get_string(&value, &["status"]).unwrap_or_default(),
        source: json_get_string(&value, &["source"]).unwrap_or_default(),
        primary_backend: json_get_string(&value, &["primary_backend"]).unwrap_or_default(),
        fallback_backend: json_get_string(&value, &["fallback_backend"]).unwrap_or_default(),
        primary_error: json_get_string(&value, &["primary_error"]).unwrap_or_default(),
        fallback_error: json_get_string(&value, &["fallback_error"]).unwrap_or_default(),
        next: value
            .get("next_actions")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_dependency_summary_extracts_fields() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("dependency_report.json"),
            r#"{"status":"missing_dependency","source":"codeagent","reason":"missing wrapper","missing":["foo","bar"],"next_actions":["install foo"]}"#,
        )
        .expect("write dependency report");

        let summary = read_dependency_summary(dir.path()).expect("dependency summary");
        assert_eq!(summary.status, "missing_dependency");
        assert_eq!(summary.source, "codeagent");
        assert_eq!(summary.reason, "missing wrapper");
        assert_eq!(summary.missing, "foo, bar");
        assert_eq!(summary.next, "install foo");
    }

    #[test]
    fn test_read_backend_failure_summary_extracts_fields() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("backend_failure_report.json"),
            r#"{"status":"fallback_failed","source":"codeagent","primary_backend":"codex","fallback_backend":"claude","primary_error":"p","fallback_error":"f","next_actions":["retry later"]}"#,
        )
        .expect("write backend report");

        let summary = read_backend_failure_summary(dir.path()).expect("backend summary");
        assert_eq!(summary.status, "fallback_failed");
        assert_eq!(summary.source, "codeagent");
        assert_eq!(summary.primary_backend, "codex");
        assert_eq!(summary.fallback_backend, "claude");
        assert_eq!(summary.primary_error, "p");
        assert_eq!(summary.fallback_error, "f");
        assert_eq!(summary.next, "retry later");
    }
}
