"""
compat_v2 适配层单元测试
"""

import unittest
import tempfile
import shutil
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.compat_v2 import (
    adapt_stop_guard,
    adapt_pretool,
    adapt_posttool,
    is_runtime_enabled,
    StopGuardResult,
    PretoolResult,
    PosttoolResult,
)
from runtime.state_machine import State, Event


class BaseTestCase(unittest.TestCase):
    """公共 setUp/tearDown"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _write_sessions(self, data: dict):
        with open(self.fusion_dir / "sessions.json", "w", encoding="utf-8") as f:
            json.dump(data, f)

    def _write_task_plan(self, content: str):
        with open(self.fusion_dir / "task_plan.md", "w", encoding="utf-8") as f:
            f.write(content)

    def _write_config(self, enabled: bool = True):
        with open(self.fusion_dir / "config.yaml", "w", encoding="utf-8") as f:
            f.write(f"runtime:\n  enabled: {str(enabled).lower()}\n  compat_mode: true\n")


class TestIsRuntimeEnabled(BaseTestCase):
    """runtime 开关检测"""

    def test_enabled_true(self):
        self._write_config(enabled=True)
        self.assertTrue(is_runtime_enabled(str(self.fusion_dir)))

    def test_enabled_false(self):
        self._write_config(enabled=False)
        self.assertFalse(is_runtime_enabled(str(self.fusion_dir)))

    def test_no_config_file(self):
        self.assertFalse(is_runtime_enabled(str(self.fusion_dir)))

    def test_no_runtime_section(self):
        with open(self.fusion_dir / "config.yaml", "w", encoding="utf-8") as f:
            f.write("backends:\n  primary: codex\n")
        self.assertFalse(is_runtime_enabled(str(self.fusion_dir)))

    def test_runtime_enabled_requires_compat_mode(self):
        with open(self.fusion_dir / "config.yaml", "w", encoding="utf-8") as f:
            f.write("runtime:\n  enabled: true\n  compat_mode: false\n")
        self.assertFalse(is_runtime_enabled(str(self.fusion_dir)))


class TestAdaptStopGuard(BaseTestCase):
    """stop-guard 适配"""

    def test_no_sessions_allows_stop(self):
        """无 sessions.json 允许停止"""
        result = adapt_stop_guard(str(self.fusion_dir))
        self.assertFalse(result.should_block)
        self.assertEqual(result.decision, "allow")

    def test_completed_status_allows_stop(self):
        """已完成的工作流允许停止"""
        self._write_sessions({"status": "completed", "current_phase": "COMPLETED"})
        result = adapt_stop_guard(str(self.fusion_dir))
        self.assertFalse(result.should_block)

    def test_in_progress_with_pending_blocks(self):
        """有剩余任务时阻止停止"""
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
        })
        self._write_task_plan(
            "### Task 1: 任务A [COMPLETED]\n"
            "### Task 2: 任务B [PENDING]\n"
            "### Task 3: 任务C [PENDING]\n"
        )
        result = adapt_stop_guard(str(self.fusion_dir))
        self.assertTrue(result.should_block)
        self.assertEqual(result.decision, "block")
        self.assertIn("2", result.system_message)

    def test_all_tasks_done_allows_stop(self):
        """所有任务完成时允许停止"""
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
        })
        self._write_task_plan(
            "### Task 1: 任务A [COMPLETED]\n"
            "### Task 2: 任务B [COMPLETED]\n"
        )
        result = adapt_stop_guard(str(self.fusion_dir))
        self.assertFalse(result.should_block)
        self.assertEqual(result.decision, "allow")

    def test_phase_correction_execute_to_verify(self):
        """阶段纠正：EXECUTE + 全部完成 → VERIFY"""
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
        })
        self._write_task_plan(
            "### Task 1: 任务A [COMPLETED]\n"
            "### Task 2: 任务B [COMPLETED]\n"
        )
        result = adapt_stop_guard(str(self.fusion_dir))
        self.assertTrue(result.phase_corrected)
        self.assertIn("ALL_TASKS_DONE", result.events_dispatched)

    def test_phase_correction_verify_to_execute(self):
        """阶段纠正：VERIFY + 有 PENDING → EXECUTE"""
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "VERIFY",
            "_runtime": {"version": "2.1.0", "state": "VERIFY", "last_event_counter": 4}
        })
        self._write_task_plan(
            "### Task 1: 任务A [COMPLETED]\n"
            "### Task 2: 任务B [PENDING]\n"
        )
        result = adapt_stop_guard(str(self.fusion_dir))
        self.assertTrue(result.phase_corrected)
        self.assertIn("VERIFY_FAIL", result.events_dispatched)

    def test_early_phase_no_task_plan(self):
        """早期阶段无 task_plan.md → 阻止停止，提示创建"""
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "DECOMPOSE",
            "_runtime": {"version": "2.1.0", "state": "DECOMPOSE", "last_event_counter": 2}
        })
        result = adapt_stop_guard(str(self.fusion_dir))
        self.assertTrue(result.should_block)
        self.assertIn("task_plan.md", result.reason)

    def test_continuation_prompt_format(self):
        """继续提示格式正确"""
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "实现用户认证",
            "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 3}
        })
        self._write_task_plan(
            "### Task 1: 创建登录API [COMPLETED]\n"
            "### Task 2: 添加JWT验证 [PENDING]\n"
        )
        result = adapt_stop_guard(str(self.fusion_dir))
        self.assertIn("实现用户认证", result.reason)
        self.assertIn("TDD", result.reason)


class TestAdaptPretool(BaseTestCase):
    """pretool 适配"""

    def test_inactive_workflow(self):
        """无活跃工作流时不输出"""
        result = adapt_pretool(str(self.fusion_dir))
        self.assertFalse(result.active)
        self.assertEqual(result.lines, [])

    def test_active_workflow_output(self):
        """活跃工作流输出上下文"""
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "实现用户认证"
        })
        self._write_task_plan(
            "### Task 1: 创建登录API [COMPLETED]\n"
            "### Task 2: 添加JWT验证 [IN_PROGRESS]\n"
            "### Task 3: 写测试 [PENDING]\n"
        )
        result = adapt_pretool(str(self.fusion_dir))
        self.assertTrue(result.active)
        self.assertTrue(len(result.lines) >= 2)
        self.assertIn("[fusion]", result.lines[0])
        self.assertIn("EXECUTE", result.lines[0])
        self.assertIn("4/8", result.lines[0])

    def test_pretool_completed_workflow(self):
        """已完成工作流不输出"""
        self._write_sessions({"status": "completed", "current_phase": "COMPLETED"})
        result = adapt_pretool(str(self.fusion_dir))
        self.assertFalse(result.active)


class TestAdaptPosttool(BaseTestCase):
    """posttool 适配"""

    def test_no_change(self):
        """无进度变化"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan("### Task 1: A [PENDING]\n")

        # 写入相同的快照
        snap_file = self.fusion_dir / ".progress_snapshot"
        snap_file.write_text("0:1:0:0")

        result = adapt_posttool(str(self.fusion_dir))
        self.assertFalse(result.changed)

    def test_task_completed(self):
        """任务完成时检测到变化"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [PENDING]\n"
        )

        # 前一个快照：0 完成
        snap_file = self.fusion_dir / ".progress_snapshot"
        snap_file.write_text("0:2:0:0")

        result = adapt_posttool(str(self.fusion_dir))
        self.assertTrue(result.changed)
        self.assertTrue(any("completed" in line.lower() or "COMPLETED" in line for line in result.lines))

    def test_task_failed(self):
        """任务失败时检测到变化"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [FAILED]\n"
        )

        snap_file = self.fusion_dir / ".progress_snapshot"
        snap_file.write_text("1:1:0:0")

        result = adapt_posttool(str(self.fusion_dir))
        self.assertTrue(result.changed)
        self.assertTrue(any("FAILED" in line for line in result.lines))

    def test_all_completed(self):
        """全部完成时提示 VERIFY"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [COMPLETED]\n"
        )

        snap_file = self.fusion_dir / ".progress_snapshot"
        snap_file.write_text("1:1:0:0")

        result = adapt_posttool(str(self.fusion_dir))
        self.assertTrue(result.changed)
        self.assertTrue(any("VERIFY" in line for line in result.lines))

    def test_inactive_workflow(self):
        """无活跃工作流"""
        result = adapt_posttool(str(self.fusion_dir))
        self.assertFalse(result.changed)
        self.assertEqual(result.lines, [])

    def test_no_progress_triggers_safe_backlog(self):
        """连续无进展达到阈值时触发 safe_backlog"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan("### Task 1: A [PENDING]\n")

        with open(self.fusion_dir / "config.yaml", "w", encoding="utf-8") as f:
            f.write(
                "runtime:\n"
                "  enabled: true\n"
                "  compat_mode: true\n"
                "safe_backlog:\n"
                "  enabled: true\n"
                "  trigger_no_progress_rounds: 2\n"
                "  max_tasks_per_run: 1\n"
                "  allowed_categories: documentation\n"
            )

        project_root = self.fusion_dir.parent
        (project_root / "README.md").write_text("# Demo\n", encoding="utf-8")
        (self.fusion_dir / ".progress_snapshot").write_text("0:1:0:0", encoding="utf-8")

        first = adapt_posttool(str(self.fusion_dir))
        self.assertFalse(first.changed)

        second = adapt_posttool(str(self.fusion_dir))
        self.assertTrue(second.changed)
        self.assertTrue(any("safe backlog" in line.lower() for line in second.lines))

        task_plan = (self.fusion_dir / "task_plan.md").read_text(encoding="utf-8")
        self.assertIn("[SAFE_BACKLOG]", task_plan)

        events_file = self.fusion_dir / "events.jsonl"
        self.assertTrue(events_file.exists())
        events = [json.loads(line) for line in events_file.read_text(encoding="utf-8").splitlines() if line.strip()]
        matching = [evt for evt in events if evt.get("type") == "SAFE_BACKLOG_INJECTED"]
        self.assertTrue(matching)
        payload = matching[-1].get("payload", {})
        self.assertEqual(payload.get("reason"), "no_progress")
        self.assertIn("stall_score", payload)
        self.assertGreaterEqual(float(payload.get("stall_score")), 0.0)

    def test_task_exhausted_triggers_safe_backlog(self):
        """任务耗尽（无 pending/in_progress）时触发托底注入"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan("### Task 1: A [COMPLETED]\n")

        with open(self.fusion_dir / "config.yaml", "w", encoding="utf-8") as f:
            f.write(
                "runtime:\n"
                "  enabled: true\n"
                "  compat_mode: true\n"
                "safe_backlog:\n"
                "  enabled: true\n"
                "  inject_on_task_exhausted: true\n"
                "  max_tasks_per_run: 1\n"
                "  allowed_categories: documentation\n"
            )

        project_root = self.fusion_dir.parent
        (project_root / "README.md").write_text("# Demo\n", encoding="utf-8")

        result = adapt_posttool(str(self.fusion_dir))
        self.assertTrue(result.changed)
        self.assertTrue(any("safe backlog" in line.lower() for line in result.lines))

        task_plan = (self.fusion_dir / "task_plan.md").read_text(encoding="utf-8")
        self.assertIn("[SAFE_BACKLOG]", task_plan)

        events_file = self.fusion_dir / "events.jsonl"
        self.assertTrue(events_file.exists())
        events = [json.loads(line) for line in events_file.read_text(encoding="utf-8").splitlines() if line.strip()]
        matching = [evt for evt in events if evt.get("type") == "SAFE_BACKLOG_INJECTED"]
        self.assertTrue(matching)
        payload = matching[-1].get("payload", {})
        self.assertEqual(payload.get("reason"), "task_exhausted")
        self.assertIn("stall_score", payload)

    def test_backoff_blocks_immediate_reinjection(self):
        """指数退避冷却期间不应重复注入，冷却后才允许"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan("### Task 1: A [PENDING]\n")

        with open(self.fusion_dir / "config.yaml", "w", encoding="utf-8") as f:
            f.write(
                "runtime:\n"
                "  enabled: true\n"
                "  compat_mode: true\n"
                "safe_backlog:\n"
                "  enabled: true\n"
                "  trigger_no_progress_rounds: 1\n"
                "  max_tasks_per_run: 1\n"
                "  allowed_categories: quality,documentation\n"
                "  backoff_enabled: true\n"
                "  backoff_base_rounds: 2\n"
                "  backoff_max_rounds: 8\n"
                "  backoff_jitter: 0\n"
                "  backoff_force_probe_rounds: 50\n"
            )

        project_root = self.fusion_dir.parent
        (project_root / "README.md").write_text("# Demo\n", encoding="utf-8")
        (project_root / "scripts/runtime/tests").mkdir(parents=True, exist_ok=True)
        (self.fusion_dir / ".progress_snapshot").write_text("0:1:0:0", encoding="utf-8")

        first = adapt_posttool(str(self.fusion_dir))
        self.assertTrue(first.changed)

        second = adapt_posttool(str(self.fusion_dir))
        self.assertFalse(second.changed)

        third = adapt_posttool(str(self.fusion_dir))
        self.assertFalse(third.changed)

        fourth = adapt_posttool(str(self.fusion_dir))
        self.assertTrue(fourth.changed)


class TestRuntimeToggle(BaseTestCase):
    """开关切换回退演练"""

    def test_disabled_runtime_returns_same_results(self):
        """禁用 runtime 时适配函数仍然可用"""
        self._write_config(enabled=False)
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
        })
        self._write_task_plan("### Task 1: A [PENDING]\n")

        # 适配函数不检查 runtime 开关（开关由 Shell 脚本检查）
        # 但它们应该正常工作
        result = adapt_stop_guard(str(self.fusion_dir))
        self.assertTrue(result.should_block)

        pretool = adapt_pretool(str(self.fusion_dir))
        self.assertTrue(pretool.active)


if __name__ == "__main__":
    unittest.main(verbosity=2)
