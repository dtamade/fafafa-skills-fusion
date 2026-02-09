"""
Fusion Runtime Budget Manager — Token/时延预算管理

追踪全局和每任务预算，超预算检测与降级建议。
v2.5.0 Phase 2 组件。
"""

from dataclasses import dataclass, field
from typing import Dict, Optional, List


@dataclass
class BudgetConfig:
    """预算配置"""
    global_token_limit: int = 100_000
    global_latency_limit_ms: int = 7_200_000  # 2 小时
    warning_threshold: float = 0.8  # 80% 时警告
    hard_limit_action: str = "serial"  # serial | pause | skip


@dataclass
class BudgetStatus:
    """预算状态快照"""
    tokens_used: int
    tokens_limit: int
    latency_used_ms: int
    latency_limit_ms: int
    over_budget: bool
    warning: bool

    @property
    def token_ratio(self) -> float:
        if self.tokens_limit == 0:
            return 0.0
        return self.tokens_used / self.tokens_limit

    @property
    def latency_ratio(self) -> float:
        if self.latency_limit_ms == 0:
            return 0.0
        return self.latency_used_ms / self.latency_limit_ms


@dataclass
class TaskUsage:
    """单任务使用量记录"""
    tokens: int = 0
    latency_ms: int = 0


class BudgetManager:
    """Token/时延预算管理器"""

    def __init__(self, config: Optional[BudgetConfig] = None):
        self._config = config or BudgetConfig()
        self._task_usage: Dict[str, TaskUsage] = {}
        self._total_tokens: int = 0
        self._total_latency_ms: int = 0

    @property
    def config(self) -> BudgetConfig:
        return self._config

    # ── 使用量记录 ──

    def record_usage(
        self, task_id: str, tokens: int, latency_ms: int
    ) -> None:
        """记录任务使用量"""
        if task_id not in self._task_usage:
            self._task_usage[task_id] = TaskUsage()

        self._task_usage[task_id].tokens += tokens
        self._task_usage[task_id].latency_ms += latency_ms
        self._total_tokens += tokens
        self._total_latency_ms += latency_ms

    def get_task_usage(self, task_id: str) -> Optional[TaskUsage]:
        return self._task_usage.get(task_id)

    # ── 预算检查 ──

    def is_over_budget(self) -> bool:
        """全局是否超预算"""
        return (
            self._total_tokens >= self._config.global_token_limit
            or self._total_latency_ms >= self._config.global_latency_limit_ms
        )

    def is_warning(self) -> bool:
        """全局是否达到警告阈值"""
        threshold = self._config.warning_threshold
        token_warn = (
            self._total_tokens
            >= self._config.global_token_limit * threshold
        )
        latency_warn = (
            self._total_latency_ms
            >= self._config.global_latency_limit_ms * threshold
        )
        return token_warn or latency_warn

    def can_execute(self, cost_budget: int = 0, latency_budget: int = 0) -> bool:
        """
        检查是否还有预算执行一个新任务。

        Args:
            cost_budget: 任务预估 token 消耗
            latency_budget: 任务预估时延 (ms)

        Returns:
            True 如果全局预算允许
        """
        if self.is_over_budget():
            return False

        # 如果任务有预算声明，检查剩余空间是否足够
        if cost_budget > 0:
            remaining_tokens = (
                self._config.global_token_limit - self._total_tokens
            )
            if cost_budget > remaining_tokens:
                return False

        if latency_budget > 0:
            remaining_latency = (
                self._config.global_latency_limit_ms - self._total_latency_ms
            )
            if latency_budget > remaining_latency:
                return False

        return True

    # ── 状态查询 ──

    def get_status(self) -> BudgetStatus:
        """获取当前预算状态"""
        return BudgetStatus(
            tokens_used=self._total_tokens,
            tokens_limit=self._config.global_token_limit,
            latency_used_ms=self._total_latency_ms,
            latency_limit_ms=self._config.global_latency_limit_ms,
            over_budget=self.is_over_budget(),
            warning=self.is_warning(),
        )

    def suggest_downgrade(self) -> Optional[str]:
        """
        根据预算状态建议降级策略。

        Returns:
            降级建议字符串，或 None (预算充足)
        """
        if self.is_over_budget():
            return (
                f"OVER_BUDGET: 执行 {self._config.hard_limit_action} 策略 "
                f"(tokens: {self._total_tokens}/{self._config.global_token_limit}, "
                f"latency: {self._total_latency_ms}/{self._config.global_latency_limit_ms}ms)"
            )

        if self.is_warning():
            return (
                "WARNING: 预算即将耗尽，建议降级后端为 claude 以节省 token"
            )

        return None

    def get_remaining(self) -> Dict[str, int]:
        """获取剩余预算"""
        return {
            "tokens": max(
                0, self._config.global_token_limit - self._total_tokens
            ),
            "latency_ms": max(
                0,
                self._config.global_latency_limit_ms - self._total_latency_ms,
            ),
        }

    def reset(self) -> None:
        """重置所有使用量（用于测试或重新开始）"""
        self._task_usage.clear()
        self._total_tokens = 0
        self._total_latency_ms = 0
