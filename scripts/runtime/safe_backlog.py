"""safe_backlog 低风险托底任务生成器。"""

from __future__ import annotations

import hashlib
import json
import random
from pathlib import Path
from typing import Any, Dict, List

from .config import load_fusion_config


def _parse_allowed_categories(raw: str) -> List[str]:
    items = [item.strip().lower() for item in (raw or "").split(",")]
    return [item for item in items if item]


def _load_state(path: Path) -> Dict[str, Any]:
    if not path.exists():
        return {
            "fingerprints": [],
            "last_category": "",
            "stats": {
                "total_injections": 0,
                "category_counts": {},
            },
            "backoff": {
                "consecutive_failures": 0,
                "consecutive_injections": 0,
                "cooldown_until_round": 0,
                "attempt_round": 0,
            },
        }
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        if isinstance(data, dict):
            fingerprints = data.get("fingerprints")
            result: Dict[str, Any] = {
                "fingerprints": [str(item) for item in fingerprints] if isinstance(fingerprints, list) else [],
                "last_category": str(data.get("last_category") or ""),
                "stats": data.get("stats") if isinstance(data.get("stats"), dict) else {
                    "total_injections": 0,
                    "category_counts": {},
                },
                "backoff": data.get("backoff") if isinstance(data.get("backoff"), dict) else {
                    "consecutive_failures": 0,
                    "consecutive_injections": 0,
                    "cooldown_until_round": 0,
                    "attempt_round": 0,
                },
            }
            stats = result.get("stats")
            if not isinstance(stats, dict):
                stats = {}
            stats.setdefault("total_injections", 0)
            stats.setdefault("category_counts", {})
            result["stats"] = stats

            backoff = result.get("backoff")
            if not isinstance(backoff, dict):
                backoff = {}
            backoff.setdefault("consecutive_failures", 0)
            backoff.setdefault("consecutive_injections", 0)
            backoff.setdefault("cooldown_until_round", 0)
            backoff.setdefault("attempt_round", 0)
            result["backoff"] = backoff
            return result
    except (json.JSONDecodeError, IOError, OSError):
        pass
    return {
        "fingerprints": [],
        "last_category": "",
        "stats": {
            "total_injections": 0,
            "category_counts": {},
        },
        "backoff": {
            "consecutive_failures": 0,
            "consecutive_injections": 0,
            "cooldown_until_round": 0,
            "attempt_round": 0,
        },
    }


def _persist_state(path: Path, state: Dict[str, Any]) -> None:
    try:
        path.write_text(
            json.dumps(state, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
    except IOError:
        pass


def _candidate_tasks(project_root: Path) -> List[Dict[str, str]]:
    candidates: List[Dict[str, str]] = []

    readme = project_root / "README.md"
    if readme.exists():
        candidates.append(
            {
                "title": "更新 README 快速开始说明",
                "category": "documentation",
                "type": "documentation",
                "execution": "Direct",
                "output": "README.md",
            }
        )

    runtime_tests = project_root / "scripts" / "runtime" / "tests"
    if runtime_tests.exists():
        candidates.append(
            {
                "title": "补充 runtime 回归测试清单",
                "category": "quality",
                "type": "verification",
                "execution": "TDD",
                "output": "scripts/runtime/tests",
            }
        )

    runtime_dir = project_root / "scripts" / "runtime"
    if runtime_dir.exists():
        candidates.append(
            {
                "title": "优化 runtime 热路径扫描开销",
                "category": "optimization",
                "type": "configuration",
                "execution": "Direct",
                "output": "scripts/runtime",
            }
        )

    # 保证至少有一个 documentation 兜底任务，避免空候选
    if not candidates:
        candidates.append(
            {
                "title": "整理实现说明与限制",
                "category": "documentation",
                "type": "documentation",
                "execution": "Direct",
                "output": "docs",
            }
        )

    return candidates


def _fingerprint(task: Dict[str, str]) -> str:
    source = f"{task.get('title', '')}|{task.get('category', '')}|{task.get('output', '')}"
    return hashlib.sha1(source.encode("utf-8")).hexdigest()


def _priority_score(task: Dict[str, str], last_category: str, category_counts: Dict[str, Any]) -> float:
    category = str(task.get("category", ""))

    base = {
        "quality": 0.82,
        "optimization": 0.79,
        "documentation": 0.72,
    }.get(category, 0.65)

    rotation_bonus = 0.08 if category and category != last_category else 0.0
    usage_count = int(category_counts.get(category, 0)) if category else 0
    repetition_penalty = min(0.25, usage_count * 0.03)

    score = base + rotation_bonus - repetition_penalty
    if score < 0.1:
        score = 0.1
    if score > 0.99:
        score = 0.99

    return round(score, 4)


def _append_task_plan(task_plan_path: Path, tasks: List[Dict[str, str]]) -> None:
    try:
        original = task_plan_path.read_text(encoding="utf-8")
    except IOError:
        return

    existing_numbers: List[int] = []
    for line in original.splitlines():
        if not line.startswith("### Task "):
            continue
        try:
            number_part = line.split(":", 1)[0].replace("### Task", "").strip()
            existing_numbers.append(int(number_part))
        except (ValueError, IndexError):
            continue

    next_index = (max(existing_numbers) if existing_numbers else 0) + 1

    chunks: List[str] = [original.rstrip("\n")]
    if chunks and chunks[0]:
        chunks.append("")

    for task in tasks:
        chunks.extend(
            [
                f"### Task {next_index}: {task['title']} [PENDING] [SAFE_BACKLOG]",
                f"- Type: {task['type']}",
                f"- Execution: {task['execution']}",
                "- Dependencies: []",
                f"- Category: {task['category']}",
                f"- Output: {task['output']}",
                "",
            ]
        )
        next_index += 1

    task_plan_path.write_text("\n".join(chunks).rstrip("\n") + "\n", encoding="utf-8")


def generate_safe_backlog(fusion_dir: str = ".fusion", project_root: str = ".") -> Dict[str, Any]:
    """按配置生成低风险托底任务并写入 task_plan。"""
    fusion_path = Path(fusion_dir)
    project_path = Path(project_root)
    cfg = load_fusion_config(fusion_dir)

    enabled = bool(cfg.get("safe_backlog_enabled", False))
    result: Dict[str, Any] = {
        "enabled": enabled,
        "added": 0,
        "tasks": [],
        "blocked_by_backoff": False,
        "backoff_state": {},
    }

    task_plan_path = fusion_path / "task_plan.md"
    state_path = fusion_path / "safe_backlog.json"

    if not enabled or not task_plan_path.exists():
        return result

    allowed = _parse_allowed_categories(str(cfg.get("safe_backlog_allowed_categories", "documentation,quality")))
    limit = int(cfg.get("safe_backlog_max_tasks_per_run", 2))
    if limit < 1:
        limit = 1

    novelty_window = int(cfg.get("safe_backlog_novelty_window", 12))
    if novelty_window < 1:
        novelty_window = 1

    diversity_rotation = bool(cfg.get("safe_backlog_diversity_rotation", True))

    state = _load_state(state_path)
    seen_list = [str(item) for item in state.get("fingerprints", [])]
    seen = set(seen_list[-novelty_window:])
    last_category = str(state.get("last_category") or "")
    stats = state.get("stats") if isinstance(state.get("stats"), dict) else {
        "total_injections": 0,
        "category_counts": {},
    }
    category_counts = stats.get("category_counts") if isinstance(stats.get("category_counts"), dict) else {}
    backoff = state.get("backoff") if isinstance(state.get("backoff"), dict) else {
        "consecutive_failures": 0,
        "consecutive_injections": 0,
        "cooldown_until_round": 0,
        "attempt_round": 0,
    }

    backoff_enabled = bool(cfg.get("safe_backlog_backoff_enabled", True))
    base_rounds = int(cfg.get("safe_backlog_backoff_base_rounds", 1))
    max_rounds = int(cfg.get("safe_backlog_backoff_max_rounds", 32))
    jitter = float(cfg.get("safe_backlog_backoff_jitter", 0.2))
    force_probe_rounds = int(cfg.get("safe_backlog_backoff_force_probe_rounds", 20))

    if base_rounds < 1:
        base_rounds = 1
    if max_rounds < base_rounds:
        max_rounds = base_rounds
    if jitter < 0:
        jitter = 0
    if jitter > 1:
        jitter = 1
    if force_probe_rounds < 1:
        force_probe_rounds = 1

    attempt_round = int(backoff.get("attempt_round", 0)) + 1
    backoff["attempt_round"] = attempt_round
    cooldown_until = int(backoff.get("cooldown_until_round", 0))

    if backoff_enabled and attempt_round <= cooldown_until:
        if attempt_round % force_probe_rounds != 0:
            state["backoff"] = backoff
            _persist_state(state_path, state)
            result["blocked_by_backoff"] = True
            result["backoff_state"] = backoff
            return result

    candidates = _candidate_tasks(project_path)
    for candidate in candidates:
        candidate["priority_score"] = _priority_score(candidate, last_category, category_counts)

    if diversity_rotation and last_category:
        rotated = [c for c in candidates if c.get("category") != last_category]
        if rotated:
            candidates = rotated + [c for c in candidates if c.get("category") == last_category]

    # 优先级：高分优先（兼顾轮转与历史重复惩罚）
    candidates.sort(key=lambda c: float(c.get("priority_score", 0.0)), reverse=True)

    selected: List[Dict[str, str]] = []
    added_fingerprints: List[str] = []

    for candidate in candidates:
        if allowed and candidate.get("category", "") not in allowed:
            continue
        fingerprint = _fingerprint(candidate)
        if fingerprint in seen:
            continue
        selected.append(candidate)
        added_fingerprints.append(fingerprint)
        if len(selected) >= limit:
            break

    if not selected:
        if backoff_enabled:
            failures = int(backoff.get("consecutive_failures", 0)) + 1
            backoff["consecutive_failures"] = failures
            backoff["consecutive_injections"] = 0
            cooldown = min(max_rounds, base_rounds * (2 ** max(0, failures - 1)))
            if jitter > 0:
                jitter_factor = random.uniform(1 - jitter, 1 + jitter)
                cooldown = max(1, int(round(cooldown * jitter_factor)))
            backoff["cooldown_until_round"] = attempt_round + cooldown
            state["backoff"] = backoff
            _persist_state(state_path, state)
            result["backoff_state"] = backoff
        return result

    _append_task_plan(task_plan_path, selected)

    merged_fingerprints = seen_list + added_fingerprints
    if len(merged_fingerprints) > novelty_window:
        merged_fingerprints = merged_fingerprints[-novelty_window:]

    state["fingerprints"] = merged_fingerprints
    state["last_category"] = selected[-1].get("category", "")

    stats["total_injections"] = int(stats.get("total_injections", 0)) + len(selected)
    for task in selected:
        category = task.get("category", "")
        if category:
            category_counts[category] = int(category_counts.get(category, 0)) + 1
    stats["category_counts"] = category_counts
    state["stats"] = stats

    if backoff_enabled:
        backoff["consecutive_failures"] = 0
        injections = int(backoff.get("consecutive_injections", 0)) + 1
        backoff["consecutive_injections"] = injections
        cooldown = min(max_rounds, base_rounds * (2 ** max(0, injections - 1)))
        if jitter > 0:
            jitter_factor = random.uniform(1 - jitter, 1 + jitter)
            cooldown = max(1, int(round(cooldown * jitter_factor)))
        backoff["cooldown_until_round"] = attempt_round + cooldown
    state["backoff"] = backoff

    _persist_state(state_path, state)

    result["added"] = len(selected)
    result["tasks"] = selected
    result["backoff_state"] = backoff
    return result


def reset_safe_backlog_backoff(fusion_dir: str = ".fusion") -> None:
    """在检测到真实进展时重置 backoff 冷却状态。"""
    state_path = Path(fusion_dir) / "safe_backlog.json"
    state = _load_state(state_path)
    backoff = state.get("backoff") if isinstance(state.get("backoff"), dict) else {}
    backoff["consecutive_failures"] = 0
    backoff["consecutive_injections"] = 0
    backoff["cooldown_until_round"] = 0
    state["backoff"] = backoff
    _persist_state(state_path, state)
