use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::process::Command;
use std::time::Instant;
use tempfile::tempdir;

use crate::hooks::evaluate_stop_guard;

const SUPPORTED_SUITES: [&str; 4] = ["phase1", "phase2", "contract", "all"];

#[derive(Clone, Serialize)]
struct ScenarioResult {
    name: String,
    passed: bool,
    duration_ms: f64,
    error: String,
}

struct NamedScenario {
    name: &'static str,
    run: fn() -> Result<()>,
}

pub(crate) fn cmd_regression(
    suite: &str,
    scenario: Option<&str>,
    runs: usize,
    min_pass_rate: f64,
    json_mode: bool,
    list_suites: bool,
) -> Result<()> {
    if list_suites {
        if json_mode {
            println!(
                "{}",
                serde_json::to_string(&json!({
                    "result": "ok",
                    "default_suite": "all",
                    "suites": SUPPORTED_SUITES,
                }))?
            );
        } else {
            for suite_name in SUPPORTED_SUITES {
                println!("{suite_name}");
            }
        }
        return Ok(());
    }

    let (suite_label, header, scenarios) = if let Some(name) = scenario {
        match name {
            "resume_reliability" => (
                "resume_reliability".to_string(),
                format!("🔄 Resume Reliability Test ({runs} runs)"),
                repeated_resume_scenarios(runs),
            ),
            _ => return Err(anyhow!("Unknown scenario: {name}")),
        }
    } else {
        select_suite(suite)?
    };

    if !json_mode {
        println!("{header}");
        println!("{}", "-".repeat(60));
    }

    let started = Instant::now();
    let results = run_suite(&scenarios, &suite_label, !json_mode);
    let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
    let payload = build_payload(&suite_label, min_pass_rate, duration_ms, &results);

    let pass_rate = payload
        .get("pass_rate")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    let ok = pass_rate >= min_pass_rate;

    if json_mode {
        println!("{}", serde_json::to_string(&payload)?);
    } else if !ok {
        eprintln!(
            "Regression suite below threshold: {:.3} < {:.3}",
            pass_rate, min_pass_rate
        );
    }

    if ok {
        Ok(())
    } else {
        Err(anyhow!(
            "regression suite {suite_label} below threshold ({pass_rate:.3} < {min_pass_rate:.3})"
        ))
    }
}

fn select_suite(suite: &str) -> Result<(String, String, Vec<NamedScenario>)> {
    match suite {
        "phase1" => Ok((
            "phase1".to_string(),
            format!(
                "🧪 Phase 1 Regression Suite ({} scenarios)",
                phase1_scenarios().len()
            ),
            phase1_scenarios(),
        )),
        "phase2" => Ok((
            "phase2".to_string(),
            format!(
                "🧪 Phase 2 Regression Suite ({} scenarios)",
                phase2_scenarios().len()
            ),
            phase2_scenarios(),
        )),
        "contract" => Ok((
            "contract".to_string(),
            format!(
                "🧪 Contract Regression Suite ({} scenarios)",
                contract_scenarios().len()
            ),
            contract_scenarios(),
        )),
        "all" => Ok((
            "all".to_string(),
            format!(
                "🧪 Full Regression Suite ({} scenarios)",
                all_scenarios().len()
            ),
            all_scenarios(),
        )),
        _ => Err(anyhow!(
            "Unknown suite: {suite}\nSupported suites: phase1|phase2|contract|all"
        )),
    }
}

fn phase1_scenarios() -> Vec<NamedScenario> {
    vec![
        NamedScenario {
            name: "S01-stop-guard-allow",
            run: scenario_stop_guard_allow,
        },
        NamedScenario {
            name: "S02-stop-guard-block",
            run: scenario_stop_guard_block,
        },
        NamedScenario {
            name: "S03-verify-fallback",
            run: scenario_verify_fallback,
        },
    ]
}

fn phase2_scenarios() -> Vec<NamedScenario> {
    vec![
        NamedScenario {
            name: "S04-hook-pretool-active",
            run: scenario_hook_pretool_active,
        },
        NamedScenario {
            name: "S05-hook-posttool-change",
            run: scenario_hook_posttool_change,
        },
    ]
}

fn contract_scenarios() -> Vec<NamedScenario> {
    let mut scenarios = phase1_scenarios();
    scenarios.extend(phase2_scenarios());
    scenarios
}

fn all_scenarios() -> Vec<NamedScenario> {
    contract_scenarios()
}

fn repeated_resume_scenarios(runs: usize) -> Vec<NamedScenario> {
    let count = runs.max(1);
    (0..count)
        .map(|_| NamedScenario {
            name: "resume-reliability",
            run: scenario_resume_reliability_single,
        })
        .collect()
}

fn run_suite(scenarios: &[NamedScenario], label: &str, verbose: bool) -> Vec<ScenarioResult> {
    let mut results = Vec::with_capacity(scenarios.len());

    for scenario in scenarios {
        let started = Instant::now();
        let outcome = (scenario.run)();
        let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
        let result = match outcome {
            Ok(()) => ScenarioResult {
                name: scenario.name.to_string(),
                passed: true,
                duration_ms,
                error: String::new(),
            },
            Err(error) => ScenarioResult {
                name: scenario.name.to_string(),
                passed: false,
                duration_ms,
                error: error.to_string(),
            },
        };

        if verbose {
            let status = if result.passed { "✅" } else { "❌" };
            let suffix = if result.error.is_empty() {
                String::new()
            } else {
                format!(" - {}", result.error)
            };
            println!(
                "  {status} {} ({:.1}ms){suffix}",
                result.name, result.duration_ms
            );
        }

        results.push(result);
    }

    if verbose {
        let passed = results.iter().filter(|result| result.passed).count();
        let total = results.len();
        let rate = if total == 0 {
            0.0
        } else {
            passed as f64 / total as f64
        };
        let total_ms: f64 = results.iter().map(|result| result.duration_ms).sum();

        println!("\n{}", "=".repeat(60));
        println!("Suite: {label}");
        println!("Passed: {passed}/{total} ({:.1}%)", rate * 100.0);
        println!("Total time: {:.1}ms", total_ms);
        println!("{}", "=".repeat(60));
    }

    results
}

fn build_payload(
    suite_label: &str,
    min_pass_rate: f64,
    duration_ms: f64,
    results: &[ScenarioResult],
) -> serde_json::Value {
    let passed = results.iter().filter(|result| result.passed).count();
    let total = results.len();
    let failed_scenarios: Vec<String> = results
        .iter()
        .filter(|result| !result.passed)
        .map(|result| result.name.clone())
        .collect();
    let pass_rate = if total == 0 {
        0.0
    } else {
        passed as f64 / total as f64
    };
    let failed_rate = if total == 0 {
        0.0
    } else {
        failed_scenarios.len() as f64 / total as f64
    };
    let longest = results
        .iter()
        .max_by(|left, right| left.duration_ms.total_cmp(&right.duration_ms))
        .cloned()
        .unwrap_or_else(empty_scenario);
    let fastest = results
        .iter()
        .min_by(|left, right| left.duration_ms.total_cmp(&right.duration_ms))
        .cloned()
        .unwrap_or_else(empty_scenario);
    let min_duration_ms = results
        .iter()
        .map(|result| result.duration_ms)
        .min_by(|left, right| left.total_cmp(right))
        .unwrap_or(0.0);
    let max_duration_ms = results
        .iter()
        .map(|result| result.duration_ms)
        .max_by(|left, right| left.total_cmp(right))
        .unwrap_or(0.0);
    let avg_duration_ms = if total == 0 {
        0.0
    } else {
        results.iter().map(|result| result.duration_ms).sum::<f64>() / total as f64
    };

    json!({
        "result": if failed_scenarios.is_empty() { "ok" } else { "error" },
        "suite": suite_label,
        "passed": passed,
        "total": total,
        "pass_rate": pass_rate,
        "min_pass_rate": min_pass_rate,
        "duration_ms": duration_ms,
        "scenario_results": results,
        "failed_scenarios": failed_scenarios,
        "longest_scenario": {
            "name": longest.name,
            "duration_ms": longest.duration_ms,
        },
        "fastest_scenario": {
            "name": fastest.name,
            "duration_ms": fastest.duration_ms,
        },
        "scenario_count_by_result": {
            "passed": passed,
            "failed": total.saturating_sub(passed),
        },
        "duration_stats": {
            "min_duration_ms": min_duration_ms,
            "max_duration_ms": max_duration_ms,
            "avg_duration_ms": avg_duration_ms,
        },
        "failed_rate": failed_rate,
        "success_rate": pass_rate,
        "success_count": passed,
        "failure_count": total.saturating_sub(passed),
        "total_scenarios": total,
        "schema_version": "v1",
        "rate_basis": total,
    })
}

fn empty_scenario() -> ScenarioResult {
    ScenarioResult {
        name: String::new(),
        passed: true,
        duration_ms: 0.0,
        error: String::new(),
    }
}

fn scenario_stop_guard_allow() -> Result<()> {
    let temp = tempdir()?;
    let fusion_dir = temp.path().join(".fusion");
    fs::create_dir_all(&fusion_dir)?;
    fs::write(
        fusion_dir.join("sessions.json"),
        "{\"status\":\"completed\",\"current_phase\":\"DELIVER\"}\n",
    )?;

    let output = evaluate_stop_guard(&fusion_dir)?;
    if output.decision != "allow" {
        return Err(anyhow!("expected allow, got {}", output.decision));
    }
    Ok(())
}

fn scenario_stop_guard_block() -> Result<()> {
    let temp = tempdir()?;
    let fusion_dir = temp.path().join(".fusion");
    fs::create_dir_all(&fusion_dir)?;
    fs::write(
        fusion_dir.join("sessions.json"),
        "{\"status\":\"in_progress\",\"current_phase\":\"EXECUTE\",\"goal\":\"contract\"}\n",
    )?;
    fs::write(fusion_dir.join("task_plan.md"), "### Task 1: A [PENDING]\n")?;

    let output = evaluate_stop_guard(&fusion_dir)?;
    if output.decision != "block" {
        return Err(anyhow!("expected block, got {}", output.decision));
    }
    Ok(())
}

fn scenario_verify_fallback() -> Result<()> {
    let temp = tempdir()?;
    let fusion_dir = temp.path().join(".fusion");
    fs::create_dir_all(&fusion_dir)?;
    fs::write(
        fusion_dir.join("sessions.json"),
        "{\"status\":\"in_progress\",\"current_phase\":\"VERIFY\",\"goal\":\"contract\"}\n",
    )?;
    fs::write(
        fusion_dir.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n",
    )?;

    let output = evaluate_stop_guard(&fusion_dir)?;
    if !output.phase_corrected {
        return Err(anyhow!("expected phase correction"));
    }
    if !output
        .events_dispatched
        .iter()
        .any(|event| event == "VERIFY_FAIL")
    {
        return Err(anyhow!("expected VERIFY_FAIL dispatch"));
    }
    Ok(())
}

fn scenario_hook_pretool_active() -> Result<()> {
    let temp = tempdir()?;
    let fusion_dir = temp.path().join(".fusion");
    fs::create_dir_all(&fusion_dir)?;
    fs::write(
        fusion_dir.join("sessions.json"),
        "{\"status\":\"in_progress\",\"current_phase\":\"EXECUTE\",\"goal\":\"contract\"}\n",
    )?;
    fs::write(fusion_dir.join("task_plan.md"), "### Task 1: A [PENDING]\n")?;

    let output = Command::new(std::env::current_exe()?)
        .arg("hook")
        .arg("pretool")
        .arg("--fusion-dir")
        .arg(&fusion_dir)
        .output()?;
    if !output.status.success() {
        return Err(anyhow!("pretool command failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("[fusion]") {
        return Err(anyhow!("expected fusion pretool output"));
    }
    Ok(())
}

fn scenario_hook_posttool_change() -> Result<()> {
    let temp = tempdir()?;
    let fusion_dir = temp.path().join(".fusion");
    fs::create_dir_all(&fusion_dir)?;
    fs::write(
        fusion_dir.join("sessions.json"),
        "{\"status\":\"in_progress\",\"current_phase\":\"EXECUTE\"}\n",
    )?;
    fs::write(
        fusion_dir.join("task_plan.md"),
        "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n- Type: implementation\n",
    )?;
    fs::write(fusion_dir.join(".progress_snapshot"), "0:2:0:0")?;

    let output = Command::new(std::env::current_exe()?)
        .arg("hook")
        .arg("posttool")
        .arg("--fusion-dir")
        .arg(&fusion_dir)
        .output()?;
    if !output.status.success() {
        return Err(anyhow!("posttool command failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("Next action: Continue task: B [PENDING]") {
        return Err(anyhow!("expected posttool next-task output"));
    }
    Ok(())
}

fn scenario_resume_reliability_single() -> Result<()> {
    scenario_stop_guard_block()
}
