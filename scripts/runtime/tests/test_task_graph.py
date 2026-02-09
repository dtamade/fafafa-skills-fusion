"""
Task Graph DAG 任务图编译器单元测试

覆盖拓扑排序、循环检测、依赖验证、task_plan.md 解析。
"""

import unittest
import tempfile
import os
from pathlib import Path

import sys
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.task_graph import TaskNode, Batch, TaskGraph


class TestTaskNode(unittest.TestCase):
    """TaskNode 数据结构"""

    def test_default_values(self):
        node = TaskNode(task_id="1", name="测试任务")
        self.assertEqual(node.status, "PENDING")
        self.assertEqual(node.task_type, "implementation")
        self.assertEqual(node.backend, "codex")
        self.assertEqual(node.dependencies, [])
        self.assertEqual(node.writeset, [])
        self.assertEqual(node.cost_budget, 0)
        self.assertEqual(node.latency_budget, 0)


class TestBatch(unittest.TestCase):
    """Batch 数据结构"""

    def test_task_ids(self):
        batch = Batch(
            batch_id=1,
            tasks=[
                TaskNode(task_id="1", name="A"),
                TaskNode(task_id="2", name="B"),
            ],
        )
        self.assertEqual(batch.task_ids, ["1", "2"])

    def test_empty_batch(self):
        batch = Batch(batch_id=1)
        self.assertEqual(batch.task_ids, [])


class TestTaskGraphBasic(unittest.TestCase):
    """TaskGraph 基本操作"""

    def test_add_and_get_node(self):
        graph = TaskGraph()
        node = TaskNode(task_id="1", name="A")
        graph.add_node(node)
        self.assertEqual(graph.get_node("1"), node)
        self.assertIsNone(graph.get_node("2"))

    def test_node_count(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
        ])
        self.assertEqual(graph.node_count, 2)

    def test_empty_graph(self):
        graph = TaskGraph()
        self.assertEqual(graph.node_count, 0)
        self.assertEqual(graph.topological_sort(), [])


class TestTaskGraphValidation(unittest.TestCase):
    """DAG 验证"""

    def test_valid_graph(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        self.assertEqual(graph.validate(), [])

    def test_unknown_dependency(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A", dependencies=["99"]),
        ])
        errors = graph.validate()
        self.assertEqual(len(errors), 1)
        self.assertIn("unknown task '99'", errors[0])

    def test_self_dependency(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A", dependencies=["1"]),
        ])
        errors = graph.validate()
        self.assertTrue(any("depends on itself" in e for e in errors))

    def test_circular_dependency(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A", dependencies=["2"]),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        errors = graph.validate()
        self.assertTrue(any("Circular dependency" in e for e in errors))

    def test_three_node_cycle(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A", dependencies=["3"]),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
            TaskNode(task_id="3", name="C", dependencies=["2"]),
        ])
        errors = graph.validate()
        self.assertTrue(any("Circular dependency" in e for e in errors))


    def test_duplicate_dependency(self):
        """重复依赖应当被去重，不影响拓扑排序"""
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1", "1"]),
        ])
        self.assertEqual(graph.validate(), [])
        batches = graph.topological_sort()
        self.assertEqual(len(batches), 2)
        self.assertEqual(batches[0].task_ids, ["1"])
        self.assertEqual(batches[1].task_ids, ["2"])


class TestTopologicalSort(unittest.TestCase):
    """拓扑排序与批次产出"""

    def test_linear_chain(self):
        """线性链: A → B → C"""
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
            TaskNode(task_id="3", name="C", dependencies=["2"]),
        ])
        batches = graph.topological_sort()
        self.assertEqual(len(batches), 3)
        self.assertEqual(batches[0].task_ids, ["1"])
        self.assertEqual(batches[1].task_ids, ["2"])
        self.assertEqual(batches[2].task_ids, ["3"])

    def test_diamond_dependency(self):
        """菱形: A → B,C → D"""
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
            TaskNode(task_id="3", name="C", dependencies=["1"]),
            TaskNode(task_id="4", name="D", dependencies=["2", "3"]),
        ])
        batches = graph.topological_sort()
        self.assertEqual(len(batches), 3)
        self.assertEqual(batches[0].task_ids, ["1"])
        self.assertCountEqual(batches[1].task_ids, ["2", "3"])
        self.assertEqual(batches[2].task_ids, ["4"])

    def test_all_independent(self):
        """全部独立 → 单批次"""
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
            TaskNode(task_id="3", name="C"),
        ])
        batches = graph.topological_sort()
        self.assertEqual(len(batches), 1)
        self.assertEqual(len(batches[0].tasks), 3)

    def test_complex_dag(self):
        """复杂 DAG: 多层依赖"""
        graph = TaskGraph([
            TaskNode(task_id="1", name="api_design"),
            TaskNode(task_id="2", name="db_schema", dependencies=["1"]),
            TaskNode(task_id="3", name="auth_types", dependencies=["1"]),
            TaskNode(task_id="4", name="auth_impl", dependencies=["2", "3"]),
            TaskNode(task_id="5", name="tests", dependencies=["4"]),
        ])
        batches = graph.topological_sort()
        self.assertEqual(len(batches), 4)
        self.assertEqual(batches[0].task_ids, ["1"])
        self.assertCountEqual(batches[1].task_ids, ["2", "3"])
        self.assertEqual(batches[2].task_ids, ["4"])
        self.assertEqual(batches[3].task_ids, ["5"])

    def test_circular_raises(self):
        """循环依赖抛出 ValueError"""
        graph = TaskGraph([
            TaskNode(task_id="1", name="A", dependencies=["2"]),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        with self.assertRaises(ValueError) as ctx:
            graph.topological_sort()
        self.assertIn("Circular dependency", str(ctx.exception))

    def test_batch_ids_sequential(self):
        """批次 ID 从 1 开始递增"""
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        batches = graph.topological_sort()
        self.assertEqual([b.batch_id for b in batches], [1, 2])


class TestGetReadyTasks(unittest.TestCase):
    """运行时就绪任务查询"""

    def test_all_pending_no_deps(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
        ])
        ready = graph.get_ready_tasks()
        self.assertEqual(len(ready), 2)

    def test_deps_not_met(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        ready = graph.get_ready_tasks()
        self.assertEqual(len(ready), 1)
        self.assertEqual(ready[0].task_id, "1")

    def test_deps_met_after_completion(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        graph.mark_completed("1")
        ready = graph.get_ready_tasks()
        self.assertEqual(len(ready), 1)
        self.assertEqual(ready[0].task_id, "2")

    def test_completed_not_ready(self):
        """已完成的任务不再出现在 ready 中"""
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
        ])
        graph.mark_completed("1")
        self.assertEqual(graph.get_ready_tasks(), [])

    def test_failed_dep_blocks(self):
        """失败的依赖阻塞下游"""
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B", dependencies=["1"]),
        ])
        graph.mark_failed("1")
        ready = graph.get_ready_tasks()
        self.assertEqual(len(ready), 0)


class TestProgressTracking(unittest.TestCase):
    """进度追踪"""

    def test_initial_progress(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
        ])
        p = graph.get_progress()
        self.assertEqual(p["total"], 2)
        self.assertEqual(p["completed"], 0)
        self.assertEqual(p["pending"], 2)

    def test_partial_progress(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
            TaskNode(task_id="3", name="C"),
        ])
        graph.mark_completed("1")
        graph.mark_failed("2")
        p = graph.get_progress()
        self.assertEqual(p["completed"], 1)
        self.assertEqual(p["failed"], 1)
        self.assertEqual(p["pending"], 1)

    def test_is_all_done(self):
        graph = TaskGraph([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
        ])
        self.assertFalse(graph.is_all_done())
        graph.mark_completed("1")
        self.assertFalse(graph.is_all_done())
        graph.mark_completed("2")
        self.assertTrue(graph.is_all_done())


class TestFromTaskPlan(unittest.TestCase):
    """从 task_plan.md 解析"""

    def test_parse_basic(self):
        content = (
            "## Tasks\n\n"
            "### Task 1: 创建登录API [COMPLETED]\n"
            "- Type: implementation\n"
            "- Backend: codex\n"
            "- Dependencies: []\n"
            "\n"
            "### Task 2: 添加JWT验证 [PENDING]\n"
            "- Type: implementation\n"
            "- Backend: codex\n"
            "- Dependencies: [1]\n"
        )
        graph = TaskGraph.from_task_plan_content(content)
        self.assertEqual(graph.node_count, 2)
        self.assertEqual(graph.get_node("1").status, "COMPLETED")
        self.assertEqual(graph.get_node("2").dependencies, ["1"])

    def test_parse_writeset_and_budget(self):
        content = (
            "### Task 1: 实现模块 [PENDING]\n"
            "- Type: implementation\n"
            "- Dependencies: []\n"
            "- Writeset: [src/auth.py, src/db.py]\n"
            "- CostBudget: 5000\n"
            "- LatencyBudget: 60000\n"
        )
        graph = TaskGraph.from_task_plan_content(content)
        node = graph.get_node("1")
        self.assertIsNotNone(node)
        self.assertEqual(node.writeset, ["src/auth.py", "src/db.py"])
        self.assertEqual(node.cost_budget, 5000)
        self.assertEqual(node.latency_budget, 60000)

    def test_parse_from_file(self):
        content = (
            "### Task 1: A [PENDING]\n"
            "- Type: design\n"
            "- Dependencies: []\n"
            "\n"
            "### Task 2: B [PENDING]\n"
            "- Type: implementation\n"
            "- Dependencies: [1]\n"
        )
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".md", delete=False
        ) as f:
            f.write(content)
            f.flush()
            try:
                graph = TaskGraph.from_task_plan(f.name)
                self.assertEqual(graph.node_count, 2)
            finally:
                os.unlink(f.name)

    def test_parse_empty_dependencies(self):
        content = "### Task 1: A [PENDING]\n- Dependencies: []\n"
        graph = TaskGraph.from_task_plan_content(content)
        self.assertEqual(graph.get_node("1").dependencies, [])

    def test_parse_special_chars_in_name(self):
        """任务名含中文、括号、冒号等特殊字符"""
        content = (
            "### Task 1: 实现用户认证(JWT+OAuth) [PENDING]\n"
            "- Type: implementation\n"
            "- Dependencies: []\n"
            "\n"
            "### Task 2: 配置: 数据库连接池 [IN_PROGRESS]\n"
            "- Type: configuration\n"
            "- Dependencies: [1]\n"
        )
        graph = TaskGraph.from_task_plan_content(content)
        self.assertEqual(graph.node_count, 2)
        self.assertIn("JWT+OAuth", graph.get_node("1").name)
        self.assertEqual(graph.get_node("2").status, "IN_PROGRESS")

    def test_parse_loose_dependency_format(self):
        """Dependencies 列表中有多余空格、不一致格式"""
        content = (
            "### Task 1: A [PENDING]\n"
            "- Dependencies: []\n"
            "\n"
            "### Task 2: B [PENDING]\n"
            "- Dependencies: []\n"
            "\n"
            "### Task 3: C [PENDING]\n"
            "- Dependencies: [ 1 , 2 ]\n"
        )
        graph = TaskGraph.from_task_plan_content(content)
        self.assertEqual(graph.get_node("3").dependencies, ["1", "2"])

    def test_parse_missing_optional_fields(self):
        """缺少可选字段时保持默认值"""
        content = (
            "### Task 1: 最小任务 [PENDING]\n"
            "- Dependencies: []\n"
        )
        graph = TaskGraph.from_task_plan_content(content)
        node = graph.get_node("1")
        self.assertEqual(node.writeset, [])
        self.assertEqual(node.cost_budget, 0)
        self.assertEqual(node.latency_budget, 0)
        self.assertEqual(node.task_type, "implementation")
        self.assertEqual(node.backend, "codex")

    def test_parse_mixed_statuses(self):
        """多种 Status 混合：COMPLETED / IN_PROGRESS / FAILED / PENDING"""
        content = (
            "### Task 1: A [COMPLETED]\n"
            "- Dependencies: []\n"
            "\n"
            "### Task 2: B [IN_PROGRESS]\n"
            "- Dependencies: [1]\n"
            "\n"
            "### Task 3: C [FAILED]\n"
            "- Dependencies: [1]\n"
            "\n"
            "### Task 4: D [PENDING]\n"
            "- Dependencies: [2, 3]\n"
        )
        graph = TaskGraph.from_task_plan_content(content)
        self.assertEqual(graph.node_count, 4)
        self.assertEqual(graph.get_node("1").status, "COMPLETED")
        self.assertEqual(graph.get_node("2").status, "IN_PROGRESS")
        self.assertEqual(graph.get_node("3").status, "FAILED")
        self.assertEqual(graph.get_node("4").status, "PENDING")
        self.assertEqual(graph.get_completed_count(), 1)
        self.assertEqual(graph.get_failed_count(), 1)


    def test_parse_name_with_brackets(self):
        """P1: 任务名含中括号不应误解析状态"""
        content = (
            "### Task 1: Fix [Auth] module [PENDING]\n"
            "- Type: implementation\n"
            "- Dependencies: []\n"
        )
        graph = TaskGraph.from_task_plan_content(content)
        node = graph.get_node("1")
        self.assertIsNotNone(node)
        self.assertEqual(node.status, "PENDING")
        self.assertIn("[Auth]", node.name)

    def test_parse_quoted_dependencies(self):
        """P2: Dependencies 含引号应去引号"""
        content = (
            '### Task 1: A [PENDING]\n'
            '- Dependencies: []\n'
            '\n'
            '### Task 2: B [PENDING]\n'
            '- Dependencies: ["1"]\n'
        )
        graph = TaskGraph.from_task_plan_content(content)
        self.assertEqual(graph.get_node("2").dependencies, ["1"])

    def test_parse_none_input_raises(self):
        """P1: None 输入应抛 TypeError"""
        with self.assertRaises(TypeError):
            TaskGraph.from_task_plan_content(None)

    def test_tasknode_none_dependencies(self):
        """P1: dependencies=None 归一化为 []"""
        node = TaskNode(task_id="1", name="A", dependencies=None)
        self.assertEqual(node.dependencies, [])

    def test_tasknode_none_writeset(self):
        """P1: writeset=None 归一化为 []"""
        node = TaskNode(task_id="1", name="A", writeset=None)
        self.assertEqual(node.writeset, [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
