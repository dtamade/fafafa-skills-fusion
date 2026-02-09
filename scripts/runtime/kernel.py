"""
Fusion Runtime Kernel

状态机执行器，负责状态转移、事件派发和状态持久化。
"""

import json
import time
import os
from pathlib import Path
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, field, asdict

from .state_machine import (
    State, Event, StateMachine, StateMachineContext,
    Transition, phase_to_state, state_to_phase
)


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
        self._current_state: State = State.IDLE
        self._context: StateMachineContext = StateMachineContext()
        self._event_counter: int = 0
        self._listeners: Dict[str, List[callable]] = {}

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
        payload: Optional[Dict[str, Any]] = None
    ) -> TransitionResult:
        """
        派发事件，触发状态转移

        Args:
            event: 要派发的事件
            payload: 事件附带数据

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
            self._emit_event("invalid_event", {
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

        # 2. 生成事件 ID
        event_id = self._generate_event_id()

        # 3. 执行转移动作 (如果有)
        if transition.action:
            try:
                transition.action(self._context)
            except Exception as e:
                # 动作失败，触发错误事件
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
                    event_id=event_id
                )

        # 4. 更新状态
        self._current_state = transition.to_state
        self._context.current_state = transition.to_state

        # 5. 持久化状态
        self._save_state()

        # 6. 记录事件
        self._append_event({
            "id": event_id,
            "type": event.name,
            "from_state": old_state.name,
            "to_state": transition.to_state.name,
            "payload": payload,
            "timestamp": time.time()
        })

        # 7. 发布状态变更事件
        self._emit_event("state_changed", {
            "from": old_state.name,
            "to": self._current_state.name,
            "event": event.name,
            "event_id": event_id
        })

        return TransitionResult(
            success=True,
            from_state=old_state,
            to_state=self._current_state,
            event=event,
            event_id=event_id
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
        """从 sessions.json 加载状态"""
        sessions_file = self.fusion_dir / "sessions.json"

        if not sessions_file.exists():
            self._current_state = State.IDLE
            return self._current_state

        try:
            with open(sessions_file, "r", encoding="utf-8") as f:
                data = json.load(f)

            # 读取 current_phase
            phase = data.get("current_phase", "IDLE")
            self._current_state = phase_to_state(phase)

            # 读取 runtime 扩展数据
            runtime_data = data.get("_runtime", {})
            self._event_counter = runtime_data.get("last_event_counter", 0)

            # 更新上下文
            self._context.current_state = self._current_state

            # 读取任务状态
            self._load_task_context()

        except (json.JSONDecodeError, IOError) as e:
            self._current_state = State.IDLE

        return self._current_state

    def _save_state(self) -> None:
        """保存状态到 sessions.json"""
        sessions_file = self.fusion_dir / "sessions.json"

        # 确保目录存在
        self.fusion_dir.mkdir(parents=True, exist_ok=True)

        # 读取现有数据
        data = {}
        if sessions_file.exists():
            try:
                with open(sessions_file, "r", encoding="utf-8") as f:
                    data = json.load(f)
            except (json.JSONDecodeError, IOError):
                pass

        # 更新状态
        data["current_phase"] = state_to_phase(self._current_state)

        # 更新 runtime 扩展
        data["_runtime"] = {
            "version": "2.1.0",
            "state": self._current_state.name,
            "last_event_counter": self._event_counter,
            "updated_at": time.time()
        }

        # 写入文件
        try:
            with open(sessions_file, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
        except IOError as e:
            self._emit_event("save_failed", {"error": str(e)})

    def _load_task_context(self) -> None:
        """从 task_plan.md 加载任务上下文"""
        task_plan = self.fusion_dir / "task_plan.md"

        if not task_plan.exists():
            return

        try:
            with open(task_plan, "r", encoding="utf-8") as f:
                content = f.read()

            # 统计任务状态
            self._context.completed_tasks = content.count("[COMPLETED]")
            self._context.pending_tasks = (
                content.count("[PENDING]") + content.count("[IN_PROGRESS]")
            )
            self._context.failed_tasks = content.count("[FAILED]")

        except IOError:
            pass

    def _generate_event_id(self) -> str:
        """生成事件 ID"""
        self._event_counter += 1
        return f"evt_{self._event_counter:06d}"

    def _append_event(self, event_data: Dict[str, Any]) -> None:
        """追加事件到日志"""
        events_file = self.fusion_dir / "events.jsonl"

        try:
            with open(events_file, "a", encoding="utf-8") as f:
                f.write(json.dumps(event_data, ensure_ascii=False) + "\n")
        except IOError:
            pass

    def _emit_event(self, event_type: str, data: Dict[str, Any]) -> None:
        """发布内部事件"""
        listeners = self._listeners.get(event_type, [])
        for listener in listeners:
            try:
                listener(data)
            except Exception:
                pass

    def on(self, event_type: str, listener: callable) -> None:
        """注册事件监听器"""
        if event_type not in self._listeners:
            self._listeners[event_type] = []
        self._listeners[event_type].append(listener)

    def off(self, event_type: str, listener: callable) -> None:
        """移除事件监听器"""
        if event_type in self._listeners:
            try:
                self._listeners[event_type].remove(listener)
            except ValueError:
                pass

    def reset(self) -> None:
        """重置内核状态"""
        self._current_state = State.IDLE
        self._context = StateMachineContext()
        self._event_counter = 0

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
                "event_counter": self._event_counter
            }
        }


def create_kernel(fusion_dir: str = ".fusion") -> FusionKernel:
    """创建并初始化内核实例"""
    kernel = FusionKernel(fusion_dir=fusion_dir)
    kernel.load_state()
    return kernel
