use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::json;
use std::env;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SHELL_SYNTAX: &str = "bash -n scripts/*.sh";
const CI_MACHINE_MODE_SMOKE: &str = "bash scripts/ci-machine-mode-smoke.sh";
const CI_CROSS_PLATFORM_SMOKE: &str = "bash scripts/ci-cross-platform-smoke.sh";
const RUST_CLIPPY: &str =
    "cd rust && cargo clippy --release --workspace --all-targets -- -D warnings";
const RUST_TEST: &str = "cd rust && cargo test --release";
const RUST_FMT: &str = "cd rust && cargo fmt --all -- --check";

#[derive(Serialize)]
struct StepResult {
    status: &'static str,
    duration_ms: u128,
    step: usize,
    started_at_ms: u128,
    finished_at_ms: u128,
    exit_code: i32,
    command: String,
}

pub(crate) fn cmd_audit(
    dry_run: bool,
    json_mode: bool,
    json_pretty: bool,
    fast: bool,
    skip_rust: bool,
) -> Result<()> {
    if json_pretty && !json_mode {
        return Err(anyhow!("--json-pretty requires --json"));
    }

    let commands = build_commands(fast, skip_rust);

    if dry_run {
        if json_mode {
            emit_json(
                &build_payload(
                    "ok",
                    true,
                    json_mode,
                    json_pretty,
                    fast,
                    skip_rust,
                    &commands,
                    &[],
                    0,
                    0,
                    None,
                    None,
                ),
                json_pretty,
            )?;
        } else {
            println!("[release-contract-audit] dry-run command plan");
            for command in &commands {
                println!("{command}");
            }
        }
        return Ok(());
    }

    if !json_mode {
        println!("[release-contract-audit] running release gates");
    }

    let total_start_ms = now_ms();
    let mut step_results = Vec::with_capacity(commands.len());

    for (index, command) in commands.iter().enumerate() {
        let step = index + 1;
        if !json_mode {
            println!("[release-contract-audit] {command}");
        }

        let step_start_ms = now_ms();
        let forced_fail = env::var("FUSION_RELEASE_AUDIT_FORCE_FAIL_STEP")
            .ok()
            .is_some_and(|value| value == step.to_string());

        let exit_code = if forced_fail {
            1
        } else {
            run_shell(command, json_mode)?
        };
        let step_end_ms = now_ms();
        let duration_ms = step_end_ms.saturating_sub(step_start_ms);
        let status = if exit_code == 0 { "ok" } else { "error" };

        step_results.push(StepResult {
            status,
            duration_ms,
            step,
            started_at_ms: step_start_ms,
            finished_at_ms: step_end_ms,
            exit_code,
            command: command.clone(),
        });

        if exit_code != 0 {
            let total_duration_ms = step_end_ms.saturating_sub(total_start_ms);
            if json_mode {
                emit_json(
                    &build_payload(
                        "error",
                        false,
                        json_mode,
                        json_pretty,
                        fast,
                        skip_rust,
                        &commands,
                        &step_results,
                        exit_code,
                        total_duration_ms,
                        Some(step),
                        Some(command.as_str()),
                    ),
                    json_pretty,
                )?;
            } else if forced_fail {
                eprintln!("[release-contract-audit] failed at step {step}: {command} (forced)");
            } else {
                eprintln!(
                    "[release-contract-audit] failed at step {step}: {command} (exit={exit_code})"
                );
            }
            return Err(anyhow!("release contract audit failed"));
        }
    }

    let total_duration_ms = now_ms().saturating_sub(total_start_ms);
    if json_mode {
        emit_json(
            &build_payload(
                "ok",
                false,
                json_mode,
                json_pretty,
                fast,
                skip_rust,
                &commands,
                &step_results,
                0,
                total_duration_ms,
                None,
                None,
            ),
            json_pretty,
        )?;
    } else {
        println!("[release-contract-audit] all gates passed");
    }

    Ok(())
}

fn build_commands(fast: bool, skip_rust: bool) -> Vec<String> {
    let mut commands = vec![SHELL_SYNTAX.to_string(), CI_MACHINE_MODE_SMOKE.to_string()];

    if !fast {
        commands.push(CI_CROSS_PLATFORM_SMOKE.to_string());
    }

    if !skip_rust {
        commands.push(RUST_CLIPPY.to_string());
        commands.push(RUST_TEST.to_string());
        commands.push(RUST_FMT.to_string());
    }

    commands
}

#[allow(clippy::too_many_arguments)]
fn build_payload(
    result: &str,
    dry_run: bool,
    json_mode: bool,
    json_pretty: bool,
    fast: bool,
    skip_rust: bool,
    commands: &[String],
    step_results: &[StepResult],
    exit_code: i32,
    total_duration_ms: u128,
    failed_step: Option<usize>,
    failed_command: Option<&str>,
) -> serde_json::Value {
    let failed_steps: Vec<usize> = step_results
        .iter()
        .filter(|item| item.status == "error")
        .map(|item| item.step)
        .collect();
    let failed_commands: Vec<String> = step_results
        .iter()
        .filter(|item| item.status == "error")
        .map(|item| item.command.clone())
        .collect();
    let success_steps_count = step_results.len().saturating_sub(failed_steps.len());
    let commands_count = commands.len();
    let steps_executed = step_results.len();
    let success_rate = if steps_executed > 0 {
        success_steps_count as f64 / steps_executed as f64
    } else {
        0.0
    };
    let failed_rate = if steps_executed > 0 {
        failed_steps.len() as f64 / steps_executed as f64
    } else {
        0.0
    };
    let success_command_rate = if commands_count > 0 {
        success_steps_count as f64 / commands_count as f64
    } else {
        0.0
    };
    let failed_command_rate = if commands_count > 0 {
        failed_commands.len() as f64 / commands_count as f64
    } else {
        0.0
    };

    let mut payload = json!({
        "schema_version": "v1",
        "result": result,
        "dry_run": dry_run,
        "flags": {
            "json": json_mode,
            "json_pretty": json_pretty,
            "fast": fast,
            "skip_rust": skip_rust,
        },
        "commands": commands,
        "exit_code": exit_code,
        "steps_executed": steps_executed,
        "step_results": step_results,
        "failed_steps": failed_steps,
        "failed_steps_count": failed_steps.len(),
        "error_step_count": failed_steps.len(),
        "failed_commands": failed_commands,
        "failed_commands_count": failed_commands.len(),
        "success_steps_count": success_steps_count,
        "commands_count": commands_count,
        "step_rate_basis": steps_executed,
        "command_rate_basis": commands_count,
        "success_rate": success_rate,
        "failed_rate": failed_rate,
        "success_command_rate": success_command_rate,
        "failed_command_rate": failed_command_rate,
        "total_duration_ms": total_duration_ms,
    });

    if let Some(step) = failed_step {
        payload["failed_step"] = json!(step);
    }
    if let Some(command) = failed_command {
        payload["failed_command"] = json!(command);
    }

    payload
}

fn emit_json(payload: &serde_json::Value, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(payload)?);
    } else {
        println!("{}", serde_json::to_string(payload)?);
    }
    Ok(())
}

fn run_shell(command: &str, json_mode: bool) -> Result<i32> {
    let output = Command::new("bash").arg("-lc").arg(command).output()?;
    if json_mode {
        eprint!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    } else {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(output.status.code().unwrap_or(1))
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix epoch")
        .as_millis()
}
