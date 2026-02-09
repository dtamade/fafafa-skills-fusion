"""
Fusion Runtime Kernel

状态机执行器，负责状态转移、事件派发和状态持久化。
v2.1.0 Week 2: 接入 EventBus 和 SessionStore。
"""

import json
import time
import os
from pathlib import Path
from typing import Optional, Dict, Any, List, Callable
from dataclasses import dataclass, field, asdict

from .state_machine import (
    State, Event, StateMachine, StateMachineContext,
    Transition, phase_to_state, state_to_phase
)
from .event_bus import EventBus, Subscription
from .session_store import SessionStore, StoredEvent


@dataclass
class TransitionResult:
    """状态转移结果"""
    success: bool
    from_state: State
    to_state: State
    event: Event
    error: Optional[str] = None
    event_id: Optional[str] = None


@dataclass
class KernelConfig:
    """内核配置"""
    enabled: bool = False
    compat_mode: bool = True
    state_lock_timeout_ms: int = 5000
    transition_timeout_ms: int = 30000
    max_events: int = 1000


class FusionKernel:
    """Fusion 运行时内核"""

    def __init__(
        self,
        fusion_dir: str = ".fusion",
        config: Optional[KernelConfig] = None
    ):
        self.fusion_dir = Path(fusion_dir)
        self.config = config or KernelConfig()
        self.state_machine = StateMachine()
        self.event_bus = EventBus()
        self.session_store = SessionStore(fusion_dir=str(self.fusion_dir))
        self._current_state: State = State.IDLE
        self._context: StateMachineContext = StateMachineContext()

    @property
    def current_state(self) -> State:
        """当前状态"""
        return self._current_state

    @property
    def context(self) -> StateMachineContext:
        """当前上下文"""
        return self._context

    def dispatch(
        self,
        event: Event,
        payload: Optional[Dict[str, Any]] = None,
        idempotency_key: Optional[str] = None,
    ) -> TransitionResult:
        """
        派发事件，触发状态转移

        Args:
            event: 要派发的事件
            payload: 事件附带数据
            idempotency_key: 幂等键（可选），相同 key 的重复 dispatch 不产生副作用

        Returns:
            TransitionResult: 转移结果
        """
        old_state = self._current_state

        # 更新上下文
        if payload:
            self._context.payload = payload

        # 1. 查找匹配的转移规则
        transition = self.state_machine.find_transition(
            self._current_state, event, self._context
        )

        if transition is None:
            self.event_bus.emit("invalid_event", {
                "state": self._current_state.name,
                "event": event.name
            })
            return TransitionResult(
                success=False,
                from_state=old_state,
                to_state=old_state,
                event=event,
                error=f"No valid transition from {old_state.name} on {event.name}"
            )

        # 2. 执行转移动作 (如果有)
        if transition.action:
            try:
                transition.action(self._context)
            except Exception as e:
                error_result = self.dispatch(Event.ERROR_OCCURRED, {
                    "error": str(e),
                    "source_event": event.name
                })
                return TransitionResult(
                    success=False,
                    from_state=old_state,
                    to_state=self._current_state,
                    event=event,
                    error=str(e),
                )

        # 3. 更新状态
        self._current_state = transition.to_state
        self._context.current_state = transition.to_state

        # 4. 通过 SessionStore 记录事件（幂等写入）
        stored = self.session_store.append_event(
            event_type=event.name,
            from_state=old_state.name,
            to_state=transition.to_state.name,
            payload=payload,
            idempotency_key=idempotency_key,
        )
        event_id = stored.id if stored else None

        # 5. 通过 SessionStore 同步快照
        self.session_store.sync_snapshot(self._current_state)

        # 6. 通过 EventBus 发布状态变更事件
        self.event_bus.emit("state_changed", {
            "from": old_state.name,
            "to": self._current_state.name,
            "event": event.name,
            "event_id": event_id,
        })

        return TransitionResult(
            success=True,
            from_state=old_state,
            to_state=self._current_state,
            event=event,
            event_id=event_id,
        )

    def can_transition(self, event: Event) -> bool:
        """检查是否可以进行转移"""
        return self.state_machine.can_transition(
            self._current_state, event, self._context
        )

    def get_valid_events(self) -> List[Event]:
        """获取当前状态可接受的事件列表"""
        return self.state_machine.get_valid_events(
            self._current_state, self._context
        )

    def load_state(self) -> State:
        """从 sessions.json 快照加载状态"""
        snapshot = self.session_store.load_snapshot()

        if not snapshot:
            self._current_state = State.IDLE
            return self._current_state

        try:
            phase = snapshot.get("current_phase", "IDLE")
            self._current_state = phase_to_state(phase)

            # 从快照恢复 SessionStore 的事件计数器
            runtime_data = snapshot.get("_runtime", {})
            self.session_store._event_counter = runtime_data.get("last_event_counter", 0)

            # 更新上下文
            self._context.current_state = self._current_state

            # 读取任务状态
            self._load_task_context()

        except Exception:
            self._current_state = State.IDLE

        return self._current_state

    def load_state_from_events(self) -> State:
        """
        从事件流重放恢复状态（完整恢复）

        比 load_state() 更可靠：不依赖 sessions.json 快照，
        直接从 events.jsonl 重建状态。
        """
        self._current_state = State.IDLE
        self._context = StateMachineContext()

        def apply_event(evt: StoredEvent):
            self._current_state = phase_to_state(evt.to_state)
            self._context.current_state = self._current_state

        replayed = self.session_store.replay(apply_fn=apply_event)

        # 读取任务上下文
        self._load_task_context()

        return self._current_state

    def resume_from_events(self, from_event_id: Optional[str] = None) -> State:
        """
        从指定事件之后恢复（增量恢复）

        先通过快照快速定位，再从 from_event_id 之后重放缺失事件。

        Args:
            from_event_id: 已处理到的最后一个事件 ID

        Returns:
            恢复后的状态
        """
        # 先加载快照作为基础
        self.load_state()

        # 再增量重放
        def apply_event(evt: StoredEvent):
            self._current_state = phase_to_state(evt.to_state)
            self._context.current_state = self._current_state

        self.session_store.replay(
            apply_fn=apply_event,
            from_event_id=from_event_id,
        )

        self._load_task_context()
        return self._current_state

    def _load_task_context(self) -> None:
        """从 task_plan.md 加载任务上下文"""
        task_plan = self.fusion_dir / "task_plan.md"

        if not task_plan.exists():
            return

        try:
            with open(task_plan, "r", encoding="utf-8") as f:
                content = f.read()

            self._context.completed_tasks = content.count("[COMPLETED]")
            self._context.pending_tasks = (
                content.count("[PENDING]") + content.count("[IN_PROGRESS]")
            )
            self._context.failed_tasks = content.count("[FAILED]")

        except IOError:
            pass

    # ── v2 兼容 API ──────────────────────────────────────
    # Week 1 的 on()/off() 接收 listener(data)，
    # 而 EventBus 的回调签名是 callback(event_type, data)。
    # 这里做适配包装，保持向后兼容。

    def on(self, event_type: str, listener: Callable) -> None:
        """注册事件监听器（v2 兼容：listener 接收 data 单参数）"""
        def wrapper(evt_type: str, data: Dict[str, Any]) -> None:
            listener(data)
        # 存储映射关系以便 off() 能正确移除
        if not hasattr(self, '_listener_wrappers'):
            self._listener_wrappers: Dict[Callable, Callable] = {}
        self._listener_wrappers[listener] = wrapper
        self.event_bus.on(event_type, wrapper)

    def off(self, event_type: str, listener: Callable) -> None:
        """移除事件监听器（v2 兼容）"""
        wrappers = getattr(self, '_listener_wrappers', {})
        wrapper = wrappers.pop(listener, None)
        if wrapper:
            self.event_bus.off(event_type, wrapper)

    def reset(self) -> None:
        """重置内核状态"""
        self._current_state = State.IDLE
        self._context = StateMachineContext()
        self.session_store.truncate()
        self.event_bus.clear()

    def get_status(self) -> Dict[str, Any]:
        """获取内核状态摘要"""
        return {
            "state": self._current_state.name,
            "phase": state_to_phase(self._current_state),
            "valid_events": [e.name for e in self.get_valid_events()],
            "context": {
                "pending_tasks": self._context.pending_tasks,
                "completed_tasks": self._context.completed_tasks,
                "failed_tasks": self._context.failed_tasks
            },
            "runtime": {
                "version": "2.1.0",
                "event_counter": self.session_store._event_counter
            }
        }


def create_kernel(fusion_dir: str = ".fusion") -> FusionKernel:
    """创建并初始化内核实例"""
    kernel = FusionKernel(fusion_dir=fusion_dir)
    kernel.load_state()
    return kernel
