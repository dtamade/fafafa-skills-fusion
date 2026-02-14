"""Validation tests for fusion control shell scripts."""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"


class TestFusionResumeValidation(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True)
        (self.fusion_dir / "sessions.json").write_text(
            json.dumps(
                {
                    "status": "paused",
                    "goal": "demo",
                    "current_phase": "EXECUTE",
                    "last_checkpoint": "2026-02-11 00:00:00",
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_resume_rejects_unknown_option(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-resume.sh"), "--bad"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown option", proc.stdout + proc.stderr)

    def test_resume_prints_hook_debug_hint_off_by_default(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-resume.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Hook debug: OFF", output)
        self.assertIn("touch .fusion/.hook_debug", output)

    def test_resume_prints_hook_debug_hint_on_with_env(self):
        env = os.environ.copy()
        env["FUSION_HOOK_DEBUG"] = "1"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-resume.sh")],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Hook debug: ON", output)
        self.assertIn(".fusion/hook-debug.log", output)


class TestFusionGitValidation(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        subprocess.run(["git", "init"], cwd=str(self.temp_dir), capture_output=True, text=True, check=False)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_git_rejects_unknown_action(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-git.sh"), "typo-action"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown action", proc.stdout + proc.stderr)


    def test_git_unknown_action_reports_to_stderr_with_usage(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-git.sh"), "--bad"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown action", proc.stderr)
        self.assertIn("Usage: fusion-git.sh", proc.stderr)

    def test_git_help_exits_zero_and_shows_usage(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-git.sh"), "--help"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("Usage: fusion-git.sh", proc.stdout + proc.stderr)

class TestFusionLogsValidation(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True)
        (self.fusion_dir / "progress.md").write_text(
            "| 2026-02-11 | EXECUTE | demo | OK | detail |\n",
            encoding="utf-8",
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_logs_rejects_non_numeric_lines(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-logs.sh"), "abc"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("LINES must be a positive integer", proc.stdout + proc.stderr)


    def test_logs_rejects_unknown_option(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-logs.sh"), "--bad"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Unknown option", output)
        self.assertIn("Usage: fusion-logs.sh", output)


    def test_logs_rejects_too_many_arguments(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-logs.sh"), "10", "extra"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Too many arguments", output)
        self.assertIn("Usage: fusion-logs.sh", output)

    def test_logs_help_exits_zero_and_shows_usage(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-logs.sh"), "--help"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("Usage: fusion-logs.sh", proc.stdout + proc.stderr)

    def test_logs_prints_hook_debug_off_by_default(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-logs.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("HOOK DEBUG", proc.stdout)
        self.assertIn("enabled: false", proc.stdout)

    def test_logs_prints_hook_debug_on_with_flag_and_log_tail(self):
        (self.fusion_dir / ".hook_debug").write_text("", encoding="utf-8")
        (self.fusion_dir / "hook-debug.log").write_text(
            "[fusion][hook-debug][pretool][2026-02-12T00:00:00Z] invoked\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-logs.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("enabled: true", proc.stdout)
        self.assertIn("hook-debug.log", proc.stdout)
        self.assertIn("[fusion][hook-debug][pretool]", proc.stdout)


class TestFusionPauseValidation(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True)
        (self.fusion_dir / "sessions.json").write_text(
            json.dumps({"status": "in_progress", "goal": "demo"}, ensure_ascii=False),
            encoding="utf-8",
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_pause_rejects_unknown_option(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-pause.sh"), "--bad"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown option", proc.stdout + proc.stderr)

    def test_pause_prints_hook_debug_hint_off_by_default(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-pause.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Hook debug: OFF", output)
        self.assertIn("touch .fusion/.hook_debug", output)

    def test_pause_prints_hook_debug_hint_on_with_env(self):
        env = os.environ.copy()
        env["FUSION_HOOK_DEBUG"] = "1"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-pause.sh")],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Hook debug: ON", output)
        self.assertIn(".fusion/hook-debug.log", output)


class TestFusionCancelValidation(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True)
        (self.fusion_dir / "sessions.json").write_text(
            json.dumps({"status": "in_progress", "goal": "demo"}, ensure_ascii=False),
            encoding="utf-8",
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_cancel_rejects_unknown_option(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-cancel.sh"), "--bad"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown option", proc.stdout + proc.stderr)


class TestFusionContinueValidation(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True)
        (self.fusion_dir / "sessions.json").write_text(
            json.dumps({"status": "in_progress", "current_phase": "EXECUTE"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (self.fusion_dir / "progress.md").write_text("| t | EXECUTE | e | OK | d |\n", encoding="utf-8")

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_continue_rejects_unknown_option(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-continue.sh"), "--bad"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown option", proc.stdout + proc.stderr)

    def test_continue_prints_hook_debug_hint_off_by_default(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-continue.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Hook debug: OFF", output)
        self.assertIn("touch .fusion/.hook_debug", output)

    def test_continue_prints_hook_debug_hint_on_with_flag(self):
        (self.fusion_dir / ".hook_debug").write_text("", encoding="utf-8")

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-continue.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Hook debug: ON", output)
        self.assertIn(".fusion/hook-debug.log", output)


if __name__ == "__main__":
    unittest.main(verbosity=2)
