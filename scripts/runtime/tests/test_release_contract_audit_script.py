"""Tests for release-contract-audit shell script."""

import json
import os
import re
import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPT = REPO_ROOT / "scripts" / "release-contract-audit.sh"


class TestReleaseContractAuditScript(unittest.TestCase):
    def run_script(self, *args: str, extra_env: dict[str, str] | None = None) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        if extra_env:
            env.update(extra_env)
        return subprocess.run(
            ["bash", str(SCRIPT), *args],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            timeout=30,
            env=env,
            check=False,
        )

    def test_help_exits_zero(self):
        proc = self.run_script("--help")
        self.assertEqual(proc.returncode, 0)
        self.assertIn("Usage: release-contract-audit.sh", proc.stdout + proc.stderr)

    def test_unknown_option_fails(self):
        proc = self.run_script("--bad")
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown option", proc.stdout + proc.stderr)

    def test_dry_run_lists_required_commands(self):
        proc = self.run_script("--dry-run")
        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("bash -n scripts/*.sh", output)
        self.assertIn("pytest -q", output)
        self.assertIn("cargo clippy --workspace --all-targets -- -D warnings", output)
        self.assertIn("cargo fmt --all -- --check", output)

    def test_dry_run_json_outputs_summary(self):
        proc = self.run_script("--dry-run", "--json", "--fast", "--skip-rust")
        self.assertEqual(proc.returncode, 0)

        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "ok")
        self.assertTrue(payload["dry_run"])
        self.assertTrue(payload["flags"]["json"])
        self.assertTrue(payload["flags"]["fast"])
        self.assertTrue(payload["flags"]["skip_rust"])
        self.assertIn("bash -n scripts/*.sh", payload["commands"])
        self.assertNotIn(
            "cargo clippy --workspace --all-targets -- -D warnings",
            payload["commands"],
        )

    def test_run_json_includes_step_metrics(self):
        proc = self.run_script("--json", "--skip-python", "--skip-rust")
        self.assertEqual(proc.returncode, 0)

        payload = json.loads(proc.stdout)
        self.assertFalse(payload["dry_run"])
        self.assertGreaterEqual(payload["steps_executed"], 1)
        self.assertGreaterEqual(payload["total_duration_ms"], 0)
        self.assertEqual(len(payload["step_results"]), payload["steps_executed"])
        self.assertIn("failed_steps", payload)
        self.assertIsInstance(payload["failed_steps"], list)
        self.assertEqual(payload["failed_steps"], [])
        self.assertIn("failed_steps_count", payload)
        self.assertEqual(payload["failed_steps_count"], 0)
        self.assertIn("error_step_count", payload)
        self.assertEqual(payload["error_step_count"], payload["failed_steps_count"])
        self.assertIn("failed_commands", payload)
        self.assertEqual(payload["failed_commands"], [])
        self.assertIn("failed_commands_count", payload)
        self.assertEqual(payload["failed_commands_count"], 0)
        self.assertIn("success_steps_count", payload)
        self.assertEqual(payload["success_steps_count"], payload["steps_executed"])
        self.assertIn("commands_count", payload)
        self.assertEqual(payload["commands_count"], len(payload["commands"]))
        self.assertIn("success_rate", payload)
        self.assertEqual(payload["success_rate"], 1.0)
        self.assertIn("failed_rate", payload)
        self.assertEqual(payload["failed_rate"], 0.0)
        self.assertIn("success_command_rate", payload)
        self.assertEqual(payload["success_command_rate"], 1.0)
        self.assertIn("failed_command_rate", payload)
        self.assertEqual(payload["failed_command_rate"], 0.0)
        self.assertIn("schema_version", payload)
        self.assertEqual(payload["schema_version"], "v1")
        self.assertIn("step_rate_basis", payload)
        self.assertEqual(payload["step_rate_basis"], payload["steps_executed"])
        self.assertIn("command_rate_basis", payload)
        self.assertEqual(payload["command_rate_basis"], payload["commands_count"])

        first = payload["step_results"][0]
        self.assertIn(first["status"], {"ok", "error"})
        self.assertIsInstance(first["command"], str)
        self.assertGreaterEqual(first["duration_ms"], 0)
        self.assertGreaterEqual(first["step"], 1)
        self.assertGreaterEqual(first["started_at_ms"], 0)
        self.assertGreaterEqual(first["finished_at_ms"], first["started_at_ms"])
        self.assertEqual(first["exit_code"], 0)

    def test_dry_run_json_pretty_outputs_multiline(self):
        proc = self.run_script("--dry-run", "--json", "--json-pretty", "--fast", "--skip-rust")
        self.assertEqual(proc.returncode, 0)
        self.assertIn("\n  \"result\":", proc.stdout)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "ok")

    def test_dry_run_fast_omits_full_pytest(self):
        proc = self.run_script("--dry-run", "--fast")
        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("test_fusion_control_script_validation.py", output)
        self.assertIsNone(re.search(r"^pytest -q$", output, re.MULTILINE))

    def test_dry_run_skip_rust_omits_rust_commands(self):
        proc = self.run_script("--dry-run", "--skip-rust")
        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertNotIn("cargo clippy --workspace --all-targets -- -D warnings", output)
        self.assertNotIn("cargo fmt --all -- --check", output)
        self.assertIn("pytest -q", output)

    def test_dry_run_skip_python_omits_pytest_commands(self):
        proc = self.run_script("--dry-run", "--skip-python")
        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertNotIn("pytest -q", output)
        self.assertIn("bash -n scripts/*.sh", output)

    def test_force_fail_step_json_reports_exit_code(self):
        proc = self.run_script(
            "--json",
            "--skip-python",
            "--skip-rust",
            extra_env={"FUSION_RELEASE_AUDIT_FORCE_FAIL_STEP": "1"},
        )
        self.assertNotEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "error")
        self.assertEqual(payload["step_results"][0]["exit_code"], 1)
        self.assertEqual(payload["failed_steps"], [1])
        self.assertEqual(payload["failed_steps_count"], 1)
        self.assertEqual(payload["error_step_count"], 1)
        self.assertEqual(payload["failed_commands"], ["bash -n scripts/*.sh"])
        self.assertEqual(payload["failed_commands_count"], 1)
        self.assertEqual(payload["success_steps_count"], 0)
        self.assertEqual(payload["commands_count"], len(payload["commands"]))
        self.assertEqual(payload["success_rate"], 0.0)
        self.assertEqual(payload["failed_rate"], 1.0)
        self.assertEqual(payload["success_command_rate"], 0.0)
        self.assertEqual(payload["failed_command_rate"], 1.0)
        self.assertEqual(payload["schema_version"], "v1")
        self.assertEqual(payload["step_rate_basis"], payload["steps_executed"])
        self.assertEqual(payload["command_rate_basis"], payload["commands_count"])

    def test_force_fail_step_reports_summary(self):
        proc = self.run_script(
            "--fast",
            "--skip-python",
            "--skip-rust",
            extra_env={"FUSION_RELEASE_AUDIT_FORCE_FAIL_STEP": "1"},
        )
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("failed at step 1", proc.stdout + proc.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
