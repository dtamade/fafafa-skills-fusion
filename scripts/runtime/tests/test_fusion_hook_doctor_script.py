"""fusion-hook-doctor.sh JSON mode tests."""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"
DOCTOR_SCRIPT = SCRIPTS_DIR / "fusion-hook-doctor.sh"


class TestFusionHookDoctorScript(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.home_dir = self.temp_dir / "home"
        self.home_dir.mkdir(parents=True)
        (self.home_dir / ".claude").mkdir(parents=True)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _run_raw(self, *args: str) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        env["HOME"] = str(self.home_dir)
        return subprocess.run(
            ["bash", str(DOCTOR_SCRIPT), *args],
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
            env=env,
        )

    def _run(self, project_root: Path) -> subprocess.CompletedProcess:
        return self._run_raw("--json", str(project_root))


    def test_json_mode_rejects_unknown_option(self):
        project = self.temp_dir / "project_unknown"
        project.mkdir(parents=True)

        proc = self._run_raw("--json", "--bad", str(project))

        self.assertNotEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("result"), "error")
        self.assertIn("Unknown option", payload.get("reason", ""))

    def test_json_mode_rejects_invalid_project_root(self):
        missing = self.temp_dir / "missing_project"

        proc = self._run_raw("--json", str(missing))

        self.assertNotEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("result"), "error")
        self.assertIn("not found", payload.get("reason", ""))


    def test_json_mode_fix_writes_project_settings(self):
        project = self.temp_dir / "project_fix"
        (project / ".fusion").mkdir(parents=True)

        (self.home_dir / ".claude" / "settings.json").write_text(
            json.dumps(
                {
                    "hooks": {
                        "PreToolUse": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-pretool.sh\""}]}],
                        "Stop": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-stop-guard.sh\""}]}],
                    }
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )

        (project / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "completed", "current_phase": "DELIVER"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project / ".fusion" / "task_plan.md").write_text("### Task 1: A [COMPLETED]\n", encoding="utf-8")

        proc = self._run_raw("--json", "--fix", str(project))

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("result"), "ok")
        self.assertTrue(payload.get("fixed"))

        settings_local = project / ".claude" / "settings.local.json"
        self.assertTrue(settings_local.exists())
        content = settings_local.read_text(encoding="utf-8")
        self.assertIn("fusion-pretool.sh", content)
        self.assertIn("fusion-posttool.sh", content)
        self.assertIn("fusion-stop-guard.sh", content)
        self.assertIn("${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh", content)
        self.assertIn("${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh", content)
        self.assertIn("${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh", content)

    def test_json_mode_reports_fixed_flag(self):
        project = self.temp_dir / "project_fixed_flag"
        (project / ".claude").mkdir(parents=True)
        (project / ".fusion").mkdir(parents=True)

        (project / ".claude" / "settings.local.json").write_text(
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

        (self.home_dir / ".claude" / "settings.json").write_text(
            json.dumps(
                {
                    "hooks": {
                        "PreToolUse": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-pretool.sh\""}]}],
                        "Stop": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-stop-guard.sh\""}]}],
                    }
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )

        (project / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "completed", "current_phase": "DELIVER"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project / ".fusion" / "task_plan.md").write_text("### Task 1: A [COMPLETED]\n", encoding="utf-8")

        proc = self._run(project)

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertIn("fixed", payload)
        self.assertFalse(payload.get("fixed"))


    def test_json_mode_fix_failure_reports_warn_and_fixed_false(self):
        project = self.temp_dir / "project_fix_fail"
        project.mkdir(parents=True)
        (project / ".fusion").mkdir(parents=True)
        (project / ".claude").write_text("occupied", encoding="utf-8")

        (project / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "completed", "current_phase": "DELIVER"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project / ".fusion" / "task_plan.md").write_text("### Task 1: A [COMPLETED]\n", encoding="utf-8")

        proc = self._run_raw("--json", "--fix", str(project))

        self.assertNotEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("result"), "warn")
        self.assertFalse(payload.get("fixed"))
        self.assertGreaterEqual(payload.get("warn_count", 0), 1)

    def test_json_mode_returns_machine_readable_summary(self):
        project = self.temp_dir / "project_ok"
        (project / ".claude").mkdir(parents=True)
        (project / ".fusion").mkdir(parents=True)

        (project / ".claude" / "settings.local.json").write_text(
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

        (self.home_dir / ".claude" / "settings.json").write_text(
            json.dumps(
                {
                    "hooks": {
                        "PreToolUse": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-pretool.sh\""}]}],
                        "Stop": [{"hooks": [{"command": "bash \"${CLAUDE_PROJECT_DIR}/scripts/fusion-stop-guard.sh\""}]}],
                    }
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )

        (project / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "completed", "current_phase": "DELIVER"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project / ".fusion" / "task_plan.md").write_text("### Task 1: A [COMPLETED]\n", encoding="utf-8")

        proc = self._run(project)

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "ok")
        self.assertEqual(payload["project_root"], str(project))
        self.assertGreaterEqual(payload["ok_count"], 1)
        self.assertEqual(payload["warn_count"], 0)

    def test_json_mode_warns_when_project_hooks_use_relative_paths(self):
        project = self.temp_dir / "project_relative"
        (project / ".claude").mkdir(parents=True)
        (project / ".fusion").mkdir(parents=True)

        (project / ".claude" / "settings.local.json").write_text(
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

        (project / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "completed", "current_phase": "DELIVER"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project / ".fusion" / "task_plan.md").write_text("### Task 1: A [COMPLETED]\n", encoding="utf-8")

        proc = self._run(project)

        self.assertNotEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload.get("result"), "warn")
        self.assertGreaterEqual(payload.get("warn_count", 0), 1)

    def test_json_mode_returns_nonzero_when_warnings_exist(self):
        project = self.temp_dir / "project_warn"
        project.mkdir(parents=True)

        proc = self._run(project)

        self.assertNotEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "warn")
        self.assertEqual(payload["project_root"], str(project))
        self.assertGreaterEqual(payload["warn_count"], 1)


if __name__ == "__main__":
    unittest.main(verbosity=2)
