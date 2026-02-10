"""runtime.config 配置加载测试"""

import unittest
import tempfile
import shutil
from pathlib import Path

from runtime.config import load_fusion_config, load_raw_config


class TestConfigLoader(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_defaults_when_missing(self):
        cfg = load_fusion_config(str(self.fusion_dir))
        self.assertFalse(cfg["runtime_enabled"])
        self.assertEqual(cfg["backend_primary"], "codex")
        self.assertEqual(cfg["scheduler_max_parallel"], 2)
        self.assertFalse(cfg["safe_backlog_enabled"])
        self.assertEqual(cfg["safe_backlog_allowed_categories"], "documentation,quality")
        self.assertEqual(cfg["safe_backlog_max_tasks_per_run"], 2)
        self.assertFalse(cfg["supervisor_enabled"])
        self.assertEqual(cfg["supervisor_mode"], "advisory")
        self.assertEqual(cfg["supervisor_persona"], "Guardian")

    def test_parses_yaml_sections(self):
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n"
            "  enabled: true\n"
            "  compat_mode: false\n"
            "backends:\n"
            "  primary: claude\n"
            "execution:\n"
            "  parallel: 5\n"
            "scheduler:\n"
            "  enabled: true\n"
            "  max_parallel: 4\n"
            "budget:\n"
            "  warning_threshold: 0.7\n",
            encoding="utf-8",
        )

        cfg = load_fusion_config(str(self.fusion_dir))
        self.assertTrue(cfg["runtime_enabled"])
        self.assertFalse(cfg["runtime_compat_mode"])
        self.assertEqual(cfg["backend_primary"], "claude")
        self.assertEqual(cfg["execution_parallel"], 5)
        self.assertTrue(cfg["scheduler_enabled"])
        self.assertEqual(cfg["scheduler_max_parallel"], 4)
        self.assertAlmostEqual(cfg["budget_warning_threshold"], 0.7)

    def test_minimal_parser_fallback_shape(self):
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n"
            "  enabled: true\n",
            encoding="utf-8",
        )
        raw = load_raw_config(str(self.fusion_dir))
        self.assertIn("runtime", raw)

    def test_safe_backlog_config_parsing(self):
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n"
            "  enabled: true\n"
            "safe_backlog:\n"
            "  enabled: true\n"
            "  trigger_no_progress_rounds: 5\n"
            "  inject_on_task_exhausted: true\n"
            "  max_tasks_per_run: 3\n"
            "  allowed_categories: documentation,quality\n"
            "  diversity_rotation: true\n"
            "  novelty_window: 9\n"
            "  max_files_touched: 6\n"
            "  max_lines_changed: 320\n",
            encoding="utf-8",
        )

        cfg = load_fusion_config(str(self.fusion_dir))
        self.assertTrue(cfg["safe_backlog_enabled"])
        self.assertEqual(cfg["safe_backlog_trigger_no_progress_rounds"], 5)
        self.assertTrue(cfg["safe_backlog_inject_on_task_exhausted"])
        self.assertEqual(cfg["safe_backlog_max_tasks_per_run"], 3)
        self.assertEqual(cfg["safe_backlog_allowed_categories"], "documentation,quality")
        self.assertTrue(cfg["safe_backlog_diversity_rotation"])
        self.assertEqual(cfg["safe_backlog_novelty_window"], 9)
        self.assertEqual(cfg["safe_backlog_max_files_touched"], 6)
        self.assertEqual(cfg["safe_backlog_max_lines_changed"], 320)

    def test_supervisor_config_parsing(self):
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n"
            "  enabled: true\n"
            "supervisor:\n"
            "  enabled: true\n"
            "  mode: advisory\n"
            "  persona: Sentinel\n"
            "  trigger_no_progress_rounds: 4\n"
            "  cadence_rounds: 3\n"
            "  force_emit_rounds: 10\n"
            "  max_suggestions: 3\n",
            encoding="utf-8",
        )

        cfg = load_fusion_config(str(self.fusion_dir))
        self.assertTrue(cfg["supervisor_enabled"])
        self.assertEqual(cfg["supervisor_mode"], "advisory")
        self.assertEqual(cfg["supervisor_persona"], "Sentinel")
        self.assertEqual(cfg["supervisor_trigger_no_progress_rounds"], 4)
        self.assertEqual(cfg["supervisor_cadence_rounds"], 3)
        self.assertEqual(cfg["supervisor_force_emit_rounds"], 10)
        self.assertEqual(cfg["supervisor_max_suggestions"], 3)


if __name__ == "__main__":
    unittest.main(verbosity=2)
