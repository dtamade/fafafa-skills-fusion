"""
Scheduler 并行调度器单元测试
"""

import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.task_graph import TaskGraph, TaskNode
from runtime.budget_manager import BudgetManager, BudgetConfig
from runtime.scheduler import Scheduler, SchedulerConfig


def _build_graph(tasks):
    """辅助: 从 TaskNode 列表构建 TaskGraph"""
    return TaskGraph(tasks)


class TestSchedulerDisabled(unittest.TestCase):
    """调度器关闭时退化为串行"""

    def test_serial_mode_one_task_at_a_time(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
            TaskNode(task_id="3", name="C"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=False),
        )
        decision = scheduler.pick_next_batch()
        self.assertIsNotNone(decision)
        self.assertEqual(len(decision.batch.tasks), 1)

    def test_serial_mode_picks_first(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=False),
        )
        decision = scheduler.pick_next_batch()
        self.assertEqual(decision.batch.task_ids, ["1"])


class TestSchedulerEnabled(unittest.TestCase):
    """调度器启用时并行调度"""

    def test_parallel_independent_tasks(self):
        """无依赖的任务可以并行"""
        graph = _build_graph([
            TaskNode(task_id="1", name="A", writeset=["a.py"]),
            TaskNode(task_id="2", name="B", writeset=["b.py"]),
            TaskNode(task_id="3", name="C", writeset=["c.py"]),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=3),
        )
        decision = scheduler.pick_next_batch()
        self.assertEqual(len(decision.batch.tasks), 3)
        self.assertEqual(decision.deferred, [])

    def test_max_parallel_limit(self):
        """并行度限制"""
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
            TaskNode(task_id="3", name="C"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=2),
        )
        decision = scheduler.pick_next_batch()
        self.assertEqual(len(decision.batch.tasks), 2)

    def test_dependency_ordering(self):
        """有依赖时只调度已就绪任务"""
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=2),
        )
        decision = scheduler.pick_next_batch()
        self.assertEqual(decision.batch.task_ids, ["1"])

    def test_conflict_defers_task(self):
        """writeset 冲突导致任务推迟"""
        graph = _build_graph([
            TaskNode(task_id="1", name="A", writeset=["shared.py"]),
            TaskNode(task_id="2", name="B", writeset=["shared.py"]),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=2),
        )
        decision = scheduler.pick_next_batch()
        self.assertEqual(len(decision.batch.tasks), 1)
        self.assertEqual(decision.batch.task_ids, ["1"])
        self.assertEqual(decision.deferred, ["2"])


class TestStrictBatchBarrier(unittest.TestCase):
    """严格批次屏障：未结算前不可派发下一批"""

    def test_active_batch_blocks_until_all_callbacks(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=2),
        )

        d1 = scheduler.pick_next_batch()
        self.assertCountEqual(d1.batch.task_ids, ["1", "2"])

        # 活动批次未结算，不允许再次派发
        self.assertIsNone(scheduler.pick_next_batch())

        scheduler.on_task_done("1")
        self.assertIsNone(scheduler.pick_next_batch())

        scheduler.on_task_done("2")
        # 第一次 pick 触发关闭活动批次；无剩余任务
        self.assertIsNone(scheduler.pick_next_batch())

        progress = scheduler.get_progress()
        self.assertEqual(progress["batches_done"], 1)

    def test_settled_batch_advances_without_on_batch_done(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True),
        )

        d1 = scheduler.pick_next_batch()
        self.assertEqual(d1.batch.task_ids, ["1"])

        scheduler.on_task_done("1")
        # 不显式调用 on_batch_done 也可前进到下一批
        d2 = scheduler.pick_next_batch()
        self.assertIsNotNone(d2)
        self.assertEqual(d2.batch.task_ids, ["2"])

    def test_on_batch_done_ignores_unsettled_batch(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=2),
        )

        scheduler.pick_next_batch()
        scheduler.on_task_done("1")

        # 仍有未结算任务时，on_batch_done 不应推进批次
        scheduler.on_batch_done()
        self.assertEqual(scheduler.get_progress()["batches_done"], 0)

        scheduler.on_task_done("2")
        scheduler.on_batch_done()
        self.assertEqual(scheduler.get_progress()["batches_done"], 1)


class TestFailFastPolicy(unittest.TestCase):
    """fail_fast 分支行为"""

    def test_fail_fast_stops_future_dispatch(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
            TaskNode(task_id="3", name="C"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=2, fail_fast=True),
        )

        d1 = scheduler.pick_next_batch()
        self.assertCountEqual(d1.batch.task_ids, ["1", "2"])

        scheduler.on_task_failed("1")
        scheduler.on_task_done("2")

        # fail_fast=true：即使存在待执行任务，也停止派发
        self.assertIsNone(scheduler.pick_next_batch())

        progress = scheduler.get_progress()
        self.assertTrue(progress["fail_fast_halted"])
        self.assertEqual(progress["pending"], 1)

    def test_non_fail_fast_continues_after_failure(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
            TaskNode(task_id="3", name="C"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=2, fail_fast=False),
        )

        d1 = scheduler.pick_next_batch()
        self.assertCountEqual(d1.batch.task_ids, ["1", "2"])

        scheduler.on_task_failed("1")
        scheduler.on_task_done("2")

        # fail_fast=false：允许继续派发未受影响任务
        d2 = scheduler.pick_next_batch()
        self.assertIsNotNone(d2)
        self.assertEqual(d2.batch.task_ids, ["3"])


class TestBudgetIntegration(unittest.TestCase):
    """预算集成"""

    def test_budget_skips_expensive_task(self):
        """超预算任务被跳过"""
        budget = BudgetManager(BudgetConfig(global_token_limit=100))
        budget.record_usage("prev", tokens=80, latency_ms=0)

        graph = _build_graph([
            TaskNode(task_id="1", name="A", cost_budget=50),  # 需要50，剩余20
            TaskNode(task_id="2", name="B", cost_budget=10),  # 需要10，剩余20
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=2),
            budget_manager=budget,
        )
        decision = scheduler.pick_next_batch()
        self.assertIn("1", decision.budget_skipped)
        self.assertIn("2", [t.task_id for t in decision.batch.tasks])

    def test_all_over_budget(self):
        """全局超预算时返回空批次"""
        budget = BudgetManager(BudgetConfig(global_token_limit=100))
        budget.record_usage("prev", tokens=100, latency_ms=0)

        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True),
            budget_manager=budget,
        )
        decision = scheduler.pick_next_batch()
        # 全局超预算，所有任务被 budget skip
        self.assertIsNotNone(decision)
        self.assertEqual(len(decision.batch.tasks), 0)

    def test_budget_warning_in_decision(self):
        """预算警告出现在决策中"""
        budget = BudgetManager(
            BudgetConfig(global_token_limit=100, warning_threshold=0.8)
        )
        budget.record_usage("prev", tokens=85, latency_ms=0)

        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True),
            budget_manager=budget,
        )
        decision = scheduler.pick_next_batch()
        self.assertTrue(len(decision.budget_warnings) > 0)


class TestRoutingIntegration(unittest.TestCase):
    """模型路由集成"""

    def test_routing_in_decision(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A", task_type="implementation"),
            TaskNode(task_id="2", name="B", task_type="documentation"),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True, max_parallel=2),
        )
        decision = scheduler.pick_next_batch()
        self.assertIn("1", decision.routing)
        self.assertIn("2", decision.routing)
        self.assertEqual(decision.routing["1"].backend, "claude")
        self.assertEqual(decision.routing["2"].backend, "claude")


class TestTaskCallbacks(unittest.TestCase):
    """任务完成回调"""

    def test_on_task_done_unlocks_dependents(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        scheduler = Scheduler(
            graph=graph,
            config=SchedulerConfig(enabled=True),
        )

        # 第一次只能调度 task 1
        d1 = scheduler.pick_next_batch()
        self.assertEqual(d1.batch.task_ids, ["1"])

        # 标记完成
        scheduler.on_task_done("1", tokens_used=100, latency_ms=50)
        scheduler.on_batch_done()

        # 现在 task 2 可以调度
        d2 = scheduler.pick_next_batch()
        self.assertIsNotNone(d2)
        self.assertEqual(d2.batch.task_ids, ["2"])

    def test_on_task_failed_records(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
        ])
        scheduler = Scheduler(graph=graph)
        scheduler.on_task_failed("1", tokens_used=50, latency_ms=100)
        self.assertTrue(scheduler.has_failures())
        self.assertTrue(scheduler.is_all_done())

    def test_no_ready_tasks_returns_none(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A", dependencies=["2"]),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        # 循环依赖 — validate 会报错但 get_ready_tasks 返回空
        # 不过 TaskGraph 构建时不做验证，只有 topological_sort 时才验证
        # get_ready_tasks 只看 PENDING + 依赖满足
        scheduler = Scheduler(graph=graph)
        decision = scheduler.pick_next_batch()
        self.assertIsNone(decision)


class TestProgress(unittest.TestCase):
    """进度追踪"""

    def test_initial_progress(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
        ])
        scheduler = Scheduler(graph=graph)
        progress = scheduler.get_progress()
        self.assertEqual(progress["total"], 2)
        self.assertEqual(progress["completed"], 0)
        self.assertEqual(progress["pending"], 2)
        self.assertEqual(progress["batches_done"], 0)
        self.assertIn("budget", progress)

    def test_progress_after_completion(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
        ])
        scheduler = Scheduler(graph=graph)
        scheduler.pick_next_batch()
        scheduler.on_task_done("1", tokens_used=500, latency_ms=100)
        scheduler.on_batch_done()
        progress = scheduler.get_progress()
        self.assertEqual(progress["completed"], 1)
        self.assertEqual(progress["batches_done"], 1)
        self.assertEqual(progress["budget"]["tokens_used"], 500)

    def test_is_all_done(self):
        graph = _build_graph([
            TaskNode(task_id="1", name="A"),
        ])
        scheduler = Scheduler(graph=graph)
        self.assertFalse(scheduler.is_all_done())
        scheduler.on_task_done("1")
        self.assertTrue(scheduler.is_all_done())


if __name__ == "__main__":
    unittest.main(verbosity=2)
