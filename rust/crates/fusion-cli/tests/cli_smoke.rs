#![allow(deprecated)]

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::tempdir;

fn write_mock_executable(
    dir: &Path,
    base_name: &str,
    unix_body: &str,
    _windows_body: &str,
) -> PathBuf {
    #[cfg(windows)]
    let path = dir.join(format!("{base_name}.cmd"));
    #[cfg(not(windows))]
    let path = dir.join(base_name);

    #[cfg(windows)]
    let content = _windows_body;
    #[cfg(not(windows))]
    let content = unix_body;

    fs::write(&path, content).expect("write mock executable");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("chmod");
    }

    path
}

fn prepend_path(dir: &Path) -> std::ffi::OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![dir.to_path_buf()];
    paths.extend(std::env::split_paths(&existing));
    std::env::join_paths(paths).expect("join PATH")
}

fn bash_script_arg(path: &Path) -> String {
    #[cfg(windows)]
    {
        let mut value = path.to_string_lossy().replace('\\', "/");
        if let Some(stripped) = value.strip_prefix("//?/") {
            value = stripped.to_string();
        }
        let bytes = value.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' {
            let drive = value[..1].to_ascii_lowercase();
            let rest = &value[2..];
            return format!("/{drive}{rest}");
        }
        value
    }
    #[cfg(not(windows))]
    {
        path.to_string_lossy().into_owned()
    }
}

fn claude_project_slug_for_test(project_path: &Path) -> String {
    let mut normalized = project_path.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = normalized.strip_prefix("//?/") {
        normalized = stripped.to_string();
    }
    let bytes = normalized.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' {
        normalized = normalized[2..].to_string();
    }
    let mut sanitized = normalized.replace('/', "-");
    if !sanitized.starts_with('-') {
        sanitized.insert(0, '-');
    }
    sanitized.replace('_', "-")
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn contains_normalized(haystack: &str, needle: &str) -> bool {
    normalize_whitespace(haystack).contains(&normalize_whitespace(needle))
}

fn line_contains_normalized(haystack: &str, needle: &str) -> bool {
    haystack
        .lines()
        .any(|line| normalize_whitespace(line).contains(&normalize_whitespace(needle)))
}

fn run_success_output(cmd: &mut Command, context: &str) -> Output {
    let output = cmd.output().expect(context);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn run_success_stdout(cmd: &mut Command, context: &str) -> String {
    let output = run_success_output(cmd, context);
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn assert_stdout_trimmed_eq(cmd: &mut Command, context: &str, expected: &str) {
    let stdout = run_success_stdout(cmd, context);
    assert_eq!(stdout.trim(), expected);
}

fn assert_stdout_lines_eq(cmd: &mut Command, context: &str, expected: &[&str]) {
    let stdout = run_success_stdout(cmd, context);
    assert_eq!(stdout.lines().collect::<Vec<_>>(), expected);
}

fn run_failure_output(cmd: &mut Command, context: &str, expected_code: Option<i32>) -> Output {
    let output = cmd.output().expect(context);
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if let Some(code) = expected_code {
        assert_eq!(
            output.status.code(),
            Some(code),
            "unexpected exit code\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output
}

fn assert_stderr_contains_normalized(
    cmd: &mut Command,
    context: &str,
    expected_code: Option<i32>,
    expected: &str,
) {
    let output = run_failure_output(cmd, context, expected_code);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        contains_normalized(&stderr, expected),
        "expected stderr to contain normalized `{expected}`\nactual stderr:\n{stderr}"
    );
}

fn pending_to_completed_wrapper(bin_dir: &Path, plan_path: &Path) -> PathBuf {
    let unix_body = format!(
        "#!/bin/bash\nset -euo pipefail\n\nplan=\"{}\"\nif grep -q '\\[PENDING\\]' \"$plan\"; then\n  sed -i.bak 's/\\[PENDING\\]/[COMPLETED]/' \"$plan\"\n  rm -f \"$plan.bak\"\nfi\necho \"ok\"\n",
        plan_path.display()
    );
    let windows_body = format!(
        "@echo off\r\nset \"PLAN={}\"\r\npowershell -NoProfile -Command \"$p=$env:PLAN; $c=Get-Content -Raw $p; $c=$c -replace '\\[PENDING\\]','[COMPLETED]'; Set-Content -Path $p -Value $c\"\r\necho ok\r\n",
        plan_path.display()
    );
    write_mock_executable(bin_dir, "codeagent-wrapper", &unix_body, &windows_body)
}

fn approve_pending_review_wrapper(bin_dir: &Path, plan_path: &Path) -> PathBuf {
    let unix_body = format!(
        "#!/bin/bash\nset -euo pipefail\n\nplan=\"{}\"\nif grep -q 'Task 2: Build API \\[IN_PROGRESS\\]' \"$plan\" && grep -q -- '- Review-Status: pending' \"$plan\"; then\n  sed -i.bak 's/Task 2: Build API \\[IN_PROGRESS\\]/Task 2: Build API [COMPLETED]/' \"$plan\"\n  sed -i.bak 's/- Review-Status: pending/- Review-Status: approved/' \"$plan\"\n  rm -f \"$plan.bak\"\nfi\necho \"ok\"\n",
        plan_path.display()
    );
    let windows_body = format!(
        "@echo off\r\nset \"PLAN={}\"\r\npowershell -NoProfile -Command \"$p=$env:PLAN; $c=Get-Content -Raw $p; if($c -match 'Task 2: Build API \\[IN_PROGRESS\\]' -and $c -match '- Review-Status: pending'){{ $c=$c -replace 'Task 2: Build API \\[IN_PROGRESS\\]','Task 2: Build API [COMPLETED]'; $c=$c -replace '- Review-Status: pending','- Review-Status: approved' }}; Set-Content -Path $p -Value $c\"\r\necho ok\r\n",
        plan_path.display()
    );
    write_mock_executable(bin_dir, "codeagent-wrapper", &unix_body, &windows_body)
}

fn init_git_repo(dir: &Path) {
    Command::new("git")
        .arg("init")
        .current_dir(dir)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "fusion@example.com"])
        .current_dir(dir)
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "Fusion Test"])
        .current_dir(dir)
        .output()
        .expect("git config name");
}

fn retired_skip_flag() -> String {
    ["--skip-", "py", "thon"].concat()
}

fn retired_skip_field() -> String {
    ["skip_", "py", "thon"].concat()
}

fn retired_test_command() -> String {
    [["py", "test"].concat(), " -q".to_string()].concat()
}

#[test]
fn inspect_json_field_reads_from_stdin_and_file() {
    let temp = tempdir().expect("tempdir");
    let payload_path = temp.path().join("payload.json");
    fs::write(
        &payload_path,
        serde_json::to_string_pretty(&json!({
            "result": "ok",
            "warn_count": 3
        }))
        .expect("json"),
    )
    .expect("payload");

    let mut file_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    file_cmd
        .arg("inspect")
        .arg("json-field")
        .arg("--file")
        .arg(&payload_path)
        .arg("--key")
        .arg("result");
    assert_stdout_trimmed_eq(&mut file_cmd, "run inspect json-field result", "ok");

    let mut number_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    number_cmd
        .arg("inspect")
        .arg("json-field")
        .arg("--file")
        .arg(&payload_path)
        .arg("--key")
        .arg("warn_count")
        .arg("--number");
    assert_stdout_trimmed_eq(&mut number_cmd, "run inspect json-field number", "3");

    let mut bool_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    bool_cmd
        .arg("inspect")
        .arg("json-field")
        .arg("--file")
        .arg(&payload_path)
        .arg("--key")
        .arg("ok")
        .arg("--bool");
    fs::write(
        &payload_path,
        serde_json::to_string_pretty(&json!({
            "ok": true,
            "warn_count": 3
        }))
        .expect("json"),
    )
    .expect("payload bool");
    assert_stdout_trimmed_eq(&mut bool_cmd, "run inspect json-field bool", "true");
}

#[test]
fn inspect_task_plan_reports_counts_next_and_type() {
    let temp = tempdir().expect("tempdir");
    let task_plan = temp.path().join("task_plan.md");
    fs::write(
        &task_plan,
        "### Task 1: Done [COMPLETED]\n\
### Task 2: Build parser [IN_PROGRESS]\n\
- Type: implementation\n\
### Task 3: Docs [PENDING]\n\
- Type: documentation\n",
    )
    .expect("task plan");

    let mut counts_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    counts_cmd
        .arg("inspect")
        .arg("task-plan")
        .arg("--file")
        .arg(&task_plan)
        .arg("counts");
    assert_stdout_trimmed_eq(&mut counts_cmd, "run inspect task-plan counts", "1:1:1:0");

    let mut next_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    next_cmd
        .arg("inspect")
        .arg("task-plan")
        .arg("--file")
        .arg(&task_plan)
        .arg("next");
    assert_stdout_trimmed_eq(&mut next_cmd, "run inspect task-plan next", "Build parser");

    let mut type_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    type_cmd
        .arg("inspect")
        .arg("task-plan")
        .arg("--file")
        .arg(&task_plan)
        .arg("task-type")
        .arg("--title")
        .arg("Build parser");
    assert_stdout_trimmed_eq(
        &mut type_cmd,
        "run inspect task-plan task-type",
        "implementation",
    );
}

#[test]
fn inspect_runtime_config_reports_runtime_fields() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:\n  enabled: true\n  compat_mode: false\n  engine: legacy\n",
    )
    .expect("config");

    let mut enabled_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    enabled_cmd
        .current_dir(temp.path())
        .arg("inspect")
        .arg("runtime-config")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--field")
        .arg("enabled");
    assert_stdout_trimmed_eq(
        &mut enabled_cmd,
        "run inspect runtime-config enabled",
        "true",
    );

    let mut engine_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    engine_cmd
        .current_dir(temp.path())
        .arg("inspect")
        .arg("runtime-config")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--field")
        .arg("engine");
    assert_stdout_trimmed_eq(&mut engine_cmd, "run inspect runtime-config engine", "rust");

    let mut compat_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    compat_cmd
        .current_dir(temp.path())
        .arg("inspect")
        .arg("runtime-config")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--field")
        .arg("compat_mode");
    assert_stdout_trimmed_eq(
        &mut compat_cmd,
        "run inspect runtime-config compat_mode",
        "false",
    );
}

#[test]
fn inspect_loop_context_reports_arrays_and_state_visits() {
    let temp = tempdir().expect("tempdir");
    let loop_context = temp.path().join("loop_context.json");
    fs::write(
        &loop_context,
        serde_json::to_string_pretty(&json!({
            "completed_count_history": [1, 2, 3],
            "state_visits": { "EXECUTE": 4, "REVIEW": 1 },
            "decision_history": [
                { "decision": "BACKOFF", "reason": "same action", "timestamp": 1 }
            ]
        }))
        .expect("json"),
    )
    .expect("loop context");

    let mut array_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    array_cmd
        .arg("inspect")
        .arg("loop-context")
        .arg("--file")
        .arg(&loop_context)
        .arg("array-values")
        .arg("--key")
        .arg("completed_count_history");
    assert_stdout_lines_eq(
        &mut array_cmd,
        "run inspect loop-context array-values",
        &["1", "2", "3"],
    );

    let mut state_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    state_cmd
        .arg("inspect")
        .arg("loop-context")
        .arg("--file")
        .arg(&loop_context)
        .arg("state-visits");
    let output = state_cmd.output().expect("run state visits");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "EXECUTE=4"));
    assert!(line_contains_normalized(&stdout, "REVIEW=1"));

    let mut decision_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    decision_cmd
        .arg("inspect")
        .arg("loop-context")
        .arg("--file")
        .arg(&loop_context)
        .arg("decision-history");
    let output = decision_cmd.output().expect("run decision history");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    let entry: serde_json::Value =
        serde_json::from_str(lines[0]).expect("parse decision history entry");
    assert_eq!(
        entry.get("decision").and_then(|v| v.as_str()),
        Some("BACKOFF")
    );
}

#[test]
fn inspect_loop_guardian_config_reports_thresholds() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::write(
        fusion.join("config.yaml"),
        "loop_guardian:\n  max_iterations: 7\n  max_no_progress: 2\n",
    )
    .expect("config");

    let mut iterations_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    iterations_cmd
        .current_dir(temp.path())
        .arg("inspect")
        .arg("loop-guardian-config")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--field")
        .arg("max_iterations");
    assert_stdout_trimmed_eq(
        &mut iterations_cmd,
        "run inspect loop-guardian-config iterations",
        "7",
    );

    let mut no_progress_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    no_progress_cmd
        .current_dir(temp.path())
        .arg("inspect")
        .arg("loop-guardian-config")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--field")
        .arg("max_no_progress");
    assert_stdout_trimmed_eq(
        &mut no_progress_cmd,
        "run inspect loop-guardian-config no_progress",
        "2",
    );
}

#[test]
fn doctor_json_fix_writes_project_settings() {
    let temp = tempdir().expect("tempdir");
    let project = temp.path().join("project_fix");
    let fusion = project.join(".fusion");
    let home = temp.path().join("home");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(home.join(".claude")).expect("create home claude");

    fs::write(
        home.join(".claude").join("settings.json"),
        serde_json::to_string_pretty(&json!({
            "hooks": {
                "PreToolUse": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-pretool.sh\""}]}],
                "Stop": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-stop-guard.sh\""}]}]
            }
        }))
        .expect("json"),
    )
    .expect("settings");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "completed",
            "current_phase": "DELIVER"
        }))
        .expect("json"),
    )
    .expect("sessions");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [COMPLETED]\n").expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("doctor")
        .arg("--json")
        .arg("--fix")
        .arg(&project)
        .env("HOME", &home);

    let output = cmd.output().expect("run doctor");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse doctor json");
    assert_eq!(payload.get("result").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(payload.get("fixed").and_then(|v| v.as_bool()), Some(true));

    let settings_local = project.join(".claude").join("settings.local.json");
    assert!(settings_local.exists());
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(settings_local).expect("settings.local"))
            .expect("parse settings.local");
    assert_eq!(
        settings.get("hooks"),
        Some(&json!({
            "PreToolUse": [{
                "matcher": "Write|Edit|Bash|Read|Glob|Grep",
                "hooks": [{
                    "type": "command",
                    "command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh\""
                }]
            }],
            "PostToolUse": [{
                "matcher": "Write|Edit",
                "hooks": [{
                    "type": "command",
                    "command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh\""
                }]
            }],
            "Stop": [{
                "hooks": [{
                    "type": "command",
                    "command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh\""
                }]
            }]
        }))
    );
}

#[test]
fn doctor_json_normalizes_stale_legacy_engine_config() {
    let temp = tempdir().expect("tempdir");
    let project = temp.path().join("project_legacy_engine");
    let fusion = project.join(".fusion");
    fs::create_dir_all(project.join(".claude")).expect("create claude");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        project.join(".claude").join("settings.local.json"),
        serde_json::to_string_pretty(&json!({
            "hooks": {
                "PreToolUse": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh\""}]}],
                "PostToolUse": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh\""}]}],
                "Stop": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh\""}]}]
            }
        }))
        .expect("json"),
    )
    .expect("settings.local");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:\n  enabled: true\n  compat_mode: true\n  engine: legacy\n",
    )
    .expect("config");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "completed",
            "current_phase": "DELIVER"
        }))
        .expect("json"),
    )
    .expect("sessions");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [COMPLETED]\n").expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("doctor")
        .arg("--json")
        .arg(&project);

    let output = cmd.output().expect("run doctor");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse doctor json");
    assert_eq!(payload.get("result").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(payload.get("warn_count").and_then(|v| v.as_u64()), Some(0));
    assert_eq!(payload.get("fixed").and_then(|v| v.as_bool()), Some(false));
}

#[test]
fn audit_dry_run_json_outputs_summary() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(&repo_root)
        .arg("audit")
        .arg("--dry-run")
        .arg("--json")
        .arg("--fast")
        .arg("--skip-rust")
        .output()
        .expect("run audit");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    let commands: Vec<&str> = payload["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .map(|item| item.as_str().expect("command string"))
        .collect();

    assert_eq!(payload["result"].as_str(), Some("ok"));
    assert_eq!(payload["dry_run"].as_bool(), Some(true));
    assert_eq!(payload["schema_version"].as_str(), Some("v1"));
    let retired_skip = retired_skip_field();
    let retired_test = retired_test_command();
    assert!(payload["flags"].get(retired_skip.as_str()).is_none());
    assert_eq!(
        commands,
        vec![
            "bash -n scripts/*.sh",
            "bash scripts/ci-machine-mode-smoke.sh",
        ]
    );
    assert_eq!(
        payload["commands_count"].as_u64(),
        Some(commands.len() as u64)
    );
    assert_eq!(payload["steps_executed"].as_u64(), Some(0));
    assert!(!commands
        .iter()
        .any(|command| command.contains(&retired_test)));
    assert!(!commands
        .iter()
        .any(|command| command.contains("test_fusion_")));
}

#[test]
fn audit_rejects_unknown_legacy_skip_flag() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let retired_skip = retired_skip_flag();

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(&repo_root).arg("audit").arg(&retired_skip);

    assert_stderr_contains_normalized(
        &mut cmd,
        "run audit with retired skip flag",
        None,
        &retired_skip,
    );
}

#[test]
fn audit_force_fail_step_json_reports_exit_code() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(&repo_root)
        .arg("audit")
        .arg("--json")
        .arg("--skip-rust")
        .env("FUSION_RELEASE_AUDIT_FORCE_FAIL_STEP", "1")
        .output()
        .expect("run audit");

    assert_eq!(output.status.code(), Some(1));
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    let commands: Vec<&str> = payload["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .map(|item| item.as_str().expect("command string"))
        .collect();
    let failed_commands: Vec<&str> = payload["failed_commands"]
        .as_array()
        .expect("failed commands array")
        .iter()
        .map(|item| item.as_str().expect("failed command string"))
        .collect();
    let failed_command_rate = payload["failed_command_rate"]
        .as_f64()
        .expect("failed command rate");
    let retired_skip = retired_skip_field();

    assert_eq!(payload["result"].as_str(), Some("error"));
    assert!(payload["flags"].get(retired_skip.as_str()).is_none());
    assert_eq!(payload["failed_steps"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["failed_steps"][0].as_u64(), Some(1));
    assert_eq!(
        commands,
        vec![
            "bash -n scripts/*.sh",
            "bash scripts/ci-machine-mode-smoke.sh",
            "bash scripts/ci-cross-platform-smoke.sh",
        ]
    );
    assert_eq!(failed_commands, vec!["bash -n scripts/*.sh"]);
    assert_eq!(
        payload["commands_count"].as_u64(),
        Some(commands.len() as u64)
    );
    assert_eq!(
        payload["command_rate_basis"].as_u64(),
        Some(commands.len() as u64)
    );
    assert!((failed_command_rate - (1.0 / 3.0)).abs() < 1e-12);
}

#[test]
fn regression_list_suites_json_outputs_machine_payload() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(&repo_root)
        .arg("regression")
        .arg("--list-suites")
        .arg("--json");

    let output = cmd.output().expect("run list suites");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse suites json");
    let suites: Vec<&str> = payload["suites"]
        .as_array()
        .expect("suites array")
        .iter()
        .map(|item| item.as_str().expect("suite string"))
        .collect();
    assert_eq!(payload.get("result").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(
        payload.get("default_suite").and_then(|v| v.as_str()),
        Some("all")
    );
    assert_eq!(suites, vec!["phase1", "phase2", "contract", "all"]);
}

#[test]
fn regression_contract_json_outputs_summary() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(&repo_root)
        .arg("regression")
        .arg("--suite")
        .arg("contract")
        .arg("--json")
        .arg("--min-pass-rate")
        .arg("0.99");

    let output = cmd.output().expect("run regression contract");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse regression json");
    assert_eq!(payload.get("result").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(
        payload.get("suite").and_then(|v| v.as_str()),
        Some("contract")
    );
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_str()),
        Some("v1")
    );
    assert!(payload.get("rate_basis").and_then(|v| v.as_u64()).is_some());
    assert!(payload
        .get("scenario_results")
        .and_then(|v| v.as_array())
        .is_some());
    assert_eq!(
        payload.get("success_rate").and_then(|v| v.as_f64()),
        Some(1.0)
    );
}

#[test]
fn selfcheck_json_quick_fix_mode_returns_ok() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "completed",
            "current_phase": "DELIVER"
        }))
        .expect("json"),
    )
    .expect("sessions");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [COMPLETED]\n").expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("selfcheck")
        .arg("--json")
        .arg("--quick")
        .arg("--fix")
        .arg(temp.path());

    let output = cmd.output().expect("run selfcheck");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse selfcheck json");
    let checks = payload["checks"].as_array().expect("checks array");
    let names: Vec<&str> = checks
        .iter()
        .map(|item| {
            item.get("name")
                .and_then(|v| v.as_str())
                .expect("check name")
        })
        .collect();
    assert_eq!(payload.get("result").and_then(|v| v.as_str()), Some("ok"));
    assert!(payload
        .get("project_root")
        .and_then(|v| v.as_str())
        .is_some());
    assert_eq!(
        names,
        vec![
            "hook_doctor",
            "stop_simulation",
            "contract_regression_suite"
        ]
    );
    assert_eq!(
        checks[2].get("skipped").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn loop_guardian_init_and_status_use_loaded_config_thresholds() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::write(
        fusion.join("config.yaml"),
        "loop_guardian:\n  max_iterations: 7\n  max_no_progress: 2\n  max_same_action: 4\n  max_same_error: 5\n  max_state_visits: 9\n  max_wall_time_ms: 12000\n  backoff_threshold: 1\n",
    )
    .expect("config");

    let mut init_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    init_cmd
        .current_dir(temp.path())
        .arg("loop-guardian")
        .arg("init")
        .arg("--fusion-dir")
        .arg(".fusion");
    init_cmd.assert().success();

    let mut status_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    status_cmd
        .current_dir(temp.path())
        .arg("loop-guardian")
        .arg("status")
        .arg("--fusion-dir")
        .arg(".fusion");
    let output = status_cmd.output().expect("run loop guardian status");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in [
        "Iterations: 0/7",
        "No-Progress Rounds: 0/2",
        "Same Action Count: 0/4",
        "Same Error Count: 0/5",
        "State Visits: 0/9",
        "Wall Time: 0s/12s",
    ] {
        assert!(line_contains_normalized(&stdout, line));
    }
}

#[test]
fn loop_guardian_record_get_and_evaluate_update_context() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "loop_guardian:\n  backoff_threshold: 1\n",
    )
    .expect("config");

    let mut init_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    init_cmd
        .current_dir(temp.path())
        .arg("loop-guardian")
        .arg("init")
        .arg("--fusion-dir")
        .arg(".fusion");
    init_cmd.assert().success();

    let mut record_one = Command::cargo_bin("fusion-bridge").expect("binary");
    record_one
        .current_dir(temp.path())
        .arg("loop-guardian")
        .arg("record")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("EXECUTE")
        .arg("task-a")
        .arg("");
    record_one.assert().success();

    let mut record_two = Command::cargo_bin("fusion-bridge").expect("binary");
    record_two
        .current_dir(temp.path())
        .arg("loop-guardian")
        .arg("record")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("EXECUTE")
        .arg("task-a")
        .arg("");
    record_two.assert().success();

    let mut get_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    get_cmd
        .current_dir(temp.path())
        .arg("loop-guardian")
        .arg("get")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg(".metrics.total_iterations");
    assert_stdout_trimmed_eq(&mut get_cmd, "run loop-guardian get", "2");

    let mut eval_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    eval_cmd
        .current_dir(temp.path())
        .arg("loop-guardian")
        .arg("evaluate")
        .arg("--fusion-dir")
        .arg(".fusion");
    assert_stdout_trimmed_eq(&mut eval_cmd, "run loop-guardian evaluate", "BACKOFF");

    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("loop_context.json")).expect("ctx"))
            .expect("parse");
    assert_eq!(payload["total_iterations"], 2);
    assert_eq!(payload["no_progress_rounds"], 1);
    assert_eq!(payload["same_action_count"], 2);
    assert_eq!(payload["same_error_count"], 0);
    assert_eq!(payload["max_state_visit_count"], 2);
}

#[test]
fn git_branch_and_status_report_repo_state() {
    let temp = tempdir().expect("tempdir");
    init_git_repo(temp.path());

    let mut branch_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    branch_cmd.current_dir(temp.path()).arg("git").arg("branch");
    branch_cmd.assert().success();

    fs::write(temp.path().join("demo.txt"), "hello\n").expect("write file");

    let mut status_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    status_cmd.current_dir(temp.path()).arg("git").arg("status");
    let status_output = status_cmd.output().expect("run git status");
    assert!(
        status_output.status.success(),
        "{}",
        String::from_utf8_lossy(&status_output.stderr)
    );
    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    assert!(line_contains_normalized(
        &status_stdout,
        "=== Fusion Git Status ==="
    ));
    assert!(line_contains_normalized(&status_stdout, "Current branch:"));
    assert!(line_contains_normalized(
        &status_stdout,
        "=== Git Status ==="
    ));
    assert!(line_contains_normalized(&status_stdout, "demo.txt"));
}

#[test]
fn git_create_branch_outputs_fusion_branch_name() {
    let temp = tempdir().expect("tempdir");
    init_git_repo(temp.path());

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("git")
        .arg("create-branch")
        .arg("demo-goal");
    let output = cmd.output().expect("run git create-branch");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "fusion/demo-goal"));
}

#[test]
fn git_changes_and_diff_report_worktree_state() {
    let temp = tempdir().expect("tempdir");
    init_git_repo(temp.path());

    let tracked = temp.path().join("tracked.txt");
    fs::write(&tracked, "before\n").expect("write tracked");
    Command::new("git")
        .args(["add", "tracked.txt"])
        .current_dir(temp.path())
        .output()
        .expect("git add tracked");
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(temp.path())
        .output()
        .expect("git commit initial");

    fs::write(&tracked, "after\n").expect("rewrite tracked");

    let mut changes_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    changes_cmd
        .current_dir(temp.path())
        .arg("git")
        .arg("changes");
    let changes_output = changes_cmd.output().expect("run git changes");
    assert!(
        changes_output.status.success(),
        "{}",
        String::from_utf8_lossy(&changes_output.stderr)
    );
    let changes_stdout = String::from_utf8_lossy(&changes_output.stdout);
    assert!(line_contains_normalized(
        &changes_stdout,
        "=== Git Status ==="
    ));
    assert!(line_contains_normalized(&changes_stdout, "tracked.txt"));
    assert!(line_contains_normalized(
        &changes_stdout,
        "=== Changed Files ==="
    ));

    let mut diff_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    diff_cmd.current_dir(temp.path()).arg("git").arg("diff");
    let diff_output = diff_cmd.output().expect("run git diff");
    assert!(
        diff_output.status.success(),
        "{}",
        String::from_utf8_lossy(&diff_output.stderr)
    );
    let diff_stdout = String::from_utf8_lossy(&diff_output.stdout);
    assert!(line_contains_normalized(&diff_stdout, "--- a/tracked.txt"));
    assert!(line_contains_normalized(&diff_stdout, "+++ b/tracked.txt"));
    assert!(line_contains_normalized(&diff_stdout, "+after"));
}

#[test]
fn git_commit_and_cleanup_follow_shell_contract() {
    let temp = tempdir().expect("tempdir");
    init_git_repo(temp.path());

    let base_branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(temp.path())
        .output()
        .expect("git show current branch");
    let base_branch = String::from_utf8(base_branch_output.stdout)
        .expect("utf8 branch")
        .trim()
        .to_string();

    let mut create_branch_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    create_branch_cmd
        .current_dir(temp.path())
        .arg("git")
        .arg("create-branch")
        .arg("cleanup-demo");
    let create_branch_output = create_branch_cmd.output().expect("run git create-branch");
    assert!(
        create_branch_output.status.success(),
        "{}",
        String::from_utf8_lossy(&create_branch_output.stderr)
    );
    let create_branch_stdout = String::from_utf8_lossy(&create_branch_output.stdout);
    assert!(line_contains_normalized(
        &create_branch_stdout,
        "fusion/cleanup-demo"
    ));

    fs::write(temp.path().join("commit.txt"), "content\n").expect("write commit file");

    let mut commit_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    commit_cmd
        .current_dir(temp.path())
        .arg("git")
        .arg("commit")
        .arg("add commit file");
    commit_cmd
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"[0-9a-f]{7,}\n").expect("short hash regex"));

    let mut cleanup_cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cleanup_cmd
        .current_dir(temp.path())
        .arg("git")
        .arg("cleanup")
        .arg(&base_branch);
    cleanup_cmd.assert().success();

    let branch_after_cleanup = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(temp.path())
        .output()
        .expect("git branch after cleanup");
    let branch_after_cleanup = String::from_utf8(branch_after_cleanup.stdout)
        .expect("utf8 cleanup branch")
        .trim()
        .to_string();
    assert_eq!(branch_after_cleanup, base_branch);
}

#[test]
fn codeagent_missing_wrapper_writes_dependency_report() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "test goal",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", "/usr/bin:/bin")
        .env_remove("CODEAGENT_WRAPPER_BIN");

    assert_stderr_contains_normalized(
        &mut cmd,
        "run codeagent without wrapper",
        Some(127),
        "Missing dependency: codeagent-wrapper",
    );

    let report = fusion.join("dependency_report.json");
    assert!(report.exists());

    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report).expect("report")).expect("parse report");
    assert_eq!(
        value.get("status").and_then(|v| v.as_str()),
        Some("blocked")
    );
}

#[test]
fn logs_prints_hook_debug_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::write(
        fusion.join("progress.md"),
        "| t | EXECUTE | demo | OK | detail |\n",
    )
    .expect("progress");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("logs")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run logs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "HOOK DEBUG"));
    assert!(line_contains_normalized(&stdout, "enabled: false"));
}

#[test]
fn logs_prints_session_info_and_hook_log_tail() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "demo goal",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "started_at": "2026-02-11T00:00:00Z",
            "last_checkpoint": "2026-02-11 00:10:00",
            "_runtime": {
                "understand": {
                    "mode": "minimal",
                    "forced": false,
                    "decision": "auto_continue"
                }
            }
        }))
        .expect("json"),
    )
    .expect("sessions");
    fs::write(
        fusion.join("progress.md"),
        "| t | EXECUTE | demo | OK | detail |\n",
    )
    .expect("progress");
    fs::write(fusion.join(".hook_debug"), "").expect("flag");
    fs::write(
        fusion.join("hook-debug.log"),
        "[fusion][hook-debug][pretool][2026-02-12T00:00:00Z] invoked\n",
    )
    .expect("hook log");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("logs")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run logs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "Goal: demo goal"));
    assert!(line_contains_normalized(&stdout, "Status: in_progress"));
    assert!(line_contains_normalized(
        &stdout,
        "UNDERSTAND: minimal (decision=auto_continue, forced=false)"
    ));
    assert!(line_contains_normalized(&stdout, "hook-debug.log"));
    assert!(line_contains_normalized(
        &stdout,
        "[fusion][hook-debug][pretool]"
    ));
}

#[test]
fn status_prints_understand_handoff_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "demo goal",
            "status": "in_progress",
            "current_phase": "INITIALIZE",
            "_runtime": {
                "state": "INITIALIZE",
                "understand": {
                    "mode": "minimal",
                    "forced": false,
                    "decision": "auto_continue"
                }
            }
        }))
        .expect("json"),
    )
    .expect("sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "## Status\n- Current Phase: INITIALIZE\n",
    )
    .expect("task plan");
    fs::write(
        fusion.join("progress.md"),
        "| t | INITIALIZE | e | OK | d |\n",
    )
    .expect("progress");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("status")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run status");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "## Runtime"));
    assert!(line_contains_normalized(
        &stdout,
        "understand: minimal (decision=auto_continue, forced=false)"
    ));
}

#[test]
fn codeagent_fallback_updates_claude_session() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "test goal",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("config.yaml"),
        "backends:\n  primary: codex\n  fallback: claude\n",
    )
    .expect("write config");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\nif [ \"$2\" = \"codex\" ]; then exit 1; fi\necho \"mock backend:$2\"\necho \"SESSION_ID: 654321\"\n",
        "@echo off\r\nif \"%2\"==\"codex\" exit /b 1\r\necho mock backend:%2\r\necho SESSION_ID: 654321\r\n",
    );
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path);

    let output = cmd.output().expect("run codeagent");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "mock backend:claude"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("claude_session").and_then(|v| v.as_str()),
        Some("654321")
    );
}

#[test]
fn codeagent_role_override_updates_role_session() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "test goal",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("config.yaml"),
        "backends:\n  primary: claude\n  fallback: codex\n",
    )
    .expect("write config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [PENDING]\n- Owner: reviewer\n",
    )
    .expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\necho \"mock backend:$2\"\necho \"SESSION_ID: 123456\"\n",
        "@echo off\r\necho mock backend:%2\r\necho SESSION_ID: 123456\r\n",
    );
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .env("FUSION_AGENT_ROLE", "reviewer");

    let output = cmd.output().expect("run codeagent");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "mock backend:codex"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions
            .get("reviewer_codex_session")
            .and_then(|v| v.as_str()),
        Some("123456")
    );
    assert_eq!(
        sessions.get("codex_session").and_then(|v| v.as_str()),
        Some("123456")
    );
}

#[test]
fn codeagent_research_task_injects_planner_owner_and_uses_role_session() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "test goal",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("config.yaml"),
        r#"backends:
  primary: codex
  fallback: claude
"#,
    )
    .expect("write config");
    fs::write(
        fusion.join("task_plan.md"),
        r#"### Task 1: Explore [PENDING]
- Type: research
- Dependencies: []
"#,
    )
    .expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\necho \"mock backend:$2\"\necho \"SESSION_ID: 333333\"\n",
        "@echo off\r\necho mock backend:%2\r\necho SESSION_ID: 333333\r\n",
    );
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path);

    let output = cmd.output().expect("run codeagent");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "mock backend:codex"));

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("task plan content");
    assert!(line_contains_normalized(&task_plan, "- Owner: planner"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions
            .get("planner_codex_session")
            .and_then(|v| v.as_str()),
        Some("333333")
    );
    assert_eq!(
        sessions.get("codex_session").and_then(|v| v.as_str()),
        Some("333333")
    );
}

#[test]
fn codeagent_writes_agent_runtime_summary_and_lifecycle_events() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "agent spine",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("config.yaml"),
        r#"backends:
  primary: codex
  fallback: claude
agents:
  enabled: true
  mode: single_orchestrator
  review_policy: high_risk
  explain_level: compact
"#,
    )
    .expect("write config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Explore [PENDING]\n- Type: research\n",
    )
    .expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\necho \"mock backend:$2\"\necho \"SESSION_ID: 111111\"\n",
        "@echo off\r\necho mock backend:%2\r\necho SESSION_ID: 111111\r\n",
    );
    let path = prepend_path(&bin_dir);

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .output()
        .expect("run codeagent");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("task plan content");
    assert!(line_contains_normalized(&task_plan, "- Owner: planner"));
    assert!(line_contains_normalized(&task_plan, "- Risk: low"));
    assert!(line_contains_normalized(&task_plan, "- Review: auto"));
    assert!(line_contains_normalized(&task_plan, "- Writes: []"));
    assert!(line_contains_normalized(&task_plan, "- Dependencies: []"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("enabled"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("mode"))
            .and_then(|v| v.as_str()),
        Some("single_orchestrator")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("current_batch_id"))
            .and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("last_decision_reason"))
            .and_then(|v| v.as_str()),
        Some("role:planner")
    );

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events read");
    assert!(line_contains_normalized(&events, "AGENT_TASK_ASSIGNED"));
    assert!(line_contains_normalized(&events, "AGENT_TASK_STARTED"));
    assert!(!line_contains_normalized(&events, "AGENT_TASK_COMPLETED"));
    assert!(line_contains_normalized(&events, "\"task_id\":\"task_1\""));
    assert!(line_contains_normalized(&events, "\"owner\":\"planner\""));
    assert!(line_contains_normalized(
        &events,
        "\"review_status\":\"none\""
    ));
    assert!(line_contains_normalized(
        &events,
        "\"decision_reason\":\"role:planner\""
    ));
}

#[test]
fn codeagent_fallback_records_agent_fallback_event_when_enabled() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "agent fallback",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("config.yaml"),
        r#"backends:
  primary: codex
  fallback: claude
agents:
  enabled: true
  mode: single_orchestrator
  review_policy: high_risk
  explain_level: compact
"#,
    )
    .expect("write config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Build [PENDING]\n- Type: implementation\n- Owner: coder\n",
    )
    .expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\nif [ \"$2\" = \"codex\" ]; then exit 1; fi\necho \"mock backend:$2\"\necho \"SESSION_ID: 222222\"\n",
        "@echo off\r\nif \"%2\"==\"codex\" exit /b 1\r\necho mock backend:%2\r\necho SESSION_ID: 222222\r\n",
    );
    let path = prepend_path(&bin_dir);

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .output()
        .expect("run codeagent");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events read");
    assert!(line_contains_normalized(&events, "AGENT_FALLBACK_USED"));
    assert!(line_contains_normalized(
        &events,
        "\"primary_backend\":\"codex\""
    ));
    assert!(line_contains_normalized(
        &events,
        "\"used_backend\":\"claude\""
    ));
}

#[test]
fn codeagent_plans_parallel_agent_batch_with_dependencies_and_review_queue() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "orchestrator core",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("config.yaml"),
        r#"backends:
  primary: codex
  fallback: claude
agents:
  enabled: true
  mode: single_orchestrator
  review_policy: high_risk
  explain_level: compact
execution:
  parallel: 3
parallel:
  enabled: true
  conflict_check: true
scheduler:
  enabled: true
  max_parallel: 2
"#,
    )
    .expect("write config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Research API [PENDING]\n- Type: research\n- Owner: planner\n- Risk: low\n- Review: auto\n- Writes: [docs/research.md]\n- Dependencies: []\n### Task 2: Build API [PENDING]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Writes: [src/api.rs]\n- Dependencies: []\n### Task 3: Polish API Docs [PENDING]\n- Type: documentation\n- Owner: coder\n- Risk: low\n- Review: auto\n- Writes: [src/api.rs]\n- Dependencies: []\n### Task 4: Verify API [PENDING]\n- Type: verification\n- Owner: reviewer\n- Risk: low\n- Review: auto\n- Writes: [tests/api.rs]\n- Dependencies: [task_2]\n",
    )
    .expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\necho \"mock backend:$2\"\necho \"SESSION_ID: 444444\"\n",
        "@echo off\r\necho mock backend:%2\r\necho SESSION_ID: 444444\r\n",
    );
    let path = prepend_path(&bin_dir);

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .output()
        .expect("run codeagent");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    let current_batch_tasks: Vec<&str> = sessions
        .get("_runtime")
        .and_then(|v| v.get("agents"))
        .and_then(|v| v.get("current_batch_tasks"))
        .and_then(|v| v.as_array())
        .expect("current batch tasks")
        .iter()
        .map(|item| item.as_str().expect("task id"))
        .collect();
    assert_eq!(current_batch_tasks, vec!["task_1", "task_2"]);
    let blocked_tasks: Vec<&str> = sessions
        .get("_runtime")
        .and_then(|v| v.get("agents"))
        .and_then(|v| v.get("blocked_tasks"))
        .and_then(|v| v.as_array())
        .expect("blocked tasks")
        .iter()
        .map(|item| item.as_str().expect("task id"))
        .collect();
    assert_eq!(blocked_tasks, vec!["task_3", "task_4"]);
    let active_roles: Vec<&str> = sessions
        .get("_runtime")
        .and_then(|v| v.get("agents"))
        .and_then(|v| v.get("active_roles"))
        .and_then(|v| v.as_array())
        .expect("active roles")
        .iter()
        .map(|item| item.as_str().expect("role"))
        .collect();
    assert_eq!(active_roles, vec!["planner", "coder"]);
    let review_queue: Vec<&str> = sessions
        .get("_runtime")
        .and_then(|v| v.get("agents"))
        .and_then(|v| v.get("review_queue"))
        .and_then(|v| v.as_array())
        .expect("review queue")
        .iter()
        .map(|item| item.as_str().expect("task id"))
        .collect();
    assert_eq!(review_queue, vec!["task_2"]);
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("review_queue_size"))
            .and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("scheduler"))
            .and_then(|v| v.get("enabled"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("scheduler"))
            .and_then(|v| v.get("current_batch_id"))
            .and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("scheduler"))
            .and_then(|v| v.get("parallel_tasks"))
            .and_then(|v| v.as_i64()),
        Some(2)
    );

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events read");
    assert!(line_contains_normalized(&events, "AGENT_BATCH_PLANNED"));
    assert!(line_contains_normalized(
        &events,
        "\"selected_tasks\":[\"task_1\",\"task_2\"]"
    ));
    assert!(line_contains_normalized(
        &events,
        "\"blocked_tasks\":[\"task_3\",\"task_4\"]"
    ));
    assert!(line_contains_normalized(
        &events,
        "\"review_queue\":[\"task_2\"]"
    ));
}

#[test]
fn codeagent_verbose_explain_records_detailed_policy_reasons() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "policy explain",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("config.yaml"),
        r#"backends:
  primary: codex
  fallback: claude
agents:
  enabled: true
  mode: single_orchestrator
  review_policy: high_risk
  explain_level: verbose
execution:
  parallel: 3
parallel:
  enabled: true
  conflict_check: true
scheduler:
  enabled: true
  max_parallel: 2
"#,
    )
    .expect("write config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Research API [PENDING]\n- Type: research\n- Owner: planner\n- Risk: low\n- Review: auto\n- Writes: [docs/research.md]\n- Dependencies: []\n### Task 2: Build API [PENDING]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Writes: [src/api.rs]\n- Dependencies: []\n### Task 3: Polish API Docs [PENDING]\n- Type: documentation\n- Owner: coder\n- Risk: low\n- Review: auto\n- Writes: [src/api.rs]\n- Dependencies: []\n### Task 4: Verify API [PENDING]\n- Type: verification\n- Owner: reviewer\n- Risk: low\n- Review: auto\n- Writes: [tests/api.rs]\n- Dependencies: [task_2]\n",
    )
    .expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\necho \"mock backend:$2\"\necho \"SESSION_ID: 444444\"\n",
        "@echo off\r\necho mock backend:%2\r\necho SESSION_ID: 444444\r\n",
    );
    let path = prepend_path(&bin_dir);

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .output()
        .expect("run codeagent");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    let policy = sessions
        .get("_runtime")
        .and_then(|v| v.get("agents"))
        .and_then(|v| v.get("policy"))
        .expect("agent policy");
    assert_eq!(
        policy.get("batch_reason").and_then(|v| v.as_str()),
        Some("ready_non_conflicting_parallel:max=2")
    );
    assert_eq!(
        policy
            .get("selected_reasons")
            .and_then(|v| v.get("task_1"))
            .and_then(|v| v.as_str()),
        Some("ready:no_dependencies")
    );
    assert_eq!(
        policy
            .get("selected_reasons")
            .and_then(|v| v.get("task_2"))
            .and_then(|v| v.as_str()),
        Some("ready:no_dependencies")
    );
    assert_eq!(
        policy
            .get("blocked_reasons")
            .and_then(|v| v.get("task_3"))
            .and_then(|v| v.as_str()),
        Some("write_conflict:src/api.rs")
    );
    assert_eq!(
        policy
            .get("blocked_reasons")
            .and_then(|v| v.get("task_4"))
            .and_then(|v| v.as_str()),
        Some("waiting_for_dependencies:task_2")
    );
    assert_eq!(
        policy
            .get("review_reasons")
            .and_then(|v| v.get("task_2"))
            .and_then(|v| v.as_str()),
        Some("review_required:risk=high+flag=required")
    );

    let batch_event: serde_json::Value = fs::read_to_string(fusion.join("events.jsonl"))
        .expect("events read")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("parse event"))
        .find(|event| event.get("type").and_then(|v| v.as_str()) == Some("AGENT_BATCH_PLANNED"))
        .expect("batch planned event");
    let event_policy = batch_event
        .get("payload")
        .and_then(|v| v.get("policy"))
        .expect("event policy");
    assert_eq!(
        event_policy.get("batch_reason").and_then(|v| v.as_str()),
        Some("ready_non_conflicting_parallel:max=2")
    );
    assert_eq!(
        event_policy
            .get("blocked_reasons")
            .and_then(|v| v.get("task_4"))
            .and_then(|v| v.as_str()),
        Some("waiting_for_dependencies:task_2")
    );
}

#[test]
fn codeagent_role_handoff_sequences_planner_coder_and_reviewer_gate() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "role handoff",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("config.yaml"),
        r#"backends:
  primary: codex
  fallback: claude
agents:
  enabled: true
  mode: role_handoff
  review_policy: high_risk
  explain_level: compact
execution:
  parallel: 2
parallel:
  enabled: true
  conflict_check: true
scheduler:
  enabled: true
  max_parallel: 2
"#,
    )
    .expect("write config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [PENDING]\n- Type: research\n- Owner: planner\n- Risk: low\n- Review: auto\n- Writes: [docs/plan.md]\n- Dependencies: []\n### Task 2: Build API [PENDING]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Writes: [src/api.rs]\n- Dependencies: []\n",
    )
    .expect("task plan");

    let unix_body = format!(
        "#!/bin/bash\nset -euo pipefail\nplan=\"{}\"\nbackend=\"$2\"\nif [ \"$backend\" = \"codex\" ] && grep -q 'Task 1: Plan API \\[PENDING\\]' \"$plan\"; then\n  sed -i.bak 's/Task 1: Plan API \\[PENDING\\]/Task 1: Plan API [COMPLETED]/' \"$plan\"\n  rm -f \"$plan.bak\"\nelif [ \"$backend\" = \"claude\" ] && grep -q 'Task 2: Build API \\[PENDING\\]' \"$plan\"; then\n  sed -i.bak 's/Task 2: Build API \\[PENDING\\]/Task 2: Build API [IN_PROGRESS]/' \"$plan\"\n  sed -i.bak 's/- Review-Status: none/- Review-Status: pending/' \"$plan\"\n  rm -f \"$plan.bak\"\nelif [ \"$backend\" = \"codex\" ] && grep -q 'Task 2: Build API \\[IN_PROGRESS\\]' \"$plan\" && grep -q -- '- Review-Status: pending' \"$plan\"; then\n  sed -i.bak 's/Task 2: Build API \\[IN_PROGRESS\\]/Task 2: Build API [COMPLETED]/' \"$plan\"\n  sed -i.bak 's/- Review-Status: pending/- Review-Status: approved/' \"$plan\"\n  rm -f \"$plan.bak\"\nfi\necho \"mock backend:$backend\"\necho \"SESSION_ID: 777777\"\n",
        fusion.join("task_plan.md").display()
    );
    let windows_body = format!(
        "@echo off\r\nset \"PLAN={}\"\r\nset \"BACK=%2\"\r\npowershell -NoProfile -Command \"$p=$env:PLAN; $b=$env:BACK; $c=Get-Content -Raw $p; if($b -eq 'codex' -and $c -match 'Task 1: Plan API \\[PENDING\\]'){{ $c=$c -replace 'Task 1: Plan API \\[PENDING\\]','Task 1: Plan API [COMPLETED]' }} elseif($b -eq 'claude' -and $c -match 'Task 2: Build API \\[PENDING\\]'){{ $c=$c -replace 'Task 2: Build API \\[PENDING\\]','Task 2: Build API [IN_PROGRESS]'; $c=$c -replace '- Review-Status: none','- Review-Status: pending' }} elseif($b -eq 'codex' -and $c -match 'Task 2: Build API \\[IN_PROGRESS\\]' -and $c -match '- Review-Status: pending'){{ $c=$c -replace 'Task 2: Build API \\[IN_PROGRESS\\]','Task 2: Build API [COMPLETED]'; $c=$c -replace '- Review-Status: pending','- Review-Status: approved' }}; Set-Content -Path $p -Value $c\"\r\necho mock backend:%2\r\necho SESSION_ID: 777777\r\n",
        fusion.join("task_plan.md").display()
    );
    let _wrapper = write_mock_executable(&bin_dir, "codeagent-wrapper", &unix_body, &windows_body);
    let path = prepend_path(&bin_dir);

    let first = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .output()
        .expect("first codeagent");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(line_contains_normalized(
        &String::from_utf8_lossy(&first.stdout),
        "mock backend:codex"
    ));

    let plan_after_first = fs::read_to_string(fusion.join("task_plan.md")).expect("task plan");
    assert!(line_contains_normalized(
        &plan_after_first,
        "- Review-Status: none"
    ));
    let first_sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse first sessions");
    assert_eq!(
        first_sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("mode"))
            .and_then(|v| v.as_str()),
        Some("role_handoff")
    );
    assert_eq!(
        first_sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("collaboration"))
            .and_then(|v| v.get("turn_role"))
            .and_then(|v| v.as_str()),
        Some("planner")
    );
    assert_eq!(
        first_sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("collaboration"))
            .and_then(|v| v.get("turn_task_id"))
            .and_then(|v| v.as_str()),
        Some("task_1")
    );

    let second = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .output()
        .expect("second codeagent");
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert!(line_contains_normalized(
        &String::from_utf8_lossy(&second.stdout),
        "mock backend:claude"
    ));

    let plan_after_second = fs::read_to_string(fusion.join("task_plan.md")).expect("task plan");
    assert!(line_contains_normalized(
        &plan_after_second,
        "Task 2: Build API [IN_PROGRESS]"
    ));
    assert!(line_contains_normalized(
        &plan_after_second,
        "- Review-Status: pending"
    ));
    let second_sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse second sessions");
    assert_eq!(
        second_sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("collaboration"))
            .and_then(|v| v.get("turn_role"))
            .and_then(|v| v.as_str()),
        Some("coder")
    );
    assert_eq!(
        second_sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("collaboration"))
            .and_then(|v| v.get("turn_task_id"))
            .and_then(|v| v.as_str()),
        Some("task_2")
    );

    let third = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .output()
        .expect("third codeagent");
    assert!(
        third.status.success(),
        "{}",
        String::from_utf8_lossy(&third.stderr)
    );
    assert!(line_contains_normalized(
        &String::from_utf8_lossy(&third.stdout),
        "mock backend:codex"
    ));

    let plan_after_third = fs::read_to_string(fusion.join("task_plan.md")).expect("task plan");
    assert!(line_contains_normalized(
        &plan_after_third,
        "Task 2: Build API [COMPLETED]"
    ));
    assert!(line_contains_normalized(
        &plan_after_third,
        "- Review-Status: approved"
    ));
    let third_sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse third sessions");
    assert_eq!(
        third_sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("collaboration"))
            .and_then(|v| v.get("turn_role"))
            .and_then(|v| v.as_str()),
        Some("reviewer")
    );
    assert_eq!(
        third_sessions
            .get("_runtime")
            .and_then(|v| v.get("agents"))
            .and_then(|v| v.get("collaboration"))
            .and_then(|v| v.get("turn_kind"))
            .and_then(|v| v.as_str()),
        Some("review_gate")
    );

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events");
    assert!(line_contains_normalized(&events, "AGENT_HANDOFF_PLANNED"));
    assert!(line_contains_normalized(&events, "AGENT_ROLE_TURN_STARTED"));
    assert!(line_contains_normalized(
        &events,
        "AGENT_ROLE_TURN_COMPLETED"
    ));
    assert!(line_contains_normalized(&events, "AGENT_REVIEW_REQUESTED"));
    assert!(line_contains_normalized(&events, "AGENT_REVIEW_APPROVED"));
}

#[test]
fn codeagent_timeout_falls_back_to_claude() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "test goal",
            "status": "in_progress",
            "current_phase": "REVIEW",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("config.yaml"),
        "backends:\n  primary: codex\n  fallback: claude\n",
    )
    .expect("write config");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\nbackend=\"$2\"\nif [ \"$backend\" = \"codex\" ]; then sleep 2; fi\necho \"mock backend:$backend\"\necho \"SESSION_ID: 222222\"\n",
        "@echo off\r\nset backend=%2\r\nif \"%backend%\"==\"codex\" powershell -NoProfile -Command \"Start-Sleep -Seconds 2\"\r\necho mock backend:%backend%\r\necho SESSION_ID: 222222\r\n",
    );
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("codeagent")
        .arg("REVIEW")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .env("FUSION_CODEAGENT_TIMEOUT_SEC", "1");

    let output = cmd.output().expect("run codeagent");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "mock backend:claude"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("claude_session").and_then(|v| v.as_str()),
        Some("222222")
    );
}

#[test]
fn codeagent_double_backend_failure_writes_backend_failure_report() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "test goal",
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "codex_session": null,
            "claude_session": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("config.yaml"),
        "backends:\n  primary: claude\n  fallback: codex\n",
    )
    .expect("write config");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\nif [ \"$2\" = \"claude\" ]; then echo \"claude-fail\" >&2; exit 11; fi\necho \"codex-fail\" >&2\nexit 12\n",
        "@echo off\r\nif \"%2\"==\"claude\" >&2 echo claude-fail && exit /b 11\r\n>&2 echo codex-fail\r\nexit /b 12\r\n",
    );
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", &path)
        .assert()
        .failure();

    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fusion.join("backend_failure_report.json")).expect("report"),
    )
    .expect("parse report");
    assert_eq!(
        report.get("status").and_then(|v| v.as_str()),
        Some("blocked")
    );
    assert_eq!(
        report.get("primary_backend").and_then(|v| v.as_str()),
        Some("claude")
    );
    assert_eq!(
        report.get("fallback_backend").and_then(|v| v.as_str()),
        Some("codex")
    );
}

#[test]
fn codeagent_missing_wrapper_clears_stale_backend_failure_report() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "goal": "test goal",
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [PENDING]
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("backend_failure_report.json"),
        serde_json::to_string_pretty(&json!({
            "status": "blocked",
            "source": "fusion-codeagent.sh",
            "primary_backend": "claude",
            "fallback_backend": "codex"
        }))
        .expect("report"),
    )
    .expect("write stale report");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", "/usr/bin:/bin")
        .env_remove("CODEAGENT_WRAPPER_BIN")
        .assert()
        .code(127);

    assert!(!fusion.join("backend_failure_report.json").exists());
    assert!(fusion.join("dependency_report.json").exists());
}

#[test]
fn status_prints_dependency_report() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("task_plan.md"),
        "## Status\n- Current Phase: EXECUTE\n",
    )
    .expect("task plan");
    fs::write(fusion.join("progress.md"), "| t | EXECUTE | e | OK | d |\n").expect("progress");
    fs::write(
        fusion.join("dependency_report.json"),
        serde_json::to_string_pretty(&json!({
            "status": "blocked",
            "source": "fusion-codeagent.sh",
            "reason": "Missing executable for backend orchestration",
            "missing": ["codeagent-wrapper"],
            "next_actions": ["Install or expose codeagent-wrapper in PATH."]
        }))
        .expect("json"),
    )
    .expect("dependency report");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("status")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run status");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "## Dependency Report"));
    assert!(line_contains_normalized(
        &stdout,
        "missing: codeagent-wrapper"
    ));
}

#[test]
fn status_prints_agent_policy_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {
                "agents": {
                    "enabled": true,
                    "mode": "single_orchestrator",
                    "explain_level": "verbose",
                    "current_batch_id": 4,
                    "active_roles": ["planner", "coder"],
                    "current_batch_tasks": ["task_1", "task_2"],
                    "review_queue": ["task_2"],
                    "review_queue_size": 1,
                    "policy": {
                        "batch_reason": "ready_non_conflicting_parallel:max=2",
                        "selected_reasons": {
                            "task_1": "ready:no_dependencies",
                            "task_2": "ready:no_dependencies"
                        },
                        "blocked_reasons": {
                            "task_3": "write_conflict:src/api.rs",
                            "task_4": "waiting_for_dependencies:task_2"
                        },
                        "review_reasons": {
                            "task_2": "review_required:risk=high+flag=required"
                        }
                    }
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("config.yaml"),
        "agents:\n  enabled: true\n  mode: single_orchestrator\n  review_policy: high_risk\n  explain_level: verbose\n",
    )
    .expect("config");
    fs::write(
        fusion.join("task_plan.md"),
        "## Status\n- Current Phase: EXECUTE\n",
    )
    .expect("task plan");
    fs::write(fusion.join("progress.md"), "| t | EXECUTE | e | OK | d |\n").expect("progress");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("status")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run status");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "agents.explain_level: verbose"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "agents.policy.batch_reason: ready_non_conflicting_parallel:max=2"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "agents.policy.selected: task_1=ready:no_dependencies; task_2=ready:no_dependencies"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "agents.policy.blocked: task_3=write_conflict:src/api.rs; task_4=waiting_for_dependencies:task_2"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "agents.policy.review: task_2=review_required:risk=high+flag=required"
    ));
}

#[test]
fn status_prints_backend_failure_report() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {
                "last_event_id": "evt_000123",
                "last_event_counter": 7,
                "scheduler": {
                    "enabled": true,
                    "current_batch_id": 3,
                    "parallel_tasks": 2
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "## Status
- Current Phase: EXECUTE
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("progress.md"),
        "| t | EXECUTE | e | OK | d |
",
    )
    .expect("progress");
    fs::write(
        fusion.join("backend_failure_report.json"),
        serde_json::to_string_pretty(&json!({
            "status": "blocked",
            "source": "fusion-codeagent.sh",
            "primary_backend": "claude",
            "fallback_backend": "codex",
            "primary_error": "claude-fail",
            "fallback_error": "codex-fail",
            "next_actions": ["retry with fallback"]
        }))
        .expect("json"),
    )
    .expect("backend report");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("status")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run status");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "## Backend Failure Report"
    ));
    assert!(line_contains_normalized(&stdout, "primary_backend: claude"));
    assert!(line_contains_normalized(&stdout, "fallback_backend: codex"));
}

#[test]
fn status_json_normalizes_stale_legacy_engine_config() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [PENDING]
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:
  enabled: true
  compat_mode: true
  engine: legacy
",
    )
    .expect("config");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("status")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--json")
        .output()
        .expect("run status");
    assert!(output.status.success());

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        value.get("runtime_engine").and_then(|v| v.as_str()),
        Some("rust")
    );
    assert!(value.get("runtime_engine_legacy").is_none());
    assert_eq!(
        value.get("runtime_compat_mode").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn status_json_outputs_machine_readable_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {
                "last_event_id": "evt_000123",
                "last_event_counter": 7,
                "scheduler": {
                    "enabled": true,
                    "current_batch_id": 3,
                    "parallel_tasks": 2
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
### Task 2: B [PENDING]
### Task 3: C [IN_PROGRESS]
### Task 4: D [FAILED]
",
    )
    .expect("task plan");
    fs::write(fusion.join(".hook_debug"), "").expect("hook debug flag");
    fs::write(fusion.join("hook-debug.log"), "a\nb\nc\nd\ne\nf\n").expect("hook debug log");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("status")
        .arg("--json")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run status");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(payload.get("result").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(
        payload.get("status").and_then(|v| v.as_str()),
        Some("in_progress")
    );
    assert_eq!(
        payload.get("phase").and_then(|v| v.as_str()),
        Some("EXECUTE")
    );
    assert_eq!(
        payload.get("task_completed").and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        payload.get("task_pending").and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        payload.get("task_in_progress").and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(payload.get("task_failed").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(
        payload.get("hook_debug_enabled").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        payload
            .get("runtime_last_event_id")
            .and_then(|v| v.as_str()),
        Some("evt_000123")
    );
    assert_eq!(
        payload
            .get("runtime_last_event_counter")
            .and_then(|v| v.as_i64()),
        Some(7)
    );
    assert_eq!(
        payload
            .get("runtime_scheduler_enabled")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        payload
            .get("runtime_scheduler_batch_id")
            .and_then(|v| v.as_i64()),
        Some(3)
    );
    assert_eq!(
        payload
            .get("runtime_scheduler_parallel_tasks")
            .and_then(|v| v.as_i64()),
        Some(2)
    );
    assert!(payload
        .get("hook_debug_flag")
        .and_then(|v| v.as_str())
        .map(|v| v.replace('\\', "/").ends_with(".fusion/.hook_debug"))
        .unwrap_or(false));
    assert!(payload
        .get("hook_debug_log")
        .and_then(|v| v.as_str())
        .map(|v| v.replace('\\', "/").ends_with(".fusion/hook-debug.log"))
        .unwrap_or(false));
    let hook_debug_tail: Vec<&str> = payload["hook_debug_tail"]
        .as_array()
        .expect("hook debug tail array")
        .iter()
        .map(|item| item.as_str().expect("hook debug tail line"))
        .collect();
    assert_eq!(hook_debug_tail, vec!["b", "c", "d", "e", "f"]);
}

#[test]
fn status_json_includes_guardian_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n",
    )
    .expect("task plan");
    fs::write(
        fusion.join("loop_context.json"),
        serde_json::to_string_pretty(&json!({
            "total_iterations": 6,
            "no_progress_rounds": 4,
            "same_action_count": 1,
            "same_error_count": 2,
            "max_state_visit_count": 5,
            "wall_time_ms": 12345
        }))
        .expect("json"),
    )
    .expect("loop context");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("status")
        .arg("--json")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run status");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        payload.get("guardian_status").and_then(|v| v.as_str()),
        Some("⚠ BACKOFF")
    );
    assert_eq!(
        payload
            .get("guardian_total_iterations")
            .and_then(|v| v.as_i64()),
        Some(6)
    );
    assert_eq!(
        payload
            .get("guardian_no_progress_rounds")
            .and_then(|v| v.as_i64()),
        Some(4)
    );
    assert_eq!(
        payload
            .get("guardian_same_action_count")
            .and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        payload
            .get("guardian_same_error_count")
            .and_then(|v| v.as_i64()),
        Some(2)
    );
    assert_eq!(
        payload
            .get("guardian_max_state_visit_count")
            .and_then(|v| v.as_i64()),
        Some(5)
    );
    assert_eq!(
        payload
            .get("guardian_wall_time_ms")
            .and_then(|v| v.as_i64()),
        Some(12345)
    );
}

#[test]
fn status_json_includes_session_identity_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "workflow_id": "fusion_123",
            "goal": "ship machine json contract",
            "started_at": "2026-03-24T08:00:00Z",
            "status": "paused",
            "current_phase": "REVIEW",
            "last_checkpoint": "2026-03-24T09:15:00Z",
            "codex_session": "codex_123",
            "claude_session": "claude_456",
            "planner_codex_session": "planner_codex_789",
            "coder_claude_session": "coder_claude_987",
            "reviewer_codex_session": "reviewer_codex_654",
            "_runtime": {
                "state": "REVIEW",
                "understand": {
                    "mode": "minimal",
                    "forced": false,
                    "decision": "auto_continue"
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n",
    )
    .expect("task plan");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("status")
        .arg("--json")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run status");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        payload.get("goal").and_then(|v| v.as_str()),
        Some("ship machine json contract")
    );
    assert_eq!(
        payload.get("workflow_id").and_then(|v| v.as_str()),
        Some("fusion_123")
    );
    assert_eq!(
        payload.get("started_at").and_then(|v| v.as_str()),
        Some("2026-03-24T08:00:00Z")
    );
    assert_eq!(
        payload.get("last_checkpoint").and_then(|v| v.as_str()),
        Some("2026-03-24T09:15:00Z")
    );
    assert_eq!(
        payload.get("runtime_state").and_then(|v| v.as_str()),
        Some("REVIEW")
    );
    assert_eq!(
        payload.get("understand_mode").and_then(|v| v.as_str()),
        Some("minimal")
    );
    assert_eq!(
        payload.get("understand_forced").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        payload.get("understand_decision").and_then(|v| v.as_str()),
        Some("auto_continue")
    );
    assert_eq!(
        payload.get("codex_session").and_then(|v| v.as_str()),
        Some("codex_123")
    );
    assert_eq!(
        payload.get("claude_session").and_then(|v| v.as_str()),
        Some("claude_456")
    );
    assert_eq!(
        payload
            .get("planner_codex_session")
            .and_then(|v| v.as_str()),
        Some("planner_codex_789")
    );
    assert_eq!(
        payload.get("coder_claude_session").and_then(|v| v.as_str()),
        Some("coder_claude_987")
    );
    assert_eq!(
        payload
            .get("reviewer_codex_session")
            .and_then(|v| v.as_str()),
        Some("reviewer_codex_654")
    );
}

#[test]
fn status_json_includes_agent_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {
                "agents": {
                    "enabled": true,
                    "mode": "single_orchestrator",
                    "explain_level": "verbose",
                    "current_batch_id": 4,
                    "active_roles": ["planner", "coder"],
                    "current_batch_tasks": ["task_1", "task_2"],
                    "review_queue": ["task_2"],
                    "review_queue_size": 1,
                    "last_decision_reason": "task_owner:review_required",
                    "policy": {
                        "batch_reason": "ready_non_conflicting_parallel:max=2",
                        "selected_reasons": {
                            "task_1": "ready:no_dependencies",
                            "task_2": "ready:no_dependencies"
                        },
                        "blocked_reasons": {
                            "task_3": "write_conflict:src/api.rs",
                            "task_4": "waiting_for_dependencies:task_2"
                        },
                        "review_reasons": {
                            "task_2": "review_required:risk=high+flag=required"
                        }
                    }
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("config.yaml"),
        "agents:\n  enabled: true\n  mode: single_orchestrator\n  review_policy: high_risk\n  explain_level: compact\n",
    )
    .expect("config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n",
    )
    .expect("task plan");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("status")
        .arg("--json")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run status");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        payload.get("agents_enabled").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        payload.get("agent_mode").and_then(|v| v.as_str()),
        Some("single_orchestrator")
    );
    assert_eq!(
        payload.get("agent_explain_level").and_then(|v| v.as_str()),
        Some("verbose")
    );
    assert_eq!(
        payload
            .get("agent_current_batch_id")
            .and_then(|v| v.as_i64()),
        Some(4)
    );
    assert_eq!(
        payload
            .get("agent_review_queue_size")
            .and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        payload
            .get("agent_last_decision_reason")
            .and_then(|v| v.as_str()),
        Some("task_owner:review_required")
    );
    assert_eq!(
        payload.get("agent_batch_reason").and_then(|v| v.as_str()),
        Some("ready_non_conflicting_parallel:max=2")
    );
    let active_roles: Vec<&str> = payload["agent_active_roles"]
        .as_array()
        .expect("active roles array")
        .iter()
        .map(|item| item.as_str().expect("role string"))
        .collect();
    assert_eq!(active_roles, vec!["planner", "coder"]);
    let current_batch_tasks: Vec<&str> = payload["agent_current_batch_tasks"]
        .as_array()
        .expect("current batch tasks array")
        .iter()
        .map(|item| item.as_str().expect("task id"))
        .collect();
    assert_eq!(current_batch_tasks, vec!["task_1", "task_2"]);
    let review_queue: Vec<&str> = payload["agent_review_queue"]
        .as_array()
        .expect("review queue array")
        .iter()
        .map(|item| item.as_str().expect("task id"))
        .collect();
    assert_eq!(review_queue, vec!["task_2"]);
    assert_eq!(
        payload
            .get("agent_selected_reasons")
            .and_then(|v| v.get("task_1"))
            .and_then(|v| v.as_str()),
        Some("ready:no_dependencies")
    );
    assert_eq!(
        payload
            .get("agent_blocked_reasons")
            .and_then(|v| v.get("task_4"))
            .and_then(|v| v.as_str()),
        Some("waiting_for_dependencies:task_2")
    );
    assert_eq!(
        payload
            .get("agent_review_reasons")
            .and_then(|v| v.get("task_2"))
            .and_then(|v| v.as_str()),
        Some("review_required:risk=high+flag=required")
    );
}

#[test]
fn status_json_includes_agent_collaboration_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {
                "agents": {
                    "enabled": true,
                    "mode": "role_handoff",
                    "explain_level": "compact",
                    "current_batch_id": 7,
                    "active_roles": ["planner", "coder", "reviewer"],
                    "current_batch_tasks": ["task_1", "task_2"],
                    "review_queue": ["task_2"],
                    "review_queue_size": 1,
                    "collaboration": {
                        "mode": "role_handoff",
                        "turn_role": "reviewer",
                        "turn_task_id": "task_2",
                        "turn_kind": "review_gate",
                        "pending_reviews": ["task_2"],
                        "blocked_handoff_reason": "awaiting_review_approval:task_2"
                    }
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("config.yaml"),
        "agents:\n  enabled: true\n  mode: role_handoff\n  review_policy: high_risk\n  explain_level: compact\n",
    )
    .expect("config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("status")
        .arg("--json")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run status");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        payload
            .get("agent_collaboration_mode")
            .and_then(|v| v.as_str()),
        Some("role_handoff")
    );
    assert_eq!(
        payload.get("agent_turn_role").and_then(|v| v.as_str()),
        Some("reviewer")
    );
    assert_eq!(
        payload.get("agent_turn_task_id").and_then(|v| v.as_str()),
        Some("task_2")
    );
    assert_eq!(
        payload.get("agent_turn_kind").and_then(|v| v.as_str()),
        Some("review_gate")
    );
    let pending_reviews: Vec<&str> = payload["agent_pending_reviews"]
        .as_array()
        .expect("pending reviews array")
        .iter()
        .map(|item| item.as_str().expect("task id"))
        .collect();
    assert_eq!(pending_reviews, vec!["task_2"]);
    assert_eq!(
        payload
            .get("agent_blocked_handoff_reason")
            .and_then(|v| v.as_str()),
        Some("awaiting_review_approval:task_2")
    );
}

#[test]
fn status_json_includes_backend_owner_and_achievement_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: 需求澄清 [COMPLETED]
- Type: research
### Task 2: 实现核心逻辑 [IN_PROGRESS]
- Type: implementation
### Task 3: 回归验证 [PENDING]
- Type: verification
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("events.jsonl"),
        serde_json::to_string(
            &json!({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 2}, "timestamp": 1.0}),
        )
        .expect("event")
            + "
" + &serde_json::to_string(
            &json!({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 3}, "timestamp": 2.0}),
        )
        .expect("event")
            + "
" + &serde_json::to_string(
            &json!({"type": "SUPERVISOR_ADVISORY", "payload": {}, "timestamp": 3.0}),
        )
        .expect("event")
            + "
",
    )
    .expect("events");
    fs::write(
        fusion.join("backend_failure_report.json"),
        serde_json::to_string_pretty(&json!({
            "status": "blocked",
            "source": "fusion-codeagent.sh",
            "primary_backend": "claude",
            "fallback_backend": "codex",
            "primary_error": "claude timeout",
            "fallback_error": "codex unavailable",
            "next_actions": ["retry with fallback"]
        }))
        .expect("json"),
    )
    .expect("backend report");
    fs::write(
        fusion.join("dependency_report.json"),
        serde_json::to_string_pretty(&json!({
            "status": "blocked",
            "source": "fusion-codeagent.sh",
            "reason": "Missing executable for backend orchestration",
            "missing": ["codeagent-wrapper"],
            "next_actions": ["Install or expose codeagent-wrapper in PATH."]
        }))
        .expect("json"),
    )
    .expect("dependency report");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("status")
        .arg("--json")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run status");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        payload.get("dependency_status").and_then(|v| v.as_str()),
        Some("blocked")
    );
    assert_eq!(
        payload.get("dependency_missing").and_then(|v| v.as_str()),
        Some("codeagent-wrapper")
    );
    assert_eq!(
        payload.get("dependency_source").and_then(|v| v.as_str()),
        Some("fusion-codeagent.sh")
    );
    assert_eq!(
        payload.get("dependency_reason").and_then(|v| v.as_str()),
        Some("Missing executable for backend orchestration")
    );
    assert_eq!(
        payload.get("dependency_next").and_then(|v| v.as_str()),
        Some("Install or expose codeagent-wrapper in PATH.")
    );
    assert_eq!(
        payload.get("backend_status").and_then(|v| v.as_str()),
        Some("blocked")
    );
    assert_eq!(
        payload.get("backend_source").and_then(|v| v.as_str()),
        Some("fusion-codeagent.sh")
    );
    assert_eq!(
        payload.get("backend_primary").and_then(|v| v.as_str()),
        Some("claude")
    );
    assert_eq!(
        payload.get("backend_fallback").and_then(|v| v.as_str()),
        Some("codex")
    );
    assert_eq!(
        payload
            .get("backend_primary_error")
            .and_then(|v| v.as_str()),
        Some("claude timeout")
    );
    assert_eq!(
        payload
            .get("backend_fallback_error")
            .and_then(|v| v.as_str()),
        Some("codex unavailable")
    );
    assert_eq!(
        payload.get("backend_next").and_then(|v| v.as_str()),
        Some("retry with fallback")
    );
    assert_eq!(
        payload
            .get("achievement_completed_tasks")
            .and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        payload
            .get("achievement_safe_total")
            .and_then(|v| v.as_i64()),
        Some(5)
    );
    assert_eq!(
        payload
            .get("achievement_advisory_total")
            .and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        payload
            .get("safe_backlog_last_added")
            .and_then(|v| v.as_i64()),
        Some(3)
    );
    assert_eq!(
        payload
            .get("safe_backlog_last_injected_at")
            .and_then(|v| v.as_f64()),
        Some(2.0)
    );
    assert_eq!(
        payload
            .get("safe_backlog_last_injected_at_iso")
            .and_then(|v| v.as_str()),
        Some("1970-01-01T00:00:02Z")
    );
    assert_eq!(
        payload.get("owner_planner").and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(payload.get("owner_coder").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(
        payload.get("owner_reviewer").and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(
        payload.get("current_role").and_then(|v| v.as_str()),
        Some("coder")
    );
    assert_eq!(
        payload.get("current_role_task").and_then(|v| v.as_str()),
        Some("实现核心逻辑")
    );
    assert_eq!(
        payload.get("current_role_status").and_then(|v| v.as_str()),
        Some("IN_PROGRESS")
    );
}

#[test]
fn status_prints_hook_debug_and_achievements_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        r#"## Status
- Current Phase: EXECUTE

### Task 1: 完成登录流程 [COMPLETED]
### Task 2: 编写回归测试 [COMPLETED]
### Task 3: 更新文档 [PENDING]
"#,
    )
    .expect("task plan");
    fs::write(
        fusion.join("progress.md"),
        "| t | EXECUTE | e | OK | d |
",
    )
    .expect("progress");
    fs::write(fusion.join(".hook_debug"), "").expect("flag");
    fs::write(
        fusion.join("hook-debug.log"),
        "[fusion][hook-debug][pretool][2026-02-12T00:00:00Z] invoked
",
    )
    .expect("hook log");
    fs::write(
        fusion.join("events.jsonl"),
        serde_json::to_string(&json!({
            "id": "evt_000010",
            "type": "SAFE_BACKLOG_INJECTED",
            "payload": {"added": 2},
            "timestamp": 1700000010.0
        }))
        .expect("event")
            + "
" + &serde_json::to_string(&json!({
            "id": "evt_000011",
            "type": "SUPERVISOR_ADVISORY",
            "payload": {"risk_score": 0.8},
            "timestamp": 1700000011.0
        }))
        .expect("event")
            + "
",
    )
    .expect("events");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("status")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run status");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "## Hook Debug"));
    assert!(line_contains_normalized(
        &stdout,
        "hook_debug.enabled: true"
    ));
    assert!(line_contains_normalized(&stdout, "hook_debug.tail:"));
    assert!(line_contains_normalized(
        &stdout,
        "[fusion][hook-debug][pretool]"
    ));
    assert!(line_contains_normalized(&stdout, "## Achievements"));
    assert!(line_contains_normalized(&stdout, "Completed tasks: 2"));
    assert!(line_contains_normalized(&stdout, "完成登录流程"));
    assert!(line_contains_normalized(&stdout, "编写回归测试"));
    assert!(line_contains_normalized(
        &stdout,
        "Safe backlog unlocked: +2 tasks"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Supervisor advisories recorded: 1"
    ));
}

#[test]
fn status_prints_top3_achievement_leaderboard() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let root = temp.path().join("projects");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&root).expect("root");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "## Status
- Current Phase: EXECUTE
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("progress.md"),
        "| t | EXECUTE | e | OK | d |
",
    )
    .expect("progress");

    let alpha = root.join("alpha").join(".fusion");
    fs::create_dir_all(&alpha).expect("alpha");
    fs::write(
        alpha.join("sessions.json"),
        json!({"status": "completed"}).to_string(),
    )
    .expect("alpha sessions");
    fs::write(
        alpha.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
### Task 2: B [COMPLETED]
",
    )
    .expect("alpha task");
    fs::write(
        alpha.join("events.jsonl"),
        json!({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 1}}).to_string()
            + "
",
    )
    .expect("alpha events");

    let gamma = root.join("gamma").join(".fusion");
    fs::create_dir_all(&gamma).expect("gamma");
    fs::write(
        gamma.join("sessions.json"),
        json!({"status": "completed"}).to_string(),
    )
    .expect("gamma sessions");
    fs::write(
        gamma.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
",
    )
    .expect("gamma task");
    fs::write(
        gamma.join("events.jsonl"),
        json!({"type": "SUPERVISOR_ADVISORY", "payload": {}}).to_string()
            + "
",
    )
    .expect("gamma events");

    let beta = root.join("beta").join(".fusion");
    fs::create_dir_all(&beta).expect("beta");
    fs::write(
        beta.join("sessions.json"),
        json!({"status": "in_progress"}).to_string(),
    )
    .expect("beta sessions");
    fs::write(
        beta.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
### Task 2: B [COMPLETED]
",
    )
    .expect("beta task");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("status")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("FUSION_LEADERBOARD_ROOT", &root);

    let output = cmd.output().expect("run status");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "## Achievement Leaderboard (Top 3)"
    ));
    assert!(line_contains_normalized(&stdout, "1) alpha | score=73"));
    assert!(line_contains_normalized(&stdout, "2) gamma | score=62"));
    assert!(line_contains_normalized(&stdout, "3) beta | score=20"));
}

#[test]
fn status_can_disable_leaderboard() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let root = temp.path().join("projects");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&root).expect("root");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "## Status
- Current Phase: EXECUTE
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("progress.md"),
        "| t | EXECUTE | e | OK | d |
",
    )
    .expect("progress");

    let alpha = root.join("alpha").join(".fusion");
    fs::create_dir_all(&alpha).expect("alpha");
    fs::write(
        alpha.join("sessions.json"),
        json!({"status": "completed"}).to_string(),
    )
    .expect("alpha sessions");
    fs::write(
        alpha.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
",
    )
    .expect("alpha task");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("status")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("FUSION_LEADERBOARD_ROOT", &root)
        .env("FUSION_STATUS_SHOW_LEADERBOARD", "0");

    let output = cmd.output().expect("run status");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!line_contains_normalized(
        &stdout,
        "## Achievement Leaderboard (Top 3)"
    ));
}

#[test]
fn achievements_print_local_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "completed",
            "current_phase": "DELIVER"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: 实现登录 [COMPLETED]
### Task 2: 增加测试 [COMPLETED]
### Task 3: 更新文档 [PENDING]
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("events.jsonl"),
        json!({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 1}}).to_string()
            + "
" + &json!({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 2}}).to_string()
            + "
" + &json!({"type": "SUPERVISOR_ADVISORY", "payload": {}}).to_string()
            + "
",
    )
    .expect("events");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("achievements")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--local-only");

    let output = cmd.output().expect("run achievements");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "=== Fusion Achievements ==="
    ));
    assert!(line_contains_normalized(
        &stdout,
        "## Current Workspace Achievements"
    ));
    assert!(line_contains_normalized(&stdout, "Workflow completed"));
    assert!(line_contains_normalized(&stdout, "Completed tasks: 2"));
    assert!(line_contains_normalized(
        &stdout,
        "Safe backlog unlocked: +3 tasks (2 rounds)"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Supervisor advisories recorded: 1"
    ));
    assert!(line_contains_normalized(&stdout, "score=81"));
    assert!(!line_contains_normalized(
        &stdout,
        "## Achievement Leaderboard"
    ));
}

#[test]
fn achievements_print_leaderboard_only() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("projects");
    let alpha = root.join("alpha").join(".fusion");
    let beta = root.join("beta").join(".fusion");
    fs::create_dir_all(&alpha).expect("alpha");
    fs::create_dir_all(&beta).expect("beta");

    fs::write(
        alpha.join("sessions.json"),
        json!({"status": "completed"}).to_string(),
    )
    .expect("alpha sessions");
    fs::write(
        alpha.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
### Task 2: B [COMPLETED]
",
    )
    .expect("alpha task");
    fs::write(
        alpha.join("events.jsonl"),
        json!({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 1}}).to_string()
            + "
",
    )
    .expect("alpha events");

    fs::write(
        beta.join("sessions.json"),
        json!({"status": "in_progress"}).to_string(),
    )
    .expect("beta sessions");
    fs::write(
        beta.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
### Task 2: B [COMPLETED]
",
    )
    .expect("beta task");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("achievements")
        .arg("--leaderboard-only")
        .arg("--root")
        .arg(&root)
        .arg("--top")
        .arg("2");

    let output = cmd.output().expect("run achievements");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "## Achievement Leaderboard"
    ));
    assert!(line_contains_normalized(&stdout, "1) alpha | score=73"));
    assert!(line_contains_normalized(&stdout, "2) beta | score=20"));
    assert!(!line_contains_normalized(
        &stdout,
        "## Current Workspace Achievements"
    ));
}

#[test]
fn achievements_reject_zero_top() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("projects");
    fs::create_dir_all(&root).expect("root");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("achievements")
        .arg("--leaderboard-only")
        .arg("--root")
        .arg(&root)
        .arg("--top")
        .arg("0");

    assert_stderr_contains_normalized(
        &mut cmd,
        "run achievements with invalid top",
        None,
        "--top must be a positive integer",
    );
}

#[test]
fn pause_updates_session_status_and_checkpoint() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "goal": "demo"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("pause")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run pause");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "Workflow paused"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("paused")
    );
    assert!(sessions
        .get("last_checkpoint")
        .and_then(|v| v.as_str())
        .is_some());
}

#[test]
fn pause_rejects_non_in_progress_workflow() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "paused",
            "goal": "demo"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("pause")
        .arg("--fusion-dir")
        .arg(".fusion");

    assert_stderr_contains_normalized(
        &mut cmd,
        "run pause against paused workflow",
        None,
        "Workflow is not in progress",
    );
}

#[test]
fn cancel_updates_session_status_and_checkpoint() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "goal": "demo"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("cancel")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run cancel");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "Workflow cancelled"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("cancelled")
    );
    assert!(sessions
        .get("last_checkpoint")
        .and_then(|v| v.as_str())
        .is_some());
}

#[test]
fn cancel_is_idempotent_for_completed_workflow() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "completed",
            "goal": "demo"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("cancel")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run cancel");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Workflow is already completed"
    ));
}

#[test]
fn continue_appends_marker_once_for_in_progress_workflow() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(fusion.join("progress.md"), "| t | EXECUTE | e | OK | d |\n").expect("progress");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let mut first = Command::cargo_bin("fusion-bridge").expect("binary");
    first
        .current_dir(temp.path())
        .arg("continue")
        .arg("--fusion-dir")
        .arg(".fusion");
    let first_output = first.output().expect("run continue");
    assert!(
        first_output.status.success(),
        "{}",
        String::from_utf8_lossy(&first_output.stderr)
    );
    let first_stdout = String::from_utf8_lossy(&first_output.stdout);
    assert!(line_contains_normalized(
        &first_stdout,
        "Current state: in_progress @ EXECUTE"
    ));
    assert!(line_contains_normalized(
        &first_stdout,
        "Next action: Continue task: A [PENDING]"
    ));
    assert!(line_contains_normalized(&first_stdout, "Hook debug: OFF"));

    let mut second = Command::cargo_bin("fusion-bridge").expect("binary");
    second
        .current_dir(temp.path())
        .arg("continue")
        .arg("--fusion-dir")
        .arg(".fusion")
        .assert()
        .success();

    let progress = fs::read_to_string(fusion.join("progress.md")).expect("progress");
    assert_eq!(progress.matches("[CONTINUE]").count(), 1);
    assert!(line_contains_normalized(
        &progress,
        "Phase: EXECUTE | Pending: 1"
    ));
}

#[test]
fn continue_prints_review_gate_next_action() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("config.yaml"),
        "agents:\n  enabled: true\n  mode: role_handoff\n  review_policy: high_risk\nsafe_backlog:\n  enabled: false\n  inject_on_task_exhausted: false\n",
    )
    .expect("config");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "review gate"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(fusion.join("progress.md"), "| t | EXECUTE | e | OK | d |\n").expect("progress");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("continue")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run continue");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Current state: in_progress @ EXECUTE"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: reviewer approve task_2 before execution continues"
    ));
    assert!(line_contains_normalized(&stdout, "Hook debug: OFF"));
}

#[test]
fn continue_noops_when_workflow_not_in_progress() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "paused",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(fusion.join("progress.md"), "| t | EXECUTE | e | OK | d |\n").expect("progress");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("continue")
        .arg("--fusion-dir")
        .arg(".fusion")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let progress = fs::read_to_string(fusion.join("progress.md")).expect("progress");
    assert!(!line_contains_normalized(&progress, "[CONTINUE]"));
}

#[test]
fn init_rejects_non_rust_engine() {
    let temp = tempdir().expect("tempdir");
    let templates = temp.path().join("templates");
    fs::create_dir_all(&templates).expect("templates dir");
    fs::write(
        templates.join("task_plan.md"),
        "## Status
",
    )
    .expect("task template");
    fs::write(
        templates.join("progress.md"),
        "| h |
",
    )
    .expect("progress template");
    fs::write(
        templates.join("findings.md"),
        "# findings
",
    )
    .expect("findings template");
    fs::write(
        templates.join("sessions.json"),
        serde_json::to_string_pretty(&json!({"status":"not_started"})).expect("sessions template"),
    )
    .expect("write sessions template");
    fs::write(
        templates.join("config.yaml"),
        "runtime:
  enabled: true
",
    )
    .expect("config template");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--templates-dir")
        .arg("templates")
        .arg("--engine")
        .arg("legacy");

    assert_stderr_contains_normalized(
        &mut cmd,
        "run init with legacy engine",
        None,
        "Invalid engine: legacy (expected: rust)",
    );
    assert!(!temp.path().join(".fusion").join("config.yaml").exists());
}

#[test]
fn catchup_reports_unsynced_context_and_next_task() {
    let temp = tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let fusion = project.join(".fusion");
    fs::create_dir_all(&fusion).expect("fusion dir");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
### Task 2: B [PENDING]
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("progress.md"),
        "## progress
",
    )
    .expect("progress");
    fs::write(
        fusion.join("findings.md"),
        "## findings
",
    )
    .expect("findings");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "demo goal",
            "codex_session": "sess_123"
        }))
        .expect("sessions json"),
    )
    .expect("sessions file");

    let canonical_project = fs::canonicalize(&project).expect("canonical project");
    let sanitized = claude_project_slug_for_test(&canonical_project);

    let claude_projects = temp
        .path()
        .join("home")
        .join(".claude")
        .join("projects")
        .join(sanitized);
    fs::create_dir_all(&claude_projects).expect("claude project dir");
    fs::write(
        claude_projects.join("session.jsonl"),
        concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Write","input":{"file_path":"/tmp/project/.fusion/progress.md"}}]}}"#,
            "
",
            r#"{"type":"user","message":{"content":"Please continue task B and verify it"}}"#,
            "
",
            r#"{"type":"assistant","message":{"content":"I updated implementation details"}}"#,
            "
"
        ),
    )
    .expect("session jsonl");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.env("HOME", temp.path().join("home"))
        .current_dir(&project)
        .arg("catchup")
        .arg("--project-path")
        .arg(&project);

    let output = cmd.output().expect("run catchup");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "SESSION RECOVERY REPORT"));
    assert!(line_contains_normalized(&stdout, "Goal: demo goal"));
    assert!(line_contains_normalized(
        &stdout,
        "Current state: in_progress @ EXECUTE"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Continue task: B [PENDING]"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Resume Codex session: sess_123"
    ));
    assert!(line_contains_normalized(&stdout, "UNSYNCED CONTEXT"));
}

#[test]
fn catchup_reports_review_gate_next_action() {
    let temp = tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let fusion = project.join(".fusion");
    fs::create_dir_all(&fusion).expect("fusion dir");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");
    fs::write(fusion.join("progress.md"), "## progress\n").expect("progress");
    fs::write(fusion.join("findings.md"), "## findings\n").expect("findings");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "review gate",
            "reviewer_codex_session": "reviewer_codex_456"
        }))
        .expect("sessions json"),
    )
    .expect("sessions file");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(&project)
        .arg("catchup")
        .arg("--project-path")
        .arg(&project);

    let output = cmd.output().expect("run catchup");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Next action: reviewer approve task_2 before execution continues"
    ));
}

#[test]
fn catchup_without_tasks_reports_decompose_next_action() {
    let temp = tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let fusion = project.join(".fusion");
    fs::create_dir_all(&fusion).expect("fusion dir");

    fs::write(fusion.join("progress.md"), "## progress\n").expect("progress");
    fs::write(fusion.join("findings.md"), "## findings\n").expect("findings");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "INITIALIZE",
            "goal": "decompose"
        }))
        .expect("sessions json"),
    )
    .expect("sessions file");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(&project)
        .arg("catchup")
        .arg("--project-path")
        .arg(&project);

    let output = cmd.output().expect("run catchup");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Create task plan and run the DECOMPOSE phase"
    ));
}

#[test]
fn catchup_completed_tasks_reports_verify_next_action() {
    let temp = tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let fusion = project.join(".fusion");
    fs::create_dir_all(&fusion).expect("fusion dir");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [COMPLETED]\n",
    )
    .expect("task plan");
    fs::write(fusion.join("progress.md"), "## progress\n").expect("progress");
    fs::write(fusion.join("findings.md"), "## findings\n").expect("findings");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "verify"
        }))
        .expect("sessions json"),
    )
    .expect("sessions file");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(&project)
        .arg("catchup")
        .arg("--project-path")
        .arg(&project);

    let output = cmd.output().expect("run catchup");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Proceed to VERIFY phase"
    ));
}

#[test]
fn init_creates_fusion_files_from_templates() {
    let temp = tempdir().expect("tempdir");
    let templates = temp.path().join("templates");
    fs::create_dir_all(&templates).expect("templates dir");
    fs::write(templates.join("task_plan.md"), "## Status\n").expect("task template");
    fs::write(templates.join("progress.md"), "| h |\n").expect("progress template");
    fs::write(templates.join("findings.md"), "# findings\n").expect("findings template");
    fs::write(
        templates.join("sessions.json"),
        serde_json::to_string_pretty(&json!({"status":"not_started"})).expect("sessions template"),
    )
    .expect("write sessions template");
    fs::write(templates.join("config.yaml"), "runtime:\n  enabled: true\n")
        .expect("config template");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--templates-dir")
        .arg("templates");

    let output = cmd.output().expect("run init");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Initialized .fusion directory"
    ));

    let fusion = temp.path().join(".fusion");
    assert!(fusion.join("task_plan.md").exists());
    assert!(fusion.join("progress.md").exists());
    assert!(fusion.join("findings.md").exists());
    assert!(fusion.join("sessions.json").exists());
    assert!(fusion.join("config.yaml").exists());
}

#[test]
fn start_without_force_sets_goal_and_phase() {
    let temp = tempdir().expect("tempdir");
    let templates = temp.path().join("templates");
    fs::create_dir_all(&templates).expect("templates dir");
    fs::write(templates.join("task_plan.md"), "## Status\n").expect("task template");
    fs::write(templates.join("progress.md"), "| h |\n").expect("progress template");
    fs::write(templates.join("findings.md"), "# findings\n").expect("findings template");
    fs::write(
        templates.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status":"not_started",
            "goal": null,
            "current_phase": null,
            "workflow_id": null,
            "started_at": null
        }))
        .expect("sessions template"),
    )
    .expect("write sessions template");
    fs::write(templates.join("config.yaml"), "runtime:\n  enabled: true\n")
        .expect("config template");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("start")
        .arg("梳理架构")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--templates-dir")
        .arg("templates");

    let output = cmd.output().expect("run start");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "UNDERSTAND runner currently minimal"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Current state: in_progress @ INITIALIZE"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Initialize workspace files and proceed to ANALYZE"
    ));
    assert!(line_contains_normalized(&stdout, "Workflow initialized"));

    let sessions: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(temp.path().join(".fusion/sessions.json")).expect("sessions"),
    )
    .expect("parse sessions");

    assert_eq!(
        sessions.get("goal").and_then(|v| v.as_str()),
        Some("梳理架构")
    );
    assert_eq!(
        sessions.get("current_phase").and_then(|v| v.as_str()),
        Some("INITIALIZE")
    );
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("in_progress")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("state"))
            .and_then(|v| v.as_str()),
        Some("INITIALIZE")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("understand"))
            .and_then(|v| v.get("mode"))
            .and_then(|v| v.as_str()),
        Some("minimal")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("understand"))
            .and_then(|v| v.get("forced"))
            .and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("understand"))
            .and_then(|v| v.get("decision"))
            .and_then(|v| v.as_str()),
        Some("auto_continue")
    );
}

#[test]
fn start_sets_goal_and_phase() {
    let temp = tempdir().expect("tempdir");
    let templates = temp.path().join("templates");
    fs::create_dir_all(&templates).expect("templates dir");
    fs::write(templates.join("task_plan.md"), "## Status\n").expect("task template");
    fs::write(templates.join("progress.md"), "| h |\n").expect("progress template");
    fs::write(templates.join("findings.md"), "# findings\n").expect("findings template");
    fs::write(
        templates.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status":"not_started",
            "goal": null,
            "current_phase": null,
            "workflow_id": null,
            "started_at": null
        }))
        .expect("sessions template"),
    )
    .expect("write sessions template");
    fs::write(templates.join("config.yaml"), "runtime:\n  enabled: true\n")
        .expect("config template");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("start")
        .arg("实现认证")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--templates-dir")
        .arg("templates")
        .arg("--force");

    let output = cmd.output().expect("run start");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Skipped UNDERSTAND (--force)"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Current state: in_progress @ INITIALIZE"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Initialize workspace files and proceed to ANALYZE"
    ));
    assert!(line_contains_normalized(&stdout, "Workflow initialized"));

    let sessions: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(temp.path().join(".fusion/sessions.json")).expect("sessions"),
    )
    .expect("parse sessions");

    assert_eq!(
        sessions.get("goal").and_then(|v| v.as_str()),
        Some("实现认证")
    );
    assert_eq!(
        sessions.get("current_phase").and_then(|v| v.as_str()),
        Some("INITIALIZE")
    );
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("in_progress")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("state"))
            .and_then(|v| v.as_str()),
        Some("INITIALIZE")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("understand"))
            .and_then(|v| v.get("mode"))
            .and_then(|v| v.as_str()),
        Some("skipped")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("understand"))
            .and_then(|v| v.get("forced"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("understand"))
            .and_then(|v| v.get("decision"))
            .and_then(|v| v.as_str()),
        Some("force_skip")
    );
}

#[test]
fn hook_pretool_prints_context_lines() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "测试 pretool"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n- Type: implementation\n",
    )
    .expect("task plan");
    fs::write(
        fusion.join("loop_context.json"),
        serde_json::to_string_pretty(&json!({
            "no_progress_rounds": 4,
            "same_action_count": 0
        }))
        .expect("loop context json"),
    )
    .expect("loop context");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("hook")
        .arg("pretool")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run hook pretool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "[fusion] Goal:"));
    assert!(line_contains_normalized(&stdout, "[fusion] Progress:"));
    assert!(line_contains_normalized(&stdout, "(type: implementation)"));
    assert!(line_contains_normalized(&stdout, "TDD flow"));
    assert!(line_contains_normalized(&stdout, "Guardian: ⚠ BACKOFF"));
}

#[test]
fn hook_pretool_prints_agent_batch_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "agent batch",
            "_runtime": {
                "agents": {
                    "enabled": true,
                    "mode": "single_orchestrator",
                    "current_batch_id": 5,
                    "active_roles": ["planner", "coder"],
                    "current_batch_tasks": ["task_1", "task_2"],
                    "review_queue": ["task_2"],
                    "review_queue_size": 1
                },
                "scheduler": {
                    "enabled": true,
                    "current_batch_id": 5,
                    "parallel_tasks": 2
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Research API [PENDING]\n- Type: research\n",
    )
    .expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("hook")
        .arg("pretool")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run hook pretool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Agent batch: 5 | Roles: planner, coder | Review queue: 1"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Agent tasks: task_1, task_2"
    ));
}

#[test]
fn hook_pretool_prints_agent_handoff_summary() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "role handoff",
            "_runtime": {
                "agents": {
                    "enabled": true,
                    "mode": "role_handoff",
                    "current_batch_id": 6,
                    "active_roles": ["planner", "coder", "reviewer"],
                    "current_batch_tasks": ["task_1", "task_2"],
                    "review_queue": ["task_2"],
                    "review_queue_size": 1,
                    "collaboration": {
                        "mode": "role_handoff",
                        "turn_role": "reviewer",
                        "turn_task_id": "task_2",
                        "turn_kind": "review_gate",
                        "pending_reviews": ["task_2"],
                        "blocked_handoff_reason": "awaiting_review_approval:task_2"
                    }
                },
                "scheduler": {
                    "enabled": true,
                    "current_batch_id": 6,
                    "parallel_tasks": 2
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("hook")
        .arg("pretool")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run hook pretool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Agent turn: reviewer -> task_2 (review_gate)"
    ));
    assert!(line_contains_normalized(&stdout, "Pending reviews: task_2"));
}

#[test]
fn hook_pretool_prints_review_gate_guidance() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "review gate"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("hook")
        .arg("pretool")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run hook pretool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Task 2/2: Build API [IN_PROGRESS] (type: implementation)"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "→ Review gate: reviewer approve task_2 before execution continues"
    ));
    assert!(!line_contains_normalized(
        &stdout,
        "→ TDD flow: RED → GREEN → REFACTOR"
    ));
}

#[test]
fn shell_pretool_fallback_prints_progress_bar_and_tdd_guidance() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "测试 pretool"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n- Type: implementation\n",
    )
    .expect("task plan");
    fs::write(
        fusion.join("loop_context.json"),
        serde_json::to_string_pretty(&json!({
            "no_progress_rounds": 4,
            "same_action_count": 0
        }))
        .expect("loop context json"),
    )
    .expect("loop context");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-pretool.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell pretool fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "[fusion] Goal:"));
    assert!(line_contains_normalized(
        &stdout,
        "Task 2/2: B [PENDING] (type: implementation)"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Progress: █████░░░░░ 50% | Guardian: ⚠ BACKOFF"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "→ TDD flow: RED → GREEN → REFACTOR"
    ));
}

#[test]
fn shell_pretool_fallback_prints_direct_execution_for_research_tasks() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "研究任务"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Research API [PENDING]\n- Type: research\n",
    )
    .expect("task plan");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-pretool.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell pretool fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Task 1/1: Research API [PENDING] (type: research)"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Progress: ░░░░░░░░░░ 0% | Guardian: OK"
    ));
    assert!(line_contains_normalized(&stdout, "→ Direct execution"));
}

#[test]
fn shell_pretool_fallback_marks_in_progress_task_and_guardian_warning() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "正在执行"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Build API [IN_PROGRESS]\n- Type: implementation\n",
    )
    .expect("task plan");
    fs::write(
        fusion.join("loop_context.json"),
        serde_json::to_string_pretty(&json!({
            "no_progress_rounds": 2,
            "same_action_count": 0
        }))
        .expect("loop context json"),
    )
    .expect("loop context");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-pretool.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell pretool fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Task 1/1: Build API [IN_PROGRESS] (type: implementation)"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Progress: ░░░░░░░░░░ 0% | Guardian: ~"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "→ TDD flow: RED → GREEN → REFACTOR"
    ));
}

#[test]
fn shell_pretool_fallback_prints_scheduler_and_agent_batch_summary() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "agent batch",
            "_runtime": {
                "agents": {
                    "enabled": true,
                    "mode": "single_orchestrator",
                    "current_batch_id": 5,
                    "active_roles": ["planner", "coder"],
                    "current_batch_tasks": ["task_1", "task_2"],
                    "review_queue": ["task_2"],
                    "review_queue_size": 1
                },
                "scheduler": {
                    "enabled": true,
                    "current_batch_id": 5,
                    "parallel_tasks": 2
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Research API [PENDING]\n- Type: research\n",
    )
    .expect("task plan");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-pretool.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell pretool fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Batch: 5 | Parallel: 2 tasks"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Agent batch: 5 | Roles: planner, coder | Review queue: 1"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Agent tasks: task_1, task_2"
    ));
}

#[test]
fn shell_pretool_fallback_prints_agent_handoff_summary() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "role handoff",
            "_runtime": {
                "agents": {
                    "enabled": true,
                    "mode": "role_handoff",
                    "current_batch_id": 6,
                    "active_roles": ["planner", "coder", "reviewer"],
                    "current_batch_tasks": ["task_1", "task_2"],
                    "review_queue": ["task_2"],
                    "review_queue_size": 1,
                    "collaboration": {
                        "mode": "role_handoff",
                        "turn_role": "reviewer",
                        "turn_task_id": "task_2",
                        "turn_kind": "review_gate",
                        "pending_reviews": ["task_2"],
                        "blocked_handoff_reason": "awaiting_review_approval:task_2"
                    }
                },
                "scheduler": {
                    "enabled": true,
                    "current_batch_id": 6,
                    "parallel_tasks": 2
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-pretool.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell pretool fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Agent turn: reviewer -> task_2 (review_gate)"
    ));
    assert!(line_contains_normalized(&stdout, "Pending reviews: task_2"));
}

#[test]
fn shell_pretool_fallback_prints_review_gate_guidance() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "review gate"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-pretool.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell pretool fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Task 2/2: Build API [IN_PROGRESS] (type: implementation)"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "→ Review gate: reviewer approve task_2 before execution continues"
    ));
    assert!(!line_contains_normalized(
        &stdout,
        "→ TDD flow: RED → GREEN → REFACTOR"
    ));
}

#[test]
fn hook_posttool_injects_safe_backlog_on_no_progress() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:\n  enabled: true\nsafe_backlog:\n  enabled: true\n  trigger_no_progress_rounds: 2\n  max_tasks_per_run: 1\n  allowed_categories: documentation\n",
    )
    .expect("config");
    fs::write(temp.path().join("README.md"), "# Demo\n").expect("readme");
    fs::write(fusion.join(".progress_snapshot"), "0:1:0:0").expect("snapshot");

    let mut first = Command::cargo_bin("fusion-bridge").expect("binary");
    first
        .current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");
    first.assert().success();

    let mut second = Command::cargo_bin("fusion-bridge").expect("binary");
    second
        .current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");
    let second_output = second.output().expect("run hook posttool");
    assert!(
        second_output.status.success(),
        "{}",
        String::from_utf8_lossy(&second_output.stderr)
    );
    let second_stdout = String::from_utf8_lossy(&second_output.stdout);
    assert!(line_contains_normalized(
        &second_stdout,
        "Safe backlog injected"
    ));

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("task plan read");
    assert!(line_contains_normalized(&task_plan, "[SAFE_BACKLOG]"));

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events read");
    assert!(line_contains_normalized(&events, "SAFE_BACKLOG_INJECTED"));
}

#[test]
fn hook_posttool_runtime_disabled_skips_safe_backlog_injection() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {
                "scheduler": {
                    "enabled": true,
                    "current_batch_id": 7,
                    "parallel_tasks": 2
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:\n  enabled: false\nsafe_backlog:\n  enabled: true\n  trigger_no_progress_rounds: 2\n  max_tasks_per_run: 1\n  allowed_categories: documentation\n",
    )
    .expect("config");
    fs::write(temp.path().join("README.md"), "# Demo\n").expect("readme");
    fs::write(fusion.join(".progress_snapshot"), "0:1:0:0").expect("snapshot");

    let mut first = Command::cargo_bin("fusion-bridge").expect("binary");
    first
        .current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");
    first.assert().success().stdout(predicates::str::is_empty());

    let mut second = Command::cargo_bin("fusion-bridge").expect("binary");
    second
        .current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");
    second
        .assert()
        .success()
        .stdout(predicates::str::is_empty());

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("task plan contents");
    assert!(!line_contains_normalized(&task_plan, "[SAFE_BACKLOG]"));
    assert!(!fusion.join("events.jsonl").exists());
    assert!(!fusion.join("safe_backlog.json").exists());
}

#[test]
fn hook_posttool_runtime_disabled_hides_runtime_only_lines() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {
                "scheduler": {
                    "enabled": true,
                    "current_batch_id": 9,
                    "parallel_tasks": 3
                }
            }
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n- Type: implementation\n",
    )
    .expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:\n  enabled: false\nsafe_backlog:\n  enabled: true\n",
    )
    .expect("config");
    fs::write(fusion.join(".progress_snapshot"), "0:2:0:0").expect("snapshot");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run hook posttool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "Task A → COMPLETED"));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Continue task: B [PENDING] | Mode: TDD"
    ));
    assert!(!line_contains_normalized(&stdout, "Batch 9 progress"));
    assert!(!line_contains_normalized(&stdout, "Safe backlog injected"));
    assert!(!line_contains_normalized(&stdout, "Advisory:"));
}

#[test]
fn hook_posttool_prints_named_completion_and_next_guidance() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n- Type: implementation\n",
    )
    .expect("task plan");
    fs::write(fusion.join(".progress_snapshot"), "0:2:0:0").expect("snapshot");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run hook posttool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "Task A → COMPLETED"));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Continue task: B [PENDING] | Mode: TDD"
    ));
}

#[test]
fn hook_posttool_prints_review_gate_next_action() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("config.yaml"),
        "agents:\n  enabled: true\n  mode: role_handoff\n  review_policy: high_risk\n",
    )
    .expect("config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");
    fs::write(fusion.join(".progress_snapshot"), "0:1:1:0").expect("snapshot");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run hook posttool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Task Plan API → COMPLETED"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: reviewer approve task_2 before execution continues"
    ));
}

#[test]
fn hook_posttool_runtime_enabled_persists_task_done_event() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
### Task 2: B [PENDING]
- Type: implementation
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:
  enabled: true
safe_backlog:
  enabled: false
",
    )
    .expect("config");
    fs::write(fusion.join(".progress_snapshot"), "0:2:0:0").expect("snapshot");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run hook posttool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "Task A → COMPLETED"));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Continue task: B [PENDING] | Mode: TDD"
    ));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("current_phase").and_then(|v| v.as_str()),
        Some("EXECUTE")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("state"))
            .and_then(|v| v.as_str()),
        Some("EXECUTE")
    );

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events read");
    assert!(line_contains_normalized(&events, "TASK_DONE"));
}

#[test]
fn hook_posttool_runtime_enabled_persists_all_tasks_done_event() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]
",
    )
    .expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:
  enabled: true
safe_backlog:
  enabled: false
",
    )
    .expect("config");
    fs::write(fusion.join(".progress_snapshot"), "0:1:0:0").expect("snapshot");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run hook posttool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "Task A → COMPLETED"));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Proceed to VERIFY phase"
    ));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("current_phase").and_then(|v| v.as_str()),
        Some("VERIFY")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("state"))
            .and_then(|v| v.as_str()),
        Some("VERIFY")
    );

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events read");
    assert!(line_contains_normalized(&events, "ALL_TASKS_DONE"));
}

#[test]
fn shell_posttool_fallback_prints_named_completion_and_next_guidance() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n- Type: implementation\n",
    )
    .expect("task plan");
    fs::write(fusion.join(".progress_snapshot"), "0:2:0:0").expect("snapshot");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-posttool.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell posttool fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "Task A → COMPLETED"));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Continue task: B [PENDING] | Mode: TDD"
    ));
}

#[test]
fn shell_posttool_fallback_prints_review_gate_next_action() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");
    fs::write(fusion.join(".progress_snapshot"), "0:1:1:0").expect("snapshot");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-posttool.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell posttool fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Task Plan API → COMPLETED"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: reviewer approve task_2 before execution continues"
    ));
}

#[test]
fn shell_posttool_fallback_completed_tasks_report_verify_next_action() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [COMPLETED]\n").expect("task plan");
    fs::write(fusion.join(".progress_snapshot"), "0:1:0:0").expect("snapshot");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-posttool.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell posttool fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(&stdout, "Task A → COMPLETED"));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Proceed to VERIFY phase"
    ));
}

#[test]
fn hook_posttool_emits_no_progress_reminder_after_five_rounds() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: A [IN_PROGRESS]\n- Type: implementation\n### Task 2: B [PENDING]\n",
    )
    .expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "safe_backlog:\n  enabled: false\n",
    )
    .expect("config");
    fs::write(fusion.join(".progress_snapshot"), "0:1:1:0").expect("snapshot");
    fs::write(fusion.join(".snapshot_unchanged_count"), "4").expect("unchanged count");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run hook posttool");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Info: 5 file edits since last task status change."
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Current: A [IN_PROGRESS]"
    ));
}

#[test]
fn hook_stop_guard_blocks_when_pending_tasks_exist() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "继续执行"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("hook")
        .arg("stop-guard")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run stop-guard");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("parse json");
    assert_eq!(
        value.get("decision").and_then(|v| v.as_str()),
        Some("block")
    );
    assert_eq!(
        value.get("systemMessage").and_then(|v| v.as_str()),
        Some("🔄 Fusion | Phase: EXECUTE | Remaining: 1 | Next: Continue task: A [PENDING]")
    );
    let reason = value
        .get("reason")
        .and_then(|v| v.as_str())
        .expect("stop guard reason");
    assert!(line_contains_normalized(
        reason,
        "Next action: Continue task: A [PENDING]"
    ));
}

#[test]
fn hook_stop_guard_review_gate_requires_reviewer_approval() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "review gate"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("config.yaml"),
        "agents:\n  enabled: true\n  mode: role_handoff\n  review_policy: high_risk\n  explain_level: compact\n",
    )
    .expect("config");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("hook")
        .arg("stop-guard")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run stop-guard");

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        value.get("decision").and_then(|v| v.as_str()),
        Some("block")
    );
    assert_eq!(
        value.get("systemMessage").and_then(|v| v.as_str()),
        Some(
            "🔄 Fusion | Phase: EXECUTE | Remaining: 1 | Next: reviewer approve task_2 before execution continues"
        )
    );
    let reason = value
        .get("reason")
        .and_then(|v| v.as_str())
        .expect("stop guard reason");
    assert!(line_contains_normalized(
        reason,
        "Review gate: reviewer approve task_2 before execution continues"
    ));
    assert!(line_contains_normalized(reason, "Task: task_2 (Build API)"));
    assert!(line_contains_normalized(
        reason,
        "If approved, set `- Review-Status: approved`"
    ));
}

#[test]
fn hook_stop_guard_without_tasks_surfaces_decompose_next_action() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "INITIALIZE",
            "goal": "拆任务"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("hook")
        .arg("stop-guard")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run stop-guard");

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        value.get("decision").and_then(|v| v.as_str()),
        Some("block")
    );
    assert_eq!(
        value.get("systemMessage").and_then(|v| v.as_str()),
        Some("🔄 Fusion | Phase: INITIALIZE | Next: Create task plan and run the DECOMPOSE phase")
    );
    let reason = value
        .get("reason")
        .and_then(|v| v.as_str())
        .expect("stop guard reason");
    assert!(line_contains_normalized(
        reason,
        "Next action: Create task plan and run the DECOMPOSE phase"
    ));
}

#[test]
fn shell_stop_guard_fallback_blocks_when_pending_tasks_exist() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "继续执行"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-stop-guard.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell stop-guard fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        value.get("decision").and_then(|v| v.as_str()),
        Some("block")
    );
    assert_eq!(
        value.get("systemMessage").and_then(|v| v.as_str()),
        Some("🔄 Fusion | Phase: EXECUTE | Remaining: 1 | Next: Continue task: A [PENDING]")
    );
    let reason = value
        .get("reason")
        .and_then(|v| v.as_str())
        .expect("stop guard reason");
    assert!(line_contains_normalized(
        reason,
        "Next action: Continue task: A [PENDING]"
    ));
}

#[test]
fn shell_stop_guard_fallback_review_gate_requires_reviewer_approval() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "review gate"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-stop-guard.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell stop-guard fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        value.get("decision").and_then(|v| v.as_str()),
        Some("block")
    );
    assert_eq!(
        value.get("systemMessage").and_then(|v| v.as_str()),
        Some(
            "🔄 Fusion | Phase: EXECUTE | Remaining: 1 | Next: reviewer approve task_2 before execution continues"
        )
    );
    let reason = value
        .get("reason")
        .and_then(|v| v.as_str())
        .expect("stop guard reason");
    assert!(line_contains_normalized(
        reason,
        "Review gate: reviewer approve task_2 before execution continues"
    ));
    assert!(line_contains_normalized(reason, "Task: task_2 (Build API)"));
    assert!(line_contains_normalized(
        reason,
        "If approved, set `- Review-Status: approved`"
    ));
}

#[test]
fn shell_stop_guard_fallback_without_tasks_surfaces_decompose_next_action() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "INITIALIZE",
            "goal": "拆任务"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    let output = Command::new("bash")
        .current_dir(temp.path())
        .arg(bash_script_arg(
            &repo_root.join("scripts/fusion-stop-guard.sh"),
        ))
        .env("FUSION_BRIDGE_DISABLE", "1")
        .output()
        .expect("run shell stop-guard fallback");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        value.get("decision").and_then(|v| v.as_str()),
        Some("block")
    );
    assert_eq!(
        value.get("systemMessage").and_then(|v| v.as_str()),
        Some("🔄 Fusion | Phase: INITIALIZE | Next: Create task plan and run the DECOMPOSE phase")
    );
    let reason = value
        .get("reason")
        .and_then(|v| v.as_str())
        .expect("stop guard reason");
    assert!(line_contains_normalized(
        reason,
        "Next action: Create task plan and run the DECOMPOSE phase"
    ));
}

#[test]
fn hook_stop_guard_runtime_disabled_marks_completed_on_allow() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "完成收尾"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [COMPLETED]\n").expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:\n  enabled: false\nsafe_backlog:\n  enabled: false\n",
    )
    .expect("config");
    fs::write(fusion.join("progress.md"), "| t | EXECUTE | e | OK | d |\n").expect("progress");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("hook")
        .arg("stop-guard")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run stop-guard");

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        value.get("decision").and_then(|v| v.as_str()),
        Some("allow")
    );

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("completed")
    );
    assert_eq!(
        sessions.get("current_phase").and_then(|v| v.as_str()),
        Some("VERIFY")
    );
    assert!(!fusion.join("events.jsonl").exists());

    let progress = fs::read_to_string(fusion.join("progress.md")).expect("progress read");
    assert!(line_contains_normalized(
        &progress,
        "| COMPLETE | Workflow finished | OK | All tasks done |"
    ));
}

#[test]
fn hook_stop_guard_runtime_enabled_persists_all_tasks_done_event() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "完成执行"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [COMPLETED]\n").expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:\n  enabled: true\nsafe_backlog:\n  enabled: false\n",
    )
    .expect("config");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("hook")
        .arg("stop-guard")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run stop-guard");

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        value.get("decision").and_then(|v| v.as_str()),
        Some("allow")
    );
    assert_eq!(
        value.get("phase_corrected").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert!(value
        .get("events_dispatched")
        .and_then(|v| v.as_array())
        .map(|arr| arr
            .iter()
            .any(|item| item.as_str() == Some("ALL_TASKS_DONE")))
        .unwrap_or(false));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("in_progress")
    );
    assert_eq!(
        sessions.get("current_phase").and_then(|v| v.as_str()),
        Some("VERIFY")
    );
    assert_eq!(
        sessions
            .get("_runtime")
            .and_then(|v| v.get("state"))
            .and_then(|v| v.as_str()),
        Some("VERIFY")
    );

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events read");
    assert!(line_contains_normalized(&events, "ALL_TASKS_DONE"));
}

#[test]
fn hook_stop_guard_runtime_enabled_injects_safe_backlog_on_task_exhausted() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "持续推进"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(fusion.join("task_plan.md"), "### Task 1: A [COMPLETED]\n").expect("task plan");
    fs::write(
        fusion.join("config.yaml"),
        "runtime:\n  enabled: true\nsafe_backlog:\n  enabled: true\n  inject_on_task_exhausted: true\n  max_tasks_per_run: 1\n  allowed_categories: documentation\n",
    )
    .expect("config");
    fs::write(temp.path().join("README.md"), "# Demo\n").expect("readme");

    let output = Command::cargo_bin("fusion-bridge")
        .expect("binary")
        .current_dir(temp.path())
        .arg("hook")
        .arg("stop-guard")
        .arg("--fusion-dir")
        .arg(".fusion")
        .output()
        .expect("run stop-guard");

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(
        value.get("decision").and_then(|v| v.as_str()),
        Some("block")
    );
    assert_eq!(
        value.get("systemMessage").and_then(|v| v.as_str()),
        Some("🔄 Fusion (safe_backlog injected) | Phase: VERIFY | Remaining: 1")
    );

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("task plan read");
    assert!(line_contains_normalized(&task_plan, "[SAFE_BACKLOG]"));

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events read");
    assert!(line_contains_normalized(&events, "ALL_TASKS_DONE"));
    assert!(line_contains_normalized(&events, "SAFE_BACKLOG_INJECTED"));
}

#[test]
fn run_continues_until_tasks_completed() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "持续执行"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let _wrapper = pending_to_completed_wrapper(&bin_dir, &fusion.join("task_plan.md"));
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("run")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", &path);

    let output = cmd.output().expect("run workflow");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Continue task: A [PENDING]"
    ));
    assert!(!line_contains_normalized(
        &stdout,
        "Next action: Resume the workflow loop from the saved checkpoint"
    ));
    assert!(line_contains_normalized(&stdout, "Loop completed"));

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("read task plan");
    assert!(line_contains_normalized(&task_plan, "[COMPLETED]"));
}

#[test]
fn run_without_task_plan_surfaces_decompose_next_action() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "INITIALIZE",
            "goal": "decompose"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        &format!(
            "#!/bin/bash\nset -euo pipefail\ncat > \"{}\" <<'EOF'\n### Task 1: Bootstrap [COMPLETED]\nEOF\necho ok\n",
            fusion.join("task_plan.md").display()
        ),
        &format!(
            "@echo off\r\nset \"PLAN={}\"\r\npowershell -NoProfile -Command \"$p=$env:PLAN; Set-Content -Path $p -Value '### Task 1: Bootstrap [COMPLETED]'\"\r\necho ok\r\n",
            fusion.join("task_plan.md").display()
        ),
    );
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("run")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", &path);

    let output = cmd.output().expect("run workflow");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Create task plan and run the DECOMPOSE phase"
    ));
    assert!(line_contains_normalized(&stdout, "Loop completed"));
}

#[test]
fn run_review_gate_surfaces_reviewer_next_action() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("config.yaml"),
        "agents:\n  enabled: true\n  mode: role_handoff\n  review_policy: high_risk\nsafe_backlog:\n  enabled: false\n  inject_on_task_exhausted: false\n",
    )
    .expect("config");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "review gate"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let _wrapper = approve_pending_review_wrapper(&bin_dir, &fusion.join("task_plan.md"));
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("run")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", &path);

    let output = cmd.output().expect("run workflow");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Next action: reviewer approve task_2 before execution continues"
    ));
    assert!(line_contains_normalized(&stdout, "Loop completed"));
}

#[test]
fn run_exits_when_no_progress_hits_limit() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "持续执行"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let _wrapper = write_mock_executable(
        &bin_dir,
        "codeagent-wrapper",
        "#!/bin/bash\nset -euo pipefail\necho \"noop\"\n",
        "@echo off\r\necho noop\r\n",
    );
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("run")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("2")
        .arg("--max-no-progress-rounds")
        .arg("2")
        .arg("--initial-backoff-ms")
        .arg("1")
        .arg("--max-backoff-ms")
        .arg("2")
        .env("PATH", &path);

    assert_stderr_contains_normalized(
        &mut cmd,
        "run workflow until no-progress limit",
        Some(2),
        "No progress rounds limit reached",
    );
}

#[test]
fn resume_from_paused_runs_until_tasks_completed() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "paused",
            "current_phase": "EXECUTE",
            "goal": "恢复执行"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let _wrapper = pending_to_completed_wrapper(&bin_dir, &fusion.join("task_plan.md"));
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("resume")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", &path);

    let output = cmd.output().expect("run resume");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Current state: paused @ EXECUTE"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Continue task: A [PENDING]"
    ));
    assert!(line_contains_normalized(&stdout, "Loop completed"));

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("read task plan");
    assert!(line_contains_normalized(&task_plan, "[COMPLETED]"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("completed")
    );
}

#[test]
fn resume_from_paused_review_gate_surfaces_reviewer_next_action() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("config.yaml"),
        "agents:\n  enabled: true\n  mode: role_handoff\n  review_policy: high_risk\nsafe_backlog:\n  enabled: false\n  inject_on_task_exhausted: false\n",
    )
    .expect("config");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "paused",
            "current_phase": "EXECUTE",
            "goal": "review gate"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let _wrapper = approve_pending_review_wrapper(&bin_dir, &fusion.join("task_plan.md"));
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("resume")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", &path);

    let output = cmd.output().expect("run resume");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Current state: paused @ EXECUTE"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: reviewer approve task_2 before execution continues"
    ));
    assert!(line_contains_normalized(&stdout, "Loop completed"));
}

#[test]
fn resume_rejects_not_started_workflow() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "not_started",
            "current_phase": null,
            "goal": null
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("resume")
        .arg("--fusion-dir")
        .arg(".fusion");

    assert_stderr_contains_normalized(
        &mut cmd,
        "run resume on not-started workflow",
        Some(1),
        "Cannot resume workflow with status: not_started",
    );
}

#[test]
fn resume_rejects_cancelled_workflow() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "cancelled",
            "current_phase": "EXECUTE",
            "goal": "已取消"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("resume")
        .arg("--fusion-dir")
        .arg(".fusion");

    assert_stderr_contains_normalized(
        &mut cmd,
        "run resume on cancelled workflow",
        Some(1),
        "Workflow was cancelled. Start a new workflow with",
    );
}

#[test]
fn resume_completed_prints_current_state_and_noop_action() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    fs::create_dir_all(&fusion).expect("create fusion");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "completed",
            "current_phase": "DELIVER",
            "goal": "已完成"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [COMPLETED]\n").expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("resume")
        .arg("--fusion-dir")
        .arg(".fusion");

    let output = cmd.output().expect("run resume");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Current state: completed @ DELIVER"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: No resume needed"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Workflow already completed. Nothing to resume."
    ));
}

#[test]
fn resume_in_progress_review_gate_surfaces_reviewer_next_action() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("config.yaml"),
        "agents:\n  enabled: true\n  mode: role_handoff\n  review_policy: high_risk\nsafe_backlog:\n  enabled: false\n  inject_on_task_exhausted: false\n",
    )
    .expect("config");
    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "review gate"
        }))
        .expect("json"),
    )
    .expect("write sessions");
    fs::write(
        fusion.join("task_plan.md"),
        "### Task 1: Plan API [COMPLETED]\n- Type: research\n- Owner: planner\n- Review-Status: none\n### Task 2: Build API [IN_PROGRESS]\n- Type: implementation\n- Owner: coder\n- Risk: high\n- Review: required\n- Review-Status: pending\n",
    )
    .expect("task plan");

    let _wrapper = approve_pending_review_wrapper(&bin_dir, &fusion.join("task_plan.md"));
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("resume")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", &path);

    let output = cmd.output().expect("run resume");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Current state: in_progress @ EXECUTE"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: reviewer approve task_2 before execution continues"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Workflow already in progress, continuing"
    ));
    assert!(line_contains_normalized(&stdout, "Loop completed"));
}

#[test]
fn resume_from_stuck_sets_in_progress_and_runs() {
    let temp = tempdir().expect("tempdir");
    let fusion = temp.path().join(".fusion");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&fusion).expect("create fusion");
    fs::create_dir_all(&bin_dir).expect("create bin");

    fs::write(
        fusion.join("sessions.json"),
        serde_json::to_string_pretty(&json!({
            "status": "stuck",
            "current_phase": "EXECUTE",
            "goal": "恢复卡住流程"
        }))
        .expect("json"),
    )
    .expect("write sessions");

    fs::write(fusion.join("task_plan.md"), "### Task 1: A [PENDING]\n").expect("task plan");

    let _wrapper = pending_to_completed_wrapper(&bin_dir, &fusion.join("task_plan.md"));
    let path = prepend_path(&bin_dir);

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("resume")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", &path);

    let output = cmd.output().expect("run resume");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(line_contains_normalized(
        &stdout,
        "Current state: stuck @ EXECUTE"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Next action: Continue task: A [PENDING]"
    ));
    assert!(!line_contains_normalized(
        &stdout,
        "Next action: Reset status to in_progress and continue the workflow loop"
    ));
    assert!(line_contains_normalized(
        &stdout,
        "Workflow is stuck. Please investigate"
    ));

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("read task plan");
    assert!(line_contains_normalized(&task_plan, "[COMPLETED]"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("completed")
    );
}
