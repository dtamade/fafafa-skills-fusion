"""
Fusion Runtime Scheduler — 并行调度器

组装 DAG 拓扑、冲突检测、预算管理、模型路由，
产出可执行的批次决策。
v2.5.0 Phase 2 核心组件。
"""

from dataclasses import dataclass, field
from typing import Dict, List, Optional

from .task_graph import TaskGraph, TaskNode, Batch
from .conflict_detector import ConflictDetector, ConflictResult
from .budget_manager import BudgetManager, BudgetConfig
from .router import Router, RouteDecision


@dataclass
class SchedulerConfig:
    """调度器配置"""
    enabled: bool = False      # 默认关闭，退化为 v2.1.0 串行
    max_parallel: int = 2      # 最大并行任务数
    fail_fast: bool = False    # 一个失败是否停止全部


@dataclass
class ScheduleDecision:
    """调度决策：一批可执行的任务"""
    batch: Batch
    deferred: List[str] = field(default_factory=list)      # 因冲突推迟
    budget_skipped: List[str] = field(default_factory=list) # 因预算跳过
    budget_warnings: List[str] = field(default_factory=list)
    routing: Dict[str, RouteDecision] = field(default_factory=dict)


class Scheduler:
    """并行调度器：综合 DAG/冲突/预算/路由做批次决策"""

    def __init__(
        self,
        graph: TaskGraph,
        config: Optional[SchedulerConfig] = None,
        conflict_detector: Optional[ConflictDetector] = None,
        budget_manager: Optional[BudgetManager] = None,
        router: Optional[Router] = None,
    ):
        self._graph = graph
        self._config = config or SchedulerConfig()
        self._conflict = conflict_detector or ConflictDetector()
        self._budget = budget_manager or BudgetManager()
        self._router = router or Router(budget_manager=self._budget)
        self._batches_done: int = 0

    @property
    def graph(self) -> TaskGraph:
        return self._graph

    @property
    def config(self) -> SchedulerConfig:
        return self._config

    # ── 调度决策 ──

    def pick_next_batch(self) -> Optional[ScheduleDecision]:
        """
        决定下一批可执行任务。

        决策管道:
        1. 从 DAG 获取所有依赖已满足的待执行任务
        2. 冲突检测：过滤 writeset 冲突
        3. 预算检查：过滤超预算任务
        4. 并行度限制：截取 max_parallel
        5. 模型路由：为每个任务选择后端

        Returns:
            ScheduleDecision 或 None (无可执行任务)
        """
        # 1. 获取就绪任务
        ready_tasks = self._graph.get_ready_tasks()
        if not ready_tasks:
            return None

        # 调度器关闭时退化为串行（只取第一个）
        if not self._config.enabled:
            task = ready_tasks[0]
            routing = self._router.route(task)
            batch = Batch(
                batch_id=self._batches_done + 1,
                tasks=[task],
            )
            return ScheduleDecision(
                batch=batch,
                routing={task.task_id: routing},
            )

        # 2. 冲突检测
        conflict_result = self._conflict.check(ready_tasks)
        safe_ids = set(conflict_result.safe_tasks)
        deferred = conflict_result.deferred_tasks

        safe_tasks = [
            t for t in ready_tasks if t.task_id in safe_ids
        ]

        # 3. 预算检查
        budget_skipped: List[str] = []
        budget_warnings: List[str] = []
        affordable_tasks: List[TaskNode] = []

        for task in safe_tasks:
            if not self._budget.can_execute(
                cost_budget=task.cost_budget,
                latency_budget=task.latency_budget,
            ):
                budget_skipped.append(task.task_id)
                continue
            affordable_tasks.append(task)

        # 预算警告
        suggestion = self._budget.suggest_downgrade()
        if suggestion:
            budget_warnings.append(suggestion)

        if not affordable_tasks:
            # 所有安全任务都超预算
            if budget_skipped:
                return ScheduleDecision(
                    batch=Batch(batch_id=self._batches_done + 1, tasks=[]),
                    deferred=deferred,
                    budget_skipped=budget_skipped,
                    budget_warnings=budget_warnings,
                )
            return None

        # 4. 并行度限制
        limited_tasks = affordable_tasks[:self._config.max_parallel]

        # 5. 模型路由
        routing = self._router.route_batch(limited_tasks)

        batch = Batch(
            batch_id=self._batches_done + 1,
            tasks=limited_tasks,
        )

        return ScheduleDecision(
            batch=batch,
            deferred=deferred,
            budget_skipped=budget_skipped,
            budget_warnings=budget_warnings,
            routing=routing,
        )

    # ── 任务完成回调 ──

    def on_task_done(self, task_id: str, tokens_used: int = 0, latency_ms: int = 0) -> None:
        """任务成功完成"""
        self._graph.mark_completed(task_id)
        self._budget.record_usage(task_id, tokens_used, latency_ms)

    def on_task_failed(self, task_id: str, tokens_used: int = 0, latency_ms: int = 0) -> None:
        """任务执行失败"""
        self._graph.mark_failed(task_id)
        self._budget.record_usage(task_id, tokens_used, latency_ms)

    def on_batch_done(self) -> None:
        """批次完成（所有任务已回调后调用）"""
        self._batches_done += 1

    # ── 状态查询 ──

    def is_all_done(self) -> bool:
        return self._graph.is_all_done()

    def has_failures(self) -> bool:
        return self._graph.get_failed_count() > 0

    def get_progress(self) -> Dict:
        progress = self._graph.get_progress()
        progress["batches_done"] = self._batches_done
        progress["budget"] = {
            "tokens_used": self._budget.get_status().tokens_used,
            "tokens_limit": self._budget.get_status().tokens_limit,
            "over_budget": self._budget.is_over_budget(),
        }
        return progress
