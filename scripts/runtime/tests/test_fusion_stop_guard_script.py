"""fusion-stop-guard.sh behavior tests."""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"
STOP_GUARD = SCRIPTS_DIR / "fusion-stop-guard.sh"


class TestFusionStopGuardScript(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _write_state(self, status: str, phase: str = "EXECUTE", goal: str = "continue", task_plan: str = "### Task 1: A [PENDING]\n"):
        fusion = self.temp_dir / ".fusion"
        fusion.mkdir(parents=True, exist_ok=True)
        (fusion / "sessions.json").write_text(
            json.dumps({"status": status, "current_phase": phase, "goal": goal}, ensure_ascii=False),
            encoding="utf-8",
        )
        if task_plan is not None:
            (fusion / "task_plan.md").write_text(task_plan, encoding="utf-8")

    def _run(self, *, mode: str | None = None, hook_input: str = "") -> subprocess.CompletedProcess:
        env = os.environ.copy()
        if mode is not None:
            env["FUSION_STOP_HOOK_MODE"] = mode
        return subprocess.run(
            ["bash", str(STOP_GUARD)],
            cwd=str(self.temp_dir),
            env=env,
            input=hook_input,
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

    def test_structured_lock_contention_returns_json_block(self):
        self._write_state("in_progress")
        (self.temp_dir / ".fusion" / ".state.lock").mkdir(parents=True)

        proc = self._run(mode="structured", hook_input="{}")

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("decision"), "block")
        self.assertIn("state", (payload.get("systemMessage") or "").lower())


    def test_structured_blocks_with_empty_stdin(self):
        self._write_state("in_progress")

        proc = self._run(mode="structured")

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("decision"), "block")

    def test_auto_mode_blocks_with_empty_stdin_using_json_contract(self):
        self._write_state("in_progress")

        proc = self._run()

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("decision"), "block")

    def test_structured_blocks_when_pending_tasks(self):
        self._write_state("in_progress")

        proc = self._run(mode="structured", hook_input="{}")

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("decision"), "block")
        self.assertIn("Continue executing the Fusion workflow", payload.get("reason", ""))

    def test_legacy_blocks_when_pending_tasks(self):
        self._write_state("in_progress")

        proc = self._run(mode="legacy")

        self.assertEqual(proc.returncode, 2)
        self.assertIn("stop blocked", (proc.stdout + proc.stderr).lower())

    def test_allows_stop_when_not_in_progress(self):
        self._write_state("completed", task_plan="### Task 1: A [COMPLETED]\n")

        proc = self._run(mode="structured", hook_input="{}")

        self.assertEqual(proc.returncode, 0)

    def test_allows_stop_when_fusion_dir_missing(self):
        proc = self._run(mode="structured", hook_input="{}")
        self.assertEqual(proc.returncode, 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
