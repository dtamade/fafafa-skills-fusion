"""fusion-start.sh / fusion-init.sh behavior tests."""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"


class TestFusionStartScript(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _run_start(self, *args: str, env_overrides=None) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        if env_overrides:
            env.update(env_overrides)
        return subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-start.sh"), *args],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
            env=env,
        )

    def _run_init(self, *args: str, env_overrides=None) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        if env_overrides:
            env.update(env_overrides)
        return subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-init.sh"), *args],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
            env=env,
        )

    def _build_minimal_path(self, commands) -> str:
        bin_dir = self.temp_dir / "mini-bin"
        bin_dir.mkdir(parents=True, exist_ok=True)
        for command_name in commands:
            target = shutil.which(command_name)
            self.assertIsNotNone(target, f"missing command on host: {command_name}")
            os.symlink(target, bin_dir / command_name)
        return str(bin_dir)

    def test_rejects_unknown_option(self):
        proc = self._run_start("--bad")

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown option", proc.stderr + proc.stdout)

    def test_rejects_multiple_goals(self):
        proc = self._run_start("goal-a", "goal-b")

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("only one goal", (proc.stderr + proc.stdout).lower())


    def test_fusion_init_supports_rust_engine(self):
        proc = self._run_init("--engine", "rust")

        self.assertEqual(proc.returncode, 0)
        config = (self.temp_dir / ".fusion" / "config.yaml").read_text(encoding="utf-8")
        self.assertIn('engine: "rust"', config)


    def test_fusion_init_json_success(self):
        proc = self._run_init("--json")

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "ok")
        self.assertEqual(payload["engine"], "python")
        self.assertTrue(payload["fusion_dir"].endswith(".fusion"))

    def test_fusion_init_json_error_on_invalid_engine(self):
        proc = self._run_init("--json", "--engine", "invalid")

        self.assertNotEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "error")
        self.assertIn("Invalid engine", payload.get("reason", ""))

    def test_fusion_init_json_fallback_without_jq_or_python3(self):
        minimal_path = self._build_minimal_path(
            ["bash", "cp", "mkdir", "chmod", "cat", "grep", "head", "cut", "ls", "dirname"]
        )

        proc = self._run_init("--json", env_overrides={"PATH": minimal_path})

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "ok")
        self.assertEqual(payload["engine"], "python")

    def test_help_exits_zero_and_shows_usage(self):
        proc = self._run_start("--help")

        self.assertEqual(proc.returncode, 0)
        self.assertIn("Usage: fusion-start.sh <goal> [--force]", proc.stdout + proc.stderr)

    def test_unknown_option_reports_usage_without_shell_redirection_error(self):
        proc = self._run_start("--bad")

        self.assertNotEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Usage: fusion-start.sh <goal> [--force]", output)
        self.assertNotIn("No such file or directory", output)

    def test_force_mode_prints_hook_selfcheck_hint(self):
        proc = self._run_start("demo goal", "--force")

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("fusion-hook-selfcheck.sh", output)
        self.assertIn("--fix .", output)

    def test_force_mode_prints_hook_debug_hint_off_by_default(self):
        proc = self._run_start("demo goal", "--force")

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Hook debug: OFF", output)
        self.assertIn("touch .fusion/.hook_debug", output)

    def test_force_mode_prints_hook_debug_hint_on_when_env_enabled(self):
        proc = self._run_start("demo goal", "--force", env_overrides={"FUSION_HOOK_DEBUG": "1"})

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Hook debug: ON", output)
        self.assertIn(".fusion/hook-debug.log", output)

    def test_force_mode_auto_fixes_missing_hooks_and_prints_restart_hint(self):
        proc = self._run_start("demo goal", "--force")

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Detected hook wiring gaps. Auto-fixing...", output)
        self.assertIn("Hook auto-fix complete.", output)
        self.assertIn("Then restart this Claude Code session and run /fusion again.", output)

        settings_local = self.temp_dir / ".claude" / "settings.local.json"
        self.assertTrue(settings_local.exists())
        content = settings_local.read_text(encoding="utf-8")
        self.assertIn("fusion-pretool.sh", content)
        self.assertIn("fusion-posttool.sh", content)
        self.assertIn("fusion-stop-guard.sh", content)

    def test_force_mode_with_existing_project_hooks_skips_restart_hint(self):
        hooks_dir = self.temp_dir / ".claude"
        hooks_dir.mkdir(parents=True, exist_ok=True)
        (hooks_dir / "settings.local.json").write_text(
            json.dumps(
                {
                    "hooks": {
                        "PreToolUse": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-pretool.sh\""}]}],
                        "PostToolUse": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-posttool.sh\""}]}],
                        "Stop": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-stop-guard.sh\""}]}],
                    }
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )

        proc = self._run_start("demo goal", "--force")

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertNotIn("Then restart this Claude Code session and run /fusion again.", output)

    def test_force_mode_upgrades_relative_hook_paths_and_prints_restart_hint(self):
        hooks_dir = self.temp_dir / ".claude"
        hooks_dir.mkdir(parents=True, exist_ok=True)
        (hooks_dir / "settings.local.json").write_text(
            json.dumps(
                {
                    "hooks": {
                        "PreToolUse": [{"hooks": [{"command": "bash scripts/fusion-pretool.sh"}]}],
                        "PostToolUse": [{"hooks": [{"command": "bash scripts/fusion-posttool.sh"}]}],
                        "Stop": [{"hooks": [{"command": "bash scripts/fusion-stop-guard.sh"}]}],
                    }
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )

        proc = self._run_start("demo goal", "--force")

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Detected hook wiring gaps. Auto-fixing...", output)
        self.assertIn("Then restart this Claude Code session and run /fusion again.", output)

        content = (hooks_dir / "settings.local.json").read_text(encoding="utf-8")
        self.assertIn("${CLAUDE_PROJECT_DIR}/scripts/fusion-pretool.sh", content)
        self.assertIn("${CLAUDE_PROJECT_DIR}/scripts/fusion-posttool.sh", content)
        self.assertIn("${CLAUDE_PROJECT_DIR}/scripts/fusion-stop-guard.sh", content)

    def test_missing_goal_reports_usage_without_shell_redirection_error(self):
        proc = self._run_start()

        self.assertNotEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Usage: fusion-start.sh <goal> [--force]", output)
        self.assertNotIn("No such file or directory", output)


if __name__ == "__main__":
    unittest.main(verbosity=2)
