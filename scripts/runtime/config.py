"""
Fusion Runtime 配置加载

统一读取 `.fusion/config.yaml`，为 runtime/bridge/understand 提供一致配置来源。
优先使用 PyYAML，失败时退回轻量行级解析。
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict


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


def _minimal_parse_yaml(path: Path) -> Dict[str, Any]:
    """
    极简 YAML 解析（仅支持 section + 一级 key:value）。

    目标是故障安全，不追求完整 YAML 语义。
    """
    data: Dict[str, Dict[str, Any]] = {}
    current_section = ""

    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except IOError:
        return {}

    for raw_line in lines:
        line = raw_line.split("#", 1)[0].rstrip()
        if not line.strip():
            continue

        if not line.startswith((" ", "\t")) and line.endswith(":"):
            current_section = line[:-1].strip()
            if current_section:
                data.setdefault(current_section, {})
            continue

        if current_section and ":" in line and line.startswith((" ", "\t")):
            key, value = line.split(":", 1)
            key = key.strip()
            value = value.strip()
            if value.startswith(("'", '"')) and value.endswith(("'", '"')) and len(value) >= 2:
                value = value[1:-1]
            data[current_section][key] = value

    return data


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
    execution = raw.get("execution") if isinstance(raw.get("execution"), dict) else {}
    scheduler = raw.get("scheduler") if isinstance(raw.get("scheduler"), dict) else {}
    budget = raw.get("budget") if isinstance(raw.get("budget"), dict) else {}
    safe_backlog = raw.get("safe_backlog") if isinstance(raw.get("safe_backlog"), dict) else {}

    execution_parallel = _to_int(execution.get("parallel"), 2)
    safe_backlog_max_tasks_per_run = _to_int(safe_backlog.get("max_tasks_per_run"), 2)
    if safe_backlog_max_tasks_per_run < 1:
        safe_backlog_max_tasks_per_run = 1

    safe_backlog_allowed_categories = safe_backlog.get("allowed_categories")
    if isinstance(safe_backlog_allowed_categories, list):
        safe_backlog_allowed_categories = ",".join(str(item) for item in safe_backlog_allowed_categories)
    elif safe_backlog_allowed_categories is None:
        safe_backlog_allowed_categories = "documentation,quality"
    else:
        safe_backlog_allowed_categories = str(safe_backlog_allowed_categories)

    return {
        "runtime_enabled": _to_bool(runtime.get("enabled"), False),
        "runtime_version": str(runtime.get("version") or "2.1.0"),
        "runtime_compat_mode": _to_bool(runtime.get("compat_mode"), True),
        "backend_primary": str(backends.get("primary") or "codex"),
        "backend_fallback": str(backends.get("fallback") or "claude"),
        "execution_parallel": execution_parallel,
        "execution_timeout_ms": _to_int(execution.get("timeout"), 7_200_000),
        "scheduler_enabled": _to_bool(scheduler.get("enabled"), False),
        "scheduler_max_parallel": _to_int(scheduler.get("max_parallel"), execution_parallel),
        "scheduler_fail_fast": _to_bool(scheduler.get("fail_fast"), False),
        "budget_global_token_limit": _to_int(budget.get("global_token_limit"), 100_000),
        "budget_global_latency_limit_ms": _to_int(budget.get("global_latency_limit_ms"), 7_200_000),
        "budget_warning_threshold": _to_float(budget.get("warning_threshold"), 0.8),
        "budget_hard_limit_action": str(budget.get("hard_limit_action") or "serial"),
        "safe_backlog_enabled": _to_bool(safe_backlog.get("enabled"), False),
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
    }
