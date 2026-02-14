"""CI contract gate workflow tests."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci-contract-gates.yml"


class TestCIContractGatesWorkflow(unittest.TestCase):
    def test_workflow_file_exists(self):
        self.assertTrue(
            WORKFLOW.exists(),
            "Missing CI workflow: .github/workflows/ci-contract-gates.yml",
        )

    def test_workflow_contains_required_commands(self):
        content = WORKFLOW.read_text(encoding="utf-8")
        required_commands = [
            "bash -n scripts/*.sh",
            "pytest -q",
            "cargo clippy --workspace --all-targets -- -D warnings",
            "cargo fmt --all -- --check",
        ]

        for command in required_commands:
            with self.subTest(command=command):
                self.assertIn(command, content)

    def test_workflow_contains_cache_steps(self):
        content = WORKFLOW.read_text(encoding="utf-8")
        self.assertRegex(content, r"cache:\s*['\"]pip['\"]")
        self.assertIn("Swatinem/rust-cache", content)

    def test_workflow_contains_machine_mode_smoke_commands(self):
        content = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("release-contract-audit.sh --dry-run --json", content)
        self.assertIn("regression_runner.py --list-suites --json", content)
        self.assertIn("regression_runner.py --suite contract --json", content)


    def test_workflow_uploads_machine_json_artifacts(self):
        content = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("actions/upload-artifact@v4", content)
        self.assertIn("/tmp/release-audit-dry-run.json", content)
        self.assertIn("/tmp/runner-suites.json", content)
        self.assertIn("/tmp/runner-contract.json", content)

    def test_workflow_contains_runner_contract_schema_smoke(self):
        content = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("python3 - <<'PY'", content)
        self.assertIn("/tmp/runner-contract.json", content)
        self.assertIn("/tmp/release-audit-dry-run.json", content)
        self.assertIn("/tmp/runner-suites.json", content)
        self.assertIn("longest_scenario", content)
        self.assertIn("fastest_scenario", content)
        self.assertIn("scenario_count_by_result", content)
        self.assertIn("duration_stats", content)
        self.assertIn("failed_rate", content)
        self.assertIn("success_rate", content)
        self.assertIn("success_count", content)
        self.assertIn("failure_count", content)
        self.assertIn("total_scenarios", content)
        self.assertIn("failed_commands", content)
        self.assertIn("failed_commands_count", content)
        self.assertIn("error_step_count", content)
        self.assertIn("success_command_rate", content)
        self.assertIn("failed_command_rate", content)
        self.assertIn("schema_version", content)
        self.assertIn("step_rate_basis", content)
        self.assertIn("command_rate_basis", content)
        self.assertIn("rate_basis", content)
        self.assertIn("success_steps_count", content)
        self.assertIn("commands_count", content)
        self.assertIn("runner_contract['rate_basis'] != runner_contract['total_scenarios']", content)
        self.assertIn("release_dry_run['step_rate_basis'] != release_dry_run['steps_executed']", content)
        self.assertIn("release_dry_run['command_rate_basis'] != release_dry_run['commands_count']", content)
        self.assertIn("default_suite", content)



if __name__ == "__main__":
    unittest.main(verbosity=2)
