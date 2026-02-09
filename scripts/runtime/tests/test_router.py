"""
Router 模型路由器单元测试
"""

import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.task_graph import TaskNode
from runtime.budget_manager import BudgetManager, BudgetConfig
from runtime.router import Router, RouteDecision, DEFAULT_ROUTING


class TestDefaultRouting(unittest.TestCase):
    """默认路由规则"""

    def setUp(self):
        self.router = Router()

    def test_implementation_to_codex(self):
        task = TaskNode(task_id="1", name="A", task_type="implementation")
        decision = self.router.route(task)
        self.assertEqual(decision.backend, "codex")
        self.assertIn("type_rule", decision.reason)

    def test_verification_to_codex(self):
        task = TaskNode(task_id="2", name="B", task_type="verification")
        decision = self.router.route(task)
        self.assertEqual(decision.backend, "codex")

    def test_documentation_to_claude(self):
        task = TaskNode(task_id="3", name="C", task_type="documentation")
        decision = self.router.route(task)
        self.assertEqual(decision.backend, "claude")

    def test_configuration_to_claude(self):
        task = TaskNode(task_id="4", name="D", task_type="configuration")
        decision = self.router.route(task)
        self.assertEqual(decision.backend, "claude")

    def test_unknown_type_uses_default(self):
        task = TaskNode(task_id="5", name="E", task_type="custom_type")
        decision = self.router.route(task)
        self.assertEqual(decision.backend, "codex")
        self.assertIn("default", decision.reason)


class TestUserOverride(unittest.TestCase):
    """用户显式指定后端"""

    def test_user_specified_claude(self):
        router = Router(default_backend="codex")
        task = TaskNode(
            task_id="1", name="A",
            task_type="implementation", backend="claude",
        )
        decision = router.route(task)
        self.assertEqual(decision.backend, "claude")
        self.assertIn("user_specified", decision.reason)

    def test_same_as_default_uses_type_rule(self):
        """backend == default_backend 时不视为用户显式指定"""
        router = Router(default_backend="codex")
        task = TaskNode(
            task_id="1", name="A",
            task_type="documentation", backend="codex",
        )
        decision = router.route(task)
        # 应走 type_rule，而不是 user_specified
        self.assertEqual(decision.backend, "claude")
        self.assertIn("type_rule", decision.reason)


class TestBudgetAwareRouting(unittest.TestCase):
    """预算感知路由"""

    def test_over_budget_forces_claude(self):
        budget = BudgetManager(BudgetConfig(global_token_limit=100))
        budget.record_usage("x", tokens=100, latency_ms=0)
        router = Router(budget_manager=budget)

        task = TaskNode(
            task_id="1", name="A", task_type="implementation",
        )
        decision = router.route(task)
        self.assertEqual(decision.backend, "claude")
        self.assertIn("budget_over", decision.reason)

    def test_warning_downgrades_to_claude(self):
        budget = BudgetManager(
            BudgetConfig(global_token_limit=100, warning_threshold=0.8)
        )
        budget.record_usage("x", tokens=85, latency_ms=0)
        router = Router(budget_manager=budget)

        task = TaskNode(
            task_id="1", name="A", task_type="implementation",
        )
        decision = router.route(task)
        self.assertEqual(decision.backend, "claude")
        self.assertIn("budget_warning", decision.reason)

    def test_user_override_beats_warning(self):
        """over_budget 优先于 user_specified，但 user_specified 优先于 warning"""
        budget = BudgetManager(
            BudgetConfig(global_token_limit=100, warning_threshold=0.8)
        )
        budget.record_usage("x", tokens=85, latency_ms=0)
        router = Router(budget_manager=budget, default_backend="codex")

        # 用户指定 claude，应走 user_specified 而不是 budget_warning
        task = TaskNode(
            task_id="1", name="A",
            task_type="implementation", backend="claude",
        )
        decision = router.route(task)
        self.assertEqual(decision.backend, "claude")
        self.assertIn("user_specified", decision.reason)

    def test_healthy_budget_uses_type_rule(self):
        budget = BudgetManager(
            BudgetConfig(global_token_limit=100000)
        )
        router = Router(budget_manager=budget)

        task = TaskNode(
            task_id="1", name="A", task_type="implementation",
        )
        decision = router.route(task)
        self.assertEqual(decision.backend, "codex")
        self.assertIn("type_rule", decision.reason)


class TestRouteBatch(unittest.TestCase):
    """批量路由"""

    def test_batch_routing(self):
        router = Router()
        tasks = [
            TaskNode(task_id="1", name="A", task_type="implementation"),
            TaskNode(task_id="2", name="B", task_type="documentation"),
        ]
        decisions = router.route_batch(tasks)
        self.assertEqual(len(decisions), 2)
        self.assertEqual(decisions["1"].backend, "codex")
        self.assertEqual(decisions["2"].backend, "claude")


if __name__ == "__main__":
    unittest.main(verbosity=2)
