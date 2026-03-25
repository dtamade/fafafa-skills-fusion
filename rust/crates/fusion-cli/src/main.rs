use clap::Parser;

mod achievements;
mod agent_handoff;
mod agent_orchestrator;
mod audit;
mod bootstrap;
mod bootstrap_config;
mod catchup;
mod catchup_render;
mod catchup_session;
mod catchup_taskplan;
mod cli;
mod dispatch;
mod doctor;
mod git;
mod hooks;
mod inspect;
mod loop_guardian;
mod models;
mod posttool;
mod posttool_progress;
mod posttool_runtime;
mod pretool;
mod regression;
mod render;
mod render_status;
mod render_taskplan;
mod render_tasks;
mod reporting;
mod runner;
mod runner_backend;
mod runner_control;
mod runner_route;
mod safe_backlog;
mod safe_backlog_core;
mod safe_backlog_support;
mod selfcheck;
mod status;
mod status_artifacts;
mod status_cmd;
mod status_owner;
mod status_render;
mod status_reports;
mod status_runtime;
mod stop_guard;
mod supervisor;

use audit::cmd_audit;
use bootstrap::{cmd_init, cmd_start};
use catchup::cmd_catchup;
use cli::{Cli, Commands};
use dispatch::{dispatch_git, dispatch_hook, dispatch_inspect};
use doctor::cmd_doctor;
use loop_guardian::dispatch_loop_guardian;
use models::RunOptions;
use regression::cmd_regression;
use reporting::{cmd_achievements, cmd_logs};
use runner::{cmd_codeagent, cmd_run};
use runner_control::{cmd_cancel, cmd_continue, cmd_pause, cmd_resume};
use selfcheck::cmd_selfcheck;
use status_cmd::cmd_status;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init {
            fusion_dir,
            templates_dir,
            engine,
        } => cmd_init(&fusion_dir, &templates_dir, &engine),
        Commands::Start {
            goal,
            fusion_dir,
            templates_dir,
            force,
            yolo,
        } => cmd_start(&fusion_dir, &templates_dir, &goal, force || yolo),
        Commands::Status { fusion_dir, json } => cmd_status(&fusion_dir, json),
        Commands::Logs { fusion_dir, lines } => cmd_logs(&fusion_dir, lines),
        Commands::Achievements {
            fusion_dir,
            local_only,
            leaderboard_only,
            root,
            top,
        } => cmd_achievements(
            &fusion_dir,
            local_only,
            leaderboard_only,
            root.as_deref(),
            top,
        ),
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
        Commands::Pause { fusion_dir } => cmd_pause(&fusion_dir),
        Commands::Cancel { fusion_dir } => cmd_cancel(&fusion_dir),
        Commands::Continue { fusion_dir } => cmd_continue(&fusion_dir),
        Commands::Catchup {
            fusion_dir,
            project_path,
        } => cmd_catchup(&fusion_dir, project_path.as_deref()),
        Commands::Codeagent {
            phase,
            prompt,
            fusion_dir,
        } => cmd_codeagent(&fusion_dir, &phase, &prompt),
        Commands::Doctor {
            json,
            fix,
            project_root,
        } => cmd_doctor(project_root.as_deref(), json, fix),
        Commands::Audit {
            dry_run,
            json,
            json_pretty,
            fast,
            skip_rust,
        } => cmd_audit(dry_run, json, json_pretty, fast, skip_rust),
        Commands::Selfcheck {
            fix,
            quick,
            json,
            project_root,
        } => cmd_selfcheck(project_root.as_deref(), fix, quick, json),
        Commands::Regression {
            suite,
            scenario,
            runs,
            min_pass_rate,
            list_suites,
            json,
        } => cmd_regression(
            &suite,
            scenario.as_deref(),
            runs,
            min_pass_rate,
            json,
            list_suites,
        ),
        Commands::LoopGuardian { command } => dispatch_loop_guardian(command),
        Commands::Git { command } => dispatch_git(command),
        Commands::Inspect { command } => dispatch_inspect(command),
        Commands::Hook { command } => dispatch_hook(command),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
