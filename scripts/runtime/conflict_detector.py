"""
Fusion Runtime Conflict Detector — 文件冲突检测

检测并行候选任务之间的 writeset 冲突，
输出可安全并行的子集和因冲突推迟的任务。
v2.5.0 Phase 2 组件。
"""

from dataclasses import dataclass, field
from typing import List, Tuple, Set

from .task_graph import TaskNode, _task_sort_key


@dataclass
class ConflictResult:
    """冲突检测结果"""
    safe_tasks: List[str] = field(default_factory=list)
    deferred_tasks: List[str] = field(default_factory=list)
    conflicts: List[Tuple[str, str, str]] = field(default_factory=list)


class ConflictDetector:
    """文件冲突检测器：基于 writeset 交集判定"""

    def has_conflict(self, task_a: TaskNode, task_b: TaskNode) -> bool:
        """两个任务的 writeset 是否有交集"""
        if not task_a.writeset or not task_b.writeset:
            return False
        return bool(set(task_a.writeset) & set(task_b.writeset))

    def get_conflicting_files(
        self, task_a: TaskNode, task_b: TaskNode
    ) -> List[str]:
        """获取两个任务冲突的文件列表"""
        if not task_a.writeset or not task_b.writeset:
            return []
        return sorted(set(task_a.writeset) & set(task_b.writeset))

    def check(self, tasks: List[TaskNode]) -> ConflictResult:
        """
        检测一组候选并行任务之间的冲突。

        使用贪心策略：按任务 ID 排序，依次尝试加入安全集合。
        如果新任务与已有安全任务冲突，则推迟。

        Args:
            tasks: 候选并行任务列表

        Returns:
            ConflictResult: 安全子集 + 推迟列表 + 冲突详情
        """
        if not tasks:
            return ConflictResult()

        result = ConflictResult()
        safe_nodes: List[TaskNode] = []
        occupied_files: Set[str] = set()

        # 按 task_id 排序，保证确定性
        sorted_tasks = sorted(tasks, key=lambda t: _task_sort_key(t.task_id))

        for task in sorted_tasks:
            task_files = set(task.writeset) if task.writeset else set()

            # 检查是否与已占用文件冲突
            overlap = task_files & occupied_files
            if overlap:
                result.deferred_tasks.append(task.task_id)
                # 记录具体冲突对
                for safe in safe_nodes:
                    conflicting = self.get_conflicting_files(task, safe)
                    for f in conflicting:
                        result.conflicts.append(
                            (safe.task_id, task.task_id, f)
                        )
            else:
                result.safe_tasks.append(task.task_id)
                safe_nodes.append(task)
                occupied_files |= task_files

        return result
