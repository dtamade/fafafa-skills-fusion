"""
State Machine Unit Tests
"""

import unittest
import sys
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.state_machine import (
    State, Event, StateMachine, StateMachineContext,
    phase_to_state, state_to_phase
)


class TestStateEnum(unittest.TestCase):
    """测试 State 枚举"""

    def test_main_states_exist(self):
        """主流程状态存在"""
        main_states = [
            State.IDLE, State.INITIALIZE, State.ANALYZE,
            State.DECOMPOSE, State.EXECUTE, State.VERIFY,
            State.REVIEW, State.COMMIT, State.DELIVER
        ]
        self.assertEqual(len(main_states), 9)

    def test_auxiliary_states_exist(self):
        """辅助状态存在"""
        aux_states = [
            State.PAUSED, State.ERROR, State.CANCELLED, State.COMPLETED
        ]
        self.assertEqual(len(aux_states), 4)


class TestEventEnum(unittest.TestCase):
    """测试 Event 枚举"""

    def test_user_events_exist(self):
        """用户操作事件存在"""
        user_events = [
            Event.START, Event.PAUSE, Event.RESUME, Event.CANCEL
        ]
        self.assertEqual(len(user_events), 4)

    def test_phase_events_exist(self):
        """阶段完成事件存在"""
        phase_events = [
            Event.INIT_DONE, Event.ANALYZE_DONE, Event.DECOMPOSE_DONE,
            Event.TASK_DONE, Event.ALL_TASKS_DONE,
            Event.VERIFY_PASS, Event.VERIFY_FAIL,
            Event.REVIEW_PASS, Event.REVIEW_FAIL,
            Event.COMMIT_DONE, Event.DELIVER_DONE
        ]
        self.assertEqual(len(phase_events), 11)


class TestStateMachine(unittest.TestCase):
    """测试 StateMachine 类"""

    def setUp(self):
        self.sm = StateMachine()

    def test_transitions_registered(self):
        """转移规则已注册"""
        count = self.sm.get_transition_count()
        self.assertGreater(count, 20)  # 至少有 20+ 条转移规则

    def test_valid_start_transition(self):
        """IDLE + START -> UNDERSTAND (Phase 0)"""
        can = self.sm.can_transition(State.IDLE, Event.START)
        self.assertTrue(can)

        t = self.sm.find_transition(State.IDLE, Event.START)
        self.assertIsNotNone(t)
        self.assertEqual(t.to_state, State.UNDERSTAND)

    def test_understand_to_initialize(self):
        """UNDERSTAND + CONFIRM -> INITIALIZE"""
        can = self.sm.can_transition(State.UNDERSTAND, Event.CONFIRM)
        self.assertTrue(can)

        t = self.sm.find_transition(State.UNDERSTAND, Event.CONFIRM)
        self.assertEqual(t.to_state, State.INITIALIZE)

    def test_skip_understand(self):
        """IDLE + SKIP_UNDERSTAND -> INITIALIZE (--force)"""
        can = self.sm.can_transition(State.IDLE, Event.SKIP_UNDERSTAND)
        self.assertTrue(can)

        t = self.sm.find_transition(State.IDLE, Event.SKIP_UNDERSTAND)
        self.assertEqual(t.to_state, State.INITIALIZE)

    def test_valid_init_done_transition(self):
        """INITIALIZE + INIT_DONE -> ANALYZE"""
        can = self.sm.can_transition(State.INITIALIZE, Event.INIT_DONE)
        self.assertTrue(can)

        t = self.sm.find_transition(State.INITIALIZE, Event.INIT_DONE)
        self.assertEqual(t.to_state, State.ANALYZE)

    def test_invalid_transition(self):
        """无效转移返回 None"""
        # IDLE 不能直接到 TASK_DONE
        can = self.sm.can_transition(State.IDLE, Event.TASK_DONE)
        self.assertFalse(can)

        t = self.sm.find_transition(State.IDLE, Event.TASK_DONE)
        self.assertIsNone(t)

    def test_cancel_from_any_active_state(self):
        """从任何活跃状态可以取消"""
        active_states = [
            State.INITIALIZE, State.ANALYZE, State.DECOMPOSE,
            State.EXECUTE, State.VERIFY, State.REVIEW
        ]
        for state in active_states:
            can = self.sm.can_transition(state, Event.CANCEL)
            self.assertTrue(can, f"Should be able to cancel from {state.name}")

    def test_error_from_any_non_terminal_state(self):
        """从任何非终态可以进入错误状态"""
        non_terminal = [
            State.IDLE, State.INITIALIZE, State.EXECUTE, State.PAUSED
        ]
        for state in non_terminal:
            can = self.sm.can_transition(state, Event.ERROR_OCCURRED)
            self.assertTrue(can, f"Should handle error from {state.name}")

    def test_pause_from_execute(self):
        """EXECUTE + PAUSE -> PAUSED"""
        can = self.sm.can_transition(State.EXECUTE, Event.PAUSE)
        self.assertTrue(can)

    def test_resume_from_paused(self):
        """PAUSED + RESUME -> EXECUTE"""
        can = self.sm.can_transition(State.PAUSED, Event.RESUME)
        self.assertTrue(can)

    def test_guard_condition_task_done(self):
        """TASK_DONE 守卫条件测试"""
        # 有待执行任务时，继续执行
        ctx = StateMachineContext(
            current_state=State.EXECUTE,
            pending_tasks=3,
            completed_tasks=2
        )
        t = self.sm.find_transition(State.EXECUTE, Event.TASK_DONE, ctx)
        self.assertIsNotNone(t)
        self.assertEqual(t.to_state, State.EXECUTE)

    def test_guard_condition_all_tasks_done(self):
        """ALL_TASKS_DONE 守卫条件测试"""
        # 所有任务完成时，进入验证
        ctx = StateMachineContext(
            current_state=State.EXECUTE,
            pending_tasks=0,
            completed_tasks=5
        )
        t = self.sm.find_transition(State.EXECUTE, Event.ALL_TASKS_DONE, ctx)
        self.assertIsNotNone(t)
        self.assertEqual(t.to_state, State.VERIFY)

    def test_get_valid_events_idle(self):
        """IDLE 状态的有效事件"""
        events = self.sm.get_valid_events(State.IDLE)
        self.assertIn(Event.START, events)
        self.assertIn(Event.ERROR_OCCURRED, events)
        self.assertNotIn(Event.TASK_DONE, events)

    def test_get_valid_events_execute(self):
        """EXECUTE 状态的有效事件"""
        events = self.sm.get_valid_events(State.EXECUTE)
        self.assertIn(Event.PAUSE, events)
        self.assertIn(Event.CANCEL, events)
        self.assertIn(Event.ERROR_OCCURRED, events)


class TestPhaseMapping(unittest.TestCase):
    """测试状态与 phase 的映射"""

    def test_state_to_phase(self):
        """State 转 phase"""
        self.assertEqual(state_to_phase(State.IDLE), "IDLE")
        self.assertEqual(state_to_phase(State.EXECUTE), "EXECUTE")
        self.assertEqual(state_to_phase(State.COMPLETED), "COMPLETED")

    def test_phase_to_state(self):
        """phase 转 State"""
        self.assertEqual(phase_to_state("IDLE"), State.IDLE)
        self.assertEqual(phase_to_state("EXECUTE"), State.EXECUTE)
        self.assertEqual(phase_to_state("execute"), State.EXECUTE)  # 大小写不敏感

    def test_unknown_phase(self):
        """未知 phase 返回 IDLE"""
        self.assertEqual(phase_to_state("UNKNOWN"), State.IDLE)


class TestStateMachineContext(unittest.TestCase):
    """测试 StateMachineContext"""

    def test_has_pending_tasks(self):
        ctx = StateMachineContext(pending_tasks=3)
        self.assertTrue(ctx.has_pending_tasks())

        ctx = StateMachineContext(pending_tasks=0)
        self.assertFalse(ctx.has_pending_tasks())

    def test_all_tasks_done(self):
        ctx = StateMachineContext(pending_tasks=0, completed_tasks=5)
        self.assertTrue(ctx.all_tasks_done())

        ctx = StateMachineContext(pending_tasks=2, completed_tasks=3)
        self.assertFalse(ctx.all_tasks_done())

        ctx = StateMachineContext(pending_tasks=0, completed_tasks=0)
        self.assertFalse(ctx.all_tasks_done())


if __name__ == "__main__":
    unittest.main(verbosity=2)
