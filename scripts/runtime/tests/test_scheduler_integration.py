"""
Scheduler 集成测试 — Kernel ↔ Scheduler 端到端

通过 Kernel 公共 API 驱动，验证：
- init_scheduler 从 task_plan.md 构建完整管道
- 串行退化 (scheduler.enabled=false)
- 并行调度循环 (enabled=true)
- 冲突处理 (writeset overlap)
- 预算耗尽 (budget exhaustion)
- 任务完成解锁依赖
- 状态同步到 sessions.json
- get_status() 包含 scheduler 信息
- compat_v2 adapt_pretool/posttool 感知批次

v2.5.0 Week 7
"""

import unittest
import tempfile
import shutil
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.kernel import FusionKernel, KernelConfig
from runtime.state_machine import State, Event
from runtime.scheduler import SchedulerConfig, ScheduleDecision
from runtime.budget_manager import BudgetConfig
from runtime.compat_v2 import adapt_pretool, adapt_posttool


# ── 辅助 ──────────────────────────────────────

TASK_PLAN_SIMPLE = """\
## Tasks

### Task 1: 创建用户模型 [PENDING]
- Type: implementation
- Backend: codex
- Dependencies: []

### Task 2: 实现认证API [PENDING]
- Type: implementation
- Backend: codex
- Dependencies: [1]

### Task 3: 编写测试 [PENDING]
- Type: verification
- Backend: codex
- Dependencies: [2]
"""

TASK_PLAN_PARALLEL = """\
## Tasks

### Task 1: 用户模块 [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/user.py]

### Task 2: 订单模块 [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/order.py]

### Task 3: 支付模块 [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/payment.py]

### Task 4: 集成测试 [PENDING]
- Type: verification
- Dependencies: [1, 2, 3]
- Writeset: [tests/integration.py]
"""

TASK_PLAN_CONFLICT = """\
## Tasks

### Task 1: 模块A写入shared [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/shared.py, src/a.py]

### Task 2: 模块B写入shared [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/shared.py, src/b.py]

### Task 3: 模块C独立 [PENDING]
- Type: implementation
- Dependencies: []
- Writeset: [src/c.py]
"""

TASK_PLAN_BUDGET = """\
## Tasks

### Task 1: 轻量任务 [PENDING]
- Type: documentation
- Dependencies: []
- CostBudget: 100

### Task 2: 重量任务 [PENDING]
- Type: implementation
- Dependencies: []
- CostBudget: 50000
"""


def _setup_fusion_dir(task_plan_content: str) -> tuple:
    """创建临时 .fusion 目录并写入 task_plan.md 和 sessions.json"""
    temp_dir = tempfile.mkdtemp()
    fusion_dir = Path(temp_dir) / ".fusion"
    fusion_dir.mkdir()

    # task_plan.md
    (fusion_dir / "task_plan.md").write_text(task_plan_content, encoding="utf-8")

    # sessions.json — 模拟 EXECUTE 阶段
    sessions = {
        "status": "in_progress",
        "goal": "集成测试目标",
        "current_phase": "EXECUTE",
        "_runtime": {
            "version": "2.1.0",
            "last_event_counter": 0,
        },
    }
    (fusion_dir / "sessions.json").write_text(
        json.dumps(sessions, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    return temp_dir, str(fusion_dir)


# ── 测试类 ──────────────────────────────────────


class TestInitScheduler(unittest.TestCase):
    """init_scheduler 从 task_plan.md 初始化"""

    def setUp(self):
        self.temp_dir, self.fusion_dir = _setup_fusion_dir(TASK_PLAN_SIMPLE)
        self.kernel = FusionKernel(fusion_dir=self.fusion_dir)
        self.kernel.load_state()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_init_creates_scheduler(self):
        """init_scheduler 返回非空 Scheduler"""
        sched = self.kernel.init_scheduler()
        self.assertIsNotNone(sched)
        self.assertIsNotNone(self.kernel.scheduler)

    def test_init_loads_all_tasks(self):
        """Scheduler 包含 task_plan.md 中的全部任务"""
        sched = self.kernel.init_scheduler()
        progress = sched.get_progress()
        self.assertEqual(progress["total"], 3)
        self.assertEqual(progress["pending"], 3)

    def test_init_without_task_plan(self):
        """无 task_plan.md 时返回 None"""
        (Path(self.fusion_dir) / "task_plan.md").unlink()
        sched = self.kernel.init_scheduler()
        self.assertIsNone(sched)
        self.assertIsNone(self.kernel.scheduler)

    def test_init_respects_config(self):
        """自定义 SchedulerConfig 被应用"""
        config = SchedulerConfig(enabled=True, max_parallel=4)
        sched = self.kernel.init_scheduler(scheduler_config=config)
        self.assertTrue(sched.config.enabled)
        self.assertEqual(sched.config.max_parallel, 4)

    def test_context_updated(self):
        """init_scheduler 更新 context.scheduler_enabled"""
        self.kernel.init_scheduler(
            scheduler_config=SchedulerConfig(enabled=True)
        )
        self.assertTrue(self.kernel.context.scheduler_enabled)


class TestSerialDegradation(unittest.TestCase):
    """scheduler.enabled=false 时退化为串行"""

    def setUp(self):
        self.temp_dir, self.fusion_dir = _setup_fusion_dir(TASK_PLAN_PARALLEL)
        self.kernel = FusionKernel(fusion_dir=self.fusion_dir)
        self.kernel.load_state()
        self.kernel.init_scheduler(
            scheduler_config=SchedulerConfig(enabled=False)
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_serial_returns_single_task(self):
        """串行模式每次只返回一个任务"""
        decision = self.kernel.get_next_batch()
        self.assertIsNotNone(decision)
        self.assertEqual(len(decision.batch.tasks), 1)

    def test_serial_picks_first_by_id(self):
        """串行模式按 ID 顺序取第一个"""
        decision = self.kernel.get_next_batch()
        self.assertEqual(decision.batch.task_ids, ["1"])


class TestParallelScheduling(unittest.TestCase):
    """并行调度循环"""

    def setUp(self):
        self.temp_dir, self.fusion_dir = _setup_fusion_dir(TASK_PLAN_PARALLEL)
        self.kernel = FusionKernel(fusion_dir=self.fusion_dir)
        self.kernel.load_state()
        self.kernel.init_scheduler(
            scheduler_config=SchedulerConfig(enabled=True, max_parallel=3)
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_first_batch_parallel(self):
        """第一批次并行调度 3 个独立任务"""
        decision = self.kernel.get_next_batch()
        self.assertIsNotNone(decision)
        self.assertEqual(len(decision.batch.tasks), 3)
        self.assertCountEqual(
            decision.batch.task_ids, ["1", "2", "3"]
        )

    def test_dependent_task_waits(self):
        """有依赖的 Task 4 在第一批次不被调度"""
        decision = self.kernel.get_next_batch()
        self.assertNotIn("4", decision.batch.task_ids)

    def test_complete_unlocks_dependent(self):
        """完成所有前置后，依赖任务解锁"""
        # 完成 batch 1
        self.kernel.complete_task("1", tokens_used=100, latency_ms=50)
        self.kernel.complete_task("2", tokens_used=100, latency_ms=50)
        self.kernel.complete_task("3", tokens_used=100, latency_ms=50)
        self.kernel.scheduler.on_batch_done()

        # batch 2 应该包含 task 4
        decision = self.kernel.get_next_batch()
        self.assertIsNotNone(decision)
        self.assertEqual(decision.batch.task_ids, ["4"])

    def test_full_lifecycle(self):
        """完整生命周期：两轮批次后 is_all_done"""
        # batch 1
        self.kernel.complete_task("1")
        self.kernel.complete_task("2")
        self.kernel.complete_task("3")
        self.kernel.scheduler.on_batch_done()

        # batch 2
        self.kernel.complete_task("4")
        self.kernel.scheduler.on_batch_done()

        self.assertTrue(self.kernel.scheduler.is_all_done())
        self.assertIsNone(self.kernel.get_next_batch())


class TestConflictHandling(unittest.TestCase):
    """文件冲突检测集成"""

    def setUp(self):
        self.temp_dir, self.fusion_dir = _setup_fusion_dir(TASK_PLAN_CONFLICT)
        self.kernel = FusionKernel(fusion_dir=self.fusion_dir)
        self.kernel.load_state()
        self.kernel.init_scheduler(
            scheduler_config=SchedulerConfig(enabled=True, max_parallel=3)
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_conflict_defers_one_task(self):
        """writeset 冲突导致 Task 1/2 不能同时调度"""
        decision = self.kernel.get_next_batch()
        batch_ids = set(decision.batch.task_ids)
        # Task 1 和 Task 2 不能同时出现在同一批次
        self.assertFalse({"1", "2"}.issubset(batch_ids))
        # 但 Task 3 应该和其中一个一起出现
        self.assertIn("3", batch_ids)

    def test_deferred_shows_in_decision(self):
        """推迟的任务出现在 deferred 列表"""
        decision = self.kernel.get_next_batch()
        self.assertTrue(len(decision.deferred) > 0)

    def test_deferred_task_scheduled_next(self):
        """推迟的任务在下一批次被调度"""
        d1 = self.kernel.get_next_batch()
        # 完成第一批次
        for task_id in d1.batch.task_ids:
            self.kernel.complete_task(task_id)
        self.kernel.scheduler.on_batch_done()

        # 被推迟的任务现在可以调度
        d2 = self.kernel.get_next_batch()
        self.assertIsNotNone(d2)
        self.assertTrue(len(d2.batch.tasks) > 0)



class TestStrictBarrierPolicy(unittest.TestCase):
    """严格并发屏障策略（Kernel 端到端）"""

    def setUp(self):
        self.temp_dir, self.fusion_dir = _setup_fusion_dir(TASK_PLAN_PARALLEL)
        self.kernel = FusionKernel(fusion_dir=self.fusion_dir)
        self.kernel.load_state()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_barrier_waits_for_whole_batch_to_settle(self):
        """并发批次未全部结算前，不派发下一批"""
        self.kernel.init_scheduler(
            scheduler_config=SchedulerConfig(enabled=True, max_parallel=3)
        )

        d1 = self.kernel.get_next_batch()
        self.assertIsNotNone(d1)
        self.assertCountEqual(d1.batch.task_ids, ["1", "2", "3"])

        # 未结算前持续阻塞
        self.assertIsNone(self.kernel.get_next_batch())

        self.kernel.complete_task("1")
        self.assertIsNone(self.kernel.get_next_batch())

        self.kernel.complete_task("2")
        self.assertIsNone(self.kernel.get_next_batch())

        self.kernel.complete_task("3")

        # 批次全部结算后才解锁依赖任务
        d2 = self.kernel.get_next_batch()
        self.assertIsNotNone(d2)
        self.assertEqual(d2.batch.task_ids, ["4"])

    def test_fail_fast_halts_after_settled_failed_batch(self):
        """fail_fast=true 时，失败批次结算后停止后续派发"""
        self.kernel.init_scheduler(
            scheduler_config=SchedulerConfig(
                enabled=True,
                max_parallel=2,
                fail_fast=True,
            )
        )

        d1 = self.kernel.get_next_batch()
        self.assertIsNotNone(d1)
        self.assertCountEqual(d1.batch.task_ids, ["1", "2"])

        self.kernel.fail_task("1")
        # 批次尚未结算（task 2 未回调），仍处于屏障
        self.assertIsNone(self.kernel.get_next_batch())

        self.kernel.complete_task("2")

        # 批次结算后进入 fail_fast 停机，不再派发 task 3
        self.assertIsNone(self.kernel.get_next_batch())

        progress = self.kernel.scheduler.get_progress()
        self.assertTrue(progress["fail_fast_halted"])
        self.assertEqual(progress["pending"], 2)


class TestBudgetExhaustion(unittest.TestCase):
    """预算耗尽集成"""

    def setUp(self):
        self.temp_dir, self.fusion_dir = _setup_fusion_dir(TASK_PLAN_BUDGET)
        self.kernel = FusionKernel(fusion_dir=self.fusion_dir)
        self.kernel.load_state()

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_budget_skips_expensive(self):
        """预算不足时跳过高代价任务"""
        budget_config = BudgetConfig(global_token_limit=1000)
        self.kernel.init_scheduler(
            scheduler_config=SchedulerConfig(enabled=True, max_parallel=2),
            budget_config=budget_config,
        )
        decision = self.kernel.get_next_batch()
        # Task 1 (cost=100) 应可调度，Task 2 (cost=50000) 应被 skip
        batch_ids = decision.batch.task_ids
        self.assertIn("1", batch_ids)
        self.assertIn("2", decision.budget_skipped)

    def test_budget_warning_propagated(self):
        """预算警告传播到决策"""
        budget_config = BudgetConfig(
            global_token_limit=200,
            warning_threshold=0.5,
        )
        self.kernel.init_scheduler(
            scheduler_config=SchedulerConfig(enabled=True),
            budget_config=budget_config,
        )
        # 先消耗 60% 预算
        self.kernel.scheduler._budget.record_usage("warmup", tokens=130, latency_ms=0)
        decision = self.kernel.get_next_batch()
        self.assertTrue(len(decision.budget_warnings) > 0)


class TestContextSync(unittest.TestCase):
    """Kernel context 和 sessions.json 同步"""

    def setUp(self):
        self.temp_dir, self.fusion_dir = _setup_fusion_dir(TASK_PLAN_SIMPLE)
        self.kernel = FusionKernel(fusion_dir=self.fusion_dir)
        self.kernel.load_state()
        self.kernel.init_scheduler(
            scheduler_config=SchedulerConfig(enabled=True, max_parallel=2)
        )

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_complete_task_updates_context(self):
        """complete_task 更新 context 计数"""
        self.kernel.complete_task("1", tokens_used=500, latency_ms=100)
        self.assertEqual(self.kernel.context.completed_tasks, 1)
        self.assertEqual(self.kernel.context.pending_tasks, 2)

    def test_fail_task_updates_context(self):
        """fail_task 更新 context 计数"""
        self.kernel.fail_task("1", tokens_used=200, latency_ms=50)
        self.assertEqual(self.kernel.context.failed_tasks, 1)

    def test_scheduler_snapshot_in_sessions(self):
        """complete_task 将 scheduler 状态写入 sessions.json"""
        self.kernel.complete_task("1", tokens_used=500, latency_ms=100)

        sessions_file = Path(self.fusion_dir) / "sessions.json"
        with open(sessions_file, "r", encoding="utf-8") as f:
            data = json.load(f)

        runtime = data.get("_runtime", {})
        sched_data = runtime.get("scheduler")
        self.assertIsNotNone(sched_data)
        self.assertTrue(sched_data["enabled"])

    def test_get_status_includes_scheduler(self):
        """get_status() 包含 scheduler 信息"""
        status = self.kernel.get_status()
        self.assertIn("scheduler", status)
        self.assertTrue(status["scheduler"]["enabled"])
        self.assertIn("progress", status["scheduler"])


class TestCompatIntegration(unittest.TestCase):
    """compat_v2 感知 scheduler"""

    def setUp(self):
        self.temp_dir, self.fusion_dir = _setup_fusion_dir(TASK_PLAN_PARALLEL)
        # 先通过 Kernel 初始化 scheduler 并同步快照
        kernel = FusionKernel(fusion_dir=self.fusion_dir)
        kernel.load_state()
        kernel.init_scheduler(
            scheduler_config=SchedulerConfig(enabled=True, max_parallel=3)
        )
        # 模拟完成一个任务并同步
        kernel.complete_task("1", tokens_used=100, latency_ms=50)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_pretool_shows_batch_info(self):
        """adapt_pretool 输出包含 Batch 信息"""
        result = adapt_pretool(self.fusion_dir)
        self.assertTrue(result.active)
        batch_line = [l for l in result.lines if "Batch" in l]
        self.assertTrue(len(batch_line) > 0, f"No batch line in: {result.lines}")

    def test_posttool_detects_completion(self):
        """adapt_posttool 在进度变化后输出完成信息"""
        # 清除旧 snapshot 以触发变化
        snap_file = Path(self.fusion_dir) / ".progress_snapshot"
        if snap_file.exists():
            snap_file.unlink()

        result = adapt_posttool(self.fusion_dir)
        # posttool 会检测到进度变化（无旧快照 vs 当前状态）
        self.assertTrue(result.changed)


class TestGetNextBatchWithoutScheduler(unittest.TestCase):
    """未初始化 scheduler 时 get_next_batch 返回 None"""

    def setUp(self):
        self.temp_dir, self.fusion_dir = _setup_fusion_dir(TASK_PLAN_SIMPLE)
        self.kernel = FusionKernel(fusion_dir=self.fusion_dir)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_returns_none(self):
        self.assertIsNone(self.kernel.get_next_batch())


if __name__ == "__main__":
    unittest.main(verbosity=2)
