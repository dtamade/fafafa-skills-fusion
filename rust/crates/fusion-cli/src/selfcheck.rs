use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::stop_guard::evaluate_stop_guard;

#[derive(Serialize)]
struct SelfcheckFlags {
    fix: bool,
    quick: bool,
    json: bool,
}

#[derive(Serialize)]
struct HookDoctorCheck {
    name: &'static str,
    ok: bool,
    exit_code: i32,
    result: String,
    warn_count: i64,
}

#[derive(Serialize)]
struct StopSimulationCheck {
    name: &'static str,
    ok: bool,
    exit_code: i32,
    decision: String,
}

#[derive(Serialize)]
struct ContractRegressionCheck {
    name: &'static str,
    ok: bool,
    exit_code: i32,
    skipped: bool,
}

pub(crate) fn cmd_selfcheck(
    project_root: Option<&Path>,
    fix_mode: bool,
    quick_mode: bool,
    json_mode: bool,
) -> Result<()> {
    let project_root = project_root
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().expect("current dir"));
    if !project_root.is_dir() {
        if json_mode {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "result": "error",
                    "reason": format!("project_root not found: {}", project_root.display()),
                }))?
            );
        }
        return Err(anyhow!(
            "project_root not found: {}",
            project_root.display()
        ));
    }
    let project_root = fs::canonicalize(project_root)?;
    let repo_root = detect_repo_root(&project_root)
        .ok_or_else(|| anyhow!("unable to locate fusion root for selfcheck command"))?;

    log_line(
        json_mode,
        &format!("[selfcheck] project_root: {}", project_root.display()),
    );

    let doctor = run_selfcheck_doctor(&project_root, fix_mode, json_mode)?;
    let stop = run_selfcheck_stop_simulation(json_mode)?;
    let regression = run_selfcheck_contract_regression(&repo_root, quick_mode, json_mode)?;

    let overall_ok = doctor.ok && stop.ok && (regression.skipped || regression.ok);
    let result = if overall_ok { "ok" } else { "error" };

    if json_mode {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "result": result,
                "project_root": project_root.display().to_string(),
                "flags": SelfcheckFlags { fix: fix_mode, quick: quick_mode, json: true },
                "checks": [serde_json::to_value(&doctor)?, serde_json::to_value(&stop)?, serde_json::to_value(&regression)?],
            }))?
        );
    } else if overall_ok {
        println!("[selfcheck] ✅ all checks passed");
    } else {
        eprintln!("[selfcheck] ❌ checks failed");
    }

    if overall_ok {
        Ok(())
    } else {
        Err(anyhow!("hook selfcheck failed"))
    }
}

fn run_selfcheck_doctor(
    project_root: &Path,
    fix_mode: bool,
    json_mode: bool,
) -> Result<HookDoctorCheck> {
    log_line(json_mode, "[selfcheck] check 1/3: fusion-hook-doctor");
    let exe = env::current_exe()?;
    let mut command = Command::new(exe);
    command.arg("doctor").arg("--json");
    if fix_mode {
        command.arg("--fix");
    }
    command.arg(project_root);

    let output = command.output()?;
    let payload = parse_json_payload(&output.stdout);
    let result = payload
        .as_ref()
        .and_then(|value| value.get("result"))
        .and_then(|value| value.as_str())
        .unwrap_or("error")
        .to_string();
    let warn_count = payload
        .as_ref()
        .and_then(|value| value.get("warn_count"))
        .and_then(|value| value.as_i64())
        .unwrap_or(999);
    let exit_code = output.status.code().unwrap_or(1);
    let ok = exit_code == 0 && result == "ok" && warn_count == 0;

    if ok {
        log_line(json_mode, "[selfcheck] ✅ doctor passed");
    } else {
        log_line(
            json_mode,
            &format!(
                "[selfcheck] ❌ doctor failed (rc={exit_code} result={result} warn_count={warn_count})"
            ),
        );
    }

    Ok(HookDoctorCheck {
        name: "hook_doctor",
        ok,
        exit_code,
        result,
        warn_count,
    })
}

fn run_selfcheck_stop_simulation(json_mode: bool) -> Result<StopSimulationCheck> {
    log_line(json_mode, "[selfcheck] check 2/3: stop-hook simulation");
    let temp_root = env::temp_dir().join(format!(
        "fusion-selfcheck-{}-{}",
        std::process::id(),
        now_ms()
    ));
    let fusion_dir = temp_root.join(".fusion");
    fs::create_dir_all(&fusion_dir)?;
    fs::write(
        fusion_dir.join("sessions.json"),
        "{\"status\":\"in_progress\",\"current_phase\":\"EXECUTE\",\"goal\":\"hook-selfcheck\"}\n",
    )?;
    fs::write(
        fusion_dir.join("task_plan.md"),
        "### Task 1: Verify Hook [PENDING]\n",
    )?;

    let (exit_code, decision) = match evaluate_stop_guard(&fusion_dir) {
        Ok(output) => (0, output.decision),
        Err(_) => (1, String::new()),
    };
    let ok = exit_code == 0 && decision == "block";

    if ok {
        log_line(json_mode, "[selfcheck] ✅ stop-hook simulation passed");
    } else {
        log_line(
            json_mode,
            &format!(
                "[selfcheck] ❌ stop-hook simulation failed (rc={exit_code} decision={decision})"
            ),
        );
    }

    let _ = fs::remove_dir_all(&temp_root);

    Ok(StopSimulationCheck {
        name: "stop_simulation",
        ok,
        exit_code,
        decision,
    })
}

fn run_selfcheck_contract_regression(
    repo_root: &Path,
    quick_mode: bool,
    json_mode: bool,
) -> Result<ContractRegressionCheck> {
    if quick_mode {
        log_line(
            json_mode,
            "[selfcheck] check 3/3: contract regression suite (skipped by --quick)",
        );
        return Ok(ContractRegressionCheck {
            name: "contract_regression_suite",
            ok: true,
            exit_code: 0,
            skipped: true,
        });
    }

    log_line(
        json_mode,
        "[selfcheck] check 3/3: contract regression suite",
    );
    let exe = env::current_exe()?;
    let output = Command::new(exe)
        .arg("regression")
        .arg("--suite")
        .arg("contract")
        .arg("--json")
        .arg("--min-pass-rate")
        .arg("0.99")
        .current_dir(repo_root)
        .output()?;
    let exit_code = output.status.code().unwrap_or(1);
    let ok = exit_code == 0;

    if ok {
        log_line(json_mode, "[selfcheck] ✅ contract regression suite passed");
    } else {
        log_line(
            json_mode,
            &format!("[selfcheck] ❌ contract regression suite failed (rc={exit_code})"),
        );
    }

    Ok(ContractRegressionCheck {
        name: "contract_regression_suite",
        ok,
        exit_code,
        skipped: false,
    })
}

fn parse_json_payload(stdout: &[u8]) -> Option<Value> {
    serde_json::from_slice(stdout).ok()
}

fn log_line(json_mode: bool, line: &str) {
    if json_mode {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}

fn detect_repo_root(project_root: &Path) -> Option<PathBuf> {
    if let Ok(exe_path) = env::current_exe() {
        for ancestor in exe_path.ancestors() {
            if ancestor.join("scripts/fusion-hook-selfcheck.sh").is_file() {
                return Some(ancestor.to_path_buf());
            }
        }
    }

    for ancestor in project_root.ancestors() {
        if ancestor.join("scripts/fusion-hook-selfcheck.sh").is_file() {
            return Some(ancestor.to_path_buf());
        }
    }

    None
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix epoch")
        .as_millis()
}
