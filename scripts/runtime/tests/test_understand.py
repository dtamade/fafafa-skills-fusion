"""UNDERSTAND 执行器测试"""

import json
import shutil
import tempfile
import unittest
from pathlib import Path

from runtime.understand import run_understand
from runtime.state_machine import State
from runtime.kernel import create_kernel


class TestUnderstandRunner(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.project_root = Path(self.temp_dir)
        self.fusion_dir = self.project_root / ".fusion"
        self.fusion_dir.mkdir(parents=True)

        (self.project_root / "package.json").write_text(
            json.dumps(
                {
                    "dependencies": {"express": "^4.0.0"},
                    "devDependencies": {"vitest": "^1.0.0"},
                }
            ),
            encoding="utf-8",
        )

        (self.fusion_dir / "sessions.json").write_text(
            json.dumps(
                {
                    "status": "in_progress",
                    "current_phase": "UNDERSTAND",
                    "goal": "初始化",
                    "_runtime": {"version": "2.1.0", "last_event_counter": 0},
                }
            ),
            encoding="utf-8",
        )
        (self.fusion_dir / "findings.md").write_text("# Findings\n", encoding="utf-8")

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_run_understand_writes_findings_and_advances_phase(self):
        result = run_understand(
            goal="实现用户登录 API 并补充测试，使用现有 Express",
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )

        self.assertGreaterEqual(result.scores.total, 7)
        self.assertTrue(result.pass_threshold)
        self.assertFalse(result.needs_confirmation)
        self.assertIn("Node.js", result.summary_md)

        findings = (self.fusion_dir / "findings.md").read_text(encoding="utf-8")
        self.assertIn("UNDERSTAND Phase", findings)
        self.assertIn("实现用户登录 API", findings)

        kernel = create_kernel(str(self.fusion_dir))
        self.assertEqual(kernel.current_state, State.INITIALIZE)

    def test_run_understand_low_score_marks_missing(self):
        result = run_understand(
            goal="优化一下",
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )
        self.assertFalse(result.pass_threshold)
        self.assertTrue(len(result.missing) > 0)
        self.assertFalse(result.needs_confirmation)

    def test_run_understand_strict_mode_stays_in_understand(self):
        (self.fusion_dir / "config.yaml").write_text(
            "understand:\n"
            "  pass_threshold: 9\n"
            "  require_confirmation: true\n",
            encoding="utf-8",
        )

        result = run_understand(
            goal="优化一下",
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )

        self.assertFalse(result.pass_threshold)
        self.assertTrue(result.require_confirmation)
        self.assertTrue(result.needs_confirmation)

        kernel = create_kernel(str(self.fusion_dir))
        self.assertEqual(kernel.current_state, State.UNDERSTAND)


if __name__ == "__main__":
    unittest.main(verbosity=2)
