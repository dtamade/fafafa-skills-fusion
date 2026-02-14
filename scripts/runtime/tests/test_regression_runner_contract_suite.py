"""Tests for regression_runner contract suite routing."""

import json
import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
RUNNER = REPO_ROOT / "scripts" / "runtime" / "regression_runner.py"


class TestRegressionRunnerContractSuite(unittest.TestCase):
    def run_runner(self, *args: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            ["python3", str(RUNNER), *args],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )

    def test_contract_suite_branch_runs(self):
        proc = self.run_runner("--suite", "contract", "--min-pass-rate", "0.99")
        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("Contract Regression Suite", output)
        self.assertIn("Suite: contract", output)
        self.assertNotIn("Full Regression Suite", output)

    def test_contract_suite_json_outputs_summary(self):
        proc = self.run_runner("--suite", "contract", "--json", "--min-pass-rate", "0.99")
        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "ok")
        self.assertEqual(payload["suite"], "contract")
        self.assertEqual(payload["passed"], payload["total"])
        self.assertGreaterEqual(payload["pass_rate"], payload["min_pass_rate"])
        self.assertGreaterEqual(payload["duration_ms"], 0)
        self.assertIn("scenario_results", payload)
        self.assertIsInstance(payload["scenario_results"], list)
        self.assertEqual(len(payload["scenario_results"]), payload["total"])
        self.assertIn("failed_scenarios", payload)
        self.assertIsInstance(payload["failed_scenarios"], list)
        self.assertIn("longest_scenario", payload)
        self.assertIn("name", payload["longest_scenario"])
        self.assertIn("duration_ms", payload["longest_scenario"])
        self.assertGreaterEqual(payload["longest_scenario"]["duration_ms"], 0)
        self.assertIn("fastest_scenario", payload)
        self.assertIn("name", payload["fastest_scenario"])
        self.assertIn("duration_ms", payload["fastest_scenario"])
        self.assertGreaterEqual(payload["fastest_scenario"]["duration_ms"], 0)
        self.assertLessEqual(
            payload["fastest_scenario"]["duration_ms"],
            payload["longest_scenario"]["duration_ms"],
        )
        self.assertIn("scenario_count_by_result", payload)
        self.assertIn("passed", payload["scenario_count_by_result"])
        self.assertIn("failed", payload["scenario_count_by_result"])
        self.assertEqual(payload["scenario_count_by_result"]["passed"], payload["passed"])
        self.assertEqual(payload["scenario_count_by_result"]["failed"], len(payload["failed_scenarios"]))
        self.assertIn("duration_stats", payload)
        self.assertIn("min_duration_ms", payload["duration_stats"])
        self.assertIn("max_duration_ms", payload["duration_stats"])
        self.assertIn("avg_duration_ms", payload["duration_stats"])
        self.assertLessEqual(payload["duration_stats"]["min_duration_ms"], payload["duration_stats"]["max_duration_ms"])
        self.assertGreaterEqual(payload["duration_stats"]["avg_duration_ms"], payload["duration_stats"]["min_duration_ms"])
        self.assertLessEqual(payload["duration_stats"]["avg_duration_ms"], payload["duration_stats"]["max_duration_ms"])
        self.assertIn("failed_rate", payload)
        expected_failed_rate = len(payload["failed_scenarios"]) / payload["total"]
        self.assertAlmostEqual(payload["failed_rate"], expected_failed_rate)
        self.assertIn("success_rate", payload)
        expected_success_rate = payload["passed"] / payload["total"]
        self.assertAlmostEqual(payload["success_rate"], expected_success_rate)
        self.assertAlmostEqual(payload["success_rate"] + payload["failed_rate"], 1.0)
        self.assertIn("success_count", payload)
        self.assertEqual(payload["success_count"], payload["passed"])
        self.assertIn("failure_count", payload)
        self.assertEqual(payload["failure_count"], len(payload["failed_scenarios"]))
        self.assertIn("total_scenarios", payload)
        self.assertEqual(payload["total_scenarios"], payload["total"])
        self.assertIn("schema_version", payload)
        self.assertEqual(payload["schema_version"], "v1")
        self.assertIn("rate_basis", payload)
        self.assertEqual(payload["rate_basis"], payload["total_scenarios"])
        first = payload["scenario_results"][0]
        self.assertIn("name", first)
        self.assertIn("passed", first)
        self.assertIn("duration_ms", first)
        self.assertIn("error", first)

    def test_unknown_suite_rejected(self):
        proc = self.run_runner("--suite", "does-not-exist", "--min-pass-rate", "0.99")
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Unknown suite", proc.stdout + proc.stderr)

    def test_list_suites_outputs_supported_set(self):
        proc = self.run_runner("--list-suites")
        self.assertEqual(proc.returncode, 0)
        output = proc.stdout + proc.stderr
        self.assertIn("phase1", output)
        self.assertIn("phase2", output)
        self.assertIn("contract", output)
        self.assertIn("all", output)

    def test_list_suites_json_outputs_machine_payload(self):
        proc = self.run_runner("--list-suites", "--json")
        self.assertEqual(proc.returncode, 0)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["result"], "ok")
        self.assertEqual(payload["default_suite"], "all")
        self.assertEqual(payload["suites"], ["phase1", "phase2", "contract", "all"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
