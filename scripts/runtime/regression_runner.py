"""
Fusion v2.1.0 Phase 1 回归测试运行器

用法:
    python3 scripts/runtime/regression_runner.py --suite phase1 --min-pass-rate 0.99
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
    parser = argparse.ArgumentParser(description="Fusion v2.1.0 回归测试运行器")
    parser.add_argument("--suite", default="phase1", help="测试套件 (phase1)")
    parser.add_argument("--scenario", help="专项场景 (resume_reliability)")
    parser.add_argument("--runs", type=int, default=20, help="重复次数 (用于可靠性测试)")
    parser.add_argument("--min-pass-rate", type=float, default=0.99, help="最低通过率")
    args = parser.parse_args()

    if args.scenario == "resume_reliability":
        print(f"🔄 Resume Reliability Test ({args.runs} runs)")
        print("-" * 60)
        scenarios = [scenario_resume_reliability_single for _ in range(args.runs)]
        results, rate = run_suite(scenarios, label="resume_reliability")
    else:
        print(f"🧪 Phase 1 Regression Suite ({len(PHASE1_SCENARIOS)} scenarios)")
        print("-" * 60)
        results, rate = run_suite(PHASE1_SCENARIOS, label=args.suite)

    if rate < args.min_pass_rate:
        print(f"\n❌ FAIL: Pass rate {rate*100:.1f}% < {args.min_pass_rate*100:.1f}%")
        sys.exit(1)
    else:
        print(f"\n✅ PASS: Pass rate {rate*100:.1f}% ≥ {args.min_pass_rate*100:.1f}%")
        sys.exit(0)


if __name__ == "__main__":
    main()
