"""fusion-status.sh 输出测试"""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"


class TestFusionStatusScript(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True)
        (self.fusion_dir / "task_plan.md").write_text("## Status\n- Current Phase: EXECUTE\n", encoding="utf-8")
        (self.fusion_dir / "progress.md").write_text("| t | EXECUTE | e | OK | d |\n", encoding="utf-8")
        (self.fusion_dir / "sessions.json").write_text(
            json.dumps(
                {
                    "status": "in_progress",
                    "current_phase": "EXECUTE",
                    "_runtime": {
                        "last_event_id": "evt_000001",
                        "last_event_counter": 1,
                        "scheduler": {
                            "enabled": True,
                            "current_batch_id": 2,
                            "parallel_tasks": 1,
                        },
                    },
                }
            ),
            encoding="utf-8",
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_status_prints_runtime_summary(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Runtime", proc.stdout)
        self.assertIn("scheduler.enabled", proc.stdout)

    def test_status_prints_hook_debug_section_and_tail(self):
        (self.fusion_dir / ".hook_debug").write_text("", encoding="utf-8")
        (self.fusion_dir / "hook-debug.log").write_text(
            "[fusion][hook-debug][pretool][2026-02-12T00:00:00Z] invoked\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Hook Debug", proc.stdout)
        self.assertIn("hook_debug.enabled: true", proc.stdout)
        self.assertIn("hook_debug.tail:", proc.stdout)
        self.assertIn("[fusion][hook-debug][pretool]", proc.stdout)

    def test_status_help_exits_zero_without_fusion_dir(self):
        shutil.rmtree(self.fusion_dir)

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--help"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("Usage: fusion-status.sh", proc.stdout + proc.stderr)

    def test_status_json_mode_outputs_machine_readable_summary(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout.strip())
        self.assertEqual(payload.get("result"), "ok")
        self.assertEqual(payload.get("status"), "in_progress")
        self.assertEqual(payload.get("phase"), "EXECUTE")

    def test_status_json_mode_reports_missing_fusion_dir(self):
        shutil.rmtree(self.fusion_dir)

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout.strip())
        self.assertEqual(payload.get("result"), "error")
        self.assertIn("No .fusion directory found", payload.get("reason", ""))


    def test_status_json_unknown_option_reports_error_object(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json", "--bad"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertNotEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout.strip())
        self.assertEqual(payload.get("result"), "error")
        self.assertIn("Unknown option", payload.get("reason", ""))

    def test_status_json_help_still_shows_usage_and_exits_zero(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json", "--help"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Usage: fusion-status.sh", output)

    def test_status_json_mode_omits_human_banner(self):
        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertNotIn("=== Fusion Status ===", proc.stdout)

    def test_status_json_includes_task_counts(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [PENDING]\n"
            "### Task 3: C [IN_PROGRESS]\n"
            "### Task 4: D [FAILED]\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout.strip())
        self.assertEqual(payload.get("task_completed"), 1)
        self.assertEqual(payload.get("task_pending"), 1)
        self.assertEqual(payload.get("task_in_progress"), 1)
        self.assertEqual(payload.get("task_failed"), 1)

    def test_status_json_includes_dependency_summary(self):
        (self.fusion_dir / "dependency_report.json").write_text(
            json.dumps(
                {
                    "status": "blocked",
                    "missing": ["codeagent-wrapper"],
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout.strip())
        self.assertEqual(payload.get("dependency_status"), "blocked")
        self.assertIn("codeagent-wrapper", payload.get("dependency_missing", ""))

    def test_status_json_includes_backend_failure_summary(self):
        (self.fusion_dir / "backend_failure_report.json").write_text(
            json.dumps(
                {
                    "status": "blocked",
                    "source": "fusion-codeagent.sh",
                    "primary_backend": "claude",
                    "fallback_backend": "codex",
                    "primary_error": "claude-fail",
                    "fallback_error": "codex-fail",
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout.strip())
        self.assertEqual(payload.get("backend_status"), "blocked")
        self.assertEqual(payload.get("backend_primary"), "claude")
        self.assertEqual(payload.get("backend_fallback"), "codex")

    def test_status_json_includes_achievement_counters(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [COMPLETED]\n",
            encoding="utf-8",
        )
        (self.fusion_dir / "events.jsonl").write_text(
            json.dumps({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 2}}, ensure_ascii=False)
            + "\n"
            + json.dumps({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 3}}, ensure_ascii=False)
            + "\n"
            + json.dumps({"type": "SUPERVISOR_ADVISORY", "payload": {}}, ensure_ascii=False)
            + "\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout.strip())
        self.assertEqual(payload.get("achievement_completed_tasks"), 2)
        self.assertEqual(payload.get("achievement_safe_total"), 5)
        self.assertEqual(payload.get("achievement_advisory_total"), 1)


    def test_status_json_includes_owner_distribution_and_current_role(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "### Task 1: 需求澄清 [COMPLETED]\n"
            "- Type: research\n"
            "### Task 2: 实现核心逻辑 [IN_PROGRESS]\n"
            "- Type: implementation\n"
            "### Task 3: 回归验证 [PENDING]\n"
            "- Type: verification\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh"), "--json"],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout.strip())
        self.assertEqual(payload.get("owner_planner"), 1)
        self.assertEqual(payload.get("owner_coder"), 1)
        self.assertEqual(payload.get("owner_reviewer"), 1)
        self.assertEqual(payload.get("current_role"), "coder")
        self.assertEqual(payload.get("current_role_task"), "实现核心逻辑")
        self.assertEqual(payload.get("current_role_status"), "IN_PROGRESS")


    def test_status_prints_team_roles_summary(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "### Task 1: 方案设计 [PENDING]\n"
            "- Type: design\n"
            "### Task 2: 编码实现 [PENDING]\n"
            "- Owner: coder\n"
            "### Task 3: 集成验证 [PENDING]\n"
            "- Type: verification\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Team Roles", proc.stdout)
        self.assertIn("owner.planner: 1", proc.stdout)
        self.assertIn("owner.coder: 1", proc.stdout)
        self.assertIn("owner.reviewer: 1", proc.stdout)
        self.assertIn("current_role: planner", proc.stdout)
        self.assertIn("current_role_task: 方案设计 [PENDING]", proc.stdout)


    def test_status_prints_safe_backlog_summary(self):
        (self.fusion_dir / "events.jsonl").write_text(
            json.dumps(
                {
                    "id": "evt_000002",
                    "idempotency_key": "safe_backlog:0:1:0:0:1",
                    "type": "SAFE_BACKLOG_INJECTED",
                    "from_state": "EXECUTE",
                    "to_state": "EXECUTE",
                    "payload": {"added": 2},
                    "timestamp": 1_700_000_000,
                },
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)
        self.assertIn("safe_backlog.last_added: 2", proc.stdout)
        self.assertIn("safe_backlog.last_injected_at:", proc.stdout)
        self.assertIn("safe_backlog.last_injected_at_iso: 2023-11-14T22:13:20Z", proc.stdout)

    def test_status_prints_dependency_report(self):
        (self.fusion_dir / "dependency_report.json").write_text(
            json.dumps(
                {
                    "status": "blocked",
                    "source": "fusion-codeagent.sh",
                    "reason": "Missing executable for backend orchestration",
                    "missing": ["codeagent-wrapper"],
                    "next_actions": ["Install or expose codeagent-wrapper in PATH."],
                },
                ensure_ascii=False,
                indent=2,
            ),
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Dependency Report", proc.stdout)
        self.assertIn("status: blocked", proc.stdout)
        self.assertIn("missing: codeagent-wrapper", proc.stdout)

    def test_status_prints_backend_failure_report(self):
        (self.fusion_dir / "backend_failure_report.json").write_text(
            json.dumps(
                {
                    "status": "blocked",
                    "source": "fusion-codeagent.sh",
                    "primary_backend": "claude",
                    "fallback_backend": "codex",
                    "primary_error": "claude-fail",
                    "fallback_error": "codex-fail",
                },
                ensure_ascii=False,
                indent=2,
            ),
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Backend Failure Report", proc.stdout)
        self.assertIn("primary_backend: claude", proc.stdout)
        self.assertIn("fallback_backend: codex", proc.stdout)


    def test_status_prints_achievements_summary(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "## Status\n"
            "- Current Phase: EXECUTE\n\n"
            "### Task 1: 完成登录流程 [COMPLETED]\n"
            "### Task 2: 编写回归测试 [COMPLETED]\n"
            "### Task 3: 更新文档 [PENDING]\n",
            encoding="utf-8",
        )
        (self.fusion_dir / "events.jsonl").write_text(
            json.dumps(
                {
                    "id": "evt_000010",
                    "type": "SAFE_BACKLOG_INJECTED",
                    "payload": {"added": 2},
                    "timestamp": 1_700_000_010,
                },
                ensure_ascii=False,
            )
            + "\n"
            + json.dumps(
                {
                    "id": "evt_000011",
                    "type": "SUPERVISOR_ADVISORY",
                    "payload": {"risk_score": 0.8},
                    "timestamp": 1_700_000_011,
                },
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh")],
            cwd=str(self.temp_dir),
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Achievements", proc.stdout)
        self.assertIn("Completed tasks: 2", proc.stdout)
        self.assertIn("完成登录流程", proc.stdout)
        self.assertIn("编写回归测试", proc.stdout)
        self.assertIn("Safe backlog unlocked: +2 tasks", proc.stdout)
        self.assertIn("Supervisor advisories recorded: 1", proc.stdout)


    def test_status_can_disable_leaderboard(self):
        root = self.temp_dir / "projects"

        alpha = root / "alpha" / ".fusion"
        alpha.mkdir(parents=True)
        (alpha / "sessions.json").write_text(json.dumps({"status": "completed"}, ensure_ascii=False), encoding="utf-8")
        (alpha / "task_plan.md").write_text("### Task 1: A [COMPLETED]\n", encoding="utf-8")

        env = os.environ.copy()
        env["FUSION_LEADERBOARD_ROOT"] = str(root)
        env["FUSION_STATUS_SHOW_LEADERBOARD"] = "0"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh")],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertNotIn("## Achievement Leaderboard (Top 3)", proc.stdout)


    def test_status_prints_top3_achievement_leaderboard(self):
        root = self.temp_dir / "projects"

        alpha = root / "alpha" / ".fusion"
        alpha.mkdir(parents=True)
        (alpha / "sessions.json").write_text(json.dumps({"status": "completed"}, ensure_ascii=False), encoding="utf-8")
        (alpha / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [COMPLETED]\n",
            encoding="utf-8",
        )
        (alpha / "events.jsonl").write_text(
            json.dumps({"type": "SAFE_BACKLOG_INJECTED", "payload": {"added": 1}}, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

        gamma = root / "gamma" / ".fusion"
        gamma.mkdir(parents=True)
        (gamma / "sessions.json").write_text(json.dumps({"status": "completed"}, ensure_ascii=False), encoding="utf-8")
        (gamma / "task_plan.md").write_text("### Task 1: A [COMPLETED]\n", encoding="utf-8")
        (gamma / "events.jsonl").write_text(
            json.dumps({"type": "SUPERVISOR_ADVISORY", "payload": {}}, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

        beta = root / "beta" / ".fusion"
        beta.mkdir(parents=True)
        (beta / "sessions.json").write_text(json.dumps({"status": "in_progress"}, ensure_ascii=False), encoding="utf-8")
        (beta / "task_plan.md").write_text(
            "### Task 1: A [COMPLETED]\n"
            "### Task 2: B [COMPLETED]\n",
            encoding="utf-8",
        )

        env = os.environ.copy()
        env["FUSION_LEADERBOARD_ROOT"] = str(root)

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-status.sh")],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Achievement Leaderboard (Top 3)", proc.stdout)
        self.assertRegex(proc.stdout, r"1\) alpha .*score=73")
        self.assertRegex(proc.stdout, r"2\) gamma .*score=62")
        self.assertRegex(proc.stdout, r"3\) beta .*score=20")


if __name__ == "__main__":
    unittest.main(verbosity=2)
