"""fusion-status.sh 输出测试"""

import json
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


if __name__ == "__main__":
    unittest.main(verbosity=2)
