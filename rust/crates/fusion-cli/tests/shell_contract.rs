use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::tempdir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

fn release_bridge_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fusion-bridge"))
}

fn run_bash_script(script: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new("bash");
    command.arg(script);
    command.args(args);
    command.current_dir(repo_root());
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("run bash script")
}

fn run_bash_commands(cwd: &Path, commands: &str, envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new("bash");
    command.arg("-lc").arg(commands).current_dir(cwd);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("run bash commands")
}

fn combined_output(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr)
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn line_contains_normalized(haystack: &str, needle: &str) -> bool {
    haystack
        .lines()
        .any(|line| normalize_whitespace(line).contains(&normalize_whitespace(needle)))
}

fn line_equals_normalized(haystack: &str, needle: &str) -> bool {
    haystack
        .lines()
        .any(|line| normalize_whitespace(line) == normalize_whitespace(needle))
}

fn retired_skip_flag() -> String {
    ["--skip-", "py", "thon"].concat()
}

fn retired_test_command() -> String {
    [["py", "test"].concat(), " -q".to_string()].concat()
}

#[cfg(unix)]
fn create_dir_alias(target: &Path, alias: &Path) {
    std::os::unix::fs::symlink(target, alias).expect("create directory alias");
}

#[test]
fn hook_adapter_avoids_nonportable_case_fallthrough_tokens() {
    let adapter = fs::read_to_string(repo_root().join("scripts/lib/fusion-hook-adapter.sh"))
        .expect("read fusion-hook-adapter.sh");

    for token in [";;&", ";&"] {
        assert!(
            !adapter.lines().any(|line| line.trim() == token),
            "fusion-hook-adapter.sh must avoid non-portable case token `{token}`"
        );
    }
}

#[test]
fn ci_cross_platform_smoke_script_help_and_end_to_end() {
    let root = repo_root();
    let script = root.join("scripts/ci-cross-platform-smoke.sh");
    let bridge = release_bridge_bin();
    let bridge_str = bridge.to_string_lossy().into_owned();
    let artifacts = tempdir().expect("artifacts tempdir");
    let artifacts_str = artifacts.path().to_string_lossy().into_owned();

    let help = run_bash_script(&script, &["--help"], &[]);
    assert!(help.status.success());
    let help_output = combined_output(&help);
    assert!(line_contains_normalized(
        &help_output,
        "Usage: ci-cross-platform-smoke.sh"
    ));
    assert!(line_contains_normalized(
        &help_output,
        "fusion-hook-selfcheck.sh --json --quick --fix"
    ));
    assert!(line_contains_normalized(
        &help_output,
        "--artifacts-dir <path>"
    ));
    assert!(line_contains_normalized(
        &help_output,
        "cross-platform-smoke-summary.json"
    ));

    let run = run_bash_script(
        &script,
        &[
            "--artifacts-dir",
            &artifacts_str,
            "--platform-label",
            "contract-test",
        ],
        &[("FUSION_BRIDGE_BIN", &bridge_str)],
    );
    assert!(
        run.status.success(),
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let output = combined_output(&run);
    assert!(line_contains_normalized(
        &output,
        "[ci-cross-platform-smoke] running fusion-start.sh"
    ));
    assert!(line_contains_normalized(
        &output,
        "[ci-cross-platform-smoke] running fusion-status.sh --json"
    ));
    assert!(line_contains_normalized(
        &output,
        "[ci-cross-platform-smoke] running fusion-hook-selfcheck.sh --json --quick --fix"
    ));
    assert!(line_contains_normalized(
        &output,
        "[ci-cross-platform-smoke] shell smoke passed"
    ));
    let summary_path = artifacts.path().join("cross-platform-smoke-summary.json");
    assert!(summary_path.is_file());
    let summary: Value = serde_json::from_str(
        &fs::read_to_string(&summary_path).expect("read cross-platform smoke summary"),
    )
    .expect("parse cross-platform smoke summary");
    assert_eq!(summary["schema_version"].as_str(), Some("v1"));
    assert_eq!(summary["result"].as_str(), Some("ok"));
    assert_eq!(summary["platform_label"].as_str(), Some("contract-test"));
    assert_eq!(summary["commands_count"].as_u64(), Some(5));
    assert_eq!(summary["completed_commands_count"].as_u64(), Some(5));
    assert_eq!(summary["runtime_engine"].as_str(), Some("rust"));
    assert_eq!(
        summary["selfcheck_contract_regression_skipped"].as_bool(),
        Some(true)
    );
}

#[cfg(unix)]
#[test]
fn ci_cross_platform_smoke_accepts_canonicalized_selfcheck_project_root() {
    let root = repo_root();
    let script = root.join("scripts/ci-cross-platform-smoke.sh");
    let bridge = release_bridge_bin();
    let bridge_str = bridge.to_string_lossy().into_owned();
    let temp = tempdir().expect("tempdir");
    let real_tmpdir = temp.path().join("real-tmpdir");
    let alias_tmpdir = temp.path().join("tmpdir-alias");
    let artifacts = temp.path().join("artifacts");
    let artifacts_str = artifacts.to_string_lossy().into_owned();

    fs::create_dir_all(&real_tmpdir).expect("create real tmpdir");
    create_dir_alias(&real_tmpdir, &alias_tmpdir);

    let alias_tmpdir_str = alias_tmpdir.to_string_lossy().into_owned();
    let run = run_bash_script(
        &script,
        &[
            "--artifacts-dir",
            &artifacts_str,
            "--platform-label",
            "canonical-path-contract",
        ],
        &[
            ("FUSION_BRIDGE_BIN", &bridge_str),
            ("TMPDIR", &alias_tmpdir_str),
        ],
    );
    assert!(
        run.status.success(),
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let summary_path = artifacts.join("cross-platform-smoke-summary.json");
    assert!(summary_path.is_file());
    let summary: Value = serde_json::from_str(
        &fs::read_to_string(&summary_path).expect("read cross-platform smoke summary"),
    )
    .expect("parse cross-platform smoke summary");
    assert_eq!(summary["result"].as_str(), Some("ok"));
    assert_eq!(summary["selfcheck_result"].as_str(), Some("ok"));
}

#[test]
fn ci_machine_mode_smoke_script_help_and_writes_artifacts() {
    let root = repo_root();
    let script = root.join("scripts/ci-machine-mode-smoke.sh");
    let bridge = release_bridge_bin();
    let bridge_str = bridge.to_string_lossy().into_owned();
    let artifacts = tempdir().expect("artifacts tempdir");
    let artifacts_str = artifacts.path().to_string_lossy().into_owned();

    let help = run_bash_script(&script, &["--help"], &[]);
    assert!(help.status.success());
    let help_output = combined_output(&help);
    assert!(line_contains_normalized(
        &help_output,
        "Usage: ci-machine-mode-smoke.sh"
    ));
    assert!(line_contains_normalized(
        &help_output,
        "release-audit-dry-run.json"
    ));

    let run = run_bash_script(
        &script,
        &["--artifacts-dir", &artifacts_str],
        &[("FUSION_BRIDGE_BIN", &bridge_str)],
    );
    assert!(
        run.status.success(),
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let output = combined_output(&run);
    assert!(line_contains_normalized(
        &output,
        "[ci-machine-mode-smoke] machine-mode smoke passed"
    ));
    assert!(artifacts
        .path()
        .join("release-audit-dry-run.json")
        .is_file());
    assert!(artifacts.path().join("runner-suites.json").is_file());
    assert!(artifacts.path().join("runner-contract.json").is_file());
}

#[test]
fn ci_machine_json_smoke_script_validates_payloads() {
    let root = repo_root();
    let script = root.join("scripts/ci-machine-json-smoke.sh");
    let temp = tempdir().expect("tempdir");
    let release_audit = temp.path().join("release-audit.json");
    let runner_suites = temp.path().join("runner-suites.json");
    let runner_contract = temp.path().join("runner-contract.json");

    let help = run_bash_script(&script, &["--help"], &[]);
    assert!(help.status.success());
    let help_output = combined_output(&help);
    assert!(line_contains_normalized(
        &help_output,
        "Usage: ci-machine-json-smoke.sh"
    ));
    assert!(line_contains_normalized(
        &help_output,
        "runner-contract.json"
    ));

    fs::write(
        &release_audit,
        serde_json::to_string(&json!({
            "schema_version": "v1",
            "result": "ok",
            "flags": {"json": true, "fast": true, "skip_rust": true},
            "commands": ["bash -n scripts/*.sh"],
            "steps_executed": 1,
            "failed_commands": [],
            "failed_commands_count": 0,
            "error_step_count": 0,
            "success_steps_count": 1,
            "commands_count": 1,
            "step_rate_basis": 1,
            "command_rate_basis": 1,
            "success_rate": 1.0,
            "failed_rate": 0.0,
            "success_command_rate": 1.0,
            "failed_command_rate": 0.0
        }))
        .expect("release audit json"),
    )
    .expect("write release audit");
    fs::write(
        &runner_suites,
        serde_json::to_string(&json!({
            "result": "ok",
            "default_suite": "all",
            "suites": ["contract", "all"]
        }))
        .expect("runner suites json"),
    )
    .expect("write runner suites");
    fs::write(
        &runner_contract,
        serde_json::to_string(&json!({
            "schema_version": "v1",
            "result": "ok",
            "suite": "contract",
            "scenario_results": [],
            "longest_scenario": null,
            "fastest_scenario": null,
            "scenario_count_by_result": {},
            "duration_stats": {"avg_ms": 1},
            "failed_rate": 0.0,
            "success_rate": 1.0,
            "success_count": 1,
            "failure_count": 0,
            "total_scenarios": 1,
            "rate_basis": 1
        }))
        .expect("runner contract json"),
    )
    .expect("write runner contract");

    let release_audit_str = release_audit.to_string_lossy().into_owned();
    let runner_suites_str = runner_suites.to_string_lossy().into_owned();
    let runner_contract_str = runner_contract.to_string_lossy().into_owned();
    let run = run_bash_script(
        &script,
        &[&release_audit_str, &runner_suites_str, &runner_contract_str],
        &[],
    );
    assert!(
        run.status.success(),
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let output = combined_output(&run);
    assert!(line_contains_normalized(
        &output,
        "[ci-machine-json-smoke] machine JSON smoke passed"
    ));

    let mut broken: Value =
        serde_json::from_slice(&fs::read(&runner_contract).expect("read runner contract"))
            .expect("parse runner contract");
    broken["rate_basis"] = json!(2);
    fs::write(
        &runner_contract,
        serde_json::to_string(&broken).expect("broken runner contract"),
    )
    .expect("rewrite runner contract");

    let failure = run_bash_script(
        &script,
        &[&release_audit_str, &runner_suites_str, &runner_contract_str],
        &[],
    );
    assert!(!failure.status.success());
    let failure_output = combined_output(&failure);
    assert!(line_contains_normalized(
        &failure_output,
        "runner-contract.json rate_basis mismatch total_scenarios"
    ));
}

#[test]
fn ci_cross_platform_json_smoke_script_validates_payloads() {
    let root = repo_root();
    let script = root.join("scripts/ci-cross-platform-json-smoke.sh");
    let temp = tempdir().expect("tempdir");
    let summary = temp.path().join("cross-platform-smoke-summary.json");

    let help = run_bash_script(&script, &["--help"], &[]);
    assert!(help.status.success());
    let help_output = combined_output(&help);
    assert!(line_contains_normalized(
        &help_output,
        "Usage: ci-cross-platform-json-smoke.sh"
    ));
    assert!(line_contains_normalized(
        &help_output,
        "cross-platform-smoke-summary.json"
    ));

    fs::write(
        &summary,
        serde_json::to_string(&json!({
            "schema_version": "v1",
            "result": "ok",
            "platform_label": "contract-test",
            "commands": [
                "fusion-start.sh",
                "fusion-status.sh --json",
                "fusion-achievements.sh --leaderboard-only",
                "fusion-hook-selfcheck.sh --json --quick --fix",
                "fusion-catchup.sh"
            ],
            "commands_count": 5,
            "completed_commands": [
                "fusion-start.sh",
                "fusion-status.sh --json",
                "fusion-achievements.sh --leaderboard-only",
                "fusion-hook-selfcheck.sh --json --quick --fix",
                "fusion-catchup.sh"
            ],
            "completed_commands_count": 5,
            "runtime_engine": "rust",
            "selfcheck_result": "ok",
            "selfcheck_contract_regression_skipped": true,
            "project_artifacts": [
                ".fusion/config.yaml",
                ".fusion/sessions.json",
                ".claude/settings.local.json"
            ],
            "failure_reason": null
        }))
        .expect("cross-platform summary json"),
    )
    .expect("write cross-platform summary");

    let summary_str = summary.to_string_lossy().into_owned();
    let run = run_bash_script(&script, &[&summary_str], &[]);
    assert!(
        run.status.success(),
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let output = combined_output(&run);
    assert!(line_contains_normalized(
        &output,
        "[ci-cross-platform-json-smoke] cross-platform JSON smoke passed"
    ));
}

#[test]
fn ci_remote_evidence_script_reports_remote_promotion_state() {
    let root = repo_root();
    let script = root.join("scripts/ci-remote-evidence.sh");
    let temp = tempdir().expect("tempdir");
    let fake_gh = temp.path().join("gh");
    let artifacts = temp.path().join("artifacts");
    let fake_gh_body = r#"#!/usr/bin/env bash
set -euo pipefail

mode="${FAKE_GH_MODE:-success}"

if [[ "$1" != "api" ]]; then
  echo "unexpected gh command: $*" >&2
  exit 1
fi

case "$2" in
  repos/example/repo/actions/workflows)
    if [[ "$mode" == "missing-workflow" ]]; then
      printf '%s\n' '{"workflows":[]}'
    else
      printf '%s\n' '{"workflows":[{"id":123,"path":".github/workflows/ci-contract-gates.yml"}]}'
    fi
    ;;
  repos/example/repo/actions/workflows/123/runs?branch=main\&per_page=10)
    printf '%s\n' '{"workflow_runs":[{"id":456,"html_url":"https://example.invalid/runs/456","status":"completed","conclusion":"success","head_sha":"abc123","created_at":"2026-03-24T10:00:00Z","updated_at":"2026-03-24T10:05:00Z"}]}'
    ;;
  repos/example/repo/actions/runs/456/jobs)
    printf '%s\n' '{"jobs":[{"name":"contract-gates","status":"completed","conclusion":"success"},{"name":"cross-platform-smoke-macos","status":"completed","conclusion":"success"},{"name":"cross-platform-smoke-windows","status":"completed","conclusion":"success"}]}'
    ;;
  *)
    echo "unexpected gh api path: $2" >&2
    exit 1
    ;;
esac
"#;
    fs::write(&fake_gh, fake_gh_body).expect("write fake gh");
    let chmod = run_bash_commands(
        temp.path(),
        &format!("chmod +x {}", fake_gh.to_string_lossy()),
        &[],
    );
    assert!(chmod.status.success());

    let help = run_bash_script(&script, &["--help"], &[]);
    assert!(help.status.success());
    let help_output = combined_output(&help);
    assert!(line_contains_normalized(
        &help_output,
        "Usage: ci-remote-evidence.sh"
    ));
    assert!(line_contains_normalized(
        &help_output,
        "--workflow-path <path>"
    ));

    let fake_gh_str = fake_gh.to_string_lossy().into_owned();
    let artifacts_str = artifacts.to_string_lossy().into_owned();
    let run = run_bash_script(
        &script,
        &[
            "--repo",
            "example/repo",
            "--branch",
            "main",
            "--json",
            "--artifacts-dir",
            &artifacts_str,
        ],
        &[("GH_BIN", &fake_gh_str)],
    );
    assert!(
        run.status.success(),
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let output = String::from_utf8_lossy(&run.stdout).to_string();
    let payload: Value = serde_json::from_str(&output).expect("parse remote evidence");
    let required_jobs: Vec<&str> = payload["required_jobs"]
        .as_array()
        .expect("required_jobs array")
        .iter()
        .map(|item| item.as_str().expect("required job string"))
        .collect();
    assert_eq!(payload["schema_version"].as_str(), Some("v1"));
    assert_eq!(payload["result"].as_str(), Some("ok"));
    assert_eq!(payload["promotion_ready"].as_bool(), Some(true));
    assert_eq!(payload["run_id"].as_u64(), Some(456));
    assert_eq!(payload["run_conclusion"].as_str(), Some("success"));
    assert_eq!(
        required_jobs,
        vec![
            "contract-gates",
            "cross-platform-smoke-macos",
            "cross-platform-smoke-windows"
        ]
    );
    assert_eq!(payload["missing_jobs"].as_array().map(Vec::len), Some(0));
    assert_eq!(payload["failed_jobs"].as_array().map(Vec::len), Some(0));
    assert!(artifacts.join("remote-ci-evidence-summary.json").is_file());

    let missing = run_bash_script(
        &script,
        &["--repo", "example/repo", "--branch", "main", "--json"],
        &[
            ("GH_BIN", &fake_gh_str),
            ("FAKE_GH_MODE", "missing-workflow"),
        ],
    );
    assert!(!missing.status.success());
    let missing_output = String::from_utf8_lossy(&missing.stdout).to_string();
    let missing_payload: Value =
        serde_json::from_str(&missing_output).expect("parse missing-workflow payload");
    assert_eq!(missing_payload["result"].as_str(), Some("error"));
    assert_eq!(
        missing_payload["reason"].as_str(),
        Some("workflow_not_found")
    );
}

#[test]
fn release_contract_audit_shell_wrapper_help_and_dry_run_json() {
    let root = repo_root();
    let script = root.join("scripts/release-contract-audit.sh");
    let bridge = release_bridge_bin();
    let bridge_str = bridge.to_string_lossy().into_owned();

    let help = run_bash_script(&script, &["--help"], &[]);
    assert!(help.status.success());
    let help_output = combined_output(&help);
    assert!(line_contains_normalized(
        &help_output,
        "Usage: release-contract-audit.sh"
    ));
    assert!(line_contains_normalized(&help_output, "5) rust test gate"));
    assert!(!line_contains_normalized(
        &help_output,
        &retired_skip_flag()
    ));

    let run = run_bash_script(
        &script,
        &["--dry-run", "--json", "--fast", "--skip-rust"],
        &[("FUSION_BRIDGE_BIN", &bridge_str)],
    );
    assert!(
        run.status.success(),
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let payload: Value = serde_json::from_slice(&run.stdout).expect("parse audit json");
    let commands: Vec<&str> = payload["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .map(|item| item.as_str().expect("command string"))
        .collect();

    assert_eq!(payload["result"].as_str(), Some("ok"));
    assert_eq!(payload["dry_run"].as_bool(), Some(true));
    assert_eq!(
        commands,
        vec![
            "bash -n scripts/*.sh",
            "bash scripts/ci-machine-mode-smoke.sh",
        ]
    );
    let retired_test = retired_test_command();
    assert!(!commands
        .iter()
        .any(|command| command.contains(&retired_test)));
}

#[test]
fn fusion_bridge_shell_helpers_use_bridge_and_release_resolution() {
    let root = repo_root();
    let bridge_lib = root.join("scripts/lib/fusion-bridge.sh");
    let temp = tempdir().expect("tempdir");
    let fusion_dir = temp.path().join(".fusion");
    fs::create_dir_all(&fusion_dir).expect("create fusion");

    fs::write(
        fusion_dir.join("config.yaml"),
        "runtime:\n  enabled: true\n  compat_mode: false\n  engine: legacy\n",
    )
    .expect("write fallback config");

    let fallback_commands = format!(
        "set -euo pipefail; source \"{}\"; FUSION_DIR=\"{}\"; printf 'enabled=%s\\nengine=%s\\ncompat=%s\\n' \"$(fusion_runtime_enabled \"$FUSION_DIR\" && echo true || echo false)\" \"$(fusion_runtime_engine \"$FUSION_DIR\")\" \"$(fusion_runtime_compat_mode \"$FUSION_DIR\")\"",
        bridge_lib.display(),
        fusion_dir.display()
    );
    let fallback = run_bash_commands(temp.path(), &fallback_commands, &[]);
    assert!(
        fallback.status.success(),
        "{}",
        String::from_utf8_lossy(&fallback.stderr)
    );
    let fallback_stdout = String::from_utf8_lossy(&fallback.stdout);
    assert!(line_equals_normalized(&fallback_stdout, "enabled=true"));
    assert!(line_equals_normalized(&fallback_stdout, "engine=rust"));
    assert!(line_equals_normalized(&fallback_stdout, "compat=false"));

    fs::write(
        fusion_dir.join("config.yaml"),
        "runtime:\n  enabled: false\n  compat_mode: true\n  engine: rust\n",
    )
    .expect("write bridge config");

    let fake_bridge = temp.path().join("fusion-bridge");
    fs::write(
        &fake_bridge,
        "#!/bin/sh\nif [ \"$1\" = \"inspect\" ] && [ \"$2\" = \"runtime-config\" ] && [ \"$6\" = \"enabled\" ]; then\n  echo true\n  exit 0\nfi\nif [ \"$1\" = \"inspect\" ] && [ \"$2\" = \"runtime-config\" ] && [ \"$6\" = \"engine\" ]; then\n  echo legacy\n  exit 0\nfi\nif [ \"$1\" = \"inspect\" ] && [ \"$2\" = \"runtime-config\" ] && [ \"$6\" = \"compat_mode\" ]; then\n  echo false\n  exit 0\nfi\nexit 1\n",
    )
    .expect("write fake bridge");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&fake_bridge)
            .expect("bridge metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_bridge, perms).expect("chmod fake bridge");
    }

    let bridge_path = fake_bridge.to_string_lossy().into_owned();
    let bridge_commands = format!(
        "set -euo pipefail; source \"{}\"; FUSION_DIR=\"{}\"; printf 'enabled=%s\\nengine=%s\\ncompat=%s\\n' \"$(fusion_runtime_enabled \"$FUSION_DIR\" && echo true || echo false)\" \"$(fusion_runtime_engine \"$FUSION_DIR\")\" \"$(fusion_runtime_compat_mode \"$FUSION_DIR\")\"",
        bridge_lib.display(),
        fusion_dir.display()
    );
    let bridge = run_bash_commands(
        temp.path(),
        &bridge_commands,
        &[("FUSION_BRIDGE_BIN", &bridge_path)],
    );
    assert!(
        bridge.status.success(),
        "{}",
        String::from_utf8_lossy(&bridge.stderr)
    );
    let bridge_stdout = String::from_utf8_lossy(&bridge.stdout);
    assert!(line_equals_normalized(&bridge_stdout, "enabled=true"));
    assert!(line_equals_normalized(&bridge_stdout, "engine=rust"));
    assert!(line_equals_normalized(&bridge_stdout, "compat=false"));

    let release_dir = temp.path().join("rust/target/release");
    let debug_dir = temp.path().join("rust/target/debug");
    let script_dir = temp.path().join("scripts");
    fs::create_dir_all(&release_dir).expect("create release dir");
    fs::create_dir_all(&debug_dir).expect("create debug dir");
    fs::create_dir_all(&script_dir).expect("create script dir");

    let debug_bridge = debug_dir.join("fusion-bridge");
    fs::write(&debug_bridge, "#!/bin/sh\necho debug\n").expect("write debug bridge");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&debug_bridge)
            .expect("debug metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&debug_bridge, perms).expect("chmod debug bridge");
    }

    let resolve_without_release = format!(
        "set -euo pipefail; source \"{}\"; resolve_fusion_bridge_bin \"{}\" || true",
        bridge_lib.display(),
        script_dir.display()
    );
    let missing_release = run_bash_commands(temp.path(), &resolve_without_release, &[]);
    assert!(missing_release.status.success());
    assert!(String::from_utf8_lossy(&missing_release.stdout)
        .trim()
        .is_empty());

    let release_bridge = release_dir.join("fusion-bridge");
    fs::write(&release_bridge, "#!/bin/sh\necho release\n").expect("write release bridge");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&release_bridge)
            .expect("release metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&release_bridge, perms).expect("chmod release bridge");
    }

    let resolve_with_release = format!(
        "set -euo pipefail; source \"{}\"; resolve_fusion_bridge_bin \"{}\"",
        bridge_lib.display(),
        script_dir.display()
    );
    let resolved = run_bash_commands(temp.path(), &resolve_with_release, &[]);
    assert!(
        resolved.status.success(),
        "{}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    assert_eq!(
        fs::canonicalize(PathBuf::from(
            String::from_utf8_lossy(&resolved.stdout).trim()
        ))
        .expect("canonicalize resolved bridge"),
        fs::canonicalize(&release_bridge).expect("canonicalize release bridge")
    );
}
