"""runtime.config 配置加载测试"""

import sys
import unittest
import tempfile
import shutil
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

import runtime.config as config_module

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
        self.assertEqual(cfg["backend_phase_routing"]["EXECUTE"], "claude")
        self.assertEqual(cfg["backend_phase_routing"]["REVIEW"], "codex")
        self.assertEqual(cfg["backend_task_type_routing"]["implementation"], "claude")
        self.assertEqual(cfg["scheduler_max_parallel"], 2)
        self.assertTrue(cfg["safe_backlog_enabled"])
        self.assertEqual(cfg["safe_backlog_allowed_categories"], "quality,documentation,optimization")
        self.assertEqual(cfg["safe_backlog_max_tasks_per_run"], 2)
        self.assertFalse(cfg["supervisor_enabled"])
        self.assertEqual(cfg["supervisor_mode"], "advisory")
        self.assertEqual(cfg["supervisor_persona"], "Guardian")
        self.assertEqual(cfg["understand_pass_threshold"], 7)
        self.assertFalse(cfg["understand_require_confirmation"])
        self.assertEqual(cfg["understand_max_questions"], 2)
        self.assertEqual(cfg["runtime_version"], "2.6.3")

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
            "  warning_threshold: 0.7\n"
            "understand:\n"
            "  pass_threshold: 8\n"
            "  require_confirmation: true\n"
            "  max_questions: 3\n",
            encoding="utf-8",
        )

        cfg = load_fusion_config(str(self.fusion_dir))
        self.assertTrue(cfg["runtime_enabled"])
        self.assertFalse(cfg["runtime_compat_mode"])
        self.assertEqual(cfg["backend_primary"], "claude")
        self.assertEqual(cfg["backend_phase_routing"]["EXECUTE"], "claude")
        self.assertEqual(cfg["backend_task_type_routing"]["implementation"], "claude")
        self.assertEqual(cfg["execution_parallel"], 5)
        self.assertTrue(cfg["scheduler_enabled"])
        self.assertEqual(cfg["scheduler_max_parallel"], 4)
        self.assertAlmostEqual(cfg["budget_warning_threshold"], 0.7)
        self.assertEqual(cfg["understand_pass_threshold"], 8)
        self.assertTrue(cfg["understand_require_confirmation"])
        self.assertEqual(cfg["understand_max_questions"], 3)

    def test_understand_threshold_clamp(self):
        (self.fusion_dir / "config.yaml").write_text(
            "understand:\n"
            "  pass_threshold: 99\n"
            "  max_questions: 0\n",
            encoding="utf-8",
        )

        cfg = load_fusion_config(str(self.fusion_dir))
        self.assertEqual(cfg["understand_pass_threshold"], 10)
        self.assertEqual(cfg["understand_max_questions"], 1)

    def test_minimal_parser_fallback_shape(self):
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n"
            "  enabled: true\n",
            encoding="utf-8",
        )
        raw = load_raw_config(str(self.fusion_dir))
        self.assertIn("runtime", raw)

    def test_minimal_parser_supports_nested_backend_routing(self):
        (self.fusion_dir / "config.yaml").write_text(
            "backend_routing:\n"
            "  phase_routing:\n"
            "    EXECUTE: codex\n"
            "  task_type_routing:\n"
            "    implementation: codex\n",
            encoding="utf-8",
        )

        original_loader = config_module.load_raw_config
        try:
            config_module.load_raw_config = lambda _fusion_dir=".fusion": config_module._minimal_parse_yaml(  # type: ignore[attr-defined]
                self.fusion_dir / "config.yaml"
            )
            cfg = load_fusion_config(str(self.fusion_dir))
        finally:
            config_module.load_raw_config = original_loader

        self.assertEqual(cfg["backend_phase_routing"]["EXECUTE"], "codex")
        self.assertEqual(cfg["backend_task_type_routing"]["implementation"], "codex")

    def test_minimal_parser_handles_booleans_numbers_and_quotes(self):
        (self.fusion_dir / "config.yaml").write_text(
            "runtime:\n"
            "  enabled: true\n"
            "  compat_mode: false\n"
            "  version: \"2.6.3\"\n"
            "budget:\n"
            "  warning_threshold: 0.7\n"
            "  global_token_limit: 123\n",
            encoding="utf-8",
        )

        original_yaml = sys.modules.get("yaml")
        try:
            sys.modules["yaml"] = None
            cfg = load_fusion_config(str(self.fusion_dir))
        finally:
            if original_yaml is not None:
                sys.modules["yaml"] = original_yaml
            else:
                sys.modules.pop("yaml", None)

        self.assertTrue(cfg["runtime_enabled"])
        self.assertFalse(cfg["runtime_compat_mode"])
        self.assertEqual(cfg["runtime_version"], "2.6.3")
        self.assertAlmostEqual(cfg["budget_warning_threshold"], 0.7)
        self.assertEqual(cfg["budget_global_token_limit"], 123)

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
