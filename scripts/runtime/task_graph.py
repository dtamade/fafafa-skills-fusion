"""
Fusion Runtime Task Graph — DAG 任务图编译器

从 task_plan.md 解析任务依赖，构建 DAG 并拓扑排序产出并行批次。
v2.5.0 Phase 2 核心组件。
"""

import re
from collections import deque
from dataclasses import dataclass, field
from typing import List, Optional, Dict, Set, Tuple


# ── 数据结构 ─────────────────────────────────────


@dataclass
class TaskNode:
    """DAG 中的任务节点"""
    task_id: str
    name: str
    status: str = "PENDING"   # PENDING / IN_PROGRESS / COMPLETED / FAILED
    task_type: str = "implementation"
    backend: str = "codex"
    dependencies: List[str] = field(default_factory=list)
    writeset: List[str] = field(default_factory=list)
    cost_budget: int = 0       # token 预算
    latency_budget: int = 0    # 毫秒


@dataclass
class Batch:
    """一组可并行执行的任务"""
    batch_id: int
    tasks: List[TaskNode] = field(default_factory=list)

    @property
    def task_ids(self) -> List[str]:
        return [t.task_id for t in self.tasks]


# ── TaskGraph ────────────────────────────────────


class TaskGraph:
    """DAG 任务图：拓扑排序、依赖验证、批次产出"""

    def __init__(self, nodes: Optional[List[TaskNode]] = None):
        self._nodes: Dict[str, TaskNode] = {}
        if nodes:
            for node in nodes:
                self._nodes[node.task_id] = node

    # ── 节点管理 ──

    def add_node(self, node: TaskNode) -> None:
        self._nodes[node.task_id] = node

    def get_node(self, task_id: str) -> Optional[TaskNode]:
        return self._nodes.get(task_id)

    @property
    def nodes(self) -> List[TaskNode]:
        return list(self._nodes.values())

    @property
    def node_count(self) -> int:
        return len(self._nodes)

    # ── 状态更新 ──

    def mark_completed(self, task_id: str) -> None:
        node = self._nodes.get(task_id)
        if node:
            node.status = "COMPLETED"

    def mark_failed(self, task_id: str) -> None:
        node = self._nodes.get(task_id)
        if node:
            node.status = "FAILED"

    def mark_in_progress(self, task_id: str) -> None:
        node = self._nodes.get(task_id)
        if node:
            node.status = "IN_PROGRESS"

    # ── 验证 ──

    def validate(self) -> List[str]:
        """
        验证 DAG 合法性。

        Returns:
            错误列表（空 = 合法）
        """
        errors: List[str] = []

        # 1. 检查悬空依赖
        all_ids = set(self._nodes.keys())
        for node in self._nodes.values():
            for dep in node.dependencies:
                if dep not in all_ids:
                    errors.append(
                        f"Task '{node.task_id}' depends on unknown task '{dep}'"
                    )

        # 2. 检查自依赖
        for node in self._nodes.values():
            if node.task_id in node.dependencies:
                errors.append(f"Task '{node.task_id}' depends on itself")

        # 3. 检查循环依赖 (Kahn 算法副产物)
        if not errors:
            in_degree = {nid: 0 for nid in self._nodes}
            for node in self._nodes.values():
                for dep in node.dependencies:
                    if dep in in_degree:
                        in_degree[node.task_id] += 1

            queue = deque(nid for nid, d in in_degree.items() if d == 0)
            visited = 0
            temp_degree = dict(in_degree)

            while queue:
                nid = queue.popleft()
                visited += 1
                # 减少依赖此节点的入度
                for node in self._nodes.values():
                    if nid in node.dependencies:
                        temp_degree[node.task_id] -= 1
                        if temp_degree[node.task_id] == 0:
                            queue.append(node.task_id)

            if visited < len(self._nodes):
                cycle_nodes = [
                    nid for nid, d in temp_degree.items() if d > 0
                ]
                errors.append(
                    f"Circular dependency detected among: {cycle_nodes}"
                )

        return errors

    # ── 拓扑排序 (Kahn 算法，按层级分批) ──

    def topological_sort(self) -> List[Batch]:
        """
        拓扑排序，返回按层级分组的批次列表。
        每个 Batch 内的任务可并行执行。

        Returns:
            List[Batch]: 批次列表（按依赖层级排序）

        Raises:
            ValueError: DAG 包含循环依赖
        """
        errors = self.validate()
        if errors:
            raise ValueError(f"Invalid DAG: {'; '.join(errors)}")

        if not self._nodes:
            return []

        # 计算入度
        in_degree: Dict[str, int] = {nid: 0 for nid in self._nodes}
        for node in self._nodes.values():
            for dep in node.dependencies:
                if dep in in_degree:
                    in_degree[node.task_id] += 1

        # BFS 按层级弹出
        batches: List[Batch] = []
        current_layer = [nid for nid, d in in_degree.items() if d == 0]

        batch_id = 1
        while current_layer:
            batch = Batch(
                batch_id=batch_id,
                tasks=[self._nodes[nid] for nid in sorted(current_layer)],
            )
            batches.append(batch)
            batch_id += 1

            next_layer: List[str] = []
            for nid in current_layer:
                for node in self._nodes.values():
                    if nid in node.dependencies:
                        in_degree[node.task_id] -= 1
                        if in_degree[node.task_id] == 0:
                            next_layer.append(node.task_id)

            current_layer = next_layer

        return batches

    # ── 运行时查询 ──

    def get_ready_tasks(self) -> List[TaskNode]:
        """获取所有依赖已满足且状态为 PENDING 的任务"""
        ready = []
        for node in self._nodes.values():
            if node.status != "PENDING":
                continue
            # 所有依赖都已完成？
            deps_met = all(
                self._nodes.get(dep) is not None
                and self._nodes[dep].status == "COMPLETED"
                for dep in node.dependencies
            )
            if deps_met:
                ready.append(node)
        return ready

    def get_completed_count(self) -> int:
        return sum(1 for n in self._nodes.values() if n.status == "COMPLETED")

    def get_pending_count(self) -> int:
        return sum(
            1 for n in self._nodes.values()
            if n.status in ("PENDING", "IN_PROGRESS")
        )

    def get_failed_count(self) -> int:
        return sum(1 for n in self._nodes.values() if n.status == "FAILED")

    def is_all_done(self) -> bool:
        """所有任务都已完成或失败"""
        return all(
            n.status in ("COMPLETED", "FAILED") for n in self._nodes.values()
        )

    def get_progress(self) -> Dict:
        return {
            "total": self.node_count,
            "completed": self.get_completed_count(),
            "pending": self.get_pending_count(),
            "failed": self.get_failed_count(),
        }

    # ── 解析 task_plan.md ──

    @classmethod
    def from_task_plan(cls, path: str) -> "TaskGraph":
        """
        从 task_plan.md 解析构建 TaskGraph。

        支持的格式:
            ### Task 1: 任务名 [STATUS]
            - Type: implementation
            - Backend: codex
            - Dependencies: [2, 3]
            - Writeset: [src/auth.py, src/db.py]
            - CostBudget: 5000
            - LatencyBudget: 60000
        """
        with open(path, "r", encoding="utf-8") as f:
            content = f.read()

        return cls.from_task_plan_content(content)

    @classmethod
    def from_task_plan_content(cls, content: str) -> "TaskGraph":
        """从 task_plan.md 文本内容解析构建 TaskGraph"""
        graph = cls()

        # 匹配任务头: ### Task N: 名称 [STATUS]
        task_pattern = re.compile(
            r"###\s+Task\s+(\d+):\s+(.+?)\s+\[(\w+)\]"
        )
        # 匹配属性行
        type_pattern = re.compile(r"-\s+Type:\s*(\S+)")
        backend_pattern = re.compile(r"-\s+Backend:\s*(\S+)")
        dep_pattern = re.compile(r"-\s+Dependencies:\s*\[([^\]]*)\]")
        writeset_pattern = re.compile(r"-\s+Writeset:\s*\[([^\]]*)\]")
        cost_pattern = re.compile(r"-\s+CostBudget:\s*(\d+)")
        latency_pattern = re.compile(r"-\s+LatencyBudget:\s*(\d+)")

        # 按任务分段
        lines = content.split("\n")
        current_node: Optional[TaskNode] = None

        for line in lines:
            task_match = task_pattern.match(line.strip())
            if task_match:
                # 保存前一个节点
                if current_node:
                    graph.add_node(current_node)

                task_num = task_match.group(1)
                task_name = task_match.group(2).strip()
                task_status = task_match.group(3)

                current_node = TaskNode(
                    task_id=task_num,
                    name=task_name,
                    status=task_status,
                )
                continue

            if current_node is None:
                continue

            # 解析属性
            m = type_pattern.match(line.strip())
            if m:
                current_node.task_type = m.group(1)
                continue

            m = backend_pattern.match(line.strip())
            if m:
                current_node.backend = m.group(1)
                continue

            m = dep_pattern.match(line.strip())
            if m:
                raw = m.group(1).strip()
                if raw:
                    current_node.dependencies = [
                        d.strip() for d in raw.split(",") if d.strip()
                    ]
                continue

            m = writeset_pattern.match(line.strip())
            if m:
                raw = m.group(1).strip()
                if raw:
                    current_node.writeset = [
                        w.strip() for w in raw.split(",") if w.strip()
                    ]
                continue

            m = cost_pattern.match(line.strip())
            if m:
                current_node.cost_budget = int(m.group(1))
                continue

            m = latency_pattern.match(line.strip())
            if m:
                current_node.latency_budget = int(m.group(1))
                continue

        # 最后一个节点
        if current_node:
            graph.add_node(current_node)

        return graph
