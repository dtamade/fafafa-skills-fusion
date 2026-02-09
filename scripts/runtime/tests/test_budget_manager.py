"""
Budget Manager 预算管理器单元测试
"""

import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.budget_manager import (
    BudgetConfig,
    BudgetStatus,
    BudgetManager,
    TaskUsage,
)


class TestBudgetConfig(unittest.TestCase):
    """配置默认值"""

    def test_defaults(self):
        config = BudgetConfig()
        self.assertEqual(config.global_token_limit, 100_000)
        self.assertEqual(config.global_latency_limit_ms, 7_200_000)
        self.assertEqual(config.warning_threshold, 0.8)
        self.assertEqual(config.hard_limit_action, "serial")


class TestBudgetStatus(unittest.TestCase):
    """状态快照"""

    def test_ratios(self):
        status = BudgetStatus(
            tokens_used=50_000,
            tokens_limit=100_000,
            latency_used_ms=3_600_000,
            latency_limit_ms=7_200_000,
            over_budget=False,
            warning=False,
        )
        self.assertAlmostEqual(status.token_ratio, 0.5)
        self.assertAlmostEqual(status.latency_ratio, 0.5)

    def test_zero_limit_ratios(self):
        status = BudgetStatus(
            tokens_used=0,
            tokens_limit=0,
            latency_used_ms=0,
            latency_limit_ms=0,
            over_budget=False,
            warning=False,
        )
        self.assertEqual(status.token_ratio, 0.0)
        self.assertEqual(status.latency_ratio, 0.0)


class TestRecordUsage(unittest.TestCase):
    """使用量记录"""

    def setUp(self):
        self.mgr = BudgetManager()

    def test_single_record(self):
        self.mgr.record_usage("1", tokens=1000, latency_ms=500)
        usage = self.mgr.get_task_usage("1")
        self.assertIsNotNone(usage)
        self.assertEqual(usage.tokens, 1000)
        self.assertEqual(usage.latency_ms, 500)

    def test_accumulate_same_task(self):
        self.mgr.record_usage("1", tokens=1000, latency_ms=500)
        self.mgr.record_usage("1", tokens=2000, latency_ms=300)
        usage = self.mgr.get_task_usage("1")
        self.assertEqual(usage.tokens, 3000)
        self.assertEqual(usage.latency_ms, 800)

    def test_multiple_tasks(self):
        self.mgr.record_usage("1", tokens=1000, latency_ms=500)
        self.mgr.record_usage("2", tokens=2000, latency_ms=300)
        status = self.mgr.get_status()
        self.assertEqual(status.tokens_used, 3000)
        self.assertEqual(status.latency_used_ms, 800)

    def test_unknown_task_returns_none(self):
        self.assertIsNone(self.mgr.get_task_usage("999"))


class TestBudgetChecks(unittest.TestCase):
    """预算检查逻辑"""

    def test_fresh_manager_not_over_budget(self):
        mgr = BudgetManager()
        self.assertFalse(mgr.is_over_budget())
        self.assertFalse(mgr.is_warning())

    def test_token_over_budget(self):
        mgr = BudgetManager(BudgetConfig(global_token_limit=1000))
        mgr.record_usage("1", tokens=1000, latency_ms=0)
        self.assertTrue(mgr.is_over_budget())

    def test_latency_over_budget(self):
        mgr = BudgetManager(BudgetConfig(global_latency_limit_ms=1000))
        mgr.record_usage("1", tokens=0, latency_ms=1000)
        self.assertTrue(mgr.is_over_budget())

    def test_warning_at_threshold(self):
        mgr = BudgetManager(
            BudgetConfig(global_token_limit=100, warning_threshold=0.8)
        )
        mgr.record_usage("1", tokens=80, latency_ms=0)
        self.assertTrue(mgr.is_warning())
        self.assertFalse(mgr.is_over_budget())

    def test_below_warning(self):
        mgr = BudgetManager(
            BudgetConfig(global_token_limit=100, warning_threshold=0.8)
        )
        mgr.record_usage("1", tokens=79, latency_ms=0)
        self.assertFalse(mgr.is_warning())


class TestCanExecute(unittest.TestCase):
    """执行预算检查"""

    def test_can_execute_fresh(self):
        mgr = BudgetManager()
        self.assertTrue(mgr.can_execute())

    def test_cannot_execute_over_budget(self):
        mgr = BudgetManager(BudgetConfig(global_token_limit=100))
        mgr.record_usage("1", tokens=100, latency_ms=0)
        self.assertFalse(mgr.can_execute())

    def test_cannot_execute_task_exceeds_remaining(self):
        mgr = BudgetManager(BudgetConfig(global_token_limit=100))
        mgr.record_usage("1", tokens=60, latency_ms=0)
        self.assertFalse(mgr.can_execute(cost_budget=50))
        self.assertTrue(mgr.can_execute(cost_budget=40))

    def test_latency_budget_check(self):
        mgr = BudgetManager(BudgetConfig(global_latency_limit_ms=1000))
        mgr.record_usage("1", tokens=0, latency_ms=800)
        self.assertFalse(mgr.can_execute(latency_budget=300))
        self.assertTrue(mgr.can_execute(latency_budget=200))

    def test_zero_budget_no_check(self):
        """cost_budget=0 不检查剩余空间"""
        mgr = BudgetManager(BudgetConfig(global_token_limit=100))
        mgr.record_usage("1", tokens=99, latency_ms=0)
        self.assertTrue(mgr.can_execute(cost_budget=0))


class TestSuggestDowngrade(unittest.TestCase):
    """降级建议"""

    def test_no_suggestion_when_healthy(self):
        mgr = BudgetManager()
        self.assertIsNone(mgr.suggest_downgrade())

    def test_warning_suggestion(self):
        mgr = BudgetManager(
            BudgetConfig(global_token_limit=100, warning_threshold=0.8)
        )
        mgr.record_usage("1", tokens=85, latency_ms=0)
        suggestion = mgr.suggest_downgrade()
        self.assertIsNotNone(suggestion)
        self.assertIn("WARNING", suggestion)

    def test_over_budget_suggestion(self):
        mgr = BudgetManager(BudgetConfig(global_token_limit=100))
        mgr.record_usage("1", tokens=100, latency_ms=0)
        suggestion = mgr.suggest_downgrade()
        self.assertIn("OVER_BUDGET", suggestion)
        self.assertIn("serial", suggestion)


class TestGetRemaining(unittest.TestCase):
    """剩余预算"""

    def test_full_remaining(self):
        mgr = BudgetManager(BudgetConfig(
            global_token_limit=1000,
            global_latency_limit_ms=5000,
        ))
        remaining = mgr.get_remaining()
        self.assertEqual(remaining["tokens"], 1000)
        self.assertEqual(remaining["latency_ms"], 5000)

    def test_partial_remaining(self):
        mgr = BudgetManager(BudgetConfig(
            global_token_limit=1000,
            global_latency_limit_ms=5000,
        ))
        mgr.record_usage("1", tokens=300, latency_ms=2000)
        remaining = mgr.get_remaining()
        self.assertEqual(remaining["tokens"], 700)
        self.assertEqual(remaining["latency_ms"], 3000)

    def test_over_budget_zero_remaining(self):
        mgr = BudgetManager(BudgetConfig(global_token_limit=100))
        mgr.record_usage("1", tokens=150, latency_ms=0)
        remaining = mgr.get_remaining()
        self.assertEqual(remaining["tokens"], 0)


class TestReset(unittest.TestCase):
    """重置"""

    def test_reset_clears_all(self):
        mgr = BudgetManager()
        mgr.record_usage("1", tokens=5000, latency_ms=1000)
        mgr.reset()
        status = mgr.get_status()
        self.assertEqual(status.tokens_used, 0)
        self.assertEqual(status.latency_used_ms, 0)
        self.assertIsNone(mgr.get_task_usage("1"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
