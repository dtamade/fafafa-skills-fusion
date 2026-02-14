"""文档新鲜度测试：避免硬编码全量测试通过数。"""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
CLI_CONTRACT_MATRIX = REPO_ROOT / "docs" / "CLI_CONTRACT_MATRIX.md"


class TestDocsFreshness(unittest.TestCase):
    def test_readme_zh_cn_avoids_hardcoded_pass_count(self):
        content = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        self.assertIsNone(
            re.search(r"全量测试：`\d+ passed`", content),
            "README.zh-CN.md contains hardcoded pass count; use dynamic wording instead.",
        )

    def test_hooks_setup_mentions_fix_flow(self):
        content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
        self.assertIn("--fix", content)
        self.assertRegex(content, r"fusion-hook-doctor\.sh\s+--json\s+--fix")

    def test_readme_mentions_hook_doctor_fix(self):
        content = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        self.assertRegex(content, r"fusion-hook-doctor\.sh\s+--json\s+--fix")

    def test_readme_zh_cn_mentions_hook_doctor_fix(self):
        content = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        self.assertRegex(content, r"fusion-hook-doctor\.sh\s+--json\s+--fix")

    def test_cli_contract_matrix_exists(self):
        self.assertTrue(
            CLI_CONTRACT_MATRIX.exists(),
            "Missing docs/CLI_CONTRACT_MATRIX.md",
        )

    def test_cli_contract_matrix_has_required_columns_and_commands(self):
        content = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")

        required_columns = [
            "| command |",
            "| valid args |",
            "| invalid args |",
            "| help exit code |",
            "| exit code |",
            "| stdout/stderr/json expectations |",
        ]
        for column in required_columns:
            with self.subTest(column=column):
                self.assertIn(column, content)

        required_commands = [
            "fusion-start.sh",
            "fusion-init.sh",
            "fusion-status.sh",
            "fusion-logs.sh",
            "fusion-git.sh",
            "fusion-codeagent.sh",
            "fusion-hook-doctor.sh",
            "fusion-achievements.sh",
            "fusion-pause.sh",
            "fusion-resume.sh",
            "fusion-cancel.sh",
            "fusion-continue.sh",
            "fusion-stop-guard.sh",
        ]

        for command in required_commands:
            with self.subTest(command=command):
                self.assertIn(command, content)

    def test_cli_contract_matrix_mentions_schema_and_basis_fields(self):
        content = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")
        self.assertIn("schema_version", content)
        self.assertIn("step_rate_basis", content)
        self.assertIn("command_rate_basis", content)
        self.assertIn("rate_basis", content)

    def test_cli_contract_matrix_has_machine_required_keys_note(self):
        content = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")
        self.assertIn("Required machine JSON keys", content)
        self.assertIn("release-contract-audit", content)
        self.assertIn("regression_runner", content)

    def test_cli_contract_matrix_mentions_schema_version_v1(self):
        content = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")
        self.assertIn("schema_version=v1", content)

    def test_cli_contract_matrix_notes_mention_ci_machine_artifacts(self):
        content = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")
        self.assertIn("/tmp/release-audit-dry-run.json", content)
        self.assertIn("/tmp/runner-contract.json", content)

    def test_hooks_setup_mentions_release_contract_audit(self):
        content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
        self.assertIn("release-contract-audit.sh", content)

    def test_hooks_setup_mentions_machine_schema_and_basis_fields(self):
        content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
        self.assertIn("schema_version", content)
        self.assertIn("step_rate_basis", content)
        self.assertIn("command_rate_basis", content)
        self.assertIn("rate_basis", content)

    def test_hooks_setup_explains_basis_denominators(self):
        content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
        self.assertIn("step_rate_basis=total_steps", content)
        self.assertIn("command_rate_basis=total_commands", content)

    def test_hooks_setup_mentions_ci_machine_artifacts(self):
        content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
        self.assertIn("/tmp/release-audit-dry-run.json", content)
        self.assertIn("/tmp/runner-contract.json", content)

    def test_hooks_setup_mentions_runner_suites_artifact(self):
        content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
        self.assertIn("/tmp/runner-suites.json", content)

    def test_hooks_setup_mentions_schema_version_v1(self):
        content = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
        self.assertIn("schema_version=v1", content)

    def test_readme_mentions_release_contract_audit_and_ci_workflow(self):
        content = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn("release-contract-audit.sh", content)
        self.assertIn("ci-contract-gates.yml", content)

    def test_readme_zh_cn_mentions_release_contract_audit_and_ci_workflow(self):
        content = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        self.assertIn("release-contract-audit.sh", content)
        self.assertIn("ci-contract-gates.yml", content)

    def test_docs_mention_release_audit_json_pretty(self):
        hooks = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
        readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")

        self.assertIn("--json-pretty", hooks)
        self.assertIn("--json-pretty", readme_en)
        self.assertIn("--json-pretty", readme_zh)

    def test_docs_mention_runner_list_suites_json(self):
        hooks = (REPO_ROOT / "docs" / "HOOKS_SETUP.md").read_text(encoding="utf-8")
        readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        matrix = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")

        self.assertIn("regression_runner.py --list-suites --json", hooks)
        self.assertIn("regression_runner.py --list-suites --json", readme_en)
        self.assertIn("regression_runner.py --list-suites --json", readme_zh)
        self.assertIn("regression_runner.py", matrix)

    def test_readme_mentions_machine_schema_and_basis_fields(self):
        content = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn("schema_version", content)
        self.assertIn("step_rate_basis", content)
        self.assertIn("command_rate_basis", content)
        self.assertIn("rate_basis", content)

    def test_readme_zh_cn_mentions_machine_schema_and_basis_fields(self):
        content = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        self.assertIn("schema_version", content)
        self.assertIn("step_rate_basis", content)
        self.assertIn("command_rate_basis", content)
        self.assertIn("rate_basis", content)

    def test_readme_en_zh_explain_basis_denominators(self):
        readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        self.assertIn("step_rate_basis=total_steps", readme_en)
        self.assertIn("command_rate_basis=total_commands", readme_en)
        self.assertIn("step_rate_basis=total_steps", readme_zh)
        self.assertIn("command_rate_basis=total_commands", readme_zh)

    def test_readme_en_zh_mention_ci_machine_artifacts(self):
        readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        for marker in ("/tmp/release-audit-dry-run.json", "/tmp/runner-contract.json"):
            self.assertIn(marker, readme_en)
            self.assertIn(marker, readme_zh)

    def test_readme_en_zh_mention_runner_suites_artifact(self):
        readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        self.assertIn("/tmp/runner-suites.json", readme_en)
        self.assertIn("/tmp/runner-suites.json", readme_zh)

    def test_readme_en_zh_mention_schema_version_v1(self):
        readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        self.assertIn("schema_version=v1", readme_en)
        self.assertIn("schema_version=v1", readme_zh)

    def test_readme_en_zh_mention_backend_failure_report(self):
        readme_en = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
        readme_zh = (REPO_ROOT / "README.zh-CN.md").read_text(encoding="utf-8")
        self.assertIn(".fusion/backend_failure_report.json", readme_en)
        self.assertIn(".fusion/backend_failure_report.json", readme_zh)

    def test_skill_md_mentions_backend_failure_report(self):
        content = (REPO_ROOT / "SKILL.md").read_text(encoding="utf-8")
        self.assertIn(".fusion/backend_failure_report.json", content)

    def test_cli_contract_matrix_mentions_backend_failure_report(self):
        content = CLI_CONTRACT_MATRIX.read_text(encoding="utf-8")
        for marker in (
            ".fusion/backend_failure_report.json",
            "backend_status",
            "backend_primary",
            "backend_fallback",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, content)


if __name__ == "__main__":
    unittest.main(verbosity=2)
