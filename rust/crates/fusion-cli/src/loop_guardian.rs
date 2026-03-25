use anyhow::Result;
use fusion_runtime_io::{read_text, write_text};
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli::LoopGuardianCommands;
use crate::render::read_task_counts;

const DEFAULT_MAX_ITERATIONS: i64 = 50;
const DEFAULT_MAX_NO_PROGRESS: i64 = 6;
const DEFAULT_MAX_SAME_ACTION: i64 = 3;
const DEFAULT_MAX_SAME_ERROR: i64 = 3;
const DEFAULT_MAX_STATE_VISITS: i64 = 8;
const DEFAULT_MAX_WALL_TIME_MS: i64 = 7_200_000;
const DEFAULT_BACKOFF_THRESHOLD: i64 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoopGuardianContext {
    iteration: i64,
    last_task_snapshot: String,
    last_completed_count: i64,
    last_action_signature: String,
    last_error_fingerprint: String,
    completed_count_history: Vec<i64>,
    action_signatures: Vec<String>,
    error_fingerprints: Vec<String>,
    state_visits: BTreeMap<String, i64>,
    started_at: i64,
    last_progress_at: i64,
    total_iterations: i64,
    no_progress_rounds: i64,
    same_action_count: i64,
    same_error_count: i64,
    wall_time_ms: i64,
    max_state_visit_count: i64,
    metrics: GuardianMetrics,
    decision_history: Vec<DecisionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GuardianMetrics {
    total_iterations: i64,
    no_progress_rounds: i64,
    same_action_count: i64,
    same_error_count: i64,
    wall_time_ms: i64,
    max_state_visit_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DecisionEntry {
    decision: String,
    reason: String,
    timestamp: i64,
}

#[derive(Debug, Clone)]
struct GuardianConfig {
    max_iterations: i64,
    max_no_progress: i64,
    max_same_action: i64,
    max_same_error: i64,
    max_state_visits: i64,
    max_wall_time_ms: i64,
    backoff_threshold: i64,
}

pub(crate) fn dispatch_loop_guardian(command: LoopGuardianCommands) -> Result<()> {
    match command {
        LoopGuardianCommands::Init { fusion_dir } => cmd_init(&fusion_dir),
        LoopGuardianCommands::Record {
            fusion_dir,
            phase,
            task,
            error,
        } => cmd_record(&fusion_dir, &phase, &task, &error),
        LoopGuardianCommands::Get { fusion_dir, key } => cmd_get(&fusion_dir, &key),
        LoopGuardianCommands::Evaluate { fusion_dir } => cmd_evaluate(&fusion_dir),
        LoopGuardianCommands::Status { fusion_dir } => cmd_status(&fusion_dir),
        LoopGuardianCommands::Reset { fusion_dir } => cmd_reset(&fusion_dir),
    }
}

fn cmd_init(fusion_dir: &Path) -> Result<()> {
    fs::create_dir_all(fusion_dir)?;
    let context_path = fusion_dir.join("loop_context.json");
    if !context_path.is_file() {
        write_context(&context_path, &default_context(now_ms()))?;
    }
    Ok(())
}

fn cmd_record(fusion_dir: &Path, phase: &str, task: &str, error: &str) -> Result<()> {
    cmd_init(fusion_dir)?;
    let config = load_config(fusion_dir);
    let context_path = fusion_dir.join("loop_context.json");
    let mut ctx = read_context(&context_path)?;
    let now = now_ms();
    let snapshot = compute_task_snapshot(fusion_dir)?;
    let completed_count = snapshot
        .split(':')
        .next()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let previous_iterations = ctx.total_iterations;
    let previous_completed = ctx.last_completed_count;
    let action_sig = short_hash(&format!("{phase}:{task}"));
    let error_fp = if error.is_empty() {
        String::new()
    } else {
        short_hash(error)
    };

    ctx.iteration += 1;
    ctx.total_iterations += 1;
    ctx.wall_time_ms = now - ctx.started_at;
    push_i64(&mut ctx.completed_count_history, completed_count, 10);
    if previous_iterations == 0 || completed_count > previous_completed {
        ctx.no_progress_rounds = 0;
        ctx.last_progress_at = now;
    } else {
        ctx.no_progress_rounds += 1;
    }
    ctx.last_completed_count = completed_count;

    if !ctx.last_action_signature.is_empty() && ctx.last_action_signature == action_sig {
        ctx.same_action_count += 1;
    } else {
        ctx.same_action_count = 1;
    }
    ctx.last_action_signature = action_sig.clone();
    push_string(&mut ctx.action_signatures, action_sig, 5);

    if error_fp.is_empty() {
        ctx.same_error_count = 0;
        ctx.last_error_fingerprint.clear();
    } else {
        if !ctx.last_error_fingerprint.is_empty() && ctx.last_error_fingerprint == error_fp {
            ctx.same_error_count += 1;
        } else {
            ctx.same_error_count = 1;
        }
        ctx.last_error_fingerprint = error_fp.clone();
        push_string(&mut ctx.error_fingerprints, error_fp, 5);
    }

    let visits = ctx.state_visits.entry(phase.to_string()).or_insert(0);
    *visits += 1;
    ctx.max_state_visit_count = ctx.state_visits.values().copied().max().unwrap_or(0);
    ctx.last_task_snapshot = snapshot;
    ctx.metrics = metrics_from_ctx(&ctx);
    let _ = config;

    write_context(&context_path, &ctx)
}

fn cmd_get(fusion_dir: &Path, key: &str) -> Result<()> {
    let context_path = fusion_dir.join("loop_context.json");
    if !context_path.is_file() {
        println!();
        return Ok(());
    }
    let ctx = read_context(&context_path)?;
    let key = key.trim_start_matches('.');
    match key {
        "iteration" => println!("{}", ctx.iteration),
        "last_task_snapshot" => println!("{}", ctx.last_task_snapshot),
        "last_completed_count" => println!("{}", ctx.last_completed_count),
        "last_action_signature" => println!("{}", ctx.last_action_signature),
        "last_error_fingerprint" => println!("{}", ctx.last_error_fingerprint),
        "started_at" => println!("{}", ctx.started_at),
        "last_progress_at" => println!("{}", ctx.last_progress_at),
        "total_iterations" | "metrics.total_iterations" => println!("{}", ctx.total_iterations),
        "no_progress_rounds" | "metrics.no_progress_rounds" => {
            println!("{}", ctx.no_progress_rounds)
        }
        "same_action_count" | "metrics.same_action_count" => println!("{}", ctx.same_action_count),
        "same_error_count" | "metrics.same_error_count" => println!("{}", ctx.same_error_count),
        "wall_time_ms" | "metrics.wall_time_ms" => println!("{}", ctx.wall_time_ms),
        "max_state_visit_count" | "metrics.max_state_visit_count" => {
            println!("{}", ctx.max_state_visit_count)
        }
        _ if key.starts_with("state_visits.") => {
            let state = key.trim_start_matches("state_visits.");
            println!("{}", ctx.state_visits.get(state).copied().unwrap_or(0));
        }
        _ => println!(),
    }
    Ok(())
}

fn cmd_evaluate(fusion_dir: &Path) -> Result<()> {
    let config = load_config(fusion_dir);
    let context_path = fusion_dir.join("loop_context.json");
    if !context_path.is_file() {
        println!("CONTINUE");
        return Ok(());
    }
    let mut ctx = read_context(&context_path)?;

    let (decision, reason) = if ctx.total_iterations >= config.max_iterations {
        (
            "ABORT_STUCK",
            format!("Max iterations ({}) reached", config.max_iterations),
        )
    } else if ctx.wall_time_ms >= config.max_wall_time_ms {
        (
            "ABORT_STUCK",
            format!("Max wall time ({}ms) exceeded", config.max_wall_time_ms),
        )
    } else if ctx.no_progress_rounds >= config.max_no_progress {
        (
            "ABORT_STUCK",
            format!(
                "No progress for {} rounds (max: {})",
                ctx.no_progress_rounds, config.max_no_progress
            ),
        )
    } else if ctx.same_action_count >= config.max_same_action {
        (
            "ESCALATE",
            format!(
                "Same action repeated {} times (max: {})",
                ctx.same_action_count, config.max_same_action
            ),
        )
    } else if ctx.same_error_count >= config.max_same_error {
        (
            "ESCALATE",
            format!(
                "Same error repeated {} times (max: {})",
                ctx.same_error_count, config.max_same_error
            ),
        )
    } else if ctx.max_state_visit_count >= config.max_state_visits {
        (
            "ESCALATE",
            format!(
                "State visited {} times (max: {})",
                ctx.max_state_visit_count, config.max_state_visits
            ),
        )
    } else if ctx.no_progress_rounds >= config.backoff_threshold {
        (
            "BACKOFF",
            format!(
                "No progress for {} rounds, slowing down",
                ctx.no_progress_rounds
            ),
        )
    } else if ctx.same_action_count >= 2 {
        (
            "BACKOFF",
            format!(
                "Same action repeated {} times, slowing down",
                ctx.same_action_count
            ),
        )
    } else {
        ("CONTINUE", String::new())
    };

    if !reason.is_empty() {
        push_decision(
            &mut ctx.decision_history,
            DecisionEntry {
                decision: decision.to_string(),
                reason,
                timestamp: now_secs(),
            },
            20,
        );
        ctx.metrics = metrics_from_ctx(&ctx);
        write_context(&context_path, &ctx)?;
    }

    println!("{decision}");
    Ok(())
}

fn cmd_status(fusion_dir: &Path) -> Result<()> {
    let config = load_config(fusion_dir);
    let context_path = fusion_dir.join("loop_context.json");
    if !context_path.is_file() {
        println!("LoopGuardian: not initialized");
        return Ok(());
    }
    let ctx = read_context(&context_path)?;
    println!("LoopGuardian Status:");
    println!(
        "  Iterations: {}/{}",
        ctx.total_iterations, config.max_iterations
    );
    println!(
        "  No-Progress Rounds: {}/{}",
        ctx.no_progress_rounds, config.max_no_progress
    );
    println!(
        "  Same Action Count: {}/{}",
        ctx.same_action_count, config.max_same_action
    );
    println!(
        "  Same Error Count: {}/{}",
        ctx.same_error_count, config.max_same_error
    );
    println!(
        "  State Visits: {}/{}",
        ctx.max_state_visit_count, config.max_state_visits
    );
    println!(
        "  Wall Time: {}s/{}s",
        ctx.wall_time_ms / 1000,
        config.max_wall_time_ms / 1000
    );
    Ok(())
}

fn cmd_reset(fusion_dir: &Path) -> Result<()> {
    let context_path = fusion_dir.join("loop_context.json");
    if context_path.exists() {
        fs::remove_file(&context_path)?;
    }
    cmd_init(fusion_dir)
}

fn load_config(fusion_dir: &Path) -> GuardianConfig {
    let path = fusion_dir.join("config.yaml");
    let raw = read_text(&path).unwrap_or_default();
    let yaml: YamlValue = serde_yaml::from_str(&raw).unwrap_or(YamlValue::Null);
    GuardianConfig {
        max_iterations: yaml_num(
            &yaml,
            &["loop_guardian", "max_iterations"],
            DEFAULT_MAX_ITERATIONS,
        ),
        max_no_progress: yaml_num(
            &yaml,
            &["loop_guardian", "max_no_progress"],
            DEFAULT_MAX_NO_PROGRESS,
        ),
        max_same_action: yaml_num(
            &yaml,
            &["loop_guardian", "max_same_action"],
            DEFAULT_MAX_SAME_ACTION,
        ),
        max_same_error: yaml_num(
            &yaml,
            &["loop_guardian", "max_same_error"],
            DEFAULT_MAX_SAME_ERROR,
        ),
        max_state_visits: yaml_num(
            &yaml,
            &["loop_guardian", "max_state_visits"],
            DEFAULT_MAX_STATE_VISITS,
        ),
        max_wall_time_ms: yaml_num(
            &yaml,
            &["loop_guardian", "max_wall_time_ms"],
            DEFAULT_MAX_WALL_TIME_MS,
        ),
        backoff_threshold: yaml_num(
            &yaml,
            &["loop_guardian", "backoff_threshold"],
            DEFAULT_BACKOFF_THRESHOLD,
        ),
    }
}

fn yaml_num(root: &YamlValue, path: &[&str], default: i64) -> i64 {
    let mut current = root;
    for segment in path {
        match current {
            YamlValue::Mapping(map) => {
                let key = YamlValue::String((*segment).to_string());
                current = map.get(&key).unwrap_or(&YamlValue::Null);
            }
            _ => return default,
        }
    }
    current.as_i64().unwrap_or(default)
}

fn compute_task_snapshot(fusion_dir: &Path) -> Result<String> {
    let counts = read_task_counts(fusion_dir)?;
    Ok(format!(
        "{}:{}:{}",
        counts.completed, counts.pending, counts.in_progress
    ))
}

fn read_context(path: &Path) -> Result<LoopGuardianContext> {
    Ok(serde_json::from_str(&read_text(path)?)?)
}

fn write_context(path: &Path, ctx: &LoopGuardianContext) -> Result<()> {
    write_text(path, &format!("{}\n", serde_json::to_string_pretty(ctx)?))
}

fn default_context(now: i64) -> LoopGuardianContext {
    LoopGuardianContext {
        iteration: 0,
        last_task_snapshot: String::new(),
        last_completed_count: 0,
        last_action_signature: String::new(),
        last_error_fingerprint: String::new(),
        completed_count_history: Vec::new(),
        action_signatures: Vec::new(),
        error_fingerprints: Vec::new(),
        state_visits: BTreeMap::new(),
        started_at: now,
        last_progress_at: now,
        total_iterations: 0,
        no_progress_rounds: 0,
        same_action_count: 0,
        same_error_count: 0,
        wall_time_ms: 0,
        max_state_visit_count: 0,
        metrics: GuardianMetrics {
            total_iterations: 0,
            no_progress_rounds: 0,
            same_action_count: 0,
            same_error_count: 0,
            wall_time_ms: 0,
            max_state_visit_count: 0,
        },
        decision_history: Vec::new(),
    }
}

fn metrics_from_ctx(ctx: &LoopGuardianContext) -> GuardianMetrics {
    GuardianMetrics {
        total_iterations: ctx.total_iterations,
        no_progress_rounds: ctx.no_progress_rounds,
        same_action_count: ctx.same_action_count,
        same_error_count: ctx.same_error_count,
        wall_time_ms: ctx.wall_time_ms,
        max_state_visit_count: ctx.max_state_visit_count,
    }
}

fn push_i64(items: &mut Vec<i64>, item: i64, limit: usize) {
    items.push(item);
    if items.len() > limit {
        let drain = items.len() - limit;
        items.drain(0..drain);
    }
}

fn push_string(items: &mut Vec<String>, item: String, limit: usize) {
    items.push(item);
    if items.len() > limit {
        let drain = items.len() - limit;
        items.drain(0..drain);
    }
}

fn push_decision(items: &mut Vec<DecisionEntry>, item: DecisionEntry, limit: usize) {
    items.push(item);
    if items.len() > limit {
        let drain = items.len() - limit;
        items.drain(0..drain);
    }
}

fn short_hash(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix epoch")
        .as_millis() as i64
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix epoch")
        .as_secs() as i64
}
