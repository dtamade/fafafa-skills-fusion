"""
Fusion Runtime Router — 模型路由

根据任务类型、复杂度和预算状态选择后端 (codex/claude)。
v2.5.0 Phase 2 组件。
"""

from dataclasses import dataclass
from typing import Dict, List, Optional

from .task_graph import TaskNode
from .budget_manager import BudgetManager


# ── 路由规则表 ─────────────────────────────────────

# 任务类型 → 默认后端
DEFAULT_ROUTING: Dict[str, str] = {
    "implementation": "codex",
    "verification": "codex",
    "design": "codex",
    "research": "codex",
    "documentation": "claude",
    "configuration": "claude",
}


@dataclass
class RouteDecision:
    """路由决策"""
    task_id: str
    backend: str  # codex | claude
    reason: str


class Router:
    """模型路由器：根据任务类型和预算状态选择后端"""

    def __init__(
        self,
        budget_manager: Optional[BudgetManager] = None,
        default_backend: str = "codex",
    ):
        self._budget_manager = budget_manager
        self._default_backend = default_backend

    def route(self, task: TaskNode) -> RouteDecision:
        """
        为单个任务选择后端。

        优先级:
        1. 用户在 task_plan.md 中显式指定的 backend
        2. 预算紧张时强制降级为 claude
        3. 任务类型对应的默认后端
        4. 全局默认后端

        Args:
            task: 待路由的任务节点

        Returns:
            RouteDecision: 路由决策
        """
        # 1. 预算硬限制：超预算全部降级
        if self._budget_manager and self._budget_manager.is_over_budget():
            return RouteDecision(
                task_id=task.task_id,
                backend="claude",
                reason="budget_over: 强制降级为 claude",
            )

        # 2. 用户显式指定（非默认值时视为用户意图）
        if task.backend and task.backend != self._default_backend:
            return RouteDecision(
                task_id=task.task_id,
                backend=task.backend,
                reason=f"user_specified: 用户指定 {task.backend}",
            )

        # 3. 预算警告时降级
        if self._budget_manager and self._budget_manager.is_warning():
            return RouteDecision(
                task_id=task.task_id,
                backend="claude",
                reason="budget_warning: 预算紧张，降级为 claude",
            )

        # 4. 按任务类型路由
        if task.task_type in DEFAULT_ROUTING:
            backend = DEFAULT_ROUTING[task.task_type]
            return RouteDecision(
                task_id=task.task_id,
                backend=backend,
                reason=f"type_rule: {task.task_type} → {backend}",
            )

        # 5. 全局默认
        return RouteDecision(
            task_id=task.task_id,
            backend=self._default_backend,
            reason=f"default: 使用默认后端 {self._default_backend}",
        )

    def route_batch(self, tasks: List[TaskNode]) -> Dict[str, RouteDecision]:
        """
        为一批任务做路由决策。

        Args:
            tasks: 任务列表

        Returns:
            Dict[task_id, RouteDecision]
        """
        return {task.task_id: self.route(task) for task in tasks}
