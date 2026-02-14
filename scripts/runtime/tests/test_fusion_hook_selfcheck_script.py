"""fusion-hook-selfcheck.sh CLI behavior tests."""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"
SELFCHECK_SCRIPT = SCRIPTS_DIR / "fusion-hook-selfcheck.sh"


class TestFusionHookSelfcheckScript(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.home_dir = self.temp_dir / "home"
        self.home_dir.mkdir(parents=True, exist_ok=True)
        (self.home_dir / ".claude").mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _run(self, *args: str) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        env["HOME"] = str(self.home_dir)
        return subprocess.run(
            ["bash", str(SELFCHECK_SCRIPT), *args],
            capture_output=True,
            text=True,
            timeout=60,
            check=False,
            env=env,
        )

    def test_help_outputs_usage(self):
        proc = self._run("--help")
        self.assertEqual(proc.returncode, 0)
        self.assertIn("Usage: fusion-hook-selfcheck.sh", proc.stdout)

    def test_unknown_option_fails_validation(self):
        proc = self._run("--bad-option")
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown option", proc.stderr)

    def test_json_quick_fix_mode_returns_ok(self):
        project = self.temp_dir / "project_ok"
        (project / ".fusion").mkdir(parents=True, exist_ok=True)
        (project / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "completed", "current_phase": "DELIVER"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project / ".fusion" / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n",
            encoding="utf-8",
        )

        proc = self._run("--json", "--quick", "--fix", str(project))

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("result"), "ok")
        self.assertEqual(payload.get("project_root"), str(project))

        checks = {item.get("name"): item for item in payload.get("checks", [])}
        self.assertTrue(checks.get("hook_doctor", {}).get("ok"))
        self.assertTrue(checks.get("stop_simulation", {}).get("ok"))
        self.assertTrue(checks.get("pytest_hook_suite", {}).get("skipped"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
