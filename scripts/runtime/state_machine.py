"""
Fusion Runtime State Machine

定义 Fusion 工作流的状态、事件和转移规则。
"""

from enum import Enum, auto
from dataclasses import dataclass, field
from typing import Optional, Callable, List, Dict, Any


class State(Enum):
    """Fusion 工作流状态"""
    # 主流程状态 (对应 8 阶段)
    IDLE = auto()           # 空闲/未初始化
    INITIALIZE = auto()     # 初始化工作流
    ANALYZE = auto()        # 分析需求和代码库
    DECOMPOSE = auto()      # 任务分解
    EXECUTE = auto()        # 执行任务
    VERIFY = auto()         # 验证结果
    REVIEW = auto()         # 代码审查
    COMMIT = auto()         # 提交代码
    DELIVER = auto()        # 交付完成

    # 辅助状态
    PAUSED = auto()         # 暂停
    ERROR = auto()          # 错误
    CANCELLED = auto()      # 取消
    COMPLETED = auto()      # 完成


class Event(Enum):
    """触发状态转移的事件"""
    # 用户操作
    START = auto()          # /fusion 启动
    PAUSE = auto()          # /fusion pause
    RESUME = auto()         # /fusion resume
    CANCEL = auto()         # /fusion cancel

    # 阶段完成事件
    INIT_DONE = auto()      # 初始化完成
    ANALYZE_DONE = auto()   # 分析完成
    DECOMPOSE_DONE = auto() # 分解完成
    TASK_DONE = auto()      # 单个任务完成
    ALL_TASKS_DONE = auto() # 所有任务完成
    VERIFY_PASS = auto()    # 验证通过
    VERIFY_FAIL = auto()    # 验证失败
    REVIEW_PASS = auto()    # 审查通过
    REVIEW_FAIL = auto()    # 审查失败
    COMMIT_DONE = auto()    # 提交完成
    DELIVER_DONE = auto()   # 交付完成

    # 系统事件
    ERROR_OCCURRED = auto() # 发生错误
    TIMEOUT = auto()        # 超时
    LOOP_DETECTED = auto()  # 检测到循环
    RECOVER = auto()        # 错误恢复


@dataclass
class Transition:
    """状态转移规则"""
    from_state: State
    event: Event
    to_state: State
    guard: Optional[Callable[['StateMachineContext'], bool]] = None
    action: Optional[Callable[['StateMachineContext'], None]] = None
    description: str = ""


@dataclass
class StateMachineContext:
    """状态机上下文，用于守卫条件和动作"""
    current_state: State = State.IDLE
    payload: Dict[str, Any] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)

    # 任务相关
    pending_tasks: int = 0
    completed_tasks: int = 0
    failed_tasks: int = 0

    # 验证/审查结果
    tests_passed: bool = False
    review_passed: bool = False

    # v2.5.0 调度器相关
    scheduler_enabled: bool = False
    current_batch_id: int = 0
    parallel_tasks: int = 0
    total_batches: int = 0

    def has_pending_tasks(self) -> bool:
        return self.pending_tasks > 0

    def all_tasks_done(self) -> bool:
        return self.pending_tasks == 0 and self.completed_tasks > 0


class StateMachine:
    """Fusion 状态机"""

    def __init__(self):
        self._transitions: List[Transition] = []
        self._register_transitions()

    def _register_transitions(self):
        """注册所有转移规则"""
        # ============ 正常流程 ============
        self._add(State.IDLE, Event.START, State.INITIALIZE,
                  description="启动工作流")

        self._add(State.INITIALIZE, Event.INIT_DONE, State.ANALYZE,
                  description="初始化完成，进入分析阶段")

        self._add(State.ANALYZE, Event.ANALYZE_DONE, State.DECOMPOSE,
                  description="分析完成，进入任务分解")

        self._add(State.DECOMPOSE, Event.DECOMPOSE_DONE, State.EXECUTE,
                  description="分解完成，开始执行任务")

        # 执行阶段 - 任务循环
        self._add(State.EXECUTE, Event.TASK_DONE, State.EXECUTE,
                  guard=lambda ctx: ctx.has_pending_tasks(),
                  description="任务完成，继续执行下一个")

        self._add(State.EXECUTE, Event.ALL_TASKS_DONE, State.VERIFY,
                  guard=lambda ctx: ctx.all_tasks_done(),
                  description="所有任务完成，进入验证")

        # 验证阶段
        self._add(State.VERIFY, Event.VERIFY_PASS, State.REVIEW,
                  description="验证通过，进入审查")

        self._add(State.VERIFY, Event.VERIFY_FAIL, State.EXECUTE,
                  description="验证失败，回退修复")

        # 审查阶段
        self._add(State.REVIEW, Event.REVIEW_PASS, State.COMMIT,
                  description="审查通过，进入提交")

        self._add(State.REVIEW, Event.REVIEW_FAIL, State.EXECUTE,
                  description="审查失败，回退修复")

        # 提交和交付
        self._add(State.COMMIT, Event.COMMIT_DONE, State.DELIVER,
                  description="提交完成，进入交付")

        self._add(State.DELIVER, Event.DELIVER_DONE, State.COMPLETED,
                  description="交付完成，工作流结束")

        # ============ 暂停/恢复 ============
        # 可暂停的状态
        pausable_states = [
            State.ANALYZE, State.DECOMPOSE, State.EXECUTE,
            State.VERIFY, State.REVIEW
        ]
        for state in pausable_states:
            self._add(state, Event.PAUSE, State.PAUSED,
                      description=f"从 {state.name} 暂停")

        self._add(State.PAUSED, Event.RESUME, State.EXECUTE,
                  description="恢复执行")

        # ============ 取消 (从任何活跃状态) ============
        active_states = [
            State.INITIALIZE, State.ANALYZE, State.DECOMPOSE,
            State.EXECUTE, State.VERIFY, State.REVIEW, State.COMMIT,
            State.DELIVER, State.PAUSED
        ]
        for state in active_states:
            self._add(state, Event.CANCEL, State.CANCELLED,
                      description=f"取消工作流 (from {state.name})")

        # ============ 错误处理 ============
        # 从任何非终态进入错误状态
        non_terminal_states = [
            State.IDLE, State.INITIALIZE, State.ANALYZE, State.DECOMPOSE,
            State.EXECUTE, State.VERIFY, State.REVIEW, State.COMMIT,
            State.DELIVER, State.PAUSED
        ]
        for state in non_terminal_states:
            self._add(state, Event.ERROR_OCCURRED, State.ERROR,
                      description=f"错误发生 (from {state.name})")

        # 错误恢复
        self._add(State.ERROR, Event.RECOVER, State.EXECUTE,
                  description="从错误恢复")

        # 循环检测
        self._add(State.EXECUTE, Event.LOOP_DETECTED, State.PAUSED,
                  description="检测到循环，自动暂停")

    def _add(
        self,
        from_state: State,
        event: Event,
        to_state: State,
        guard: Optional[Callable[[StateMachineContext], bool]] = None,
        action: Optional[Callable[[StateMachineContext], None]] = None,
        description: str = ""
    ):
        """添加转移规则"""
        self._transitions.append(Transition(
            from_state=from_state,
            event=event,
            to_state=to_state,
            guard=guard,
            action=action,
            description=description
        ))

    def find_transition(
        self,
        current_state: State,
        event: Event,
        context: Optional[StateMachineContext] = None
    ) -> Optional[Transition]:
        """
        查找匹配的转移规则

        Args:
            current_state: 当前状态
            event: 触发事件
            context: 状态机上下文 (用于守卫条件)

        Returns:
            匹配的转移规则，或 None
        """
        ctx = context or StateMachineContext(current_state=current_state)

        for t in self._transitions:
            if t.from_state != current_state:
                continue
            if t.event != event:
                continue
            # 检查守卫条件
            if t.guard is not None:
                try:
                    if not t.guard(ctx):
                        continue
                except Exception:
                    continue
            return t

        return None

    def can_transition(
        self,
        current_state: State,
        event: Event,
        context: Optional[StateMachineContext] = None
    ) -> bool:
        """检查是否可以进行转移"""
        return self.find_transition(current_state, event, context) is not None

    def get_valid_events(
        self,
        current_state: State,
        context: Optional[StateMachineContext] = None
    ) -> List[Event]:
        """获取当前状态可接受的事件列表"""
        ctx = context or StateMachineContext(current_state=current_state)
        valid = []

        for t in self._transitions:
            if t.from_state != current_state:
                continue
            # 检查守卫条件
            if t.guard is not None:
                try:
                    if not t.guard(ctx):
                        continue
                except Exception:
                    continue
            if t.event not in valid:
                valid.append(t.event)

        return valid

    def get_all_transitions(self) -> List[Transition]:
        """获取所有转移规则"""
        return self._transitions.copy()

    def get_transition_count(self) -> int:
        """获取转移规则数量"""
        return len(self._transitions)


# 状态到 sessions.json 的映射
STATE_TO_PHASE: Dict[State, str] = {
    State.IDLE: "IDLE",
    State.INITIALIZE: "INITIALIZE",
    State.ANALYZE: "ANALYZE",
    State.DECOMPOSE: "DECOMPOSE",
    State.EXECUTE: "EXECUTE",
    State.VERIFY: "VERIFY",
    State.REVIEW: "REVIEW",
    State.COMMIT: "COMMIT",
    State.DELIVER: "DELIVER",
    State.PAUSED: "PAUSED",
    State.ERROR: "ERROR",
    State.CANCELLED: "CANCELLED",
    State.COMPLETED: "COMPLETED",
}

PHASE_TO_STATE: Dict[str, State] = {v: k for k, v in STATE_TO_PHASE.items()}


def phase_to_state(phase: str) -> State:
    """将 sessions.json 的 current_phase 转换为 State"""
    return PHASE_TO_STATE.get(phase.upper(), State.IDLE)


def state_to_phase(state: State) -> str:
    """将 State 转换为 sessions.json 的 current_phase"""
    return STATE_TO_PHASE.get(state, "IDLE")
