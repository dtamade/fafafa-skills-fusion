"""
Conflict Detector 文件冲突检测器单元测试
"""

import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.task_graph import TaskNode
from runtime.conflict_detector import ConflictDetector, ConflictResult


class TestHasConflict(unittest.TestCase):
    """两任务冲突检测"""

    def setUp(self):
        self.detector = ConflictDetector()

    def test_no_overlap(self):
        a = TaskNode(task_id="1", name="A", writeset=["src/a.py"])
        b = TaskNode(task_id="2", name="B", writeset=["src/b.py"])
        self.assertFalse(self.detector.has_conflict(a, b))

    def test_overlap(self):
        a = TaskNode(task_id="1", name="A", writeset=["src/a.py", "src/common.py"])
        b = TaskNode(task_id="2", name="B", writeset=["src/b.py", "src/common.py"])
        self.assertTrue(self.detector.has_conflict(a, b))

    def test_empty_writeset_no_conflict(self):
        a = TaskNode(task_id="1", name="A", writeset=[])
        b = TaskNode(task_id="2", name="B", writeset=["src/b.py"])
        self.assertFalse(self.detector.has_conflict(a, b))

    def test_both_empty_no_conflict(self):
        a = TaskNode(task_id="1", name="A")
        b = TaskNode(task_id="2", name="B")
        self.assertFalse(self.detector.has_conflict(a, b))


class TestGetConflictingFiles(unittest.TestCase):
    """冲突文件获取"""

    def setUp(self):
        self.detector = ConflictDetector()

    def test_returns_conflicting_files(self):
        a = TaskNode(task_id="1", name="A", writeset=["x.py", "shared.py"])
        b = TaskNode(task_id="2", name="B", writeset=["y.py", "shared.py"])
        files = self.detector.get_conflicting_files(a, b)
        self.assertEqual(files, ["shared.py"])

    def test_multiple_conflicts(self):
        a = TaskNode(task_id="1", name="A", writeset=["a.py", "b.py", "c.py"])
        b = TaskNode(task_id="2", name="B", writeset=["b.py", "c.py", "d.py"])
        files = self.detector.get_conflicting_files(a, b)
        self.assertEqual(files, ["b.py", "c.py"])

    def test_no_overlap_returns_empty(self):
        a = TaskNode(task_id="1", name="A", writeset=["a.py"])
        b = TaskNode(task_id="2", name="B", writeset=["b.py"])
        self.assertEqual(self.detector.get_conflicting_files(a, b), [])


class TestCheck(unittest.TestCase):
    """批量冲突检测 (贪心分区)"""

    def setUp(self):
        self.detector = ConflictDetector()

    def test_empty_list(self):
        result = self.detector.check([])
        self.assertEqual(result.safe_tasks, [])
        self.assertEqual(result.deferred_tasks, [])

    def test_single_task_always_safe(self):
        result = self.detector.check([
            TaskNode(task_id="1", name="A", writeset=["a.py"]),
        ])
        self.assertEqual(result.safe_tasks, ["1"])
        self.assertEqual(result.deferred_tasks, [])

    def test_no_conflict_all_safe(self):
        result = self.detector.check([
            TaskNode(task_id="1", name="A", writeset=["a.py"]),
            TaskNode(task_id="2", name="B", writeset=["b.py"]),
            TaskNode(task_id="3", name="C", writeset=["c.py"]),
        ])
        self.assertEqual(result.safe_tasks, ["1", "2", "3"])
        self.assertEqual(result.deferred_tasks, [])

    def test_pairwise_conflict_defers_later(self):
        """两任务冲突 → ID 小的安全，ID 大的推迟"""
        result = self.detector.check([
            TaskNode(task_id="1", name="A", writeset=["shared.py"]),
            TaskNode(task_id="2", name="B", writeset=["shared.py"]),
        ])
        self.assertEqual(result.safe_tasks, ["1"])
        self.assertEqual(result.deferred_tasks, ["2"])
        self.assertEqual(len(result.conflicts), 1)
        self.assertEqual(result.conflicts[0], ("1", "2", "shared.py"))

    def test_chain_conflict(self):
        """三任务链式冲突: A↔B 冲突，B 被推迟后 C 与已有安全集不冲突"""
        result = self.detector.check([
            TaskNode(task_id="1", name="A", writeset=["x.py"]),
            TaskNode(task_id="2", name="B", writeset=["x.py", "y.py"]),
            TaskNode(task_id="3", name="C", writeset=["y.py"]),
        ])
        # B 因与 A 共享 x.py 被推迟；C 的 y.py 不在安全集中（B 已被推迟）
        self.assertCountEqual(result.safe_tasks, ["1", "3"])
        self.assertEqual(result.deferred_tasks, ["2"])

    def test_partial_conflict(self):
        """部分冲突: A 和 C 不冲突，B 和 A 冲突"""
        result = self.detector.check([
            TaskNode(task_id="1", name="A", writeset=["a.py"]),
            TaskNode(task_id="2", name="B", writeset=["a.py", "b.py"]),
            TaskNode(task_id="3", name="C", writeset=["c.py"]),
        ])
        self.assertCountEqual(result.safe_tasks, ["1", "3"])
        self.assertEqual(result.deferred_tasks, ["2"])

    def test_empty_writeset_no_conflict(self):
        """无 writeset 的任务永远安全"""
        result = self.detector.check([
            TaskNode(task_id="1", name="A"),
            TaskNode(task_id="2", name="B"),
            TaskNode(task_id="3", name="C", writeset=["shared.py"]),
        ])
        self.assertEqual(result.safe_tasks, ["1", "2", "3"])
        self.assertEqual(result.deferred_tasks, [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
