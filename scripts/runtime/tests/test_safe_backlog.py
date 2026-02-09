"""safe_backlog 低风险托底任务生成测试"""

import json
import shutil
import tempfile
import unittest
from pathlib import Path

from runtime.safe_backlog import generate_safe_backlog, reset_safe_backlog_backoff


class TestSafeBacklog(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.project_root = Path(self.temp_dir)
        self.fusion_dir = self.project_root / ".fusion"
        self.fusion_dir.mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _write_config(
        self,
        enabled: bool = True,
        categories: str = "documentation,quality",
        backoff_enabled: bool = False,
    ):
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n"
            "  enabled: true\n"
            "safe_backlog:\n"
            f"  enabled: {'true' if enabled else 'false'}\n"
            "  trigger_no_progress_rounds: 3\n"
            "  max_tasks_per_run: 2\n"
            f"  allowed_categories: \"{categories}\"\n"
            "  inject_on_task_exhausted: true\n"
            "  diversity_rotation: true\n"
            "  novelty_window: 12\n"
            f"  backoff_enabled: {'true' if backoff_enabled else 'false'}\n"
            "  backoff_base_rounds: 1\n"
            "  backoff_max_rounds: 32\n"
            "  backoff_jitter: 0\n"
            "  backoff_force_probe_rounds: 20\n"
            "  max_files_touched: 4\n"
            "  max_lines_changed: 200\n",
            encoding="utf-8",
        )

    def _write_task_plan(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "# Fusion Task Plan\n\n"
            "## Tasks\n\n"
            "### Task 1: Existing Work [PENDING]\n"
            "- Type: implementation\n"
            "- Execution: TDD\n"
            "- Dependencies: []\n",
            encoding="utf-8",
        )

    def test_generate_adds_safe_tasks_and_persists_state(self):
        self._write_config(enabled=True, categories="documentation,quality")
        self._write_task_plan()
        (self.project_root / "README.md").write_text("# Demo\n", encoding="utf-8")
        (self.project_root / "scripts/runtime/tests").mkdir(parents=True, exist_ok=True)

        result = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )

        self.assertTrue(result["enabled"])
        self.assertGreater(result["added"], 0)
        self.assertTrue((self.fusion_dir / "safe_backlog.json").exists())

        task_plan = (self.fusion_dir / "task_plan.md").read_text(encoding="utf-8")
        self.assertIn("[SAFE_BACKLOG]", task_plan)

    def test_generate_is_idempotent_with_fingerprint_dedup(self):
        self._write_config(enabled=True, categories="documentation")
        self._write_task_plan()
        (self.project_root / "README.md").write_text("# Demo\n", encoding="utf-8")

        first = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )
        second = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )

        self.assertGreaterEqual(first["added"], 1)
        self.assertEqual(second["added"], 0)

    def test_disabled_returns_without_modification(self):
        self._write_config(enabled=False)
        self._write_task_plan()
        original = (self.fusion_dir / "task_plan.md").read_text(encoding="utf-8")

        result = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )

        self.assertFalse(result["enabled"])
        self.assertEqual(result["added"], 0)
        self.assertEqual((self.fusion_dir / "task_plan.md").read_text(encoding="utf-8"), original)

    def test_category_filter_only_keeps_allowed_categories(self):
        self._write_config(enabled=True, categories="documentation")
        self._write_task_plan()
        (self.project_root / "README.md").write_text("# Demo\n", encoding="utf-8")
        (self.project_root / "scripts/runtime/tests").mkdir(parents=True, exist_ok=True)

        result = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )

        self.assertGreaterEqual(result["added"], 1)
        categories = {item.get("category") for item in result["tasks"]}
        self.assertEqual(categories, {"documentation"})

        state = json.loads((self.fusion_dir / "safe_backlog.json").read_text(encoding="utf-8"))
        self.assertIn("fingerprints", state)

    def test_supports_optimization_category_generation(self):
        self._write_config(enabled=True, categories="optimization")
        self._write_task_plan()
        (self.project_root / "scripts/runtime").mkdir(parents=True, exist_ok=True)

        result = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )

        self.assertGreaterEqual(result["added"], 1)
        categories = {item.get("category") for item in result["tasks"]}
        self.assertEqual(categories, {"optimization"})

    def test_rotation_avoids_same_category_every_injection(self):
        self._write_config(enabled=True, categories="documentation,quality,optimization")
        self._write_task_plan()
        (self.project_root / "README.md").write_text("# Demo\n", encoding="utf-8")
        (self.project_root / "scripts/runtime/tests").mkdir(parents=True, exist_ok=True)
        (self.project_root / "scripts/runtime").mkdir(parents=True, exist_ok=True)

        first = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )
        second = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )

        self.assertGreaterEqual(first["added"], 1)
        self.assertGreaterEqual(second["added"], 1)
        self.assertNotEqual(first["tasks"][0]["category"], second["tasks"][0]["category"])

        state = json.loads((self.fusion_dir / "safe_backlog.json").read_text(encoding="utf-8"))
        self.assertIn("last_category", state)
        self.assertIn("stats", state)

    def test_tasks_include_priority_score_for_orchestration(self):
        self._write_config(enabled=True, categories="documentation,quality,optimization")
        self._write_task_plan()
        (self.project_root / "README.md").write_text("# Demo\n", encoding="utf-8")
        (self.project_root / "scripts/runtime/tests").mkdir(parents=True, exist_ok=True)
        (self.project_root / "scripts/runtime").mkdir(parents=True, exist_ok=True)

        result = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )

        self.assertGreaterEqual(result["added"], 1)
        for task in result["tasks"]:
            self.assertIn("priority_score", task)
            self.assertIsInstance(task["priority_score"], float)

    def test_backoff_state_persists_and_reset(self):
        self._write_config(enabled=True, categories="documentation", backoff_enabled=True)
        self._write_task_plan()
        (self.project_root / "README.md").write_text("# Demo\n", encoding="utf-8")

        first = generate_safe_backlog(
            fusion_dir=str(self.fusion_dir),
            project_root=str(self.project_root),
        )
        self.assertGreaterEqual(first["added"], 1)

        state_before = json.loads((self.fusion_dir / "safe_backlog.json").read_text(encoding="utf-8"))
        self.assertGreater(int(state_before.get("backoff", {}).get("cooldown_until_round", 0)), 0)

        reset_safe_backlog_backoff(str(self.fusion_dir))
        state_after = json.loads((self.fusion_dir / "safe_backlog.json").read_text(encoding="utf-8"))
        backoff = state_after.get("backoff", {})
        self.assertEqual(int(backoff.get("consecutive_failures", -1)), 0)
        self.assertEqual(int(backoff.get("consecutive_injections", -1)), 0)
        self.assertEqual(int(backoff.get("cooldown_until_round", -1)), 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
