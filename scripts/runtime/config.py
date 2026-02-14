"""
Fusion Runtime 配置加载

统一读取 `.fusion/config.yaml`，为 runtime/bridge/understand 提供一致配置来源。
优先使用 PyYAML，失败时退回轻量行级解析。
"""

from __future__ import annotations

from pathlib import Path
import re
from typing import Any, Dict


DEFAULT_BACKEND_PHASE_ROUTING: Dict[str, str] = {
    "UNDERSTAND": "codex",
    "INITIALIZE": "codex",
    "ANALYZE": "codex",
    "DECOMPOSE": "codex",
    "EXECUTE": "claude",
    "VERIFY": "codex",
    "REVIEW": "codex",
    "COMMIT": "claude",
    "DELIVER": "claude",
}

DEFAULT_BACKEND_TASK_TYPE_ROUTING: Dict[str, str] = {
    "implementation": "claude",
    "verification": "claude",
    "design": "codex",
    "research": "codex",
    "documentation": "claude",
    "configuration": "claude",
}


def _to_bool(value: Any, default: bool) -> bool:
    if isinstance(value, bool):
        return value
    if isinstance(value, (int, float)):
        return bool(value)
    if isinstance(value, str):
        normalized = value.strip().lower()
        if normalized in ("true", "yes", "on", "1"):
            return True
        if normalized in ("false", "no", "off", "0"):
            return False
    return default


def _to_int(value: Any, default: int) -> int:
    try:
        return int(value)
    except (TypeError, ValueError):
        return default


def _to_float(value: Any, default: float) -> float:
    try:
        return float(value)
    except (TypeError, ValueError):
        return default


def _normalize_backend(value: Any, default: str) -> str:
    if isinstance(value, str):
        normalized = value.strip().lower()
        if normalized in ("codex", "claude"):
            return normalized
    return default


def _merge_backend_map(
    defaults: Dict[str, str],
    raw_map: Any,
    *,
    uppercase_keys: bool,
) -> Dict[str, str]:
    merged = dict(defaults)
    if not isinstance(raw_map, dict):
        return merged

    for key, value in raw_map.items():
        if not isinstance(key, str):
            continue
        backend = _normalize_backend(value, "")
        if not backend:
            continue
        normalized_key = key.strip().upper() if uppercase_keys else key.strip().lower()
        if normalized_key:
            merged[normalized_key] = backend

    return merged


def _parse_scalar(value: str) -> Any:
    v = value.strip()
    if not v:
        return ""

    if (v.startswith('"') and v.endswith('"')) or (v.startswith("'") and v.endswith("'")):
        return v[1:-1]

    lowered = v.lower()
    if lowered in ("true", "false"):
        return lowered == "true"
    if lowered in ("null", "none"):
        return None

    if re.fullmatch(r"[-+]?\d+", v):
        try:
            return int(v)
        except ValueError:
            return v

    if re.fullmatch(r"[-+]?\d+\.\d+", v):
        try:
            return float(v)
        except ValueError:
            return v

    return v


def _minimal_parse_yaml(path: Path) -> Dict[str, Any]:
    """
    极简 YAML 解析（支持按缩进构建嵌套 dict，覆盖常见配置结构）。

    目标是故障安全，不追求完整 YAML 语义。
    """
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except IOError:
        return {}

    root: Dict[str, Any] = {}
    stack: list[tuple[int, Dict[str, Any]]] = [(-1, root)]

    for raw_line in lines:
        line = raw_line.split("#", 1)[0].rstrip()
        if not line.strip() or ":" not in line:
            continue

        indent = len(line) - len(line.lstrip(" \t"))
        stripped = line.lstrip(" \t")

        while len(stack) > 1 and indent <= stack[-1][0]:
            stack.pop()

        parent = stack[-1][1]
        key, value = stripped.split(":", 1)
        key = key.strip()
        value = value.strip()

        if not key:
            continue

        if value == "":
            next_map: Dict[str, Any] = {}
            parent[key] = next_map
            stack.append((indent, next_map))
        else:
            parent[key] = _parse_scalar(value)

    return root


def load_raw_config(fusion_dir: str = ".fusion") -> Dict[str, Any]:
    """读取 config.yaml 原始内容（dict）。"""
    config_path = Path(fusion_dir) / "config.yaml"
    if not config_path.exists():
        return {}

    try:
        import yaml  # type: ignore

        loaded = yaml.safe_load(config_path.read_text(encoding="utf-8"))
        return loaded if isinstance(loaded, dict) else {}
    except Exception:
        return _minimal_parse_yaml(config_path)


def load_fusion_config(fusion_dir: str = ".fusion") -> Dict[str, Any]:
    """
    读取并规范化 Fusion 配置。

    返回扁平化字段，避免业务层重复解析。
    """
    raw = load_raw_config(fusion_dir)

    runtime = raw.get("runtime") if isinstance(raw.get("runtime"), dict) else {}
    backends = raw.get("backends") if isinstance(raw.get("backends"), dict) else {}
    backend_routing = raw.get("backend_routing") if isinstance(raw.get("backend_routing"), dict) else {}
    execution = raw.get("execution") if isinstance(raw.get("execution"), dict) else {}
    scheduler = raw.get("scheduler") if isinstance(raw.get("scheduler"), dict) else {}
    budget = raw.get("budget") if isinstance(raw.get("budget"), dict) else {}
    safe_backlog = raw.get("safe_backlog") if isinstance(raw.get("safe_backlog"), dict) else {}
    supervisor = raw.get("supervisor") if isinstance(raw.get("supervisor"), dict) else {}
    understand = raw.get("understand") if isinstance(raw.get("understand"), dict) else {}

    execution_parallel = _to_int(execution.get("parallel"), 2)
    safe_backlog_max_tasks_per_run = _to_int(safe_backlog.get("max_tasks_per_run"), 2)
    if safe_backlog_max_tasks_per_run < 1:
        safe_backlog_max_tasks_per_run = 1

    safe_backlog_allowed_categories = safe_backlog.get("allowed_categories")
    if isinstance(safe_backlog_allowed_categories, list):
        safe_backlog_allowed_categories = ",".join(str(item) for item in safe_backlog_allowed_categories)
    elif safe_backlog_allowed_categories is None:
        safe_backlog_allowed_categories = "quality,documentation,optimization"
    else:
        safe_backlog_allowed_categories = str(safe_backlog_allowed_categories)

    understand_threshold = _to_int(understand.get("pass_threshold"), 7)
    if understand_threshold < 0:
        understand_threshold = 0
    if understand_threshold > 10:
        understand_threshold = 10

    understand_max_questions = _to_int(understand.get("max_questions"), 2)
    if understand_max_questions < 1:
        understand_max_questions = 1

    backend_primary = _normalize_backend(backends.get("primary"), "codex")
    backend_fallback = _normalize_backend(backends.get("fallback"), "claude")
    if backend_fallback == backend_primary:
        backend_fallback = "claude" if backend_primary == "codex" else "codex"

    phase_routing_raw = None
    task_type_routing_raw = None

    if isinstance(backends.get("phase_routing"), dict):
        phase_routing_raw = backends.get("phase_routing")
    elif isinstance(backend_routing.get("phase_routing"), dict):
        phase_routing_raw = backend_routing.get("phase_routing")
    elif isinstance(backend_routing.get("phase"), dict):
        phase_routing_raw = backend_routing.get("phase")

    if isinstance(backends.get("task_type_routing"), dict):
        task_type_routing_raw = backends.get("task_type_routing")
    elif isinstance(backend_routing.get("task_type_routing"), dict):
        task_type_routing_raw = backend_routing.get("task_type_routing")
    elif isinstance(backend_routing.get("task_type"), dict):
        task_type_routing_raw = backend_routing.get("task_type")

    backend_phase_routing = _merge_backend_map(
        DEFAULT_BACKEND_PHASE_ROUTING,
        phase_routing_raw,
        uppercase_keys=True,
    )
    backend_task_type_routing = _merge_backend_map(
        DEFAULT_BACKEND_TASK_TYPE_ROUTING,
        task_type_routing_raw,
        uppercase_keys=False,
    )

    return {
        "runtime_enabled": _to_bool(runtime.get("enabled"), False),
        "runtime_version": str(runtime.get("version") or "2.6.3"),
        "runtime_compat_mode": _to_bool(runtime.get("compat_mode"), True),
        "runtime_engine": str(runtime.get("engine") or "python"),
        "backend_primary": backend_primary,
        "backend_fallback": backend_fallback,
        "backend_phase_routing": backend_phase_routing,
        "backend_task_type_routing": backend_task_type_routing,
        "execution_parallel": execution_parallel,
        "execution_timeout_ms": _to_int(execution.get("timeout"), 7_200_000),
        "scheduler_enabled": _to_bool(scheduler.get("enabled"), False),
        "scheduler_max_parallel": _to_int(scheduler.get("max_parallel"), execution_parallel),
        "scheduler_fail_fast": _to_bool(scheduler.get("fail_fast"), False),
        "budget_global_token_limit": _to_int(budget.get("global_token_limit"), 100_000),
        "budget_global_latency_limit_ms": _to_int(budget.get("global_latency_limit_ms"), 7_200_000),
        "budget_warning_threshold": _to_float(budget.get("warning_threshold"), 0.8),
        "budget_hard_limit_action": str(budget.get("hard_limit_action") or "serial"),
        "safe_backlog_enabled": _to_bool(safe_backlog.get("enabled"), True),
        "safe_backlog_trigger_no_progress_rounds": _to_int(safe_backlog.get("trigger_no_progress_rounds"), 3),
        "safe_backlog_max_tasks_per_run": safe_backlog_max_tasks_per_run,
        "safe_backlog_allowed_categories": safe_backlog_allowed_categories,
        "safe_backlog_inject_on_task_exhausted": _to_bool(safe_backlog.get("inject_on_task_exhausted"), True),
        "safe_backlog_diversity_rotation": _to_bool(safe_backlog.get("diversity_rotation"), True),
        "safe_backlog_novelty_window": _to_int(safe_backlog.get("novelty_window"), 12),
        "safe_backlog_backoff_enabled": _to_bool(safe_backlog.get("backoff_enabled"), True),
        "safe_backlog_backoff_base_rounds": _to_int(safe_backlog.get("backoff_base_rounds"), 1),
        "safe_backlog_backoff_max_rounds": _to_int(safe_backlog.get("backoff_max_rounds"), 32),
        "safe_backlog_backoff_jitter": _to_float(safe_backlog.get("backoff_jitter"), 0.2),
        "safe_backlog_backoff_force_probe_rounds": _to_int(safe_backlog.get("backoff_force_probe_rounds"), 20),
        "safe_backlog_max_files_touched": _to_int(safe_backlog.get("max_files_touched"), 4),
        "safe_backlog_max_lines_changed": _to_int(safe_backlog.get("max_lines_changed"), 200),
        "supervisor_enabled": _to_bool(supervisor.get("enabled"), False),
        "supervisor_mode": str(supervisor.get("mode") or "advisory"),
        "supervisor_persona": str(supervisor.get("persona") or "Guardian"),
        "supervisor_trigger_no_progress_rounds": _to_int(supervisor.get("trigger_no_progress_rounds"), 2),
        "supervisor_cadence_rounds": _to_int(supervisor.get("cadence_rounds"), 2),
        "supervisor_force_emit_rounds": _to_int(supervisor.get("force_emit_rounds"), 12),
        "supervisor_max_suggestions": _to_int(supervisor.get("max_suggestions"), 2),
        "understand_pass_threshold": understand_threshold,
        "understand_require_confirmation": _to_bool(understand.get("require_confirmation"), False),
        "understand_max_questions": understand_max_questions,
    }
