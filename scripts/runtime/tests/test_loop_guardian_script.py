"""Direct tests for loop-guardian.sh behavior."""

import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
LOOP_GUARDIAN = REPO_ROOT / "scripts" / "loop-guardian.sh"


class TestLoopGuardianStatus(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True)
        (self.fusion_dir / "config.yaml").write_text(
            "loop_guardian:\n"
            "  max_iterations: 7\n"
            "  max_no_progress: 2\n"
            "  max_same_action: 4\n"
            "  max_same_error: 5\n"
            "  max_state_visits: 9\n"
            "  max_wall_time_ms: 12000\n"
            "  backoff_threshold: 1\n",
            encoding="utf-8",
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _run_guardian(self, commands: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [
                "bash",
                "-lc",
                f'set -euo pipefail; source "{LOOP_GUARDIAN}"; {commands}',
            ],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

    def test_status_uses_loaded_config_thresholds(self):
        proc = self._run_guardian("guardian_init; guardian_status")

        self.assertEqual(proc.returncode, 0)
        self.assertIn("Iterations: 0/7", proc.stdout)
        self.assertIn("No-Progress Rounds: 0/2", proc.stdout)
        self.assertIn("Same Action Count: 0/4", proc.stdout)
        self.assertIn("Same Error Count: 0/5", proc.stdout)

    def test_status_includes_state_and_walltime_thresholds(self):
        proc = self._run_guardian("guardian_init; guardian_status")

        self.assertEqual(proc.returncode, 0)
        self.assertIn("State Visits: 0/9", proc.stdout)
        self.assertIn("Wall Time: 0s/12s", proc.stdout)




class TestLoopGuardianInit(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _run_guardian(self, commands: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [
                "bash",
                "-lc",
                f'set -euo pipefail; source "{LOOP_GUARDIAN}"; {commands}',
            ],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

    def test_init_creates_fusion_dir_when_missing(self):
        proc = self._run_guardian("guardian_init; test -f .fusion/loop_context.json")

        self.assertEqual(proc.returncode, 0)
        self.assertTrue((self.temp_dir / ".fusion" / "loop_context.json").exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
