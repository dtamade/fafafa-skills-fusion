use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fusion-bridge")]
#[command(about = "Fusion Rust bridge binary", long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    Init {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        #[arg(long, default_value = "templates")]
        templates_dir: PathBuf,
        #[arg(long, default_value = "rust")]
        engine: String,
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
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Logs {
        #[arg(default_value_t = 50)]
        lines: usize,
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Achievements {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        #[arg(long, default_value_t = false)]
        local_only: bool,
        #[arg(long, default_value_t = false)]
        leaderboard_only: bool,
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long, default_value_t = 10)]
        top: usize,
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
    Pause {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Cancel {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Continue {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Catchup {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        #[arg(long)]
        project_path: Option<PathBuf>,
    },
    Codeagent {
        #[arg(default_value = "EXECUTE")]
        phase: String,
        #[arg(trailing_var_arg = true)]
        prompt: Vec<String>,
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Doctor {
        #[arg(long, default_value_t = false)]
        json: bool,
        #[arg(long, default_value_t = false)]
        fix: bool,
        project_root: Option<PathBuf>,
    },
    Audit {
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
        #[arg(long, default_value_t = false)]
        json_pretty: bool,
        #[arg(long, default_value_t = false)]
        fast: bool,
        #[arg(long, default_value_t = false)]
        skip_rust: bool,
    },
    Selfcheck {
        #[arg(long, default_value_t = false)]
        fix: bool,
        #[arg(long, default_value_t = false)]
        quick: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
        project_root: Option<PathBuf>,
    },
    Regression {
        #[arg(long, default_value = "all")]
        suite: String,
        #[arg(long)]
        scenario: Option<String>,
        #[arg(long, default_value_t = 20)]
        runs: usize,
        #[arg(long, default_value_t = 0.99)]
        min_pass_rate: f64,
        #[arg(long, default_value_t = false)]
        list_suites: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    LoopGuardian {
        #[command(subcommand)]
        command: LoopGuardianCommands,
    },
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
    Inspect {
        #[command(subcommand)]
        command: InspectCommands,
    },
    Hook {
        #[command(subcommand)]
        command: HookCommands,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum InspectCommands {
    JsonField {
        #[arg(long)]
        key: String,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        number: bool,
        #[arg(long, default_value_t = false)]
        bool: bool,
    },
    RuntimeConfig {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        #[arg(long)]
        field: String,
    },
    LoopContext {
        #[arg(long)]
        file: PathBuf,
        #[command(subcommand)]
        query: LoopContextInspectCommands,
    },
    LoopGuardianConfig {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        #[arg(long)]
        field: String,
    },
    TaskPlan {
        #[arg(long)]
        file: PathBuf,
        #[command(subcommand)]
        query: TaskPlanInspectCommands,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum TaskPlanInspectCommands {
    Counts,
    First {
        #[arg(long)]
        status: String,
    },
    Last {
        #[arg(long)]
        status: String,
    },
    Next,
    TaskType {
        #[arg(long)]
        title: String,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum LoopContextInspectCommands {
    ArrayValues {
        #[arg(long)]
        key: String,
    },
    StateVisits,
    DecisionHistory,
}

#[derive(Subcommand, Debug)]
pub(crate) enum HookCommands {
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

#[derive(Subcommand, Debug)]
pub(crate) enum LoopGuardianCommands {
    Init {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Record {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        phase: String,
        task: String,
        error: String,
    },
    Get {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
        key: String,
    },
    Evaluate {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Status {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
    Reset {
        #[arg(long, default_value = ".fusion")]
        fusion_dir: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum GitCommands {
    Status,
    CreateBranch {
        goal_slug: String,
    },
    Commit {
        message: String,
        task_id: Option<String>,
    },
    Branch,
    Changes,
    Diff,
    Cleanup {
        original_branch: Option<String>,
    },
}
