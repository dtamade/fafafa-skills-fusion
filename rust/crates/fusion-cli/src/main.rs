use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fusion_provider::{extract_session_id, run_backend, session_key_for_backend};
use fusion_runtime_io::{
    append_event, ensure_fusion_dir, json_get_bool, json_get_string, json_set_string,
    load_backends_from_config, load_flat_config, read_json, read_text,
    remove_dependency_report_if_exists, utc_now_iso, write_dependency_report, write_json_pretty,
    write_text, DependencyReport, FlatConfig,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha1::{Digest, Sha1};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "fusion-bridge")]
#[command(about = "Fusion Rust bridge binary", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        #[arg(long, default_value = "templates")]
        templates_dir: PathBuf,
    },
    Start {
        goal: String,
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        #[arg(long, default_value = "templates")]
        templates_dir: PathBuf,
        #[arg(long, default_value_t = false)]
        force: bool,
        #[arg(long, default_value_t = false)]
        yolo: bool,
    },
    Status {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Run {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        #[arg(long, default_value_t = 50)]
        max_iterations: i64,
        #[arg(long, default_value_t = 6)]
        max_no_progress_rounds: i64,
        #[arg(long, default_value_t = 250)]
        initial_backoff_ms: u64,
        #[arg(long, default_value_t = 5000)]
        max_backoff_ms: u64,
    },
    Resume {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        #[arg(long, default_value_t = 50)]
        max_iterations: i64,
        #[arg(long, default_value_t = 6)]
        max_no_progress_rounds: i64,
        #[arg(long, default_value_t = 250)]
        initial_backoff_ms: u64,
        #[arg(long, default_value_t = 5000)]
        max_backoff_ms: u64,
    },
    Codeagent {
        #[arg(default_value = "EXECUTE")]
        phase: String,
        #[arg(trailing_var_arg = true)]
        prompt: Vec<String>,
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Hook {
        #[command(subcommand)]
        command: HookCommands,
    },
}

#[derive(Subcommand, Debug)]
enum HookCommands {
    Pretool {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Posttool {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    StopGuard {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    SetGoal {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        goal: String,
    },
}

#[derive(Debug, Clone, Copy, Default)]
struct TaskCounts {
    completed: i64,
    pending: i64,
    in_progress: i64,
    failed: i64,
}

impl TaskCounts {
    fn total(&self) -> i64 {
        self.completed + self.pending + self.in_progress + self.failed
    }

    fn pending_like(&self) -> i64 {
        self.pending + self.in_progress
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SafeTask {
    title: String,
    category: String,
    #[serde(rename = "type")]
    task_type: String,
    execution: String,
    output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SafeBacklogResult {
    enabled: bool,
    added: i64,
    tasks: Vec<SafeTask>,
    blocked_by_backoff: bool,
    backoff_state: SafeBackoffState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SafeBackoffState {
    consecutive_failures: i64,
    consecutive_injections: i64,
    cooldown_until_round: i64,
    attempt_round: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SafeBacklogStats {
    total_injections: i64,
    category_counts: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SafeBacklogState {
    fingerprints: Vec<String>,
    last_category: String,
    stats: SafeBacklogStats,
    backoff: SafeBackoffState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SupervisorState {
    last_advice_round: i64,
    last_digest: String,
    last_risk_score: f64,
    updated_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SupervisorSuggestion {
    category: String,
    title: String,
    rationale: String,
}

#[derive(Debug, Clone)]
struct SupervisorAdvice {
    line: String,
    payload: Value,
    risk_score: f64,
}

#[derive(Debug, Clone)]
struct RunOptions {
    max_iterations: i64,
    max_no_progress_rounds: i64,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
}

#[derive(Debug, Clone)]
struct CodeagentExecution {
    output: String,
    exit_code: i32,
}

#[derive(Debug, Serialize)]
struct StopGuardOutput {
    decision: String,
    should_block: bool,
    reason: String,
    #[serde(rename = "systemMessage")]
    system_message: String,
    phase_corrected: bool,
    events_dispatched: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init {
            fusion_dir,
            templates_dir,
        } => cmd_init(&fusion_dir, &templates_dir),
        Commands::Start {
            goal,
            fusion_dir,
            templates_dir,
            force,
            yolo,
        } => cmd_start(&fusion_dir, &templates_dir, &goal, force || yolo),
        Commands::Status { fusion_dir } => cmd_status(&fusion_dir),
        Commands::Run {
            fusion_dir,
            max_iterations,
            max_no_progress_rounds,
            initial_backoff_ms,
            max_backoff_ms,
        } => cmd_run(
            &fusion_dir,
            RunOptions {
                max_iterations,
                max_no_progress_rounds,
                initial_backoff_ms,
                max_backoff_ms,
            },
        ),
        Commands::Resume {
            fusion_dir,
            max_iterations,
            max_no_progress_rounds,
            initial_backoff_ms,
            max_backoff_ms,
        } => cmd_resume(
            &fusion_dir,
            RunOptions {
                max_iterations,
                max_no_progress_rounds,
                initial_backoff_ms,
                max_backoff_ms,
            },
        ),
        Commands::Codeagent {
            phase,
            prompt,
            fusion_dir,
        } => cmd_codeagent(&fusion_dir, &phase, &prompt),
        Commands::Hook { command } => match command {
            HookCommands::Pretool { fusion_dir } => cmd_hook_pretool(&fusion_dir),
            HookCommands::Posttool { fusion_dir } => cmd_hook_posttool(&fusion_dir),
            HookCommands::StopGuard { fusion_dir } => cmd_hook_stop_guard(&fusion_dir),
            HookCommands::SetGoal { fusion_dir, goal } => cmd_hook_set_goal(&fusion_dir, &goal),
        },
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn cmd_init(fusion_dir: &Path, templates_dir: &Path) -> Result<()> {
    init_fusion_dir(fusion_dir, templates_dir)?;

    println!("[fusion] Initialized .fusion directory");
    println!("[fusion] Files created:");

    let mut names: Vec<String> = fs::read_dir(fusion_dir)
        .with_context(|| format!("failed reading dir: {}", fusion_dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    names.sort();
    for name in names {
        println!("- {name}");
    }

    Ok(())
}

fn cmd_start(fusion_dir: &Path, templates_dir: &Path, goal: &str, force_mode: bool) -> Result<()> {
    init_fusion_dir(fusion_dir, templates_dir)?;

    let sessions_path = fusion_dir.join("sessions.json");
    let mut sessions = read_json(&sessions_path)?;

    let workflow_id = format!("fusion_{}", chrono::Utc::now().timestamp());
    let timestamp = utc_now_iso();

    json_set_string(&mut sessions, "goal", goal);
    json_set_string(&mut sessions, "started_at", &timestamp);
    json_set_string(&mut sessions, "workflow_id", &workflow_id);
    json_set_string(&mut sessions, "status", "in_progress");
    json_set_string(&mut sessions, "current_phase", "INITIALIZE");

    write_json_pretty(&sessions_path, &sessions)?;

    if force_mode {
        println!("[fusion] ⚠️ Skipped UNDERSTAND (--force)");
    } else {
        println!("[fusion] UNDERSTAND runner (Rust) currently minimal; proceed to INITIALIZE");
    }

    println!();
    println!("[FUSION] Workflow initialized.");
    println!("Goal: {goal}");

    Ok(())
}

fn init_fusion_dir(fusion_dir: &Path, templates_dir: &Path) -> Result<()> {
    if fusion_dir.is_symlink() {
        anyhow::bail!(
            "❌ Security: {} is a symlink, refusing to use",
            fusion_dir.display()
        );
    }

    let sessions_path = fusion_dir.join("sessions.json");
    if sessions_path.is_file() {
        let sessions = read_json(&sessions_path)?;
        let status =
            json_get_string(&sessions, &["status"]).unwrap_or_else(|| "unknown".to_string());
        if status == "in_progress" || status == "paused" {
            anyhow::bail!(
                "❌ Cannot reinitialize: workflow is {status}\n   Use /fusion cancel or /fusion resume"
            );
        }
    }

    fs::create_dir_all(fusion_dir)
        .with_context(|| format!("failed creating dir: {}", fusion_dir.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(fusion_dir)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(fusion_dir, perms)?;
    }

    copy_if_exists(
        &templates_dir.join("task_plan.md"),
        &fusion_dir.join("task_plan.md"),
    )?;
    copy_if_exists(
        &templates_dir.join("progress.md"),
        &fusion_dir.join("progress.md"),
    )?;
    copy_if_exists(
        &templates_dir.join("findings.md"),
        &fusion_dir.join("findings.md"),
    )?;

    let config_template = templates_dir.join("config.yaml");
    if config_template.is_file() {
        copy_if_exists(&config_template, &fusion_dir.join("config.yaml"))?;
    } else {
        write_text(
            &fusion_dir.join("config.yaml"),
            "runtime:\n  enabled: true\n  compat_mode: true\n  version: \"2.6.3\"\n\nbackends:\n  primary: codex\n  fallback: claude\n",
        )?;
    }

    let sessions_template = templates_dir.join("sessions.json");
    if sessions_template.is_file() {
        copy_if_exists(&sessions_template, &sessions_path)?;
    } else if !sessions_path.exists() {
        write_text(
            &sessions_path,
            r#"{
  "workflow_id": null,
  "goal": null,
  "started_at": null,
  "status": "not_started",
  "current_phase": null,
  "codex_session": null,
  "claude_session": null,
  "tasks": {},
  "strikes": {
    "current_task": null,
    "count": 0,
    "history": []
  },
  "git": {
    "branch": null,
    "commits": []
  },
  "last_checkpoint": null
}"#,
        )?;
    }

    ensure_gitignore_has_fusion()?;

    Ok(())
}

fn copy_if_exists(src: &Path, dest: &Path) -> Result<()> {
    if src.is_file() {
        fs::copy(src, dest).with_context(|| {
            format!(
                "failed copying template {} -> {}",
                src.display(),
                dest.display()
            )
        })?;
    }
    Ok(())
}

fn ensure_gitignore_has_fusion() -> Result<()> {
    let gitignore = PathBuf::from(".gitignore");
    if !gitignore.is_file() {
        return Ok(());
    }

    let mut content = read_text(&gitignore)?;
    if content.lines().any(|line| line.trim() == ".fusion/") {
        return Ok(());
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n# Fusion working directory\n.fusion/\n");
    write_text(&gitignore, &content)
}

fn cmd_status(fusion_dir: &Path) -> Result<()> {
    if !fusion_dir.is_dir() {
        println!("[fusion] No .fusion directory found. Run /fusion to start.");
        std::process::exit(1);
    }

    println!("=== Fusion Status ===");
    println!();

    let task_plan = fusion_dir.join("task_plan.md");
    if task_plan.is_file() {
        println!("## Task Plan");
        let content = read_text(&task_plan)?;
        if let Some(block) = extract_status_block(&content) {
            print!("{block}");
            if !block.ends_with('\n') {
                println!();
            }
        } else {
            println!("No status found");
        }
        println!();
    }

    let progress = fusion_dir.join("progress.md");
    if progress.is_file() {
        println!("## Recent Progress (last 10 entries)");
        let content = read_text(&progress)?;
        let mut rows: Vec<&str> = content
            .lines()
            .filter(|line| line.starts_with('|'))
            .collect();
        if rows.len() > 12 {
            rows = rows[rows.len() - 12..].to_vec();
        }
        for row in rows {
            println!("{row}");
        }
        println!();

        let error_lines: Vec<&str> = content
            .lines()
            .filter(|line| line.contains("ERROR") || line.contains("FAILED"))
            .collect();
        if !error_lines.is_empty() {
            println!("## Errors: {} found", error_lines.len());
            for row in error_lines.iter().rev().take(5).rev() {
                println!("{row}");
            }
        }
    }

    let sessions_path = fusion_dir.join("sessions.json");
    if sessions_path.is_file() {
        println!("## Active Sessions");
        let sessions_text = read_text(&sessions_path)?;
        for line in sessions_text.lines().take(5) {
            println!("{line}");
        }

        println!();
        println!("## Runtime");
        let sessions = read_json(&sessions_path)?;
        if let Some(status) = json_get_string(&sessions, &["status"]) {
            println!("status: {status}");
        }
        if let Some(phase) = json_get_string(&sessions, &["current_phase"]) {
            println!("phase: {phase}");
        }
        if let Some(last_event_id) = json_get_string(&sessions, &["_runtime", "last_event_id"]) {
            println!("last_event_id: {last_event_id}");
        }
        if let Some(counter) = sessions
            .get("_runtime")
            .and_then(|v| v.get("last_event_counter"))
            .and_then(|v| v.as_i64())
        {
            println!("event_counter: {counter}");
        }

        if let Some(enabled) = json_get_bool(&sessions, &["_runtime", "scheduler", "enabled"]) {
            println!("scheduler.enabled: {enabled}");
            if let Some(batch_id) = sessions
                .get("_runtime")
                .and_then(|v| v.get("scheduler"))
                .and_then(|v| v.get("current_batch_id"))
                .and_then(|v| v.as_i64())
            {
                println!("scheduler.batch_id: {batch_id}");
            }
            if let Some(parallel_tasks) = sessions
                .get("_runtime")
                .and_then(|v| v.get("scheduler"))
                .and_then(|v| v.get("parallel_tasks"))
                .and_then(|v| v.as_i64())
            {
                println!("scheduler.parallel_tasks: {parallel_tasks}");
            }
        }

        let events = fusion_dir.join("events.jsonl");
        if events.is_file() {
            if let Some((added, timestamp)) = last_safe_backlog(&events)? {
                println!("safe_backlog.last_added: {added}");
                println!("safe_backlog.last_injected_at: {timestamp}");
                if let Some(iso) = epoch_to_iso(timestamp) {
                    println!("safe_backlog.last_injected_at_iso: {iso}");
                }
            }
        }
    }

    let dep = fusion_dir.join("dependency_report.json");
    if dep.is_file() {
        println!();
        println!("## Dependency Report");
        let value = read_json(&dep)?;

        if let Some(status) = json_get_string(&value, &["status"]) {
            println!("status: {status}");
        }
        if let Some(source) = json_get_string(&value, &["source"]) {
            println!("source: {source}");
        }
        if let Some(reason) = json_get_string(&value, &["reason"]) {
            println!("reason: {reason}");
        }

        if let Some(missing) = value.get("missing").and_then(|v| v.as_array()) {
            let joined = missing
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            if !joined.is_empty() {
                println!("missing: {joined}");
            }
        }

        if let Some(next) = value
            .get("next_actions")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
        {
            println!("next: {next}");
        }
    }

    Ok(())
}

fn execute_codeagent(
    fusion_dir: &Path,
    phase: &str,
    prompt_args: &[String],
) -> Result<CodeagentExecution> {
    ensure_fusion_dir(fusion_dir)?;

    let cwd = std::env::current_dir().context("failed reading cwd")?;
    let explicit_bin = std::env::var("CODEAGENT_WRAPPER_BIN").ok();

    let wrapper = match fusion_provider::resolve_wrapper_bin(explicit_bin.as_deref(), &cwd) {
        Ok(resolved) => resolved,
        Err(_) => {
            let report = DependencyReport {
                status: "blocked".to_string(),
                source: "fusion-bridge".to_string(),
                timestamp: utc_now_iso(),
                missing: vec!["codeagent-wrapper".to_string()],
                reason: "Missing executable for backend orchestration".to_string(),
                auto_attempted: vec![
                    explicit_bin.unwrap_or_default(),
                    "codeagent-wrapper in PATH".to_string(),
                    "./node_modules/.bin/codeagent-wrapper".to_string(),
                    "~/.local/bin/codeagent-wrapper".to_string(),
                    "~/.npm-global/bin/codeagent-wrapper".to_string(),
                ],
                next_actions: vec![
                    "Install or expose codeagent-wrapper in PATH.".to_string(),
                    "Or set CODEAGENT_WRAPPER_BIN to an executable path.".to_string(),
                    "Re-run: fusion-bridge codeagent EXECUTE".to_string(),
                ],
                agent_prompt: Some(
                    "Dependency missing: codeagent-wrapper. Resolve installation/path and retry fusion-bridge codeagent."
                        .to_string(),
                ),
            };

            let path = write_dependency_report(fusion_dir, &report)?;
            let message = format!(
                "[fusion][deps] Missing dependency: codeagent-wrapper
[fusion][deps] Report written: {}
",
                path.display()
            );
            return Ok(CodeagentExecution {
                output: message,
                exit_code: 127,
            });
        }
    };

    remove_dependency_report_if_exists(fusion_dir)?;

    let sessions_path = fusion_dir.join("sessions.json");
    let mut sessions = read_json(&sessions_path)?;

    let goal = json_get_string(&sessions, &["goal"]).unwrap_or_default();
    let (primary, fallback) = load_backends_from_config(fusion_dir);

    let primary_session_key = session_key_for_backend(&primary);
    let primary_session = json_get_string(&sessions, &[primary_session_key]);

    let prompt = if prompt_args.is_empty() {
        render_prompt(fusion_dir, phase, &goal)?
    } else {
        prompt_args.join(" ")
    };

    let mut used_backend = primary.clone();
    let mut run_result = run_backend(
        &wrapper.bin,
        &primary,
        &prompt,
        primary_session.as_deref(),
        &cwd,
    )?;

    if run_result.exit_code != 0 {
        eprintln!("[fusion] primary backend failed, fallback to {fallback}");
        used_backend = fallback.clone();
        run_result = run_backend(&wrapper.bin, &fallback, &prompt, None, &cwd)?;
    }

    if let Some(session_id) = extract_session_id(&run_result.output) {
        let key = session_key_for_backend(&used_backend);
        json_set_string(&mut sessions, key, &session_id);
        write_json_pretty(&sessions_path, &sessions)?;
    }

    Ok(CodeagentExecution {
        output: run_result.output,
        exit_code: run_result.exit_code,
    })
}

fn cmd_codeagent(fusion_dir: &Path, phase: &str, prompt_args: &[String]) -> Result<()> {
    let run = execute_codeagent(fusion_dir, phase, prompt_args)?;

    if run.exit_code != 0 {
        eprint!("{}", run.output);
        std::process::exit(run.exit_code.max(1));
    }

    print!("{}", run.output);
    Ok(())
}

fn cmd_resume(fusion_dir: &Path, options: RunOptions) -> Result<()> {
    ensure_fusion_dir(fusion_dir)?;

    let sessions_path = fusion_dir.join("sessions.json");
    let mut sessions = read_json(&sessions_path)?;
    let status = json_get_string(&sessions, &["status"]).unwrap_or_else(|| "unknown".to_string());

    match status.as_str() {
        "paused" => {
            json_set_string(&mut sessions, "status", "in_progress");
            if json_get_string(&sessions, &["current_phase"]).is_none() {
                json_set_string(&mut sessions, "current_phase", "EXECUTE");
            }
            write_json_pretty(&sessions_path, &sessions)?;
            println!("[fusion] Workflow resumed from paused state");
        }
        "stuck" => {
            println!("⚠️ Workflow is stuck. Please investigate:");
            println!("   - Check .fusion/progress.md for errors");
            println!("   - Fix the issue and run /fusion resume again");
            println!("   - Or cancel with: ./scripts/fusion-cancel.sh");

            json_set_string(&mut sessions, "status", "in_progress");
            if json_get_string(&sessions, &["current_phase"]).is_none() {
                json_set_string(&mut sessions, "current_phase", "EXECUTE");
            }
            write_json_pretty(&sessions_path, &sessions)?;
            println!("[fusion] Status has been set to 'in_progress'. Continuing.");
        }
        "in_progress" => {
            println!("[fusion] Workflow already in progress, continuing");
        }
        "completed" => {
            println!("✅ Workflow already completed. Nothing to resume.");
            return Ok(());
        }
        "cancelled" => {
            anyhow::bail!(
                "❌ Workflow was cancelled. Start a new workflow with:\n   /fusion \"<new goal>\""
            );
        }
        _ => {
            anyhow::bail!("Cannot resume workflow with status: {status}");
        }
    }

    cmd_run(fusion_dir, options)
}

fn cmd_run(fusion_dir: &Path, options: RunOptions) -> Result<()> {
    ensure_fusion_dir(fusion_dir)?;

    let max_iterations = options.max_iterations.max(1);
    let max_no_progress_rounds = options.max_no_progress_rounds.max(1);
    let mut no_progress_rounds: i64 = 0;
    let mut backoff_ms = options.initial_backoff_ms.max(1);

    let mut last_counts = read_task_counts(fusion_dir)?;

    for iteration in 1..=max_iterations {
        let guard = evaluate_stop_guard(fusion_dir)?;
        if guard.decision == "allow" || !guard.should_block {
            println!("[fusion][run] Loop completed after {iteration} iteration(s)");
            return Ok(());
        }

        println!(
            "[fusion][run] Iteration {}/{} | {}",
            iteration, max_iterations, guard.system_message
        );

        cmd_hook_pretool(fusion_dir)?;

        let sessions = read_json(&fusion_dir.join("sessions.json"))?;
        let phase =
            json_get_string(&sessions, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());

        let step = execute_codeagent(fusion_dir, &phase, &[])?;
        if !step.output.is_empty() {
            print!("{}", step.output);
        }
        if step.exit_code != 0 {
            eprintln!(
                "[fusion][run] codeagent step failed with exit code {}",
                step.exit_code
            );
            std::process::exit(step.exit_code.max(1));
        }

        cmd_hook_posttool(fusion_dir)?;

        let current_counts = read_task_counts(fusion_dir)?;
        let changed = current_counts.completed != last_counts.completed
            || current_counts.pending != last_counts.pending
            || current_counts.in_progress != last_counts.in_progress
            || current_counts.failed != last_counts.failed;

        if changed {
            no_progress_rounds = 0;
            backoff_ms = options.initial_backoff_ms.max(1);
            last_counts = current_counts;
            continue;
        }

        no_progress_rounds += 1;
        if no_progress_rounds >= max_no_progress_rounds {
            eprintln!(
                "[fusion][run] No progress rounds limit reached ({}/{})",
                no_progress_rounds, max_no_progress_rounds
            );
            std::process::exit(2);
        }

        let bounded_max_backoff = options
            .max_backoff_ms
            .max(options.initial_backoff_ms.max(1));
        let current_sleep = backoff_ms.min(bounded_max_backoff);
        println!(
            "[fusion][run] No progress detected ({}/{}), backoff {}ms",
            no_progress_rounds, max_no_progress_rounds, current_sleep
        );
        thread::sleep(Duration::from_millis(current_sleep));
        backoff_ms = backoff_ms.saturating_mul(2).min(bounded_max_backoff);
    }

    eprintln!(
        "[fusion][run] Max iterations reached ({}) before completion",
        max_iterations
    );
    std::process::exit(3);
}

fn cmd_hook_pretool(fusion_dir: &Path) -> Result<()> {
    let sessions_path = fusion_dir.join("sessions.json");
    if !sessions_path.is_file() {
        return Ok(());
    }

    let snapshot = read_json(&sessions_path)?;
    if json_get_string(&snapshot, &["status"]).as_deref() != Some("in_progress") {
        return Ok(());
    }

    let goal = truncate_chars(
        json_get_string(&snapshot, &["goal"]).unwrap_or_else(|| "?".to_string()),
        60,
    );
    let phase =
        json_get_string(&snapshot, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());
    let phase_num = phase_num(&phase);

    println!("[fusion] Goal: {goal} | Phase: {phase} ({phase_num})");

    let counts = read_task_counts(fusion_dir)?;
    let total = counts.total();
    if total > 0 {
        let next_task = find_next_task(fusion_dir)?;
        let task_index = counts.completed + 1;
        let percent = counts.completed * 100 / total;
        let filled = counts.completed * 10 / total;
        let bar = format!(
            "{}{}",
            "█".repeat(filled as usize),
            "░".repeat((10 - filled) as usize)
        );
        let task_status = if counts.in_progress > 0 {
            "IN_PROGRESS"
        } else {
            "PENDING"
        };

        println!("[fusion] Task {task_index}/{total}: {next_task} [{task_status}]");
        println!("[fusion] Progress: {bar} {percent}% | Guardian: OK");
    }

    let sched = snapshot.get("_runtime").and_then(|v| v.get("scheduler"));
    if let Some(enabled) = sched
        .and_then(|v| v.get("enabled"))
        .and_then(|v| v.as_bool())
    {
        if enabled {
            let batch_id = sched
                .and_then(|v| v.get("current_batch_id"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let parallel = sched
                .and_then(|v| v.get("parallel_tasks"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            if batch_id > 0 || parallel > 0 {
                println!("[fusion] Batch: {batch_id} | Parallel: {parallel} tasks");
            }
        }
    }

    Ok(())
}

fn cmd_hook_posttool(fusion_dir: &Path) -> Result<()> {
    let sessions_path = fusion_dir.join("sessions.json");
    if !sessions_path.is_file() {
        return Ok(());
    }

    let snapshot = read_json(&sessions_path)?;
    if json_get_string(&snapshot, &["status"]).as_deref() != Some("in_progress") {
        return Ok(());
    }

    let cfg = load_flat_config(fusion_dir);
    let counts = read_task_counts(fusion_dir)?;
    let total = counts.total();
    let pending_like = counts.pending_like();
    let current_snap = format!(
        "{}:{}:{}:{}",
        counts.completed, counts.pending, counts.in_progress, counts.failed
    );

    let snap_file = fusion_dir.join(".progress_snapshot");
    let unchanged_file = fusion_dir.join(".snapshot_unchanged_count");

    if cfg.safe_backlog_enabled
        && cfg.safe_backlog_inject_on_task_exhausted
        && total > 0
        && pending_like == 0
    {
        if let Some(lines) = try_inject_safe_backlog(
            fusion_dir,
            &snapshot,
            &cfg,
            SafeBacklogTrigger {
                counts: &counts,
                pending_like,
                current_snap: &current_snap,
                reason: "task_exhausted",
                no_progress_rounds: 0,
                snap_file: &snap_file,
            },
        )? {
            let _ = write_text(&unchanged_file, "0");
            for line in lines {
                println!("{line}");
            }
            return Ok(());
        }
    }

    let prev_snap = if snap_file.is_file() {
        read_text(&snap_file).unwrap_or_default().trim().to_string()
    } else {
        String::new()
    };

    let _ = write_text(&snap_file, &current_snap);

    if current_snap == prev_snap {
        let mut unchanged = if unchanged_file.is_file() {
            read_text(&unchanged_file)
                .ok()
                .and_then(|s| s.trim().parse::<i64>().ok())
                .unwrap_or(0)
        } else {
            0
        };
        unchanged += 1;
        let _ = write_text(&unchanged_file, &unchanged.to_string());

        let trigger_rounds = cfg.safe_backlog_trigger_no_progress_rounds.max(1);
        let mut advisory_lines: Vec<String> = Vec::new();

        if let Some(advice) =
            generate_supervisor_advice(fusion_dir, &cfg, unchanged, counts, pending_like)?
        {
            advisory_lines.push(advice.line.clone());
            let current_phase = json_get_string(&snapshot, &["current_phase"])
                .unwrap_or_else(|| "EXECUTE".to_string());
            let key = format!(
                "supervisor:{current_snap}:{unchanged}:{}",
                (advice.risk_score.clamp(0.0, 1.0) * 1000.0).round() as i64
            );
            let _ = append_event(
                fusion_dir,
                "SUPERVISOR_ADVISORY",
                &current_phase,
                &current_phase,
                advice.payload,
                &key,
            );
        }

        if unchanged >= trigger_rounds && cfg.safe_backlog_enabled {
            if let Some(lines) = try_inject_safe_backlog(
                fusion_dir,
                &snapshot,
                &cfg,
                SafeBacklogTrigger {
                    counts: &counts,
                    pending_like,
                    current_snap: &current_snap,
                    reason: "no_progress",
                    no_progress_rounds: unchanged,
                    snap_file: &snap_file,
                },
            )? {
                let _ = write_text(&unchanged_file, "0");
                for line in lines {
                    println!("{line}");
                }
                return Ok(());
            }
        }

        for line in advisory_lines {
            println!("{line}");
        }
        return Ok(());
    }

    reset_safe_backlog_backoff(fusion_dir)?;
    let _ = write_text(&unchanged_file, "0");

    let prev_parts: Vec<&str> = if prev_snap.is_empty() {
        vec!["0", "0", "0", "0"]
    } else {
        prev_snap.split(':').collect()
    };

    let prev_completed = prev_parts
        .first()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let prev_failed = prev_parts
        .get(3)
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);

    let completed_delta = counts.completed - prev_completed;
    let failed_delta = counts.failed - prev_failed;

    let mut lines: Vec<String> = Vec::new();
    if completed_delta > 0 {
        lines.push(format!(
            "[fusion] Task completed ({}/{total} done)",
            counts.completed
        ));
        if counts.pending_like() > 0 {
            lines.push(format!("[fusion] Next: {}", find_next_task(fusion_dir)?));
        } else {
            lines.push("[fusion] All tasks completed! Proceed to VERIFY phase.".to_string());
        }
    }

    if failed_delta > 0 {
        lines.push("[fusion] Task FAILED. Apply 3-Strike protocol.".to_string());
    }

    let sched = snapshot.get("_runtime").and_then(|v| v.get("scheduler"));
    if sched
        .and_then(|v| v.get("enabled"))
        .and_then(|v| v.as_bool())
        == Some(true)
    {
        let batch_id = sched
            .and_then(|v| v.get("current_batch_id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        if batch_id > 0 && completed_delta > 0 {
            lines.push(format!(
                "[fusion] Batch {batch_id} progress: +{completed_delta} tasks completed"
            ));
        }
    }

    for line in lines {
        println!("{line}");
    }

    Ok(())
}

fn evaluate_stop_guard(fusion_dir: &Path) -> Result<StopGuardOutput> {
    let sessions_path = fusion_dir.join("sessions.json");
    if !sessions_path.is_file() {
        return Ok(StopGuardOutput {
            decision: "allow".to_string(),
            should_block: false,
            reason: String::new(),
            system_message: String::new(),
            phase_corrected: false,
            events_dispatched: vec![],
        });
    }

    let mut snapshot = read_json(&sessions_path)?;
    let status = json_get_string(&snapshot, &["status"]).unwrap_or_default();

    if status != "in_progress" {
        return Ok(StopGuardOutput {
            decision: "allow".to_string(),
            should_block: false,
            reason: String::new(),
            system_message: String::new(),
            phase_corrected: false,
            events_dispatched: vec![],
        });
    }

    let counts = read_task_counts(fusion_dir)?;
    let total_remaining = counts.pending + counts.in_progress + counts.failed;
    let total = counts.total();
    let next_task = find_next_task(fusion_dir)?;

    let mut current_phase =
        json_get_string(&snapshot, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());
    let mut phase_corrected = false;
    let mut events_dispatched: Vec<String> = Vec::new();

    if current_phase == "EXECUTE" && total_remaining == 0 && counts.completed > 0 {
        json_set_string(&mut snapshot, "current_phase", "VERIFY");
        current_phase = "VERIFY".to_string();
        phase_corrected = true;
        events_dispatched.push("ALL_TASKS_DONE".to_string());
    } else if matches!(
        current_phase.as_str(),
        "VERIFY" | "REVIEW" | "COMMIT" | "DELIVER"
    ) && counts.pending > 0
    {
        json_set_string(&mut snapshot, "current_phase", "EXECUTE");
        if current_phase == "VERIFY" {
            events_dispatched.push("VERIFY_FAIL".to_string());
        } else if current_phase == "REVIEW" {
            events_dispatched.push("REVIEW_FAIL".to_string());
        } else {
            events_dispatched.push("ERROR_OCCURRED".to_string());
            events_dispatched.push("RECOVER".to_string());
        }
        current_phase = "EXECUTE".to_string();
        phase_corrected = true;
    }

    if phase_corrected {
        write_json_pretty(&sessions_path, &snapshot)?;
    }

    if total_remaining == 0 && counts.completed > 0 {
        return Ok(StopGuardOutput {
            decision: "allow".to_string(),
            should_block: false,
            reason: String::new(),
            system_message: String::new(),
            phase_corrected,
            events_dispatched,
        });
    }

    if total == 0
        && matches!(
            current_phase.as_str(),
            "INITIALIZE" | "ANALYZE" | "DECOMPOSE"
        )
    {
        let goal = json_get_string(&snapshot, &["goal"]).unwrap_or_else(|| "(not set)".to_string());
        return Ok(StopGuardOutput {
            decision: "block".to_string(),
            should_block: true,
            reason: format!(
                "Continue with task decomposition for goal: {}. Create .fusion/task_plan.md with tasks.",
                if goal.is_empty() { "(not set)" } else { &goal }
            ),
            system_message: format!("🔄 Fusion | Phase: {current_phase} | Create task_plan.md"),
            phase_corrected,
            events_dispatched,
        });
    }

    let goal = json_get_string(&snapshot, &["goal"]).unwrap_or_else(|| "(not set)".to_string());
    let mut reason = format!(
        "Continue executing the Fusion workflow.

Goal: {}
Phase: {}
Remaining: {} tasks
Next task: {}

Instructions:
1. Read .fusion/task_plan.md
2. Find next PENDING or IN_PROGRESS task
3. Execute based on task type:
   - implementation/verification → TDD flow (RED→GREEN→REFACTOR)
   - design/documentation/configuration/research → direct execution
4. Update task status to [COMPLETED]
5. Continue until all tasks done

Only ask user if 3-Strike exhausted.",
        if goal.is_empty() { "(not set)" } else { &goal },
        current_phase,
        total_remaining,
        next_task
    );

    if phase_corrected {
        reason.push_str(&format!(
            "

Note: Phase auto-corrected to {} based on task states.",
            current_phase
        ));
    }

    Ok(StopGuardOutput {
        decision: "block".to_string(),
        should_block: true,
        reason,
        system_message: format!(
            "🔄 Fusion | Phase: {} | Remaining: {} | Next: {}",
            current_phase, total_remaining, next_task
        ),
        phase_corrected,
        events_dispatched,
    })
}

fn cmd_hook_stop_guard(fusion_dir: &Path) -> Result<()> {
    let output = evaluate_stop_guard(fusion_dir)?;
    output_stop_guard(output)
}

fn cmd_hook_set_goal(fusion_dir: &Path, goal: &str) -> Result<()> {
    let sessions_path = fusion_dir.join("sessions.json");
    let mut data = if sessions_path.is_file() {
        read_json(&sessions_path)?
    } else {
        Value::Object(Map::new())
    };
    json_set_string(&mut data, "goal", goal);
    write_json_pretty(&sessions_path, &data)?;
    println!("Goal set: {}...", truncate_chars(goal.to_string(), 60));
    Ok(())
}

struct SafeBacklogTrigger<'a> {
    counts: &'a TaskCounts,
    pending_like: i64,
    current_snap: &'a str,
    reason: &'a str,
    no_progress_rounds: i64,
    snap_file: &'a Path,
}

fn try_inject_safe_backlog(
    fusion_dir: &Path,
    snapshot: &Value,
    cfg: &FlatConfig,
    trigger: SafeBacklogTrigger<'_>,
) -> Result<Option<Vec<String>>> {
    let project_root = fusion_dir
        .canonicalize()
        .ok()
        .and_then(|p| p.parent().map(|x| x.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let counts = trigger.counts;
    let pending_like = trigger.pending_like;
    let current_snap = trigger.current_snap;
    let reason = trigger.reason;
    let no_progress_rounds = trigger.no_progress_rounds;
    let snap_file = trigger.snap_file;

    let backlog_result = generate_safe_backlog(fusion_dir, &project_root, cfg)?;
    if backlog_result.blocked_by_backoff || backlog_result.added <= 0 {
        return Ok(None);
    }

    let current_phase =
        json_get_string(snapshot, &["current_phase"]).unwrap_or_else(|| "EXECUTE".to_string());
    let stall_score = compute_stall_score(no_progress_rounds, pending_like, counts.failed, reason);
    let payload = json!({
        "reason": reason,
        "stall_score": stall_score,
        "added": backlog_result.added,
        "tasks": backlog_result.tasks,
    });

    let key = format!(
        "safe_backlog:{}:{}:{}",
        reason, current_snap, backlog_result.added
    );

    let _ = append_event(
        fusion_dir,
        "SAFE_BACKLOG_INJECTED",
        &current_phase,
        &current_phase,
        payload,
        &key,
    );

    let latest_counts = read_task_counts(fusion_dir)?;
    let latest_snap = format!(
        "{}:{}:{}:{}",
        latest_counts.completed,
        latest_counts.pending,
        latest_counts.in_progress,
        latest_counts.failed
    );
    let _ = write_text(snap_file, &latest_snap);

    Ok(Some(vec![format!(
        "[fusion] Safe backlog injected: +{} task(s)",
        backlog_result.added
    )]))
}

fn compute_stall_score(
    no_progress_rounds: i64,
    pending_like: i64,
    failed_tasks: i64,
    reason: &str,
) -> f64 {
    let mut score = 0.2;

    if reason == "task_exhausted" {
        score += 0.45;
    }
    if reason == "no_progress" {
        score += (no_progress_rounds as f64 * 0.12).min(0.4);
    }

    if pending_like == 0 {
        score += 0.2;
    }
    if failed_tasks > 0 {
        score += (failed_tasks as f64 * 0.05).min(0.15);
    }

    score.clamp(0.0, 1.0)
}

fn parse_allowed_categories(raw: &str) -> HashSet<String> {
    raw.split(',')
        .map(|item| item.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn fingerprint(task: &SafeTask) -> String {
    let source = format!("{}|{}|{}", task.title, task.category, task.output);
    let mut hasher = Sha1::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn priority_score(
    task: &SafeTask,
    last_category: &str,
    category_counts: &HashMap<String, i64>,
) -> f64 {
    let base = match task.category.as_str() {
        "quality" => 0.82,
        "optimization" => 0.79,
        "documentation" => 0.72,
        _ => 0.65,
    };

    let rotation_bonus = if !task.category.is_empty() && task.category != last_category {
        0.08
    } else {
        0.0
    };
    let usage_count = *category_counts.get(&task.category).unwrap_or(&0) as f64;
    let repetition_penalty = (usage_count * 0.03).min(0.25);

    (base + rotation_bonus - repetition_penalty).clamp(0.1, 0.99)
}

fn candidate_tasks(project_root: &Path) -> Vec<SafeTask> {
    let mut candidates: Vec<SafeTask> = Vec::new();

    if project_root.join("README.md").exists() {
        candidates.push(SafeTask {
            title: "更新 README 快速开始说明".to_string(),
            category: "documentation".to_string(),
            task_type: "documentation".to_string(),
            execution: "Direct".to_string(),
            output: "README.md".to_string(),
            priority_score: None,
        });
    }

    if project_root.join("scripts/runtime/tests").exists() {
        candidates.push(SafeTask {
            title: "补充 runtime 回归测试清单".to_string(),
            category: "quality".to_string(),
            task_type: "verification".to_string(),
            execution: "TDD".to_string(),
            output: "scripts/runtime/tests".to_string(),
            priority_score: None,
        });
    }

    if project_root.join("scripts/runtime").exists() {
        candidates.push(SafeTask {
            title: "优化 runtime 热路径扫描开销".to_string(),
            category: "optimization".to_string(),
            task_type: "configuration".to_string(),
            execution: "Direct".to_string(),
            output: "scripts/runtime".to_string(),
            priority_score: None,
        });
    }

    if candidates.is_empty() {
        candidates.push(SafeTask {
            title: "整理实现说明与限制".to_string(),
            category: "documentation".to_string(),
            task_type: "documentation".to_string(),
            execution: "Direct".to_string(),
            output: "docs".to_string(),
            priority_score: None,
        });
    }

    candidates
}

fn load_safe_backlog_state(path: &Path) -> SafeBacklogState {
    if !path.is_file() {
        return SafeBacklogState::default();
    }

    read_text(path)
        .ok()
        .and_then(|text| serde_json::from_str::<SafeBacklogState>(&text).ok())
        .unwrap_or_default()
}

fn persist_safe_backlog_state(path: &Path, state: &SafeBacklogState) {
    if let Ok(text) = serde_json::to_string_pretty(state) {
        let _ = write_text(path, &text);
    }
}

fn append_task_plan(task_plan_path: &Path, tasks: &[SafeTask]) -> Result<()> {
    let original = read_text(task_plan_path)?;

    let mut existing_numbers: Vec<i64> = Vec::new();
    for line in original.lines() {
        if !line.starts_with("### Task ") {
            continue;
        }
        let prefix = line.split(':').next().unwrap_or("");
        let number = prefix.replace("### Task", "").trim().parse::<i64>().ok();
        if let Some(number) = number {
            existing_numbers.push(number);
        }
    }

    let mut next_index = existing_numbers.into_iter().max().unwrap_or(0) + 1;

    let mut chunks: Vec<String> = vec![original.trim_end_matches('\n').to_string()];
    if !chunks[0].is_empty() {
        chunks.push(String::new());
    }

    for task in tasks {
        chunks.push(format!(
            "### Task {next_index}: {} [PENDING] [SAFE_BACKLOG]",
            task.title
        ));
        chunks.push(format!("- Type: {}", task.task_type));
        chunks.push(format!("- Execution: {}", task.execution));
        chunks.push("- Dependencies: []".to_string());
        chunks.push(format!("- Category: {}", task.category));
        chunks.push(format!("- Output: {}", task.output));
        chunks.push(String::new());
        next_index += 1;
    }

    let merged = format!("{}\n", chunks.join("\n").trim_end_matches('\n'));
    write_text(task_plan_path, &merged)?;
    Ok(())
}

fn generate_safe_backlog(
    fusion_dir: &Path,
    project_root: &Path,
    cfg: &FlatConfig,
) -> Result<SafeBacklogResult> {
    let mut result = SafeBacklogResult {
        enabled: cfg.safe_backlog_enabled,
        added: 0,
        tasks: vec![],
        blocked_by_backoff: false,
        backoff_state: SafeBackoffState::default(),
    };

    let task_plan_path = fusion_dir.join("task_plan.md");
    let state_path = fusion_dir.join("safe_backlog.json");

    if !cfg.safe_backlog_enabled || !task_plan_path.is_file() {
        return Ok(result);
    }

    let allowed = parse_allowed_categories(&cfg.safe_backlog_allowed_categories);
    let limit = cfg.safe_backlog_max_tasks_per_run.max(1) as usize;
    let novelty_window = cfg.safe_backlog_novelty_window.max(1) as usize;

    let mut state = load_safe_backlog_state(&state_path);
    let seen_slice_start = state.fingerprints.len().saturating_sub(novelty_window);
    let seen: HashSet<String> = state.fingerprints[seen_slice_start..]
        .iter()
        .cloned()
        .collect();

    let mut backoff = state.backoff.clone();
    let base_rounds = cfg.safe_backlog_backoff_base_rounds.max(1);
    let max_rounds = cfg
        .safe_backlog_backoff_max_rounds
        .max(cfg.safe_backlog_backoff_base_rounds.max(1));
    let jitter = cfg.safe_backlog_backoff_jitter.clamp(0.0, 1.0);
    let force_probe_rounds = cfg.safe_backlog_backoff_force_probe_rounds.max(1);

    backoff.attempt_round += 1;

    if cfg.safe_backlog_backoff_enabled
        && backoff.attempt_round <= backoff.cooldown_until_round
        && (backoff.attempt_round % force_probe_rounds != 0)
    {
        state.backoff = backoff.clone();
        persist_safe_backlog_state(&state_path, &state);
        result.blocked_by_backoff = true;
        result.backoff_state = backoff;
        return Ok(result);
    }

    let mut candidates = candidate_tasks(project_root);
    for candidate in &mut candidates {
        let score = priority_score(
            candidate,
            &state.last_category,
            &state.stats.category_counts,
        );
        candidate.priority_score = Some((score * 10000.0).round() / 10000.0);
    }

    if cfg.safe_backlog_diversity_rotation && !state.last_category.is_empty() {
        let mut rotated: Vec<SafeTask> = candidates
            .iter()
            .filter(|task| task.category != state.last_category)
            .cloned()
            .collect();
        if !rotated.is_empty() {
            rotated.extend(
                candidates
                    .iter()
                    .filter(|task| task.category == state.last_category)
                    .cloned(),
            );
            candidates = rotated;
        }
    }

    candidates.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut selected: Vec<SafeTask> = Vec::new();
    let mut added_fingerprints: Vec<String> = Vec::new();

    for candidate in candidates {
        if !allowed.is_empty() && !allowed.contains(&candidate.category.to_lowercase()) {
            continue;
        }
        let fp = fingerprint(&candidate);
        if seen.contains(&fp) {
            continue;
        }

        selected.push(candidate);
        added_fingerprints.push(fp);
        if selected.len() >= limit {
            break;
        }
    }

    if selected.is_empty() {
        if cfg.safe_backlog_backoff_enabled {
            backoff.consecutive_failures += 1;
            backoff.consecutive_injections = 0;
            let mut cooldown = (base_rounds
                * (2_i64.pow((backoff.consecutive_failures - 1).max(0) as u32)))
            .min(max_rounds);
            if jitter > 0.0 {
                let factor = rand::thread_rng().gen_range((1.0 - jitter)..=(1.0 + jitter));
                cooldown = ((cooldown as f64) * factor).round().max(1.0) as i64;
            }
            backoff.cooldown_until_round = backoff.attempt_round + cooldown;
        }

        state.backoff = backoff.clone();
        persist_safe_backlog_state(&state_path, &state);
        result.backoff_state = backoff;
        return Ok(result);
    }

    append_task_plan(&task_plan_path, &selected)?;

    state.fingerprints.extend(added_fingerprints);
    if state.fingerprints.len() > novelty_window {
        let start = state.fingerprints.len() - novelty_window;
        state.fingerprints = state.fingerprints[start..].to_vec();
    }

    state.last_category = selected
        .last()
        .map(|task| task.category.clone())
        .unwrap_or_default();

    state.stats.total_injections += selected.len() as i64;
    for task in &selected {
        *state
            .stats
            .category_counts
            .entry(task.category.clone())
            .or_insert(0) += 1;
    }

    if cfg.safe_backlog_backoff_enabled {
        backoff.consecutive_failures = 0;
        backoff.consecutive_injections += 1;
        let mut cooldown = (base_rounds
            * (2_i64.pow((backoff.consecutive_injections - 1).max(0) as u32)))
        .min(max_rounds);
        if jitter > 0.0 {
            let factor = rand::thread_rng().gen_range((1.0 - jitter)..=(1.0 + jitter));
            cooldown = ((cooldown as f64) * factor).round().max(1.0) as i64;
        }
        backoff.cooldown_until_round = backoff.attempt_round + cooldown;
    }

    state.backoff = backoff.clone();
    persist_safe_backlog_state(&state_path, &state);

    result.added = selected.len() as i64;
    result.tasks = selected;
    result.backoff_state = backoff;
    Ok(result)
}

fn reset_safe_backlog_backoff(fusion_dir: &Path) -> Result<()> {
    let state_path = fusion_dir.join("safe_backlog.json");
    let mut state = load_safe_backlog_state(&state_path);
    state.backoff.consecutive_failures = 0;
    state.backoff.consecutive_injections = 0;
    state.backoff.cooldown_until_round = 0;
    persist_safe_backlog_state(&state_path, &state);
    Ok(())
}

fn build_supervisor_suggestions(
    no_progress_rounds: i64,
    counts: TaskCounts,
    pending_like: i64,
    max_suggestions: i64,
) -> Vec<SupervisorSuggestion> {
    let mut suggestions: Vec<SupervisorSuggestion> = Vec::new();

    if counts.failed > 0 {
        suggestions.push(SupervisorSuggestion {
            category: "quality".to_string(),
            title: "先收敛失败任务并补最小回归用例".to_string(),
            rationale: "失败任务会放大停滞，先修复失败路径可以最快恢复主循环。".to_string(),
        });
    }

    if pending_like > 0 {
        suggestions.push(SupervisorSuggestion {
            category: "documentation".to_string(),
            title: "为当前 IN_PROGRESS 任务补充完成判据".to_string(),
            rationale: "明确完成标准能减少反复修改导致的无进展回合。".to_string(),
        });
    }

    if no_progress_rounds >= 4 {
        suggestions.push(SupervisorSuggestion {
            category: "optimization".to_string(),
            title: "执行一次低风险热路径体检并记录基线".to_string(),
            rationale: "避免在同一路径重复试错，先用基线定位瓶颈再继续开发。".to_string(),
        });
    }

    if suggestions.is_empty() {
        suggestions.push(SupervisorSuggestion {
            category: "documentation".to_string(),
            title: "整理当前阶段的假设与限制".to_string(),
            rationale: "在不改变业务行为的前提下沉淀上下文，降低后续漂移风险。".to_string(),
        });
    }

    let mut unique: Vec<SupervisorSuggestion> = Vec::new();
    let mut seen_titles: HashSet<String> = HashSet::new();
    for suggestion in suggestions {
        if seen_titles.insert(suggestion.title.clone()) {
            unique.push(suggestion);
        }
        if unique.len() >= max_suggestions.max(1) as usize {
            break;
        }
    }

    unique
}

fn suggestion_digest(suggestions: &[SupervisorSuggestion]) -> String {
    let source = suggestions
        .iter()
        .map(|s| format!("{}:{}", s.category, s.title))
        .collect::<Vec<_>>()
        .join("|");
    let mut hasher = Sha1::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn generate_supervisor_advice(
    fusion_dir: &Path,
    cfg: &FlatConfig,
    no_progress_rounds: i64,
    counts: TaskCounts,
    pending_like: i64,
) -> Result<Option<SupervisorAdvice>> {
    if !cfg.supervisor_enabled {
        return Ok(None);
    }

    let mut mode = cfg.supervisor_mode.trim().to_lowercase();
    if mode != "advisory" {
        mode = "advisory".to_string();
    }

    let trigger_rounds = cfg.supervisor_trigger_no_progress_rounds.max(1);
    let cadence_rounds = cfg.supervisor_cadence_rounds.max(1);
    let force_emit_rounds = cfg.supervisor_force_emit_rounds.max(1);
    let max_suggestions = cfg.supervisor_max_suggestions.max(1);
    let persona = if cfg.supervisor_persona.trim().is_empty() {
        "Guardian".to_string()
    } else {
        cfg.supervisor_persona.trim().to_string()
    };

    if no_progress_rounds < trigger_rounds {
        return Ok(None);
    }

    let state_path = fusion_dir.join("supervisor_state.json");
    let mut state = if state_path.is_file() {
        read_text(&state_path)
            .ok()
            .and_then(|s| serde_json::from_str::<SupervisorState>(&s).ok())
            .unwrap_or_default()
    } else {
        SupervisorState::default()
    };

    if state.last_advice_round > 0
        && no_progress_rounds - state.last_advice_round < cadence_rounds
        && (no_progress_rounds % force_emit_rounds != 0)
    {
        return Ok(None);
    }

    let suggestions =
        build_supervisor_suggestions(no_progress_rounds, counts, pending_like, max_suggestions);
    if suggestions.is_empty() {
        return Ok(None);
    }

    let digest = suggestion_digest(&suggestions);
    if digest == state.last_digest && (no_progress_rounds % force_emit_rounds != 0) {
        return Ok(None);
    }

    let denominator = (trigger_rounds * 3).max(1) as f64;
    let stagnation_score = (no_progress_rounds as f64 / denominator).clamp(0.0, 1.0);
    let failed = counts.failed.max(0) as f64;
    let repeat_denominator = (pending_like + counts.failed).max(1) as f64;
    let repeat_pressure = (failed / repeat_denominator).clamp(0.0, 1.0);
    let risk_score = (0.65 * stagnation_score + 0.35 * repeat_pressure).clamp(0.0, 1.0);

    let lead = suggestions
        .first()
        .map(|s| s.title.clone())
        .unwrap_or_else(|| "收敛当前任务".to_string());

    let line = format!(
        "[fusion][{}] Advisory: no-progress={}, risk={:.2}, next={}",
        persona, no_progress_rounds, risk_score, lead
    );

    let payload = json!({
        "mode": mode,
        "persona": persona,
        "no_progress_rounds": no_progress_rounds,
        "stagnation_score": stagnation_score,
        "repeat_pressure": repeat_pressure,
        "risk_score": (risk_score * 1000.0).round() / 1000.0,
        "suggestions": suggestions,
    });

    state.last_advice_round = no_progress_rounds;
    state.last_digest = digest;
    state.last_risk_score = risk_score;
    state.updated_at = Some(chrono::Utc::now().timestamp_millis() as f64 / 1000.0);

    if let Ok(text) = serde_json::to_string_pretty(&state) {
        let _ = write_text(&state_path, &text);
    }

    Ok(Some(SupervisorAdvice {
        line,
        payload,
        risk_score,
    }))
}

fn output_stop_guard(output: StopGuardOutput) -> Result<()> {
    let json = serde_json::to_string(&output)?;
    println!("{json}");
    Ok(())
}

fn phase_num(phase: &str) -> &'static str {
    match phase {
        "UNDERSTAND" => "0/8",
        "INITIALIZE" => "1/8",
        "ANALYZE" => "2/8",
        "DECOMPOSE" => "3/8",
        "EXECUTE" => "4/8",
        "VERIFY" => "5/8",
        "REVIEW" => "6/8",
        "COMMIT" => "7/8",
        "DELIVER" => "8/8",
        _ => "?/8",
    }
}

fn truncate_chars(input: String, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}

fn read_task_counts(fusion_dir: &Path) -> Result<TaskCounts> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok(TaskCounts::default());
    }
    let content = read_text(&task_plan)?;
    Ok(TaskCounts {
        completed: content.matches("[COMPLETED]").count() as i64,
        pending: content.matches("[PENDING]").count() as i64,
        in_progress: content.matches("[IN_PROGRESS]").count() as i64,
        failed: content.matches("[FAILED]").count() as i64,
    })
}

fn find_next_task(fusion_dir: &Path) -> Result<String> {
    let task_plan = fusion_dir.join("task_plan.md");
    if !task_plan.is_file() {
        return Ok("unknown".to_string());
    }

    let content = read_text(&task_plan)?;
    for line in content.lines() {
        if !(line.contains("[IN_PROGRESS]") || line.contains("[PENDING]")) {
            continue;
        }
        if !line.contains("### Task") {
            continue;
        }

        let mut name = if let Some((_, right)) = line.split_once(':') {
            right.trim().to_string()
        } else {
            line.to_string()
        };

        for tag in ["[IN_PROGRESS]", "[PENDING]", "[COMPLETED]", "[FAILED]"] {
            name = name.replace(tag, "").trim().to_string();
        }

        if !name.is_empty() {
            return Ok(name);
        }
    }

    Ok("unknown".to_string())
}

fn render_prompt(fusion_dir: &Path, phase: &str, goal: &str) -> Result<String> {
    let task_plan_path = fusion_dir.join("task_plan.md");
    let task_plan = if task_plan_path.is_file() {
        read_text(&task_plan_path)?
    } else {
        String::new()
    };

    Ok(format!(
        "[Fusion Runner]\nPhase: {phase}\nGoal: {goal}\n\n请在当前仓库执行下一步工作，并更新：\n1) .fusion/task_plan.md\n2) .fusion/progress.md\n\n当前 task_plan 内容：\n{task_plan}"
    ))
}

fn extract_status_block(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let idx = lines
        .iter()
        .position(|line| line.starts_with("## Status"))?;
    let end = usize::min(idx + 6, lines.len());
    let block = lines[idx..end].join("\n");
    Some(format!("{block}\n"))
}

fn last_safe_backlog(events_file: &Path) -> Result<Option<(i64, f64)>> {
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

fn epoch_to_iso(timestamp: f64) -> Option<String> {
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
    use tempfile::tempdir;

    #[test]
    fn test_extract_status_block() {
        let input = "A\n## Status\n- a\n- b\n- c\n- d\n- e\n- f\n";
        let block = extract_status_block(input).expect("status block");
        assert!(block.contains("## Status"));
        assert!(!block.contains("- f"));
    }

    #[test]
    fn test_render_prompt_contains_task_plan() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(dir.path().join("task_plan.md"), "### Task 1: A [PENDING]\n")
            .expect("write task plan");

        let prompt = render_prompt(dir.path(), "EXECUTE", "my goal").expect("render prompt");
        assert!(prompt.contains("Phase: EXECUTE"));
        assert!(prompt.contains("my goal"));
        assert!(prompt.contains("Task 1"));
    }

    #[test]
    fn test_compute_stall_score_bounds() {
        let score = compute_stall_score(10, 0, 4, "no_progress");
        assert!((0.0..=1.0).contains(&score));
    }
}
