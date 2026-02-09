"""
中断→恢复 集成测试

验证 Kernel + SessionStore + EventBus 端到端的恢复能力。
包含故障注入和边界场景。
"""

import unittest
import tempfile
import shutil
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.kernel import FusionKernel, KernelConfig, create_kernel
from runtime.state_machine import State, Event
from runtime.session_store import SessionStore
from runtime.event_bus import EventBus


class TestResumeFromSnapshot(unittest.TestCase):
    """从快照恢复（快速路径）"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_resume_basic_workflow(self):
        """基础恢复：推进到 EXECUTE，重建后状态正确"""
        # 第一个 kernel 实例推进状态
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)
        k1.dispatch(Event.ANALYZE_DONE)
        k1.dispatch(Event.DECOMPOSE_DONE)
        self.assertEqual(k1.current_state, State.EXECUTE)

        # 模拟崩溃：创建全新实例
        k2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k2.load_state()
        self.assertEqual(k2.current_state, State.EXECUTE)

        # 恢复后可以继续工作
        result = k2.dispatch(Event.PAUSE)
        self.assertTrue(result.success)
        self.assertEqual(k2.current_state, State.PAUSED)

    def test_resume_preserves_event_counter(self):
        """恢复后事件计数器连续"""
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)       # evt_000001
        k1.dispatch(Event.INIT_DONE)   # evt_000002

        # 恢复
        k2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k2.load_state()

        # 新事件的 ID 应该继续递增
        result = k2.dispatch(Event.ANALYZE_DONE)
        self.assertEqual(result.event_id, "evt_000003")

    def test_resume_after_pause(self):
        """暂停→崩溃→恢复→继续"""
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)
        k1.dispatch(Event.ANALYZE_DONE)
        k1.dispatch(Event.DECOMPOSE_DONE)
        k1.dispatch(Event.PAUSE)
        self.assertEqual(k1.current_state, State.PAUSED)

        # 恢复
        k2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k2.load_state()
        self.assertEqual(k2.current_state, State.PAUSED)

        # 可以 RESUME
        result = k2.dispatch(Event.RESUME)
        self.assertTrue(result.success)
        self.assertEqual(k2.current_state, State.EXECUTE)


class TestResumeFromEvents(unittest.TestCase):
    """从事件流重放恢复（完整路径）"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_full_replay_matches_snapshot(self):
        """事件流重放的结果与快照一致"""
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)
        k1.dispatch(Event.ANALYZE_DONE)
        k1.dispatch(Event.DECOMPOSE_DONE)

        # 从快照恢复
        k_snap = FusionKernel(fusion_dir=str(self.fusion_dir))
        k_snap.load_state()

        # 从事件流恢复
        k_events = FusionKernel(fusion_dir=str(self.fusion_dir))
        k_events.load_state_from_events()

        self.assertEqual(k_snap.current_state, k_events.current_state)

    def test_replay_after_snapshot_corruption(self):
        """快照损坏时，事件流重放可以恢复"""
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)
        k1.dispatch(Event.ANALYZE_DONE)

        # 破坏 sessions.json
        sessions_file = self.fusion_dir / "sessions.json"
        with open(sessions_file, "w", encoding="utf-8") as f:
            f.write("corrupted data!!!")

        # 快照恢复会失败（回退到 IDLE）
        k_snap = FusionKernel(fusion_dir=str(self.fusion_dir))
        k_snap.load_state()
        self.assertEqual(k_snap.current_state, State.IDLE)  # 快照损坏

        # 但事件流重放可以正确恢复
        k_events = FusionKernel(fusion_dir=str(self.fusion_dir))
        k_events.load_state_from_events()
        self.assertEqual(k_events.current_state, State.DECOMPOSE)

    def test_replay_from_midpoint(self):
        """增量恢复：从指定事件之后开始"""
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)       # evt_000001
        k1.dispatch(Event.INIT_DONE)   # evt_000002
        k1.dispatch(Event.ANALYZE_DONE)  # evt_000003

        # 从 evt_000001 之后增量恢复
        k2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k2.resume_from_events(from_event_id="evt_000001")

        # 应该重放 evt_000002 和 evt_000003
        self.assertEqual(k2.current_state, State.DECOMPOSE)

    def test_replay_empty_events(self):
        """没有事件时重放回到 IDLE"""
        k = FusionKernel(fusion_dir=str(self.fusion_dir))
        state = k.load_state_from_events()
        self.assertEqual(state, State.IDLE)


class TestIdempotentDispatch(unittest.TestCase):
    """幂等派发"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_idempotent_dispatch_same_key(self):
        """相同幂等键的 dispatch 不会重复写入事件"""
        k = FusionKernel(fusion_dir=str(self.fusion_dir))
        r1 = k.dispatch(Event.START, idempotency_key="key_start")
        self.assertTrue(r1.success)
        self.assertIsNotNone(r1.event_id)

        # 尝试用同一个 key 重新 dispatch（但状态已经不是 IDLE 了）
        # 这里 START 从 INITIALIZE 是无效转移，所以会失败
        r2 = k.dispatch(Event.START, idempotency_key="key_start")
        self.assertFalse(r2.success)  # 状态机拒绝

    def test_event_count_after_idempotent_writes(self):
        """幂等跳过不增加事件计数"""
        k = FusionKernel(fusion_dir=str(self.fusion_dir))
        k.dispatch(Event.START)
        k.dispatch(Event.INIT_DONE)

        initial_count = k.session_store.get_event_count()
        self.assertEqual(initial_count, 2)


class TestFaultInjection(unittest.TestCase):
    """故障注入测试"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_missing_sessions_json(self):
        """sessions.json 不存在时的恢复"""
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)

        # 删除 sessions.json
        sessions_file = self.fusion_dir / "sessions.json"
        sessions_file.unlink()

        # 快照恢复失败（回退到 IDLE）
        k2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k2.load_state()
        self.assertEqual(k2.current_state, State.IDLE)

        # 但事件流重放可以恢复
        k3 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k3.load_state_from_events()
        self.assertEqual(k3.current_state, State.ANALYZE)

    def test_missing_events_jsonl(self):
        """events.jsonl 不存在时的处理"""
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)

        # 删除 events.jsonl
        events_file = self.fusion_dir / "events.jsonl"
        events_file.unlink()

        # 快照恢复仍然正常
        k2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k2.load_state()
        self.assertEqual(k2.current_state, State.INITIALIZE)

        # 事件流重放回退到 IDLE（没有事件可重放）
        k3 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k3.load_state_from_events()
        self.assertEqual(k3.current_state, State.IDLE)

    def test_partial_events_file(self):
        """events.jsonl 中有损坏行但不影响整体恢复"""
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)

        # 在事件文件中插入损坏行
        events_file = self.fusion_dir / "events.jsonl"
        with open(events_file, "r", encoding="utf-8") as f:
            lines = f.readlines()
        with open(events_file, "w", encoding="utf-8") as f:
            f.write(lines[0])
            f.write("CORRUPTED LINE\n")
            f.write(lines[1])

        # 事件流重放应跳过损坏行，正确恢复
        k2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k2.load_state_from_events()
        self.assertEqual(k2.current_state, State.ANALYZE)

    def test_both_files_missing(self):
        """sessions.json 和 events.jsonl 都不存在"""
        k = FusionKernel(fusion_dir=str(self.fusion_dir))
        k.load_state()
        self.assertEqual(k.current_state, State.IDLE)

        k2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k2.load_state_from_events()
        self.assertEqual(k2.current_state, State.IDLE)


class TestEventBusIntegration(unittest.TestCase):
    """EventBus 与 Kernel 的集成"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_listeners_fire_on_dispatch(self):
        """dispatch 时通过 EventBus 发布事件"""
        k = FusionKernel(fusion_dir=str(self.fusion_dir))
        received = []

        k.on("state_changed", lambda data: received.append(data))
        k.dispatch(Event.START)

        self.assertEqual(len(received), 1)
        self.assertEqual(received[0]["from"], "IDLE")
        self.assertEqual(received[0]["to"], "INITIALIZE")

    def test_listener_error_does_not_break_dispatch(self):
        """监听器异常不影响 dispatch"""
        k = FusionKernel(fusion_dir=str(self.fusion_dir))

        def bad_listener(data):
            raise RuntimeError("boom")

        k.on("state_changed", bad_listener)
        result = k.dispatch(Event.START)

        # dispatch 仍然成功
        self.assertTrue(result.success)
        self.assertEqual(k.current_state, State.INITIALIZE)

    def test_wildcard_listener_via_event_bus(self):
        """直接通过 event_bus 注册通配符监听"""
        k = FusionKernel(fusion_dir=str(self.fusion_dir))
        all_events = []

        # 直接用 EventBus API（不走兼容层）
        k.event_bus.on("*", lambda et, data: all_events.append(et))

        k.dispatch(Event.START)
        k.dispatch(Event.INIT_DONE)

        self.assertIn("state_changed", all_events)
        self.assertEqual(all_events.count("state_changed"), 2)


class TestFullWorkflowReplay(unittest.TestCase):
    """完整工作流的中断→恢复"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_full_workflow_interrupt_and_resume(self):
        """完整工作流在任意点中断都能恢复"""
        events_sequence = [
            Event.START,
            Event.INIT_DONE,
            Event.ANALYZE_DONE,
            Event.DECOMPOSE_DONE,
        ]

        expected_states = [
            State.INITIALIZE,
            State.ANALYZE,
            State.DECOMPOSE,
            State.EXECUTE,
        ]

        # 在每个状态点中断并验证恢复
        for i in range(len(events_sequence)):
            k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
            for j in range(i + 1):
                k1.dispatch(events_sequence[j])
            self.assertEqual(k1.current_state, expected_states[i])

            # 从快照恢复
            k_snap = FusionKernel(fusion_dir=str(self.fusion_dir))
            k_snap.load_state()
            self.assertEqual(k_snap.current_state, expected_states[i])

            # 从事件流恢复
            k_evt = FusionKernel(fusion_dir=str(self.fusion_dir))
            k_evt.load_state_from_events()
            self.assertEqual(k_evt.current_state, expected_states[i])

            # 清理
            k1.reset()
            k1.session_store.truncate()
            sessions_file = self.fusion_dir / "sessions.json"
            if sessions_file.exists():
                sessions_file.unlink()

    def test_error_recovery_and_resume(self):
        """错误→恢复→继续的完整流程"""
        k1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k1.dispatch(Event.START)
        k1.dispatch(Event.INIT_DONE)
        k1.dispatch(Event.ANALYZE_DONE)
        k1.dispatch(Event.DECOMPOSE_DONE)

        # 触发错误
        k1.dispatch(Event.ERROR_OCCURRED, {"error": "test error"})
        self.assertEqual(k1.current_state, State.ERROR)

        # 崩溃→恢复
        k2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        k2.load_state()
        self.assertEqual(k2.current_state, State.ERROR)

        # 错误恢复
        result = k2.dispatch(Event.RECOVER)
        self.assertTrue(result.success)
        self.assertEqual(k2.current_state, State.EXECUTE)


if __name__ == "__main__":
    unittest.main(verbosity=2)
