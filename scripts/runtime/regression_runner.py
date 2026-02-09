"""
Fusion v2.5.0 回归测试运行器

用法:
    python3 scripts/runtime/regression_runner.py --suite phase1 --min-pass-rate 0.99
    python3 scripts/runtime/regression_runner.py --suite phase2 --min-pass-rate 0.99
    python3 scripts/runtime/regression_runner.py --suite all --min-pass-rate 0.99
    python3 scripts/runtime/regression_runner.py --scenario resume_reliability --runs 20 --min-pass-rate 0.95
"""

import argparse
import sys
import time
import tempfile
import shutil
import json
import subprocess
import unittest
from pathlib import Path
from dataclasses import dataclass

# 确保 runtime 可导入
sys.path.insert(0, str(Path(__file__).parent.parent))

from runtime.state_machine import StateMachine, State, Event
from runtime.kernel import FusionKernel
from runtime.session_store import SessionStore
from runtime.event_bus import EventBus
from runtime.compat_v2 import adapt_stop_guard, adapt_pretool, adapt_posttool
from runtime.task_graph import TaskGraph, TaskNode
from runtime.conflict_detector import ConflictDetector
from runtime.budget_manager import BudgetManager, BudgetConfig
from runtime.router import Router
from runtime.scheduler import Scheduler, SchedulerConfig


@dataclass
class ScenarioResult:
    name: str
    passed: bool
    duration_ms: float
    error: str = ""


def _make_fusion_dir() -> Path:
    """创建临时 .fusion 目录"""
    tmp = Path(tempfile.mkdtemp())
    fusion = tmp / ".fusion"
    fusion.mkdir()
    return fusion


def _cleanup(fusion_dir: Path):
    shutil.rmtree(fusion_dir.parent, ignore_errors=True)


def _write_all_done_task_plan(fusion_dir: Path):
    """写入一个全部完成的 task_plan.md（满足 all_tasks_done 守卫条件）"""
    (fusion_dir / "task_plan.md").write_text(
        "### Task 1: A [COMPLETED]\n### Task 2: B [COMPLETED]\n"
    )


def _advance_kernel_to_execute(fusion_dir: Path) -> 'FusionKernel':
    """创建 Kernel 并推进到 EXECUTE 状态"""
    k = FusionKernel(fusion_dir=str(fusion_dir))
    for evt in [Event.START, Event.INIT_DONE, Event.ANALYZE_DONE, Event.DECOMPOSE_DONE]:
        r = k.dispatch(evt)
        assert r.success, f"Advance failed at {evt}: {r.error}"
    assert k.current_state == State.EXECUTE
    return k


def _advance_kernel_to_verify(fusion_dir: Path) -> 'FusionKernel':
    """创建 Kernel 并推进到 VERIFY 状态"""
    k = _advance_kernel_to_execute(fusion_dir)
    _write_all_done_task_plan(fusion_dir)
    k._load_task_context()
    r = k.dispatch(Event.ALL_TASKS_DONE)
    assert r.success, f"ALL_TASKS_DONE failed: {r.error}"
    assert k.current_state == State.VERIFY
    return k


# ──────────────────────────────────────────────
# 场景定义: Phase 1 全覆盖
# ──────────────────────────────────────────────

def scenario_fsm_basic_transitions() -> ScenarioResult:
    """S01: Kernel 基本状态转移链 (IDLE→INIT→ANALYZE→DECOMPOSE→EXECUTE→VERIFY→REVIEW→COMMIT→DELIVER→COMPLETED)"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        transitions = [
            Event.START, Event.INIT_DONE, Event.ANALYZE_DONE,
            Event.DECOMPOSE_DONE,
        ]
        for evt in transitions:
            result = k.dispatch(evt)
            assert result.success, f"Failed at {evt}: {result.error}"

        # ALL_TASKS_DONE 需要 task_plan.md 中的任务全部完成
        _write_all_done_task_plan(fusion_dir)
        k._load_task_context()

        for evt in [Event.ALL_TASKS_DONE, Event.VERIFY_PASS, Event.REVIEW_PASS,
                    Event.COMMIT_DONE, Event.DELIVER_DONE]:
            result = k.dispatch(evt)
            assert result.success, f"Failed at {evt}: {result.error}"
        assert k.current_state == State.COMPLETED
        return ScenarioResult("S01-fsm-basic", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S01-fsm-basic", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_fsm_error_recovery() -> ScenarioResult:
    """S02: Kernel 错误→恢复路径"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        k.dispatch(Event.START)
        k.dispatch(Event.INIT_DONE)
        k.dispatch(Event.ANALYZE_DONE)
        k.dispatch(Event.DECOMPOSE_DONE)
        assert k.current_state == State.EXECUTE

        r = k.dispatch(Event.ERROR_OCCURRED, {"error": "test"})
        assert r.success
        assert k.current_state == State.ERROR

        r = k.dispatch(Event.RECOVER)
        assert r.success
        assert k.current_state == State.EXECUTE
        return ScenarioResult("S02-error-recovery", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S02-error-recovery", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_fsm_pause_resume() -> ScenarioResult:
    """S03: Kernel 暂停→恢复"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        k.dispatch(Event.START)
        k.dispatch(Event.INIT_DONE)
        k.dispatch(Event.ANALYZE_DONE)
        k.dispatch(Event.DECOMPOSE_DONE)

        r = k.dispatch(Event.PAUSE)
        assert r.success
        assert k.current_state == State.PAUSED

        r = k.dispatch(Event.RESUME)
        assert r.success
        assert k.current_state == State.EXECUTE
        return ScenarioResult("S03-pause-resume", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S03-pause-resume", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_fsm_cancel() -> ScenarioResult:
    """S04: Kernel 取消"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        k.dispatch(Event.START)
        k.dispatch(Event.INIT_DONE)
        r = k.dispatch(Event.CANCEL)
        assert r.success
        assert k.current_state == State.CANCELLED
        return ScenarioResult("S04-cancel", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S04-cancel", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_fsm_verify_fail_loop() -> ScenarioResult:
    """S05: VERIFY 失败退回 EXECUTE"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = _advance_kernel_to_verify(fusion_dir)

        r = k.dispatch(Event.VERIFY_FAIL)
        assert r.success
        assert k.current_state == State.EXECUTE
        return ScenarioResult("S05-verify-fail", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S05-verify-fail", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_fsm_review_fail_loop() -> ScenarioResult:
    """S06: REVIEW 失败退回 EXECUTE"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = _advance_kernel_to_verify(fusion_dir)
        r = k.dispatch(Event.VERIFY_PASS)
        assert r.success
        assert k.current_state == State.REVIEW

        r = k.dispatch(Event.REVIEW_FAIL)
        assert r.success
        assert k.current_state == State.EXECUTE
        return ScenarioResult("S06-review-fail", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S06-review-fail", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_fsm_invalid_transition() -> ScenarioResult:
    """S07: 非法转移不崩溃"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        r = k.dispatch(Event.ALL_TASKS_DONE)
        assert not r.success
        assert k.current_state == State.IDLE
        return ScenarioResult("S07-invalid-transition", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S07-invalid-transition", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_fsm_guard_conditions() -> ScenarioResult:
    """S08: 守卫条件检查"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = _advance_kernel_to_execute(fusion_dir)

        # TASK_DONE + has_pending_tasks → 留在 EXECUTE
        (fusion_dir / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n"
        )
        k._load_task_context()
        r = k.dispatch(Event.TASK_DONE)
        assert r.success
        assert k.current_state == State.EXECUTE

        # ALL_TASKS_DONE + all_tasks_done → 进入 VERIFY
        _write_all_done_task_plan(fusion_dir)
        k._load_task_context()
        r = k.dispatch(Event.ALL_TASKS_DONE)
        assert r.success
        assert k.current_state == State.VERIFY
        return ScenarioResult("S08-guard-conditions", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S08-guard-conditions", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_dispatch_and_persist() -> ScenarioResult:
    """S09: Kernel 派发并持久化"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        r = k.dispatch(Event.START)
        assert r.success
        assert k.current_state == State.INITIALIZE

        # 验证 sessions.json 已更新
        sessions = json.loads((fusion_dir / "sessions.json").read_text())
        assert sessions.get("_runtime", {}).get("state") == "INITIALIZE"

        # 验证 events.jsonl 有事件
        events_file = fusion_dir / "events.jsonl"
        assert events_file.exists()
        lines = events_file.read_text().strip().split("\n")
        assert len(lines) >= 1
        return ScenarioResult("S09-kernel-persist", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S09-kernel-persist", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_full_workflow() -> ScenarioResult:
    """S10: Kernel 完整工作流 IDLE→COMPLETED"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = _advance_kernel_to_execute(fusion_dir)

        _write_all_done_task_plan(fusion_dir)
        k._load_task_context()

        for evt in [Event.ALL_TASKS_DONE, Event.VERIFY_PASS, Event.REVIEW_PASS,
                    Event.COMMIT_DONE, Event.DELIVER_DONE]:
            r = k.dispatch(evt)
            assert r.success, f"Failed at {evt}: {r.error}"
        assert k.current_state == State.COMPLETED
        return ScenarioResult("S10-kernel-full", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S10-kernel-full", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_load_state() -> ScenarioResult:
    """S11: Kernel 保存后加载状态恢复"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k1 = FusionKernel(fusion_dir=str(fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)
        k1.dispatch(Event.ANALYZE_DONE)
        expected = k1.current_state  # 应该是 DECOMPOSE

        # 新实例加载
        k2 = FusionKernel(fusion_dir=str(fusion_dir))
        k2.load_state()
        assert k2.current_state == expected, f"Expected {expected}, got {k2.current_state}"
        return ScenarioResult("S11-kernel-load", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S11-kernel-load", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_event_replay() -> ScenarioResult:
    """S12: Kernel 从事件日志重放恢复"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k1 = FusionKernel(fusion_dir=str(fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)
        k1.dispatch(Event.ANALYZE_DONE)
        k1.dispatch(Event.DECOMPOSE_DONE)

        # 删除快照
        sessions_file = fusion_dir / "sessions.json"
        sessions_file.unlink()

        # 从事件重放
        k2 = FusionKernel(fusion_dir=str(fusion_dir))
        k2.load_state_from_events()
        assert k2.current_state == State.EXECUTE
        return ScenarioResult("S12-event-replay", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S12-event-replay", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_idempotent_dispatch() -> ScenarioResult:
    """S13: Kernel 幂等派发"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        r1 = k.dispatch(Event.START, idempotency_key="start-001")
        assert r1.success

        r2 = k.dispatch(Event.START, idempotency_key="start-001")
        # 幂等重复应跳过
        assert not r2.success or k.current_state == State.INITIALIZE
        return ScenarioResult("S13-idempotent", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S13-idempotent", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_event_bus_pubsub() -> ScenarioResult:
    """S14: EventBus 发布/订阅"""
    t0 = time.monotonic()
    try:
        bus = EventBus()
        received = []
        bus.on("test.event", lambda et, d: received.append(d))
        bus.emit("test.event", {"value": 42})
        assert len(received) == 1
        assert received[0]["value"] == 42
        return ScenarioResult("S14-eventbus-pubsub", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S14-eventbus-pubsub", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_event_bus_error_isolation() -> ScenarioResult:
    """S15: EventBus 错误隔离"""
    t0 = time.monotonic()
    try:
        bus = EventBus()
        received = []

        def bad_handler(et, d):
            raise ValueError("boom")

        def good_handler(et, d):
            received.append("ok")

        bus.on("test", bad_handler)
        bus.on("test", good_handler)
        errors = bus.emit("test", {})
        assert len(errors) == 1
        assert len(received) == 1
        return ScenarioResult("S15-bus-error-isolation", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S15-bus-error-isolation", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_event_bus_wildcard() -> ScenarioResult:
    """S16: EventBus 通配符订阅"""
    t0 = time.monotonic()
    try:
        bus = EventBus()
        received = []
        bus.on("*", lambda et, d: received.append(et))
        bus.emit("event.a", {})
        bus.emit("event.b", {})
        assert len(received) == 2
        assert "event.a" in received
        assert "event.b" in received
        return ScenarioResult("S16-bus-wildcard", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S16-bus-wildcard", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_session_store_append_load() -> ScenarioResult:
    """S17: SessionStore 追加/加载"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        store = SessionStore(fusion_dir=str(fusion_dir))
        store.append_event("START", "IDLE", "INITIALIZE", {"goal": "test"})
        store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")

        events = store.load_events()
        assert len(events) == 2
        assert events[0].event_type == "START"
        assert events[1].event_type == "INIT_DONE"
        return ScenarioResult("S17-store-append-load", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S17-store-append-load", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_session_store_idempotency() -> ScenarioResult:
    """S18: SessionStore 幂等写入"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        store = SessionStore(fusion_dir=str(fusion_dir))
        e1 = store.append_event("START", "IDLE", "INIT", idempotency_key="key-a")
        assert e1 is not None
        e2 = store.append_event("START", "IDLE", "INIT", idempotency_key="key-a")
        assert e2 is None

        events = store.load_events()
        assert len(events) == 1
        return ScenarioResult("S18-store-idempotency", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S18-store-idempotency", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_session_store_replay() -> ScenarioResult:
    """S19: SessionStore replay"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        store = SessionStore(fusion_dir=str(fusion_dir))
        store.append_event("A", "S0", "S1")
        e2 = store.append_event("B", "S1", "S2")
        store.append_event("C", "S2", "S3")

        replayed = []
        store.replay(apply_fn=lambda ev: replayed.append(ev.event_type), from_event_id=e2.id)

        # replay from e2 应只执行 e2 之后的事件 (C)
        assert "C" in replayed
        assert "A" not in replayed
        return ScenarioResult("S19-store-replay", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S19-store-replay", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_session_store_snapshot() -> ScenarioResult:
    """S20: SessionStore 快照同步"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        store = SessionStore(fusion_dir=str(fusion_dir))
        store.sync_snapshot(State.EXECUTE, {"goal": "test"})
        snap = store.load_snapshot()
        assert snap is not None
        assert snap.get("_runtime", {}).get("state") == "EXECUTE"
        return ScenarioResult("S20-store-snapshot", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S20-store-snapshot", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_stop_guard_allow() -> ScenarioResult:
    """S21: compat stop-guard 允许停止"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        result = adapt_stop_guard(str(fusion_dir))
        assert not result.should_block
        assert result.decision == "allow"
        return ScenarioResult("S21-stop-guard-allow", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S21-stop-guard-allow", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_stop_guard_block() -> ScenarioResult:
    """S22: compat stop-guard 阻止停止"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        with open(fusion_dir / "sessions.json", "w") as f:
            json.dump({
                "status": "in_progress", "current_phase": "EXECUTE",
                "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
            }, f)
        with open(fusion_dir / "task_plan.md", "w") as f:
            f.write("### Task 1: A [PENDING]\n")

        result = adapt_stop_guard(str(fusion_dir))
        assert result.should_block
        assert result.decision == "block"
        return ScenarioResult("S22-stop-guard-block", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S22-stop-guard-block", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_phase_correction() -> ScenarioResult:
    """S23: compat 阶段纠正 EXECUTE→VERIFY"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        with open(fusion_dir / "sessions.json", "w") as f:
            json.dump({
                "status": "in_progress", "current_phase": "EXECUTE",
                "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
            }, f)
        with open(fusion_dir / "task_plan.md", "w") as f:
            f.write("### Task 1: A [COMPLETED]\n### Task 2: B [COMPLETED]\n")

        result = adapt_stop_guard(str(fusion_dir))
        assert result.phase_corrected
        assert "ALL_TASKS_DONE" in result.events_dispatched
        return ScenarioResult("S23-phase-correction", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S23-phase-correction", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_pretool_active() -> ScenarioResult:
    """S24: compat pretool 活跃输出"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        with open(fusion_dir / "sessions.json", "w") as f:
            json.dump({"status": "in_progress", "current_phase": "EXECUTE", "goal": "test"}, f)
        with open(fusion_dir / "task_plan.md", "w") as f:
            f.write("### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n")

        result = adapt_pretool(str(fusion_dir))
        assert result.active
        assert len(result.lines) >= 2
        assert "[fusion]" in result.lines[0]
        return ScenarioResult("S24-pretool-active", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S24-pretool-active", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_pretool_inactive() -> ScenarioResult:
    """S25: compat pretool 非活跃"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        result = adapt_pretool(str(fusion_dir))
        assert not result.active
        return ScenarioResult("S25-pretool-inactive", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S25-pretool-inactive", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_posttool_change() -> ScenarioResult:
    """S26: compat posttool 进度变化"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        with open(fusion_dir / "sessions.json", "w") as f:
            json.dump({"status": "in_progress", "current_phase": "EXECUTE"}, f)
        with open(fusion_dir / "task_plan.md", "w") as f:
            f.write("### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n")
        (fusion_dir / ".progress_snapshot").write_text("0:2:0:0")

        result = adapt_posttool(str(fusion_dir))
        assert result.changed
        return ScenarioResult("S26-posttool-change", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S26-posttool-change", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_posttool_no_change() -> ScenarioResult:
    """S27: compat posttool 无变化"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        with open(fusion_dir / "sessions.json", "w") as f:
            json.dump({"status": "in_progress", "current_phase": "EXECUTE"}, f)
        with open(fusion_dir / "task_plan.md", "w") as f:
            f.write("### Task 1: A [PENDING]\n")
        (fusion_dir / ".progress_snapshot").write_text("0:1:0:0")

        result = adapt_posttool(str(fusion_dir))
        assert not result.changed
        return ScenarioResult("S27-posttool-nochange", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S27-posttool-nochange", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_error_then_resume() -> ScenarioResult:
    """S28: Kernel 错误→恢复→继续"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = _advance_kernel_to_execute(fusion_dir)
        k.dispatch(Event.ERROR_OCCURRED, {"error": "test"})
        assert k.current_state == State.ERROR
        k.dispatch(Event.RECOVER)
        assert k.current_state == State.EXECUTE

        # 继续完成
        _write_all_done_task_plan(fusion_dir)
        k._load_task_context()
        k.dispatch(Event.ALL_TASKS_DONE)
        assert k.current_state == State.VERIFY
        return ScenarioResult("S28-error-resume", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S28-error-resume", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_pause_crash_resume() -> ScenarioResult:
    """S29: Kernel 暂停→崩溃→恢复"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k1 = FusionKernel(fusion_dir=str(fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)
        k1.dispatch(Event.ANALYZE_DONE)
        k1.dispatch(Event.DECOMPOSE_DONE)
        k1.dispatch(Event.PAUSE)
        assert k1.current_state == State.PAUSED

        # 模拟崩溃：新实例
        k2 = FusionKernel(fusion_dir=str(fusion_dir))
        k2.load_state()
        assert k2.current_state == State.PAUSED

        k2.dispatch(Event.RESUME)
        assert k2.current_state == State.EXECUTE
        return ScenarioResult("S29-pause-crash-resume", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S29-pause-crash-resume", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_snapshot_corruption_replay() -> ScenarioResult:
    """S30: 快照损坏→事件重放恢复"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k1 = FusionKernel(fusion_dir=str(fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)
        k1.dispatch(Event.ANALYZE_DONE)
        expected = k1.current_state  # DECOMPOSE

        # 破坏快照
        (fusion_dir / "sessions.json").write_text("CORRUPT!!!")

        k2 = FusionKernel(fusion_dir=str(fusion_dir))
        k2.load_state_from_events()
        assert k2.current_state == expected, f"Expected {expected}, got {k2.current_state}"
        return ScenarioResult("S30-corruption-replay", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S30-corruption-replay", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_listener_integration() -> ScenarioResult:
    """S31: Kernel EventBus 监听器集成"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        events_received = []
        k.on("state_changed", lambda data: events_received.append(data))

        k.dispatch(Event.START)
        k.dispatch(Event.INIT_DONE)

        assert len(events_received) >= 2
        return ScenarioResult("S31-listener-integration", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S31-listener-integration", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_cli_stop_guard() -> ScenarioResult:
    """S32: compat_v2 CLI stop-guard 输出 JSON"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        with open(fusion_dir / "sessions.json", "w") as f:
            json.dump({
                "status": "in_progress", "current_phase": "EXECUTE",
                "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
            }, f)
        with open(fusion_dir / "task_plan.md", "w") as f:
            f.write("### Task 1: A [PENDING]\n")

        scripts_dir = str(Path(__file__).parent.parent)
        proc = subprocess.run(
            [sys.executable, "-m", "runtime.compat_v2", "stop-guard", str(fusion_dir)],
            capture_output=True, text=True, cwd=scripts_dir, timeout=10
        )
        assert proc.returncode == 0, f"exit={proc.returncode} stderr={proc.stderr}"
        output = json.loads(proc.stdout)
        assert output["decision"] == "block"
        return ScenarioResult("S32-cli-stop-guard", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S32-cli-stop-guard", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_cli_pretool() -> ScenarioResult:
    """S33: compat_v2 CLI pretool 输出行"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        with open(fusion_dir / "sessions.json", "w") as f:
            json.dump({"status": "in_progress", "current_phase": "EXECUTE", "goal": "test"}, f)
        with open(fusion_dir / "task_plan.md", "w") as f:
            f.write("### Task 1: A [PENDING]\n")

        scripts_dir = str(Path(__file__).parent.parent)
        proc = subprocess.run(
            [sys.executable, "-m", "runtime.compat_v2", "pretool", str(fusion_dir)],
            capture_output=True, text=True, cwd=scripts_dir, timeout=10
        )
        assert proc.returncode == 0, f"exit={proc.returncode} stderr={proc.stderr}"
        assert "[fusion]" in proc.stdout
        return ScenarioResult("S33-cli-pretool", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S33-cli-pretool", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_verify_fallback() -> ScenarioResult:
    """S34: VERIFY + PENDING → 退回 EXECUTE"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        with open(fusion_dir / "sessions.json", "w") as f:
            json.dump({
                "status": "in_progress", "current_phase": "VERIFY",
                "_runtime": {"version": "2.1.0", "state": "VERIFY", "last_event_counter": 4}
            }, f)
        with open(fusion_dir / "task_plan.md", "w") as f:
            f.write("### Task 1: A [COMPLETED]\n### Task 2: B [PENDING]\n")

        result = adapt_stop_guard(str(fusion_dir))
        assert result.phase_corrected
        assert "VERIFY_FAIL" in result.events_dispatched
        return ScenarioResult("S34-verify-fallback", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S34-verify-fallback", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_full_interrupt_every_step() -> ScenarioResult:
    """S35: 全工作流每步中断→恢复"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        # 阶段1: IDLE → EXECUTE（每步新建 kernel 模拟中断）
        early_events = [Event.START, Event.INIT_DONE, Event.ANALYZE_DONE, Event.DECOMPOSE_DONE]
        for i, evt in enumerate(early_events):
            k = FusionKernel(fusion_dir=str(fusion_dir))
            k.load_state()
            r = k.dispatch(evt)
            assert r.success, f"Step {i} failed at {evt}: {r.error}"

        # 阶段2: EXECUTE → COMPLETED（需要 task context）
        _write_all_done_task_plan(fusion_dir)
        late_events = [Event.ALL_TASKS_DONE, Event.VERIFY_PASS, Event.REVIEW_PASS,
                       Event.COMMIT_DONE, Event.DELIVER_DONE]
        for i, evt in enumerate(late_events):
            k = FusionKernel(fusion_dir=str(fusion_dir))
            k.load_state()
            r = k.dispatch(evt)
            assert r.success, f"Late step {i} failed at {evt}: {r.error}"

        k_final = FusionKernel(fusion_dir=str(fusion_dir))
        k_final.load_state()
        assert k_final.current_state == State.COMPLETED
        return ScenarioResult("S35-interrupt-every-step", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("S35-interrupt-every-step", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


# ──────────────────────────────────────────────
# 恢复可靠性专项
# ──────────────────────────────────────────────

def scenario_resume_reliability_single() -> ScenarioResult:
    """恢复可靠性单次测试"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_dir()
    try:
        import random
        early_events = [Event.START, Event.INIT_DONE, Event.ANALYZE_DONE, Event.DECOMPOSE_DONE]
        late_events = [Event.ALL_TASKS_DONE, Event.VERIFY_PASS, Event.REVIEW_PASS,
                       Event.COMMIT_DONE, Event.DELIVER_DONE]
        all_events = early_events + late_events

        # 随机中断点
        interrupt_at = random.randint(1, len(all_events) - 1)

        k1 = FusionKernel(fusion_dir=str(fusion_dir))
        for i, evt in enumerate(all_events[:interrupt_at]):
            # 在 ALL_TASKS_DONE 前设置 task context
            if evt == Event.ALL_TASKS_DONE:
                _write_all_done_task_plan(fusion_dir)
                k1._load_task_context()
            r = k1.dispatch(evt)
            assert r.success, f"Step {i} failed: {r.error}"

        # 模拟崩溃：新实例恢复
        k2 = FusionKernel(fusion_dir=str(fusion_dir))
        k2.load_state()

        # 继续
        for evt in all_events[interrupt_at:]:
            if evt == Event.ALL_TASKS_DONE:
                _write_all_done_task_plan(fusion_dir)
                k2._load_task_context()
            r = k2.dispatch(evt)
            assert r.success, f"Resume failed at {evt}: {r.error}"

        assert k2.current_state == State.COMPLETED
        return ScenarioResult(f"resume-reliability(int@{interrupt_at})", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("resume-reliability", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


# ──────────────────────────────────────────────
# Phase 2 场景: DAG / 冲突 / 预算 / 路由 / 调度 / 集成
# ──────────────────────────────────────────────

_TASK_PLAN_PARALLEL = """\
## Tasks

### Task 1: 用户模块 [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/user.py]

### Task 2: 订单模块 [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/order.py]

### Task 3: 支付模块 [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/payment.py]

### Task 4: 集成测试 [PENDING]
- Type: verification
- Dependencies: [1, 2, 3]
- Writeset: [tests/integration.py]
"""

_TASK_PLAN_CONFLICT = """\
## Tasks

### Task 1: 模块A写入shared [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/shared.py, src/a.py]

### Task 2: 模块B写入shared [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/shared.py, src/b.py]

### Task 3: 模块C独立 [PENDING]
- Type: documentation
- Dependencies: []
- Writeset: [src/c.py]
"""


def _make_fusion_with_tasks(task_plan_content: str) -> Path:
    """创建带 task_plan.md 和 sessions.json 的 .fusion 目录"""
    fusion_dir = _make_fusion_dir()
    (fusion_dir / "task_plan.md").write_text(task_plan_content, encoding="utf-8")
    (fusion_dir / "sessions.json").write_text(json.dumps({
        "status": "in_progress", "goal": "regression test",
        "current_phase": "EXECUTE",
        "_runtime": {"version": "2.1.0", "last_event_counter": 0},
    }, ensure_ascii=False), encoding="utf-8")
    return fusion_dir


def scenario_dag_topological_sort() -> ScenarioResult:
    """P01: DAG 拓扑排序产出正确批次"""
    t0 = time.monotonic()
    try:
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
            TaskNode(task_id="3", name="C", dependencies=["1"]),
            TaskNode(task_id="4", name="D", dependencies=["2", "3"]),
        ])
        batches = graph.topological_sort()
        assert len(batches) == 3
        assert batches[0].task_ids == ["1"]
        assert set(batches[1].task_ids) == {"2", "3"}
        assert batches[2].task_ids == ["4"]
        return ScenarioResult("P01-dag-topo-sort", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P01-dag-topo-sort", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_dag_circular_detection() -> ScenarioResult:
    """P02: DAG 循环依赖检测"""
    t0 = time.monotonic()
    try:
        graph = TaskGraph([
            TaskNode(task_id="1", name="A", dependencies=["2"]),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        errors = graph.validate()
        assert any("Circular" in e for e in errors)
        return ScenarioResult("P02-dag-cycle-detect", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P02-dag-cycle-detect", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_dag_from_task_plan() -> ScenarioResult:
    """P03: 从 task_plan.md 解析 DAG"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_with_tasks(_TASK_PLAN_PARALLEL)
    try:
        graph = TaskGraph.from_task_plan(str(fusion_dir / "task_plan.md"))
        assert graph.node_count == 4
        assert graph.get_node("4").dependencies == ["1", "2", "3"]
        return ScenarioResult("P03-dag-from-taskplan", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P03-dag-from-taskplan", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_dag_ready_tasks() -> ScenarioResult:
    """P04: DAG 就绪任务查询"""
    t0 = time.monotonic()
    try:
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        ready = graph.get_ready_tasks()
        assert len(ready) == 1
        assert ready[0].task_id == "1"

        graph.mark_completed("1")
        ready = graph.get_ready_tasks()
        assert len(ready) == 1
        assert ready[0].task_id == "2"
        return ScenarioResult("P04-dag-ready-tasks", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P04-dag-ready-tasks", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_dag_duplicate_deps() -> ScenarioResult:
    """P05: DAG 重复依赖去重"""
    t0 = time.monotonic()
    try:
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1", "1"]),
        ])
        assert graph.validate() == []
        batches = graph.topological_sort()
        assert len(batches) == 2
        return ScenarioResult("P05-dag-dup-deps", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P05-dag-dup-deps", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_conflict_detection() -> ScenarioResult:
    """P06: 文件冲突检测"""
    t0 = time.monotonic()
    try:
        detector = ConflictDetector()
        tasks = [
            TaskNode(task_id="1", name="A", writeset=["shared.py"]),
            TaskNode(task_id="2", name="B", writeset=["shared.py"]),
            TaskNode(task_id="3", name="C", writeset=["other.py"]),
        ]
        result = detector.check(tasks)
        assert "1" in result.safe_tasks or "2" in result.safe_tasks
        assert "3" in result.safe_tasks
        assert len(result.deferred_tasks) > 0
        return ScenarioResult("P06-conflict-detect", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P06-conflict-detect", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_conflict_no_overlap() -> ScenarioResult:
    """P07: 无冲突全部安全"""
    t0 = time.monotonic()
    try:
        detector = ConflictDetector()
        tasks = [
            TaskNode(task_id="1", name="A", writeset=["a.py"]),
            TaskNode(task_id="2", name="B", writeset=["b.py"]),
        ]
        result = detector.check(tasks)
        assert len(result.safe_tasks) == 2
        assert len(result.deferred_tasks) == 0
        return ScenarioResult("P07-no-conflict", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P07-no-conflict", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_budget_tracking() -> ScenarioResult:
    """P08: 预算追踪与超预算检测"""
    t0 = time.monotonic()
    try:
        bm = BudgetManager(BudgetConfig(global_token_limit=1000))
        assert not bm.is_over_budget()
        bm.record_usage("t1", tokens=600, latency_ms=100)
        assert not bm.is_over_budget()
        bm.record_usage("t2", tokens=500, latency_ms=100)
        assert bm.is_over_budget()
        return ScenarioResult("P08-budget-tracking", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P08-budget-tracking", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_budget_can_execute() -> ScenarioResult:
    """P09: 预算可执行判断"""
    t0 = time.monotonic()
    try:
        bm = BudgetManager(BudgetConfig(global_token_limit=1000))
        bm.record_usage("t1", tokens=900, latency_ms=0)
        assert not bm.can_execute(cost_budget=200, latency_budget=0)
        assert bm.can_execute(cost_budget=50, latency_budget=0)
        return ScenarioResult("P09-budget-can-exec", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P09-budget-can-exec", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_budget_warning() -> ScenarioResult:
    """P10: 预算警告"""
    t0 = time.monotonic()
    try:
        bm = BudgetManager(BudgetConfig(global_token_limit=1000, warning_threshold=0.8))
        bm.record_usage("t1", tokens=850, latency_ms=0)
        assert bm.is_warning()
        suggestion = bm.suggest_downgrade()
        assert suggestion is not None
        return ScenarioResult("P10-budget-warning", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P10-budget-warning", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_router_type_based() -> ScenarioResult:
    """P11: 路由按任务类型分配"""
    t0 = time.monotonic()
    try:
        router = Router()
        impl_task = TaskNode(task_id="1", name="A", task_type="implementation")
        doc_task = TaskNode(task_id="2", name="B", task_type="documentation")
        assert router.route(impl_task).backend == "codex"
        assert router.route(doc_task).backend == "claude"
        return ScenarioResult("P11-router-type", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P11-router-type", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_router_budget_downgrade() -> ScenarioResult:
    """P12: 路由超预算降级"""
    t0 = time.monotonic()
    try:
        bm = BudgetManager(BudgetConfig(global_token_limit=100))
        bm.record_usage("prev", tokens=100, latency_ms=0)
        router = Router(budget_manager=bm)
        task = TaskNode(task_id="1", name="A", task_type="implementation")
        decision = router.route(task)
        assert decision.backend == "claude"
        return ScenarioResult("P12-router-downgrade", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P12-router-downgrade", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_scheduler_serial_mode() -> ScenarioResult:
    """P13: 调度器关闭时串行"""
    t0 = time.monotonic()
    try:
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
            TaskNode(task_id="3", name="C"),
        ])
        sched = Scheduler(graph=graph, config=SchedulerConfig(enabled=False))
        decision = sched.pick_next_batch()
        assert decision is not None
        assert len(decision.batch.tasks) == 1
        return ScenarioResult("P13-sched-serial", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P13-sched-serial", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_scheduler_parallel_mode() -> ScenarioResult:
    """P14: 调度器启用时并行"""
    t0 = time.monotonic()
    try:
        graph = TaskGraph([
            TaskNode(task_id="1", name="A", writeset=["a.py"]),
            TaskNode(task_id="2", name="B", writeset=["b.py"]),
        ])
        sched = Scheduler(graph=graph, config=SchedulerConfig(enabled=True, max_parallel=2))
        decision = sched.pick_next_batch()
        assert len(decision.batch.tasks) == 2
        return ScenarioResult("P14-sched-parallel", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P14-sched-parallel", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_scheduler_conflict_defer() -> ScenarioResult:
    """P15: 调度器冲突推迟"""
    t0 = time.monotonic()
    try:
        graph = TaskGraph([
            TaskNode(task_id="1", name="A", writeset=["shared.py"]),
            TaskNode(task_id="2", name="B", writeset=["shared.py"]),
        ])
        sched = Scheduler(graph=graph, config=SchedulerConfig(enabled=True, max_parallel=2))
        decision = sched.pick_next_batch()
        assert len(decision.batch.tasks) == 1
        assert len(decision.deferred) == 1
        return ScenarioResult("P15-sched-conflict", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P15-sched-conflict", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_scheduler_budget_skip() -> ScenarioResult:
    """P16: 调度器预算跳过"""
    t0 = time.monotonic()
    try:
        bm = BudgetManager(BudgetConfig(global_token_limit=100))
        bm.record_usage("prev", tokens=80, latency_ms=0)
        graph = TaskGraph([
            TaskNode(task_id="1", name="A", cost_budget=50),
            TaskNode(task_id="2", name="B", cost_budget=10),
        ])
        sched = Scheduler(graph=graph, config=SchedulerConfig(enabled=True), budget_manager=bm)
        decision = sched.pick_next_batch()
        assert "1" in decision.budget_skipped
        assert "2" in decision.batch.task_ids
        return ScenarioResult("P16-sched-budget-skip", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P16-sched-budget-skip", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_scheduler_lifecycle() -> ScenarioResult:
    """P17: 调度器完整生命周期"""
    t0 = time.monotonic()
    try:
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        sched = Scheduler(graph=graph, config=SchedulerConfig(enabled=True))
        d1 = sched.pick_next_batch()
        assert d1.batch.task_ids == ["1"]

        sched.on_task_done("1", tokens_used=100)
        sched.on_batch_done()

        d2 = sched.pick_next_batch()
        assert d2.batch.task_ids == ["2"]

        sched.on_task_done("2", tokens_used=100)
        assert sched.is_all_done()
        return ScenarioResult("P17-sched-lifecycle", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P17-sched-lifecycle", False, (time.monotonic() - t0) * 1000, str(e))


def scenario_kernel_init_scheduler() -> ScenarioResult:
    """P18: Kernel 初始化 Scheduler"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_with_tasks(_TASK_PLAN_PARALLEL)
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        k.load_state()
        sched = k.init_scheduler(scheduler_config=SchedulerConfig(enabled=True, max_parallel=3))
        assert sched is not None
        assert k.context.scheduler_enabled
        progress = sched.get_progress()
        assert progress["total"] == 4
        return ScenarioResult("P18-kernel-init-sched", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P18-kernel-init-sched", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_parallel_batch() -> ScenarioResult:
    """P19: Kernel 并行批次调度"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_with_tasks(_TASK_PLAN_PARALLEL)
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        k.load_state()
        k.init_scheduler(scheduler_config=SchedulerConfig(enabled=True, max_parallel=3))

        decision = k.get_next_batch()
        assert decision is not None
        assert len(decision.batch.tasks) == 3
        assert "4" not in decision.batch.task_ids
        return ScenarioResult("P19-kernel-parallel-batch", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P19-kernel-parallel-batch", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_complete_task() -> ScenarioResult:
    """P20: Kernel complete_task 更新上下文"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_with_tasks(_TASK_PLAN_PARALLEL)
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        k.load_state()
        k.init_scheduler(scheduler_config=SchedulerConfig(enabled=True, max_parallel=3))

        k.complete_task("1", tokens_used=500, latency_ms=100)
        assert k.context.completed_tasks == 1
        assert k.context.pending_tasks == 3

        # 验证 sessions.json 有 scheduler 数据
        data = json.loads((fusion_dir / "sessions.json").read_text())
        assert "scheduler" in data.get("_runtime", {})
        return ScenarioResult("P20-kernel-complete-task", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P20-kernel-complete-task", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_scheduler_conflict() -> ScenarioResult:
    """P21: Kernel 调度器冲突处理"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_with_tasks(_TASK_PLAN_CONFLICT)
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        k.load_state()
        k.init_scheduler(scheduler_config=SchedulerConfig(enabled=True, max_parallel=3))

        decision = k.get_next_batch()
        assert not {"1", "2"}.issubset(set(decision.batch.task_ids))
        assert "3" in decision.batch.task_ids
        return ScenarioResult("P21-kernel-sched-conflict", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P21-kernel-sched-conflict", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_status_with_scheduler() -> ScenarioResult:
    """P22: Kernel get_status 包含 scheduler"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_with_tasks(_TASK_PLAN_PARALLEL)
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        k.load_state()
        k.init_scheduler(scheduler_config=SchedulerConfig(enabled=True))
        status = k.get_status()
        assert "scheduler" in status
        assert status["scheduler"]["enabled"]
        return ScenarioResult("P22-kernel-status-sched", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P22-kernel-status-sched", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_compat_pretool_batch() -> ScenarioResult:
    """P23: compat pretool 显示批次信息"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_with_tasks(_TASK_PLAN_PARALLEL)
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        k.load_state()
        k.init_scheduler(scheduler_config=SchedulerConfig(enabled=True, max_parallel=3))
        k.complete_task("1", tokens_used=100, latency_ms=50)

        result = adapt_pretool(str(fusion_dir))
        assert result.active
        batch_lines = [l for l in result.lines if "Batch" in l]
        assert len(batch_lines) > 0
        return ScenarioResult("P23-compat-pretool-batch", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P23-compat-pretool-batch", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_kernel_no_scheduler() -> ScenarioResult:
    """P24: Kernel 未初始化 scheduler 时 get_next_batch 返回 None"""
    t0 = time.monotonic()
    fusion_dir = _make_fusion_with_tasks(_TASK_PLAN_PARALLEL)
    try:
        k = FusionKernel(fusion_dir=str(fusion_dir))
        assert k.get_next_batch() is None
        assert k.scheduler is None
        return ScenarioResult("P24-kernel-no-sched", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P24-kernel-no-sched", False, (time.monotonic() - t0) * 1000, str(e))
    finally:
        _cleanup(fusion_dir)


def scenario_dag_parse_brackets_in_name() -> ScenarioResult:
    """P25: task_plan.md 任务名含中括号"""
    t0 = time.monotonic()
    try:
        content = "### Task 1: Fix [Auth] module [PENDING]\n- Dependencies: []\n"
        graph = TaskGraph.from_task_plan_content(content)
        node = graph.get_node("1")
        assert node is not None
        assert node.status == "PENDING"
        assert "[Auth]" in node.name
        return ScenarioResult("P25-dag-brackets-name", True, (time.monotonic() - t0) * 1000)
    except Exception as e:
        return ScenarioResult("P25-dag-brackets-name", False, (time.monotonic() - t0) * 1000, str(e))


# ──────────────────────────────────────────────
# 场景注册表
# ──────────────────────────────────────────────

PHASE1_SCENARIOS = [
    scenario_fsm_basic_transitions,
    scenario_fsm_error_recovery,
    scenario_fsm_pause_resume,
    scenario_fsm_cancel,
    scenario_fsm_verify_fail_loop,
    scenario_fsm_review_fail_loop,
    scenario_fsm_invalid_transition,
    scenario_fsm_guard_conditions,
    scenario_kernel_dispatch_and_persist,
    scenario_kernel_full_workflow,
    scenario_kernel_load_state,
    scenario_kernel_event_replay,
    scenario_kernel_idempotent_dispatch,
    scenario_event_bus_pubsub,
    scenario_event_bus_error_isolation,
    scenario_event_bus_wildcard,
    scenario_session_store_append_load,
    scenario_session_store_idempotency,
    scenario_session_store_replay,
    scenario_session_store_snapshot,
    scenario_compat_stop_guard_allow,
    scenario_compat_stop_guard_block,
    scenario_compat_phase_correction,
    scenario_compat_pretool_active,
    scenario_compat_pretool_inactive,
    scenario_compat_posttool_change,
    scenario_compat_posttool_no_change,
    scenario_kernel_error_then_resume,
    scenario_kernel_pause_crash_resume,
    scenario_kernel_snapshot_corruption_replay,
    scenario_kernel_listener_integration,
    scenario_compat_cli_stop_guard,
    scenario_compat_cli_pretool,
    scenario_compat_verify_fallback,
    scenario_full_interrupt_every_step,
]

PHASE2_SCENARIOS = [
    scenario_dag_topological_sort,
    scenario_dag_circular_detection,
    scenario_dag_from_task_plan,
    scenario_dag_ready_tasks,
    scenario_dag_duplicate_deps,
    scenario_conflict_detection,
    scenario_conflict_no_overlap,
    scenario_budget_tracking,
    scenario_budget_can_execute,
    scenario_budget_warning,
    scenario_router_type_based,
    scenario_router_budget_downgrade,
    scenario_scheduler_serial_mode,
    scenario_scheduler_parallel_mode,
    scenario_scheduler_conflict_defer,
    scenario_scheduler_budget_skip,
    scenario_scheduler_lifecycle,
    scenario_kernel_init_scheduler,
    scenario_kernel_parallel_batch,
    scenario_kernel_complete_task,
    scenario_kernel_scheduler_conflict,
    scenario_kernel_status_with_scheduler,
    scenario_compat_pretool_batch,
    scenario_kernel_no_scheduler,
    scenario_dag_parse_brackets_in_name,
]

ALL_SCENARIOS = PHASE1_SCENARIOS + PHASE2_SCENARIOS


def run_suite(scenarios, label="phase1"):
    """运行场景列表并输出报告"""
    results = []
    for fn in scenarios:
        r = fn()
        results.append(r)
        status = "✅" if r.passed else "❌"
        print(f"  {status} {r.name} ({r.duration_ms:.1f}ms){'' if r.passed else ' - ' + r.error}")

    passed = sum(1 for r in results if r.passed)
    total = len(results)
    rate = passed / total if total > 0 else 0
    total_ms = sum(r.duration_ms for r in results)

    print(f"\n{'='*60}")
    print(f"Suite: {label}")
    print(f"Passed: {passed}/{total} ({rate*100:.1f}%)")
    print(f"Total time: {total_ms:.1f}ms")
    print(f"{'='*60}")

    return results, rate


def main():
    parser = argparse.ArgumentParser(description="Fusion v2.5.0 回归测试运行器")
    parser.add_argument("--suite", default="all", help="测试套件 (phase1|phase2|all)")
    parser.add_argument("--scenario", help="专项场景 (resume_reliability)")
    parser.add_argument("--runs", type=int, default=20, help="重复次数 (用于可靠性测试)")
    parser.add_argument("--min-pass-rate", type=float, default=0.99, help="最低通过率")
    args = parser.parse_args()

    if args.scenario == "resume_reliability":
        print(f"🔄 Resume Reliability Test ({args.runs} runs)")
        print("-" * 60)
        scenarios = [scenario_resume_reliability_single for _ in range(args.runs)]
        results, rate = run_suite(scenarios, label="resume_reliability")
    elif args.suite == "phase1":
        print(f"🧪 Phase 1 Regression Suite ({len(PHASE1_SCENARIOS)} scenarios)")
        print("-" * 60)
        results, rate = run_suite(PHASE1_SCENARIOS, label="phase1")
    elif args.suite == "phase2":
        print(f"🧪 Phase 2 Regression Suite ({len(PHASE2_SCENARIOS)} scenarios)")
        print("-" * 60)
        results, rate = run_suite(PHASE2_SCENARIOS, label="phase2")
    else:
        print(f"🧪 Full Regression Suite ({len(ALL_SCENARIOS)} scenarios)")
        print("-" * 60)
        results, rate = run_suite(ALL_SCENARIOS, label="all")

    if rate < args.min_pass_rate:
        print(f"\n❌ FAIL: Pass rate {rate*100:.1f}% < {args.min_pass_rate*100:.1f}%")
        sys.exit(1)
    else:
        print(f"\n✅ PASS: Pass rate {rate*100:.1f}% ≥ {args.min_pass_rate*100:.1f}%")
        sys.exit(0)


if __name__ == "__main__":
    main()
