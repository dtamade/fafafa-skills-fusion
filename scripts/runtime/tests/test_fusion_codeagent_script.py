"""fusion-codeagent.sh 脚本测试"""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"


class TestFusionCodeagentScript(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True)
        (self.fusion_dir / "task_plan.md").write_text("### Task 1: A [PENDING]\n", encoding="utf-8")
        (self.fusion_dir / "sessions.json").write_text(
            json.dumps(
                {
                    "goal": "测试 goal",
                    "status": "in_progress",
                    "current_phase": "EXECUTE",
                    "codex_session": None,
                }
            ),
            encoding="utf-8",
        )
        (self.fusion_dir / "config.yaml").write_text(
            "backends:\n"
            "  primary: codex\n"
            "  fallback: claude\n",
            encoding="utf-8",
        )

        # mock codeagent-wrapper
        self.bin_dir = self.temp_dir / "bin"
        self.bin_dir.mkdir()
        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text(
            "#!/bin/bash\n"
            "echo \"mock backend:$2\"\n"
            "echo \"SESSION_ID: 123456\"\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o755)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_execute_phase_prefers_claude_and_updates_session_id(self):
        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)
        self.assertIn("mock backend", proc.stdout)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("claude_session"), "123456")

    def test_execute_phase_stores_uuid_session_id(self):
        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text(
            "#!/bin/bash\n"
            "echo \"mock backend:$2\"\n"
            "echo \"SESSION_ID: 283e89c0-48ef-4f0b-b66a-4d9dc66473a7\"\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o755)

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("claude_session"), "283e89c0-48ef-4f0b-b66a-4d9dc66473a7")

    def test_fallback_updates_claude_session(self):
        # 覆盖 mock: codex 失败, claude 成功
        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text(
            "#!/bin/bash\n"
            "if [ \"$2\" = \"codex\" ]; then exit 1; fi\n"
            "echo \"mock backend:$2\"\n"
            "echo \"SESSION_ID: 654321\"\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o755)

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("claude_session"), "654321")

    def test_execute_phase_resume_failure_retries_without_resume(self):
        sessions = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        sessions["claude_session"] = "1610419"
        (self.fusion_dir / "sessions.json").write_text(json.dumps(sessions), encoding="utf-8")

        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text(
            "#!/bin/bash\n"
            "backend=\"$2\"\n"
            "cmd=\"$3\"\n"
            "sid=\"$4\"\n"
            "\n"
            "if [ \"$backend\" = \"claude\" ] && [ \"$cmd\" = \"resume\" ] && [ \"$sid\" = \"1610419\" ]; then\n"
            "  echo \"resume failed\" >&2\n"
            "  exit 1\n"
            "fi\n"
            "\n"
            "if [ \"$backend\" = \"codex\" ]; then\n"
            "  echo \"codex should not be used\" >&2\n"
            "  exit 2\n"
            "fi\n"
            "\n"
            "echo \"mock backend:$backend\"\n"
            "echo \"SESSION_ID: 283e89c0-48ef-4f0b-b66a-4d9dc66473a7\"\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o755)

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("mock backend:claude", proc.stdout)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("claude_session"), "283e89c0-48ef-4f0b-b66a-4d9dc66473a7")

    def test_timeout_falls_back_to_claude(self):
        if shutil.which("timeout") is None and shutil.which("gtimeout") is None:
            self.skipTest("timeout command not available")

        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text(
            "#!/bin/bash\n"
            "backend=\"$2\"\n"
            "\n"
            "if [ \"$backend\" = \"codex\" ]; then\n"
            "  sleep 2\n"
            "  echo \"mock backend:$backend\"\n"
            "  echo \"SESSION_ID: 111111\"\n"
            "  exit 0\n"
            "fi\n"
            "\n"
            "echo \"mock backend:$backend\"\n"
            "echo \"SESSION_ID: 222222\"\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o755)

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"
        env["FUSION_CODEAGENT_TIMEOUT_SEC"] = "1"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "REVIEW"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("mock backend:claude", proc.stdout)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("claude_session"), "222222")

    def test_double_backend_failure_returns_nonzero(self):
        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text("#!/bin/bash\nexit 1\n", encoding="utf-8")
        wrapper.chmod(0o755)

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertNotEqual(proc.returncode, 0)

    def test_double_backend_failure_writes_backend_failure_report(self):
        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text(
            "#!/bin/bash\n"
            "if [ \"$2\" = \"codex\" ]; then\n"
            "  echo \"codex-fail\" >&2\n"
            "  exit 11\n"
            "fi\n"
            "echo \"claude-fail\" >&2\n"
            "exit 12\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o755)

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertNotEqual(proc.returncode, 0)

        report_file = self.fusion_dir / "backend_failure_report.json"
        self.assertTrue(report_file.exists())

        report = json.loads(report_file.read_text(encoding="utf-8"))
        self.assertEqual(report.get("status"), "blocked")
        self.assertEqual(report.get("source"), "fusion-codeagent.sh")
        self.assertEqual(report.get("primary_backend"), "claude")
        self.assertEqual(report.get("fallback_backend"), "codex")
        self.assertIn("claude-fail", report.get("primary_error", ""))
        self.assertIn("codex-fail", report.get("fallback_error", ""))

    def test_success_clears_stale_backend_failure_report(self):
        stale_report = self.fusion_dir / "backend_failure_report.json"
        stale_report.write_text(
            json.dumps({"status": "blocked", "source": "fusion-codeagent.sh"}, ensure_ascii=False),
            encoding="utf-8",
        )

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)
        self.assertFalse(stale_report.exists())

    def test_review_phase_prefers_codex(self):
        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "REVIEW"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)
        self.assertIn("mock backend:codex", proc.stdout)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("codex_session"), "123456")

    def test_execute_phase_owner_planner_routes_to_codex_and_stores_role_session(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "### Task 1: Planning [PENDING]\n"
            "- Owner: planner\n"
            "- Type: implementation\n",
            encoding="utf-8",
        )

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)
        self.assertIn("mock backend:codex", proc.stdout)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("planner_codex_session"), "123456")
        self.assertEqual(data.get("codex_session"), "123456")

    def test_execute_phase_owner_session_is_used_for_resume(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "### Task 1: Planning [PENDING]\n"
            "- Owner: planner\n"
            "- Type: research\n",
            encoding="utf-8",
        )

        sessions = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        sessions["planner_codex_session"] = "999999"
        (self.fusion_dir / "sessions.json").write_text(json.dumps(sessions), encoding="utf-8")

        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text(
            "#!/bin/bash\n"
            "echo \"$@\"\n"
            "echo \"SESSION_ID: 222222\"\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o755)

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("resume 999999", proc.stdout)

    def test_env_role_override_routes_to_reviewer_backend(self):
        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"
        env["FUSION_AGENT_ROLE"] = "reviewer"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("mock backend:codex", proc.stdout)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("reviewer_codex_session"), "123456")

    def test_execute_phase_without_owner_keeps_type_based_routing(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "### Task 1: Explore Design [PENDING]\n"
            "- Type: research\n",
            encoding="utf-8",
        )

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        self.assertIn("mock backend:codex", proc.stdout)

    def test_execute_phase_auto_inserts_owner_for_pending_tasks(self):
        (self.fusion_dir / "task_plan.md").write_text(
            "### Task 1: Explore Design [PENDING]\n"
            "- Type: research\n"
            "- Dependencies: []\n",
            encoding="utf-8",
        )

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )

        self.assertEqual(proc.returncode, 0)
        content = (self.fusion_dir / "task_plan.md").read_text(encoding="utf-8")
        self.assertIn("- Owner: planner", content)

    def test_help_exits_zero_without_routing(self):
        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "--help"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        output = proc.stdout + proc.stderr
        self.assertEqual(proc.returncode, 0)
        self.assertIn("Usage: fusion-codeagent.sh", output)
        self.assertNotIn("[fusion] route:", output)


    def test_unknown_option_exits_nonzero_without_routing(self):
        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "--bad"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=6,
            check=False,
        )

        output = proc.stdout + proc.stderr
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown option", output)
        self.assertIn("Usage: fusion-codeagent.sh", output)
        self.assertNotIn("[fusion] route:", output)

    def test_missing_wrapper_emits_dependency_report(self):
        (self.bin_dir / "codeagent-wrapper").unlink(missing_ok=True)

        env = dict(os.environ)
        # Keep core commands available but avoid accidental wrapper from custom PATHs
        env["PATH"] = "/usr/bin:/bin"
        env.pop("CODEAGENT_WRAPPER_BIN", None)

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 127)
        self.assertIn("Missing dependency: codeagent-wrapper", proc.stderr)

        report_file = self.fusion_dir / "dependency_report.json"
        self.assertTrue(report_file.exists())

        report = json.loads(report_file.read_text(encoding="utf-8"))
        self.assertEqual(report.get("status"), "blocked")
        self.assertIn("codeagent-wrapper", report.get("missing", []))

    def test_missing_wrapper_clears_stale_backend_failure_report(self):
        (self.bin_dir / "codeagent-wrapper").unlink(missing_ok=True)

        stale_report = self.fusion_dir / "backend_failure_report.json"
        stale_report.write_text(
            json.dumps(
                {
                    "status": "blocked",
                    "source": "fusion-codeagent.sh",
                    "primary_backend": "claude",
                    "fallback_backend": "codex",
                    "primary_error": "old error",
                    "fallback_error": "old error",
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )

        env = dict(os.environ)
        env["PATH"] = "/usr/bin:/bin"
        env.pop("CODEAGENT_WRAPPER_BIN", None)

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )

        self.assertEqual(proc.returncode, 127)
        self.assertFalse(stale_report.exists())

        report_file = self.fusion_dir / "dependency_report.json"
        self.assertTrue(report_file.exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
