"""supervisor 虚拟监督官（默认 advisory，仅增补不接管执行）。"""

from __future__ import annotations

import hashlib
import json
import time
from pathlib import Path
from typing import Any, Dict, List

from .config import load_fusion_config


def _default_state() -> Dict[str, Any]:
    return {
        "last_advice_round": 0,
        "last_digest": "",
        "last_risk_score": 0.0,
    }


def _load_state(path: Path) -> Dict[str, Any]:
    if not path.exists():
        return _default_state()

    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, IOError, OSError):
        return _default_state()

    if not isinstance(data, dict):
        return _default_state()

    state = _default_state()
    state["last_advice_round"] = int(data.get("last_advice_round", 0) or 0)
    state["last_digest"] = str(data.get("last_digest") or "")
    try:
        state["last_risk_score"] = float(data.get("last_risk_score", 0.0) or 0.0)
    except (TypeError, ValueError):
        state["last_risk_score"] = 0.0
    return state


def _persist_state(path: Path, state: Dict[str, Any]) -> None:
    try:
        path.write_text(json.dumps(state, ensure_ascii=False, indent=2), encoding="utf-8")
    except IOError:
        pass


def _build_suggestions(
    *,
    no_progress_rounds: int,
    counts: Dict[str, int],
    pending_like: int,
    max_suggestions: int,
) -> List[Dict[str, str]]:
    suggestions: List[Dict[str, str]] = []

    failed = int(counts.get("failed", 0))
    if failed > 0:
        suggestions.append(
            {
                "category": "quality",
                "title": "先收敛失败任务并补最小回归用例",
                "rationale": "失败任务会放大停滞，先修复失败路径可以最快恢复主循环。",
            }
        )

    if pending_like > 0:
        suggestions.append(
            {
                "category": "documentation",
                "title": "为当前 IN_PROGRESS 任务补充完成判据",
                "rationale": "明确完成标准能减少反复修改导致的无进展回合。",
            }
        )

    if no_progress_rounds >= 4:
        suggestions.append(
            {
                "category": "optimization",
                "title": "执行一次低风险热路径体检并记录基线",
                "rationale": "避免在同一路径重复试错，先用基线定位瓶颈再继续开发。",
            }
        )

    if not suggestions:
        suggestions.append(
            {
                "category": "documentation",
                "title": "整理当前阶段的假设与限制",
                "rationale": "在不改变业务行为的前提下沉淀上下文，降低后续漂移风险。",
            }
        )

    unique: List[Dict[str, str]] = []
    seen = set()
    for item in suggestions:
        title = item.get("title", "")
        if title in seen:
            continue
        seen.add(title)
        unique.append(item)
        if len(unique) >= max_suggestions:
            break
    return unique


def _suggestion_digest(suggestions: List[Dict[str, str]]) -> str:
    source = "|".join(f"{item.get('category','')}:{item.get('title','')}" for item in suggestions)
    return hashlib.sha1(source.encode("utf-8")).hexdigest()


def _stagnation_score(no_progress_rounds: int, trigger_rounds: int) -> float:
    denominator = max(trigger_rounds * 3, 1)
    return min(1.0, max(0.0, no_progress_rounds / denominator))


def _repeat_pressure(*, counts: Dict[str, int], pending_like: int) -> float:
    failed = max(0, int(counts.get("failed", 0)))
    denominator = max(1, pending_like + failed)
    return min(1.0, failed / denominator)


def generate_supervisor_advice(
    *,
    fusion_dir: str = ".fusion",
    no_progress_rounds: int,
    counts: Dict[str, int],
    pending_like: int,
) -> Dict[str, Any]:
    """生成监督建议（默认 advisory），仅输出建议不直接改任务。"""
    cfg = load_fusion_config(fusion_dir)
    enabled = bool(cfg.get("supervisor_enabled", False))

    result: Dict[str, Any] = {
        "enabled": enabled,
        "mode": "advisory",
        "emit": False,
        "line": "",
        "suggestions": [],
        "payload": {},
    }

    if not enabled:
        return result

    mode = str(cfg.get("supervisor_mode") or "advisory").strip().lower()
    if mode not in {"advisory", "enforced"}:
        mode = "advisory"
    if mode != "advisory":
        mode = "advisory"
    result["mode"] = mode

    trigger_rounds = int(cfg.get("supervisor_trigger_no_progress_rounds", 2))
    cadence_rounds = int(cfg.get("supervisor_cadence_rounds", 2))
    force_emit_rounds = int(cfg.get("supervisor_force_emit_rounds", 12))
    max_suggestions = int(cfg.get("supervisor_max_suggestions", 2))
    persona = str(cfg.get("supervisor_persona") or "Guardian").strip() or "Guardian"

    if trigger_rounds < 1:
        trigger_rounds = 1
    if cadence_rounds < 1:
        cadence_rounds = 1
    if force_emit_rounds < 1:
        force_emit_rounds = 1
    if max_suggestions < 1:
        max_suggestions = 1

    if no_progress_rounds < trigger_rounds:
        return result

    state_path = Path(fusion_dir) / "supervisor_state.json"
    state = _load_state(state_path)
    last_advice_round = int(state.get("last_advice_round", 0) or 0)

    if (
        last_advice_round > 0
        and no_progress_rounds - last_advice_round < cadence_rounds
        and (no_progress_rounds % force_emit_rounds != 0)
    ):
        return result

    suggestions = _build_suggestions(
        no_progress_rounds=no_progress_rounds,
        counts=counts,
        pending_like=pending_like,
        max_suggestions=max_suggestions,
    )
    if not suggestions:
        return result

    digest = _suggestion_digest(suggestions)
    if digest == str(state.get("last_digest") or "") and (no_progress_rounds % force_emit_rounds != 0):
        return result

    stagnation_score = _stagnation_score(no_progress_rounds, trigger_rounds)
    repeat_pressure = _repeat_pressure(counts=counts, pending_like=pending_like)
    risk_score = round(min(1.0, max(0.0, 0.65 * stagnation_score + 0.35 * repeat_pressure)), 3)

    lead = suggestions[0].get("title") or "收敛当前任务"
    line = (
        f"[fusion][{persona}] Advisory: no-progress={no_progress_rounds}, "
        f"risk={risk_score:.2f}, next={lead}"
    )

    payload = {
        "mode": mode,
        "persona": persona,
        "no_progress_rounds": no_progress_rounds,
        "stagnation_score": stagnation_score,
        "repeat_pressure": repeat_pressure,
        "risk_score": risk_score,
        "suggestions": suggestions,
    }

    state["last_advice_round"] = no_progress_rounds
    state["last_digest"] = digest
    state["last_risk_score"] = risk_score
    state["updated_at"] = time.time()
    _persist_state(state_path, state)

    result["emit"] = True
    result["line"] = line
    result["suggestions"] = suggestions
    result["payload"] = payload
    return result

