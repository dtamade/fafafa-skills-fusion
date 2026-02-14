"""
Hook stdin 处理测试

验证 pretool/posttool/stop-guard 正确读取和处理 stdin 输入。
这是对 hook stdin 修复的回归测试。
"""

import unittest
import tempfile
import shutil
import json
import subprocess
from pathlib import Path


class TestHookStdinHandling(unittest.TestCase):
    """测试 Shell hook 的 stdin 处理"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.scripts_dir = Path(__file__).parent.parent.parent

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _write_sessions(self, data: dict):
        with open(self.fusion_dir / "sessions.json", "w", encoding="utf-8") as f:
            json.dump(data, f)

    def _write_task_plan(self, content: str):
        with open(self.fusion_dir / "task_plan.md", "w", encoding="utf-8") as f:
            f.write(content)

    def _write_config(self):
        """写入最小配置"""
        config = """
runtime:
  enabled: true
  engine: "python"
"""
        with open(self.fusion_dir / "config.yaml", "w", encoding="utf-8") as f:
            f.write(config)

    def test_pretool_consumes_stdin(self):
        """pretool 正确消费 stdin 输入"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE", "goal": "test"})
        self._write_task_plan("### Task 1: A [PENDING]\n")
        self._write_config()

        hook_input = json.dumps({"tool_name": "Write", "tool_input": {"file_path": "test.txt"}})

        proc = subprocess.run(
            ["bash", str(self.scripts_dir / "fusion-pretool.sh")],
            input=hook_input,
            capture_output=True,
            text=True,
            cwd=self.temp_dir,
            timeout=5
        )

        # 应该成功执行（不会因为 stdin 阻塞而超时）
        self.assertEqual(proc.returncode, 0)
        # 应该有输出（活跃工作流）
        self.assertIn("[fusion]", proc.stdout)

    def test_posttool_consumes_stdin(self):
        """posttool 正确消费 stdin 输入"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan("### Task 1: A [COMPLETED]\n")
        self._write_config()
        (self.fusion_dir / ".progress_snapshot").write_text("0:1:0:0")

        hook_input = json.dumps({"tool_name": "Write", "tool_input": {"file_path": "test.txt"}})

        proc = subprocess.run(
            ["bash", str(self.scripts_dir / "fusion-posttool.sh")],
            input=hook_input,
            capture_output=True,
            text=True,
            cwd=self.temp_dir,
            timeout=5
        )

        # 应该成功执行（不会因为 stdin 阻塞而超时）
        self.assertEqual(proc.returncode, 0)

    def test_stop_guard_consumes_stdin(self):
        """stop-guard 正确消费 stdin 输入"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE", "goal": "test"})
        self._write_task_plan("### Task 1: A [PENDING]\n")
        self._write_config()

        hook_input = json.dumps({"session_id": "test123"})

        proc = subprocess.run(
            ["bash", str(self.scripts_dir / "fusion-stop-guard.sh")],
            input=hook_input,
            capture_output=True,
            text=True,
            cwd=self.temp_dir,
            timeout=5
        )

        # 应该成功执行（不会因为 stdin 阻塞而超时）
        self.assertEqual(proc.returncode, 0)
        # 应该输出 JSON（structured mode）
        output = json.loads(proc.stdout)
        self.assertEqual(output["decision"], "block")

    def test_pretool_handles_empty_stdin(self):
        """pretool 处理空 stdin 不崩溃"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE", "goal": "test"})
        self._write_task_plan("### Task 1: A [PENDING]\n")
        self._write_config()

        proc = subprocess.run(
            ["bash", str(self.scripts_dir / "fusion-pretool.sh")],
            input="",
            capture_output=True,
            text=True,
            cwd=self.temp_dir,
            timeout=5
        )

        # 应该成功执行
        self.assertEqual(proc.returncode, 0)

    def test_posttool_handles_malformed_json(self):
        """posttool 处理格式错误的 JSON 不崩溃"""
        self._write_sessions({"status": "in_progress", "current_phase": "EXECUTE"})
        self._write_task_plan("### Task 1: A [PENDING]\n")
        self._write_config()

        proc = subprocess.run(
            ["bash", str(self.scripts_dir / "fusion-posttool.sh")],
            input="{invalid json}",
            capture_output=True,
            text=True,
            cwd=self.temp_dir,
            timeout=5
        )

        # 应该成功执行（hook 不应该因为输入格式错误而失败）
        self.assertEqual(proc.returncode, 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
