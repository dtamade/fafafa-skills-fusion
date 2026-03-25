use fusion_provider::{session_key_for_backend, session_key_for_backend_role};
use fusion_runtime_io::{json_get_string, FlatConfig};
use serde_json::Value;

use crate::render::{backend_for_role, default_role_for_phase, normalize_role};

#[derive(Debug, Clone)]
pub(crate) struct CodeagentRoute {
    pub(crate) role: String,
    pub(crate) role_source: String,
    pub(crate) role_is_explicit: bool,
    pub(crate) primary: String,
    pub(crate) fallback: String,
    pub(crate) route_reason: String,
}

fn opposite_backend(backend: &str) -> String {
    if backend == "codex" {
        "claude".to_string()
    } else {
        "codex".to_string()
    }
}

pub(crate) fn resolve_codeagent_route(
    cfg: &FlatConfig,
    phase: &str,
    next_task_type: &str,
    next_task_owner: &str,
) -> CodeagentRoute {
    let mut role =
        normalize_role(&std::env::var("FUSION_AGENT_ROLE").unwrap_or_default()).unwrap_or_default();
    let mut role_is_explicit = false;
    let role_source = if !role.is_empty() {
        role_is_explicit = true;
        "env".to_string()
    } else if phase == "EXECUTE" {
        if let Some(owner_role) = normalize_role(next_task_owner) {
            role = owner_role;
            role_is_explicit = true;
            "task_owner".to_string()
        } else {
            role = default_role_for_phase(phase).to_string();
            "phase_default".to_string()
        }
    } else {
        role = default_role_for_phase(phase).to_string();
        "phase_default".to_string()
    };

    let mut primary = cfg
        .backend_phase_routing
        .get(phase)
        .cloned()
        .unwrap_or_default();
    let mut route_reason = if primary.is_empty() {
        String::new()
    } else {
        format!("phase:{phase}")
    };

    if primary.is_empty() && phase == "EXECUTE" && !next_task_type.is_empty() {
        if let Some(task_backend) = cfg.backend_task_type_routing.get(next_task_type) {
            primary = task_backend.clone();
            route_reason = format!("task_type:{next_task_type}");
        }
    }

    if primary != "codex" && primary != "claude" {
        primary = if cfg.backend_primary == "claude" {
            "claude".to_string()
        } else {
            "codex".to_string()
        };
        route_reason = format!("default:{primary}");
    }

    let mut fallback = cfg.backend_fallback.clone();
    if (fallback != "codex" && fallback != "claude") || fallback == primary {
        fallback = if cfg.backend_primary == "codex" || cfg.backend_primary == "claude" {
            if cfg.backend_primary != primary {
                cfg.backend_primary.clone()
            } else {
                opposite_backend(&primary)
            }
        } else {
            opposite_backend(&primary)
        };
    }

    if cfg.agent_enabled && cfg.agent_mode == "single_orchestrator" {
        primary = if cfg.backend_primary == "claude" {
            "claude".to_string()
        } else {
            "codex".to_string()
        };
        fallback = if cfg.backend_fallback == "codex" || cfg.backend_fallback == "claude" {
            if cfg.backend_fallback == primary {
                opposite_backend(&primary)
            } else {
                cfg.backend_fallback.clone()
            }
        } else {
            opposite_backend(&primary)
        };
        if role_is_explicit {
            route_reason = format!("role:{role}");
        }
    } else if cfg.agent_enabled && cfg.agent_mode == "role_handoff" && role_is_explicit {
        if let Some(role_backend) = backend_for_role(&role) {
            primary = role_backend.to_string();
            fallback = opposite_backend(&primary);
            route_reason = format!("role_handoff:{role}");
        }
    } else if role_is_explicit {
        if let Some(role_backend) = backend_for_role(&role) {
            primary = role_backend.to_string();
            fallback = opposite_backend(&primary);
            route_reason = format!("role:{role}");
        }
    }

    if primary == fallback {
        fallback = opposite_backend(&primary);
    }

    CodeagentRoute {
        role,
        role_source,
        role_is_explicit,
        primary,
        fallback,
        route_reason,
    }
}

pub(crate) fn lookup_primary_session(
    sessions: &Value,
    primary: &str,
    session_role: Option<&str>,
) -> Option<String> {
    let primary_session_key = session_key_for_backend_role(primary, session_role);
    let mut primary_session = json_get_string(sessions, &[primary_session_key.as_str()]);
    if primary_session.is_none() && session_role.is_some() {
        primary_session = json_get_string(sessions, &[session_key_for_backend(primary)]);
    }
    primary_session
}
