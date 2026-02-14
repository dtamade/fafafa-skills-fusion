#![allow(deprecated)]

use assert_cmd::prelude::*;
use predicates::str::contains;
use serde_json::json;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::tempdir;

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

    cmd.assert()
        .code(127)
        .stderr(contains("Missing dependency: codeagent-wrapper"));

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

    let wrapper = bin_dir.join("codeagent-wrapper");
    fs::write(
        &wrapper,
        "#!/bin/bash\nif [ \"$2\" = \"codex\" ]; then exit 1; fi\necho \"mock backend:$2\"\necho \"SESSION_ID: 654321\"\n",
    )
    .expect("write wrapper");
    let mut perms = fs::metadata(&wrapper).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&wrapper, perms).expect("chmod");

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("codeagent")
        .arg("EXECUTE")
        .arg("--fusion-dir")
        .arg(".fusion")
        .env("PATH", path);

    cmd.assert()
        .success()
        .stdout(contains("mock backend:claude"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("claude_session").and_then(|v| v.as_str()),
        Some("654321")
    );
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

    cmd.assert()
        .success()
        .stdout(contains("## Dependency Report"))
        .stdout(contains("missing: codeagent-wrapper"));
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

    cmd.assert()
        .success()
        .stdout(contains("Initialized .fusion directory"));

    let fusion = temp.path().join(".fusion");
    assert!(fusion.join("task_plan.md").exists());
    assert!(fusion.join("progress.md").exists());
    assert!(fusion.join("findings.md").exists());
    assert!(fusion.join("sessions.json").exists());
    assert!(fusion.join("config.yaml").exists());
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

    cmd.assert()
        .success()
        .stdout(contains("Workflow initialized"));

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
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n",
    )
    .expect("task plan");

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("hook")
        .arg("pretool")
        .arg("--fusion-dir")
        .arg(".fusion");

    cmd.assert()
        .success()
        .stdout(contains("[fusion] Goal:"))
        .stdout(contains("[fusion] Progress:"));
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
        "safe_backlog:\n  enabled: true\n  trigger_no_progress_rounds: 2\n  max_tasks_per_run: 1\n  allowed_categories: documentation\n",
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
    second
        .assert()
        .success()
        .stdout(contains("Safe backlog injected"));

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("task plan read");
    assert!(task_plan.contains("[SAFE_BACKLOG]"));

    let events = fs::read_to_string(fusion.join("events.jsonl")).expect("events read");
    assert!(events.contains("SAFE_BACKLOG_INJECTED"));
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

    let wrapper = bin_dir.join("codeagent-wrapper");
    fs::write(
        &wrapper,
        format!(
            "#!/bin/bash\nset -euo pipefail\n\nplan=\"{}\"\nif grep -q '\\[PENDING\\]' \"$plan\"; then\n  sed -i 's/\\[PENDING\\]/[COMPLETED]/' \"$plan\"\nfi\necho \"ok\"\n",
            fusion.join("task_plan.md").display()
        ),
    )
    .expect("write wrapper");
    let mut perms = fs::metadata(&wrapper).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&wrapper, perms).expect("chmod");

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("run")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", path);

    cmd.assert().success().stdout(contains("Loop completed"));

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("read task plan");
    assert!(task_plan.contains("[COMPLETED]"));
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

    let wrapper = bin_dir.join("codeagent-wrapper");
    fs::write(&wrapper, "#!/bin/bash\nset -euo pipefail\necho \"noop\"\n").expect("write wrapper");
    let mut perms = fs::metadata(&wrapper).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&wrapper, perms).expect("chmod");

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

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
        .env("PATH", path);

    cmd.assert()
        .code(2)
        .stderr(contains("No progress rounds limit reached"));
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

    let wrapper = bin_dir.join("codeagent-wrapper");
    fs::write(
        &wrapper,
        format!(
            "#!/bin/bash\nset -euo pipefail\n\nplan=\"{}\"\nif grep -q '\\[PENDING\\]' \"$plan\"; then\n  sed -i 's/\\[PENDING\\]/[COMPLETED]/' \"$plan\"\nfi\necho \"ok\"\n",
            fusion.join("task_plan.md").display()
        ),
    )
    .expect("write wrapper");
    let mut perms = fs::metadata(&wrapper).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&wrapper, perms).expect("chmod");

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("resume")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", path);

    cmd.assert().success().stdout(contains("Loop completed"));

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("read task plan");
    assert!(task_plan.contains("[COMPLETED]"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("in_progress")
    );
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

    cmd.assert()
        .code(1)
        .stderr(contains("Cannot resume workflow with status: not_started"));
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

    cmd.assert().code(1).stderr(contains(
        "Workflow was cancelled. Start a new workflow with",
    ));
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

    let wrapper = bin_dir.join("codeagent-wrapper");
    fs::write(
        &wrapper,
        format!(
            "#!/bin/bash\nset -euo pipefail\n\nplan=\"{}\"\nif grep -q '\\[PENDING\\]' \"$plan\"; then\n  sed -i 's/\\[PENDING\\]/[COMPLETED]/' \"$plan\"\nfi\necho \"ok\"\n",
            fusion.join("task_plan.md").display()
        ),
    )
    .expect("write wrapper");
    let mut perms = fs::metadata(&wrapper).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&wrapper, perms).expect("chmod");

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let mut cmd = Command::cargo_bin("fusion-bridge").expect("binary");
    cmd.current_dir(temp.path())
        .arg("resume")
        .arg("--fusion-dir")
        .arg(".fusion")
        .arg("--max-iterations")
        .arg("4")
        .env("PATH", path);

    cmd.assert()
        .success()
        .stdout(contains("Workflow is stuck. Please investigate"));

    let task_plan = fs::read_to_string(fusion.join("task_plan.md")).expect("read task plan");
    assert!(task_plan.contains("[COMPLETED]"));

    let sessions: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fusion.join("sessions.json")).expect("sessions"))
            .expect("parse sessions");
    assert_eq!(
        sessions.get("status").and_then(|v| v.as_str()),
        Some("in_progress")
    );
}
