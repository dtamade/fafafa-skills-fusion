"""
Hook 适配器端到端测试

测试 Shell→Python CLI 入口的输入输出格式，
验证 compat_v2 模块的 CLI 接口正确性。
"""

import unittest
import tempfile
import shutil
import json
import sys
import subprocess
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

SCRIPTS_DIR = str(Path(__file__).parent.parent.parent)


class BaseHookTestCase(unittest.TestCase):
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

    def _run_compat(self, command: str, fusion_dir: str = None) -> subprocess.CompletedProcess:
        """运行 compat_v2 CLI"""
        fd = fusion_dir or str(self.fusion_dir)
        return subprocess.run(
            [sys.executable, "-m", "runtime.compat_v2", command, fd],
            capture_output=True, text=True, cwd=SCRIPTS_DIR, timeout=10
        )


class TestStopGuardCLI(BaseHookTestCase):
    """stop-guard CLI 输出格式"""

    def test_allow_output_is_json(self):
        """allow 输出有效 JSON"""
        proc = self._run_compat("stop-guard")
        self.assertEqual(proc.returncode, 0)
        output = json.loads(proc.stdout)
        self.assertEqual(output["decision"], "allow")
        self.assertFalse(output["should_block"])

    def test_block_output_is_json(self):
        """block 输出有效 JSON"""
        self._write_sessions({
            "status": "in_progress", "current_phase": "EXECUTE",
            "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
        })
        self._write_task_plan("### Task 1: A [PENDING]\n")

        proc = self._run_compat("stop-guard")
        self.assertEqual(proc.returncode, 0)
        output = json.loads(proc.stdout)
        self.assertEqual(output["decision"], "block")
        self.assertTrue(output["should_block"])
        self.assertIn("reason", output)
        self.assertIn("systemMessage", output)

    def test_block_json_has_continuation_prompt(self):
        """block JSON 包含继续提示"""
        self._write_sessions({
            "status": "in_progress", "current_phase": "EXECUTE",
            "goal": "测试目标",
            "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
        })
        self._write_task_plan("### Task 1: A [PENDING]\n")

        proc = self._run_compat("stop-guard")
        output = json.loads(proc.stdout)
        self.assertIn("测试目标", output["reason"])
        self.assertIn("TDD", output["reason"])

    def test_phase_correction_in_output(self):
        """阶段纠正反映在输出中"""
        self._write_sessions({
            "status": "in_progress", "current_phase": "EXECUTE",
            "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 0}
        })
        self._write_task_plan(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [COMPLETED]\n"
        )

        proc = self._run_compat("stop-guard")
        output = json.loads(proc.stdout)
        self.assertTrue(output["phase_corrected"])
        self.assertIn("ALL_TASKS_DONE", output["events_dispatched"])


class TestPretoolCLI(BaseHookTestCase):
    """pretool CLI 输出格式"""

    def test_inactive_no_output(self):
        """非活跃工作流无输出"""
        proc = self._run_compat("pretool")
        self.assertEqual(proc.returncode, 0)
        self.assertEqual(proc.stdout.strip(), "")

    def test_active_outputs_fusion_lines(self):
        """活跃工作流输出 [fusion] 行"""
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "测试"
        })
        self._write_task_plan(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [PENDING]\n"
        )

        proc = self._run_compat("pretool")
        self.assertEqual(proc.returncode, 0)
        lines = proc.stdout.strip().split("\n")
        self.assertTrue(len(lines) >= 2)
        self.assertIn("[fusion]", lines[0])
        self.assertIn("EXECUTE", lines[0])

    def test_progress_bar_format(self):
        """进度条格式正确"""
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "测试"
        })
        self._write_task_plan(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [COMPLETED]\n"
            "### Task 3: C [PENDING]\n"
            "### Task 4: D [PENDING]\n"
        )

        proc = self._run_compat("pretool")
        lines = proc.stdout.strip().split("\n")
        # 应该有进度行
        progress_line = [l for l in lines if "Progress" in l]
        self.assertTrue(len(progress_line) > 0)
        self.assertIn("50%", progress_line[0])


class TestPosttoolCLI(BaseHookTestCase):
    """posttool CLI 输出格式"""

    def test_inactive_no_output(self):
        """非活跃工作流无输出"""
        proc = self._run_compat("posttool")
        self.assertEqual(proc.returncode, 0)
        self.assertEqual(proc.stdout.strip(), "")

    def test_change_detected_output(self):
        """进度变化时输出"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [PENDING]\n"
        )
        (self.fusion_dir / ".progress_snapshot").write_text("0:2:0:0")

        proc = self._run_compat("posttool")
        self.assertEqual(proc.returncode, 0)
        self.assertIn("[fusion]", proc.stdout)
        self.assertIn("completed", proc.stdout.lower())

    def test_no_change_no_output(self):
        """无变化时无输出"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan("### Task 1: A [PENDING]\n")
        (self.fusion_dir / ".progress_snapshot").write_text("0:1:0:0")

        proc = self._run_compat("posttool")
        self.assertEqual(proc.returncode, 0)
        self.assertEqual(proc.stdout.strip(), "")


class TestCLIErrorHandling(BaseHookTestCase):
    """CLI 错误处理"""

    def test_unknown_command_exits_1(self):
        """未知命令退出码 1"""
        proc = self._run_compat("unknown-command")
        self.assertEqual(proc.returncode, 1)
        self.assertIn("Unknown command", proc.stderr)

    def test_no_args_exits_1(self):
        """无参数退出码 1"""
        proc = subprocess.run(
            [sys.executable, "-m", "runtime.compat_v2"],
            capture_output=True, text=True, cwd=SCRIPTS_DIR, timeout=10
        )
        self.assertEqual(proc.returncode, 1)
        self.assertIn("Usage", proc.stderr)

    def test_nonexistent_fusion_dir(self):
        """不存在的 fusion_dir 不崩溃"""
        proc = self._run_compat("pretool", "/tmp/nonexistent-dir-12345")
        # 应该返回 0（非活跃工作流）或 1（错误）但不崩溃
        self.assertIn(proc.returncode, [0, 1])


if __name__ == "__main__":
    unittest.main(verbosity=2)
