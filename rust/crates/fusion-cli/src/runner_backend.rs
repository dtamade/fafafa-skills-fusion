use anyhow::{Context, Result};
use fusion_provider::{
    extract_session_id, run_backend, session_key_for_backend, session_key_for_backend_role,
    WrapperResolution,
};
use fusion_runtime_io::{
    json_set_string, utc_now_iso, write_dependency_report, write_json_pretty, DependencyReport,
};
use serde_json::json;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct BackendRunOutcome {
    pub(crate) output: String,
    pub(crate) exit_code: i32,
    pub(crate) used_backend: String,
    pub(crate) primary_ok: bool,
    pub(crate) primary_error: String,
    pub(crate) fallback_error: String,
}

pub(crate) fn clear_backend_failure_report(fusion_dir: &Path) -> Result<()> {
    let path = fusion_dir.join("backend_failure_report.json");
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("failed removing file: {}", path.display()))?;
    }
    Ok(())
}

pub(crate) fn resolve_wrapper_or_dependency_error(
    fusion_dir: &Path,
    cwd: &Path,
    explicit_bin: Option<&str>,
    report_source: &str,
    rerun_command: &str,
    clear_backend_failure_report: impl Fn(&Path) -> Result<()>,
) -> Result<Result<WrapperResolution, (String, i32)>> {
    match fusion_provider::resolve_wrapper_bin(explicit_bin, cwd) {
        Ok(resolved) => Ok(Ok(resolved)),
        Err(_) => {
            clear_backend_failure_report(fusion_dir)?;

            let report = DependencyReport {
                status: "blocked".to_string(),
                source: report_source.to_string(),
                timestamp: utc_now_iso(),
                missing: vec!["codeagent-wrapper".to_string()],
                reason: "Missing executable for backend orchestration".to_string(),
                auto_attempted: vec![
                    explicit_bin.unwrap_or_default().to_string(),
                    "codeagent-wrapper in PATH".to_string(),
                    "./node_modules/.bin/codeagent-wrapper".to_string(),
                    "~/.local/bin/codeagent-wrapper".to_string(),
                    "~/.npm-global/bin/codeagent-wrapper".to_string(),
                ],
                next_actions: vec![
                    "Install or expose codeagent-wrapper in PATH.".to_string(),
                    "Or set CODEAGENT_WRAPPER_BIN to an executable path.".to_string(),
                    format!("Re-run: {rerun_command}"),
                ],
                agent_prompt: Some(format!(
                    "Dependency missing: codeagent-wrapper. Resolve installation/path and retry {rerun_command}."
                )),
            };

            let path = write_dependency_report(fusion_dir, &report)?;
            let message = format!(
                "[fusion][deps] Missing dependency: codeagent-wrapper\n[fusion][deps] Report written: {}\n",
                path.display()
            );
            Ok(Err((message, 127)))
        }
    }
}

pub(crate) fn write_backend_failure_report(
    fusion_dir: &Path,
    report_source: &str,
    primary_backend: &str,
    fallback_backend: &str,
    primary_error: &str,
    fallback_error: &str,
) -> Result<()> {
    let report = json!({
        "status": "blocked",
        "source": report_source,
        "timestamp": utc_now_iso(),
        "primary_backend": primary_backend,
        "fallback_backend": fallback_backend,
        "primary_error": primary_error,
        "fallback_error": fallback_error,
        "next_actions": ["Check backend network/credentials and retry with explicit backend override."],
    });
    write_json_pretty(&fusion_dir.join("backend_failure_report.json"), &report)
}

pub(crate) fn persist_backend_success(
    fusion_dir: &Path,
    sessions_path: &Path,
    sessions: &mut serde_json::Value,
    output: &str,
    used_backend: &str,
    session_role: Option<&str>,
) -> Result<()> {
    clear_backend_failure_report(fusion_dir)?;
    if let Some(session_id) = extract_session_id(output) {
        let session_key = session_key_for_backend_role(used_backend, session_role);
        json_set_string(sessions, &session_key, &session_id);
        if session_role.is_some() {
            let legacy_key = session_key_for_backend(used_backend);
            json_set_string(sessions, legacy_key, &session_id);
        }
        write_json_pretty(sessions_path, sessions)?;
    }
    Ok(())
}

pub(crate) fn execute_backend_with_fallback(
    wrapper: &WrapperResolution,
    primary: &str,
    fallback: &str,
    prompt: &str,
    primary_session: Option<&str>,
    cwd: &Path,
    timeout_sec: Option<u64>,
) -> Result<BackendRunOutcome> {
    let mut used_backend = primary.to_string();
    let mut primary_ok = false;
    let mut primary_error = String::new();
    let mut fallback_error = String::new();

    let first_result = run_backend(
        &wrapper.bin,
        primary,
        prompt,
        primary_session,
        cwd,
        timeout_sec,
    )?;
    let mut exit_code = first_result.exit_code;
    let output = if first_result.exit_code == 0 {
        primary_ok = true;
        first_result.output
    } else {
        primary_error = first_result.output;
        if primary_session.is_some_and(|sid| !sid.is_empty()) {
            eprintln!("[fusion] primary resume failed, retry without resume on {primary}");
            let retry_result = run_backend(&wrapper.bin, primary, prompt, None, cwd, timeout_sec)?;
            exit_code = retry_result.exit_code;
            if retry_result.exit_code == 0 {
                primary_ok = true;
                retry_result.output
            } else {
                if !primary_error.is_empty() && !retry_result.output.is_empty() {
                    primary_error.push('\n');
                }
                primary_error.push_str(&retry_result.output);
                eprintln!("[fusion] primary backend failed, fallback to {fallback}");
                used_backend = fallback.to_string();
                let fallback_result =
                    run_backend(&wrapper.bin, fallback, prompt, None, cwd, timeout_sec)?;
                exit_code = fallback_result.exit_code;
                if fallback_result.exit_code == 0 {
                    primary_ok = true;
                    fallback_result.output
                } else {
                    fallback_error = fallback_result.output;
                    fallback_error.clone()
                }
            }
        } else {
            eprintln!("[fusion] primary backend failed, fallback to {fallback}");
            used_backend = fallback.to_string();
            let fallback_result =
                run_backend(&wrapper.bin, fallback, prompt, None, cwd, timeout_sec)?;
            exit_code = fallback_result.exit_code;
            if fallback_result.exit_code == 0 {
                primary_ok = true;
                fallback_result.output
            } else {
                fallback_error = fallback_result.output;
                fallback_error.clone()
            }
        }
    };

    Ok(BackendRunOutcome {
        output,
        exit_code,
        used_backend,
        primary_ok,
        primary_error,
        fallback_error,
    })
}
