use anyhow::Result;
use fusion_runtime_io::{read_text, write_text, FlatConfig};
use serde_json::json;
use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::path::Path;

use crate::models::{SupervisorAdvice, SupervisorState, SupervisorSuggestion, TaskCounts};

pub(crate) fn generate_supervisor_advice(
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
            .and_then(|text| serde_json::from_str::<SupervisorState>(&text).ok())
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
        .map(|suggestion| suggestion.title.clone())
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
        .map(|suggestion| format!("{}:{}", suggestion.category, suggestion.title))
        .collect::<Vec<_>>()
        .join("|");
    let mut hasher = Sha1::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}
