"""fusion-achievements.sh 输出测试"""

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"


class TestFusionAchievementsScript(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def _run(self, cwd: Path, *args: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-achievements.sh"), *args],
            cwd=str(cwd),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

    def test_local_achievements_summary(self):
        fusion_dir = self.temp_dir / ".fusion"
        fusion_dir.mkdir(parents=True)

        (fusion_dir / "sessions.json").write_text(
            json.dumps({"status": "completed", "current_phase": "DELIVER"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (fusion_dir / "task_plan.md").write_text(
            "### Task 1: 实现登录 [COMPLETED]\n"
            "### Task 2: 增加测试 [COMPLETED]\n"
            "### Task 3: 更新文档 [PENDING]\n",
            encoding="utf-8",
        )
        (fusion_dir / "events.jsonl").write_text(
            json.dumps({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 1}}, ensure_ascii=False)
            + "\n"
            + json.dumps({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 2}}, ensure_ascii=False)
            + "\n"
            + json.dumps({"type": "SUPERVISOR_ADVISORY", "payload": {}}, ensure_ascii=False)
            + "\n",
            encoding="utf-8",
        )

        proc = self._run(self.temp_dir, "--local-only")

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Current Workspace Achievements", proc.stdout)
        self.assertIn("Workflow completed", proc.stdout)
        self.assertIn("Completed tasks: 2", proc.stdout)
        self.assertIn("Safe backlog unlocked: +3 tasks (2 rounds)", proc.stdout)
        self.assertIn("Supervisor advisories recorded: 1", proc.stdout)
        self.assertIn("score=81", proc.stdout)

    def test_local_summary_handles_zero_match_counts(self):
        fusion_dir = self.temp_dir / ".fusion"
        fusion_dir.mkdir(parents=True)

        (fusion_dir / "sessions.json").write_text(
            json.dumps({"status": "in_progress"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (fusion_dir / "task_plan.md").write_text(
            "### Task 1: A [PENDING]\n",
            encoding="utf-8",
        )
        (fusion_dir / "events.jsonl").write_text(
            json.dumps({"type": "OTHER_EVENT", "payload": {}}, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

        proc = self._run(self.temp_dir, "--local-only")

        self.assertEqual(proc.returncode, 0)
        self.assertEqual(proc.stderr.strip(), "")
        self.assertIn("score=0", proc.stdout)
        self.assertIn("(no achievements yet)", proc.stdout)

    def test_leaderboard_ranks_projects_by_score(self):
        root = self.temp_dir / "projects"
        project_alpha = root / "alpha"
        project_beta = root / "beta"

        (project_alpha / ".fusion").mkdir(parents=True)
        (project_alpha / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "completed"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project_alpha / ".fusion" / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [COMPLETED]\n",
            encoding="utf-8",
        )
        (project_alpha / ".fusion" / "events.jsonl").write_text(
            json.dumps({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 1}}, ensure_ascii=False)
            + "\n",
            encoding="utf-8",
        )

        (project_beta / ".fusion").mkdir(parents=True)
        (project_beta / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "in_progress"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project_beta / ".fusion" / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [COMPLETED]\n",
            encoding="utf-8",
        )

        proc = self._run(self.temp_dir, "--leaderboard-only", "--root", str(root), "--top", "5")

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Achievement Leaderboard", proc.stdout)
        self.assertRegex(proc.stdout, r"1\) alpha .*score=73")
        self.assertRegex(proc.stdout, r"2\) beta .*score=20")


    def test_rejects_empty_root_equals_value(self):
        proc = self._run(self.temp_dir, "--leaderboard-only", "--root=", "--top", "1")

        self.assertNotEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Missing value for --root", output)
        self.assertNotIn("=== Fusion Achievements ===", proc.stdout)

    def test_rejects_empty_top_equals_value(self):
        root = self.temp_dir / "projects"
        root.mkdir(parents=True)

        proc = self._run(self.temp_dir, "--leaderboard-only", "--root", str(root), "--top=")

        self.assertNotEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Missing value for --top", output)
        self.assertNotIn("=== Fusion Achievements ===", proc.stdout)

    def test_unknown_option_shows_usage_without_banner(self):
        proc = self._run(self.temp_dir, "--not-a-real-option")

        self.assertNotEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Unknown option", output)
        self.assertIn("Usage: fusion-achievements.sh", output)
        self.assertNotIn("=== Fusion Achievements ===", proc.stdout)

    def test_rejects_non_numeric_top_value(self):
        root = self.temp_dir / "projects"
        root.mkdir(parents=True)

        proc = self._run(self.temp_dir, "--leaderboard-only", "--root", str(root), "--top", "abc")

        self.assertNotEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("--top must be a positive integer", output)
        self.assertNotIn("head: invalid number of lines", output)
        self.assertNotIn("=== Fusion Achievements ===", proc.stdout)

    def test_rejects_missing_root_value(self):
        proc = self._run(self.temp_dir, "--root")

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Missing value for --root", proc.stdout + proc.stderr)

    def test_rejects_missing_top_value(self):
        root = self.temp_dir / "projects"
        root.mkdir(parents=True)

        proc = self._run(self.temp_dir, "--leaderboard-only", "--root", str(root), "--top")

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Missing value for --top", proc.stdout + proc.stderr)

    def test_supports_top_equals_syntax(self):
        root = self.temp_dir / "projects"
        project_alpha = root / "alpha"
        (project_alpha / ".fusion").mkdir(parents=True)
        (project_alpha / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "completed"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project_alpha / ".fusion" / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n",
            encoding="utf-8",
        )

        proc = self._run(self.temp_dir, "--leaderboard-only", "--root", str(root), "--top=1")

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Achievement Leaderboard", proc.stdout)
        self.assertRegex(proc.stdout, r"1\) alpha .*score=60")

    def test_supports_root_equals_syntax(self):
        root = self.temp_dir / "projects"
        project_alpha = root / "alpha"
        (project_alpha / ".fusion").mkdir(parents=True)
        (project_alpha / ".fusion" / "sessions.json").write_text(
            json.dumps({"status": "completed"}, ensure_ascii=False),
            encoding="utf-8",
        )
        (project_alpha / ".fusion" / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n",
            encoding="utf-8",
        )

        proc = self._run(self.temp_dir, "--leaderboard-only", f"--root={root}", "--top", "1")

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Achievement Leaderboard", proc.stdout)
        self.assertRegex(proc.stdout, r"1\) alpha .*score=60")


if __name__ == "__main__":
    unittest.main(verbosity=2)
