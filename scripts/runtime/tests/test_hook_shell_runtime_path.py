"""
Hook Shell 脚本运行时路径测试

验证从任意工作目录调用 Hook 脚本时，runtime compat_v2 仍可被正确加载。
该用例覆盖真实入口（bash 脚本），避免 PYTHONPATH 断层回归。
"""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"


class TestHookShellRuntimePath(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _write_sessions(self, data: dict):
        with open(self.fusion_dir / "sessions.json", "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False)

    def _write_task_plan(self, content: str):
        (self.fusion_dir / "task_plan.md").write_text(content, encoding="utf-8")

    def _enable_runtime(self):
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n  enabled: true\n",
            encoding="utf-8",
        )

    def _run_hook(self, script_name: str) -> subprocess.CompletedProcess:
        env = dict(os.environ)
        env.pop("PYTHONPATH", None)
        return subprocess.run(
            ["bash", str(SCRIPTS_DIR / script_name)],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

    def test_pretool_uses_runtime_adapter_from_external_cwd(self):
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "测试 runtime 路径",
        })
        self._write_task_plan(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [PENDING]\n"
        )
        self._enable_runtime()

        proc = self._run_hook("fusion-pretool.sh")
        self.assertEqual(proc.returncode, 0)
        self.assertIn("█", proc.stdout)

    def test_posttool_uses_runtime_adapter_from_external_cwd(self):
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
        })
        self._write_task_plan(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [PENDING]\n"
        )
        self._enable_runtime()
        (self.fusion_dir / ".progress_snapshot").write_text("0:2:0:0", encoding="utf-8")

        proc = self._run_hook("fusion-posttool.sh")
        self.assertEqual(proc.returncode, 0)
        self.assertIn("Task completed", proc.stdout)

    def test_posttool_injects_safe_backlog_on_no_progress_threshold(self):
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
        })
        self._write_task_plan("### Task 1: A [PENDING]\n")
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n"
            "  enabled: true\n"
            "  compat_mode: true\n"
            "safe_backlog:\n"
            "  enabled: true\n"
            "  trigger_no_progress_rounds: 2\n"
            "  max_tasks_per_run: 1\n"
            "  allowed_categories: documentation\n",
            encoding="utf-8",
        )
        (self.temp_dir / "README.md").write_text("# Demo\n", encoding="utf-8")
        (self.fusion_dir / ".progress_snapshot").write_text("0:1:0:0", encoding="utf-8")

        first = self._run_hook("fusion-posttool.sh")
        self.assertEqual(first.returncode, 0)
        self.assertEqual(first.stdout.strip(), "")

        second = self._run_hook("fusion-posttool.sh")
        self.assertEqual(second.returncode, 0)
        self.assertIn("Safe backlog injected", second.stdout)

        task_plan = (self.fusion_dir / "task_plan.md").read_text(encoding="utf-8")
        self.assertIn("[SAFE_BACKLOG]", task_plan)

        events_file = self.fusion_dir / "events.jsonl"
        self.assertTrue(events_file.exists())
        events = [
            json.loads(line)
            for line in events_file.read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
        self.assertTrue(any(evt.get("type") == "SAFE_BACKLOG_INJECTED" for evt in events))

    def test_stop_guard_uses_runtime_adapter_from_external_cwd(self):
        self._write_sessions({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "继续执行",
        })
        self._write_task_plan("### Task 1: A [PENDING]\n")
        self._enable_runtime()

        proc = self._run_hook("fusion-stop-guard.sh")
        self.assertEqual(proc.returncode, 0)
        output = json.loads(proc.stdout)
        self.assertEqual(output.get("decision"), "block")


if __name__ == "__main__":
    unittest.main(verbosity=2)
