"""runtime.supervisor 虚拟监督官测试"""

import json
import shutil
import tempfile
import unittest
from pathlib import Path

from runtime.supervisor import generate_supervisor_advice


class TestSupervisor(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.project_root = Path(self.temp_dir)
        self.fusion_dir = self.project_root / ".fusion"
        self.fusion_dir.mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _write_config(self, enabled: bool = True, cadence: int = 2):
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n"
            "  enabled: true\n"
            "supervisor:\n"
            f"  enabled: {'true' if enabled else 'false'}\n"
            "  mode: advisory\n"
            "  persona: Sentinel\n"
            "  trigger_no_progress_rounds: 2\n"
            f"  cadence_rounds: {cadence}\n"
            "  force_emit_rounds: 12\n"
            "  max_suggestions: 2\n",
            encoding="utf-8",
        )

    def test_disabled_by_default(self):
        result = generate_supervisor_advice(
            fusion_dir=str(self.fusion_dir),
            no_progress_rounds=4,
            counts={"completed": 0, "pending": 1, "in_progress": 0, "failed": 0},
            pending_like=1,
        )

        self.assertFalse(result["enabled"])
        self.assertFalse(result["emit"])
        self.assertEqual(result["suggestions"], [])

    def test_emits_advisory_when_enabled_and_threshold_met(self):
        self._write_config(enabled=True, cadence=1)

        result = generate_supervisor_advice(
            fusion_dir=str(self.fusion_dir),
            no_progress_rounds=2,
            counts={"completed": 0, "pending": 1, "in_progress": 0, "failed": 0},
            pending_like=1,
        )

        self.assertTrue(result["enabled"])
        self.assertTrue(result["emit"])
        self.assertIn("[Sentinel]", result["line"])
        self.assertGreaterEqual(len(result["suggestions"]), 1)
        self.assertLessEqual(len(result["suggestions"]), 2)

        state_file = self.fusion_dir / "supervisor_state.json"
        self.assertTrue(state_file.exists())
        state = json.loads(state_file.read_text(encoding="utf-8"))
        self.assertEqual(state.get("last_advice_round"), 2)

    def test_cadence_blocks_too_frequent_advice(self):
        self._write_config(enabled=True, cadence=3)

        first = generate_supervisor_advice(
            fusion_dir=str(self.fusion_dir),
            no_progress_rounds=2,
            counts={"completed": 0, "pending": 1, "in_progress": 0, "failed": 1},
            pending_like=1,
        )
        second = generate_supervisor_advice(
            fusion_dir=str(self.fusion_dir),
            no_progress_rounds=3,
            counts={"completed": 0, "pending": 1, "in_progress": 0, "failed": 1},
            pending_like=1,
        )

        self.assertTrue(first["emit"])
        self.assertFalse(second["emit"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
