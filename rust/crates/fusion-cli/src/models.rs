use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TaskCounts {
    pub(crate) completed: i64,
    pub(crate) pending: i64,
    pub(crate) in_progress: i64,
    pub(crate) failed: i64,
}

impl TaskCounts {
    pub(crate) fn total(&self) -> i64 {
        self.completed + self.pending + self.in_progress + self.failed
    }

    pub(crate) fn pending_like(&self) -> i64 {
        self.pending + self.in_progress
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SafeTask {
    pub(crate) title: String,
    pub(crate) category: String,
    #[serde(rename = "type")]
    pub(crate) task_type: String,
    pub(crate) execution: String,
    pub(crate) output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) priority_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SafeBacklogResult {
    pub(crate) enabled: bool,
    pub(crate) added: i64,
    pub(crate) tasks: Vec<SafeTask>,
    pub(crate) blocked_by_backoff: bool,
    pub(crate) backoff_state: SafeBackoffState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SafeBackoffState {
    pub(crate) consecutive_failures: i64,
    pub(crate) consecutive_injections: i64,
    pub(crate) cooldown_until_round: i64,
    pub(crate) attempt_round: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SafeBacklogStats {
    pub(crate) total_injections: i64,
    pub(crate) category_counts: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SafeBacklogState {
    pub(crate) fingerprints: Vec<String>,
    pub(crate) last_category: String,
    pub(crate) stats: SafeBacklogStats,
    pub(crate) backoff: SafeBackoffState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SupervisorState {
    pub(crate) last_advice_round: i64,
    pub(crate) last_digest: String,
    pub(crate) last_risk_score: f64,
    pub(crate) updated_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SupervisorSuggestion {
    pub(crate) category: String,
    pub(crate) title: String,
    pub(crate) rationale: String,
}

#[derive(Debug, Clone)]
pub(crate) struct SupervisorAdvice {
    pub(crate) line: String,
    pub(crate) payload: Value,
    pub(crate) risk_score: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct RunOptions {
    pub(crate) max_iterations: i64,
    pub(crate) max_no_progress_rounds: i64,
    pub(crate) initial_backoff_ms: u64,
    pub(crate) max_backoff_ms: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct CodeagentExecution {
    pub(crate) output: String,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Serialize)]
pub(crate) struct StopGuardOutput {
    pub(crate) decision: String,
    pub(crate) should_block: bool,
    pub(crate) reason: String,
    #[serde(rename = "systemMessage")]
    pub(crate) system_message: String,
    pub(crate) phase_corrected: bool,
    pub(crate) events_dispatched: Vec<String>,
}
