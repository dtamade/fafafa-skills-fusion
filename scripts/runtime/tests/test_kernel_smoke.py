"""
Kernel Smoke Tests
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


class TestKernelBasic(unittest.TestCase):
    """Kernel 基础功能测试"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.kernel = FusionKernel(fusion_dir=str(self.fusion_dir))

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_initial_state_is_idle(self):
        """初始状态为 IDLE"""
        self.assertEqual(self.kernel.current_state, State.IDLE)

    def test_dispatch_start_event(self):
        """派发 START 事件 -> UNDERSTAND"""
        result = self.kernel.dispatch(Event.START)
        self.assertTrue(result.success)
        self.assertEqual(result.from_state, State.IDLE)
        self.assertEqual(result.to_state, State.UNDERSTAND)
        self.assertEqual(self.kernel.current_state, State.UNDERSTAND)

    def test_skip_understand(self):
        """派发 SKIP_UNDERSTAND 事件跳过理解确认"""
        result = self.kernel.dispatch(Event.SKIP_UNDERSTAND)
        self.assertTrue(result.success)
        self.assertEqual(result.from_state, State.IDLE)
        self.assertEqual(result.to_state, State.INITIALIZE)
        self.assertEqual(self.kernel.current_state, State.INITIALIZE)

    def test_dispatch_invalid_event(self):
        """派发无效事件"""
        result = self.kernel.dispatch(Event.TASK_DONE)
        self.assertFalse(result.success)
        self.assertIsNotNone(result.error)
        self.assertEqual(self.kernel.current_state, State.IDLE)

    def test_can_transition(self):
        """检查转移可行性"""
        self.assertTrue(self.kernel.can_transition(Event.START))
        self.assertFalse(self.kernel.can_transition(Event.TASK_DONE))

    def test_get_valid_events(self):
        """获取有效事件列表"""
        events = self.kernel.get_valid_events()
        self.assertIn(Event.START, events)
        self.assertIn(Event.SKIP_UNDERSTAND, events)
        self.assertIn(Event.ERROR_OCCURRED, events)

    def test_get_status(self):
        """获取状态摘要"""
        status = self.kernel.get_status()
        self.assertEqual(status["state"], "IDLE")
        self.assertIn("valid_events", status)
        self.assertIn("context", status)


class TestKernelWorkflow(unittest.TestCase):
    """Kernel 工作流测试"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.kernel = FusionKernel(fusion_dir=str(self.fusion_dir))

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_basic_workflow(self):
        """基础工作流：SKIP_UNDERSTAND -> INIT_DONE -> ANALYZE_DONE"""
        # SKIP_UNDERSTAND (跳过理解确认，直接到 INITIALIZE)
        result = self.kernel.dispatch(Event.SKIP_UNDERSTAND)
        self.assertTrue(result.success)
        self.assertEqual(self.kernel.current_state, State.INITIALIZE)

        # INIT_DONE
        result = self.kernel.dispatch(Event.INIT_DONE)
        self.assertTrue(result.success)
        self.assertEqual(self.kernel.current_state, State.ANALYZE)

        # ANALYZE_DONE
        result = self.kernel.dispatch(Event.ANALYZE_DONE)
        self.assertTrue(result.success)
        self.assertEqual(self.kernel.current_state, State.DECOMPOSE)

    def test_understand_workflow(self):
        """理解确认流程：START -> CONFIRM -> INIT_DONE"""
        # START -> UNDERSTAND
        result = self.kernel.dispatch(Event.START)
        self.assertTrue(result.success)
        self.assertEqual(self.kernel.current_state, State.UNDERSTAND)

        # CONFIRM -> INITIALIZE
        result = self.kernel.dispatch(Event.CONFIRM)
        self.assertTrue(result.success)
        self.assertEqual(self.kernel.current_state, State.INITIALIZE)

        # INIT_DONE
        result = self.kernel.dispatch(Event.INIT_DONE)
        self.assertTrue(result.success)
        self.assertEqual(self.kernel.current_state, State.ANALYZE)

    def test_pause_resume(self):
        """暂停和恢复"""
        # 进入 EXECUTE 状态 (使用 SKIP_UNDERSTAND 跳过理解确认)
        self.kernel.dispatch(Event.SKIP_UNDERSTAND)
        self.kernel.dispatch(Event.INIT_DONE)
        self.kernel.dispatch(Event.ANALYZE_DONE)
        self.kernel.dispatch(Event.DECOMPOSE_DONE)
        self.assertEqual(self.kernel.current_state, State.EXECUTE)

        # PAUSE
        result = self.kernel.dispatch(Event.PAUSE)
        self.assertTrue(result.success)
        self.assertEqual(self.kernel.current_state, State.PAUSED)

        # RESUME
        result = self.kernel.dispatch(Event.RESUME)
        self.assertTrue(result.success)
        self.assertEqual(self.kernel.current_state, State.EXECUTE)

    def test_cancel_workflow(self):
        """取消工作流"""
        self.kernel.dispatch(Event.SKIP_UNDERSTAND)
        self.kernel.dispatch(Event.INIT_DONE)

        result = self.kernel.dispatch(Event.CANCEL)
        self.assertTrue(result.success)
        self.assertEqual(self.kernel.current_state, State.CANCELLED)


class TestKernelPersistence(unittest.TestCase):
    """Kernel 持久化测试"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_save_and_load_state(self):
        """保存和加载状态"""
        # 创建内核并推进状态 (使用 SKIP_UNDERSTAND)
        kernel1 = FusionKernel(fusion_dir=str(self.fusion_dir))
        kernel1.dispatch(Event.SKIP_UNDERSTAND)
        kernel1.dispatch(Event.INIT_DONE)
        self.assertEqual(kernel1.current_state, State.ANALYZE)

        # 创建新内核并加载状态
        kernel2 = FusionKernel(fusion_dir=str(self.fusion_dir))
        loaded_state = kernel2.load_state()
        self.assertEqual(loaded_state, State.ANALYZE)
        self.assertEqual(kernel2.current_state, State.ANALYZE)

    def test_sessions_json_updated(self):
        """sessions.json 被正确更新"""
        kernel = FusionKernel(fusion_dir=str(self.fusion_dir))
        kernel.dispatch(Event.SKIP_UNDERSTAND)

        # 检查文件
        sessions_file = self.fusion_dir / "sessions.json"
        self.assertTrue(sessions_file.exists())

        with open(sessions_file, "r", encoding="utf-8") as f:
            data = json.load(f)

        self.assertEqual(data["current_phase"], "INITIALIZE")
        self.assertIn("_runtime", data)
        self.assertEqual(data["_runtime"]["version"], "2.6.3")

    def test_events_logged(self):
        """事件被记录到日志"""
        kernel = FusionKernel(fusion_dir=str(self.fusion_dir))
        kernel.dispatch(Event.SKIP_UNDERSTAND)
        kernel.dispatch(Event.INIT_DONE)

        # 检查事件日志
        events_file = self.fusion_dir / "events.jsonl"
        self.assertTrue(events_file.exists())

        with open(events_file, "r", encoding="utf-8") as f:
            lines = f.readlines()

        self.assertEqual(len(lines), 2)

        event1 = json.loads(lines[0])
        self.assertEqual(event1["type"], "SKIP_UNDERSTAND")
        self.assertEqual(event1["from_state"], "IDLE")
        self.assertEqual(event1["to_state"], "INITIALIZE")


class TestKernelEventListener(unittest.TestCase):
    """Kernel 事件监听测试"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.kernel = FusionKernel(fusion_dir=str(self.fusion_dir))
        self.events_received = []

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_state_changed_event(self):
        """state_changed 事件被发布"""
        def listener(data):
            self.events_received.append(data)

        self.kernel.on("state_changed", listener)
        self.kernel.dispatch(Event.SKIP_UNDERSTAND)

        self.assertEqual(len(self.events_received), 1)
        self.assertEqual(self.events_received[0]["from"], "IDLE")
        self.assertEqual(self.events_received[0]["to"], "INITIALIZE")


class TestCreateKernel(unittest.TestCase):
    """create_kernel 工厂函数测试"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

        # 创建预存状态
        sessions_file = self.fusion_dir / "sessions.json"
        with open(sessions_file, "w", encoding="utf-8") as f:
            json.dump({"current_phase": "EXECUTE"}, f)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_create_kernel_loads_state(self):
        """create_kernel 自动加载状态"""
        kernel = create_kernel(fusion_dir=str(self.fusion_dir))
        self.assertEqual(kernel.current_state, State.EXECUTE)

    def test_create_kernel_auto_init_scheduler_from_config(self):
        """create_kernel 自动从 config.yaml 初始化 scheduler"""
        (self.fusion_dir / "task_plan.md").write_text(
            "### Task 1: A [PENDING]\n"
            "- Type: implementation\n"
            "- Dependencies: []\n",
            encoding="utf-8",
        )
        (self.fusion_dir / "config.yaml").write_text(
            "scheduler:\n"
            "  enabled: true\n"
            "  max_parallel: 3\n"
            "backends:\n"
            "  primary: claude\n"
            "budget:\n"
            "  global_token_limit: 321\n",
            encoding="utf-8",
        )

        kernel = create_kernel(fusion_dir=str(self.fusion_dir))

        self.assertIsNotNone(kernel.scheduler)
        self.assertTrue(kernel.scheduler.config.enabled)
        self.assertEqual(kernel.scheduler.config.max_parallel, 3)
        # default backend should come from config
        decision = kernel.get_next_batch()
        self.assertIsNotNone(decision)
        task_id = decision.batch.task_ids[0]
        # implementation 类型默认路由 claude（执行写码优先宿主）
        self.assertEqual(decision.routing[task_id].backend, "claude")


if __name__ == "__main__":
    unittest.main(verbosity=2)
