"""
Fusion Runtime Kernel

状态机执行器，负责状态转移、事件派发和状态持久化。
v2.1.0 Week 2: 接入 EventBus 和 SessionStore。
"""

import json
import time
import os
import tempfile
from pathlib import Path
from typing import Optional, Dict, Any, List, Callable
from dataclasses import dataclass, field, asdict

from .state_machine import (
    State, Event, StateMachine, StateMachineContext,
    Transition, phase_to_state, state_to_phase
)
from .event_bus import EventBus, Subscription
from .session_store import SessionStore, StoredEvent
from .task_graph import TaskGraph, TaskNode
from .scheduler import Scheduler, SchedulerConfig, ScheduleDecision
from .conflict_detector import ConflictDetector
from .budget_manager import BudgetManager, BudgetConfig
from .router import Router
from .config import load_fusion_config


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
        _recursion_depth: int = 0,
    ) -> TransitionResult:
        """
        派发事件，触发状态转移

        Args:
            event: 要派发的事件
            payload: 事件附带数据
            idempotency_key: 幂等键（可选），相同 key 的重复 dispatch 不产生副作用
            _recursion_depth: 内部参数，用于防止无限递归

        Returns:
            TransitionResult: 转移结果
        """
        # 防止递归错误处理导致无限循环
        MAX_RECURSION_DEPTH = 3
        if _recursion_depth >= MAX_RECURSION_DEPTH:
            return TransitionResult(
                success=False,
                from_state=self._current_state,
                to_state=self._current_state,
                event=event,
                error=f"Max recursion depth ({MAX_RECURSION_DEPTH}) exceeded in error handling"
            )

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
                # 递归深度 +1 防止无限循环
                self.dispatch(Event.ERROR_OCCURRED, {
                    "error": str(e),
                    "source_event": event.name
                }, _recursion_depth=_recursion_depth + 1)
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

    # ── v2.5.0 Scheduler 集成 ──────────────────────────

    def init_scheduler(
        self,
        scheduler_config: Optional[SchedulerConfig] = None,
        budget_config: Optional[BudgetConfig] = None,
        default_backend: str = "codex",
    ) -> Optional[Scheduler]:
        """
        从 task_plan.md 初始化 Scheduler。

        当 scheduler_config.enabled=False 时仍会创建 Scheduler，
        但 pick_next_batch() 会退化为串行模式。

        Returns:
            Scheduler 实例，或 None（task_plan.md 不存在时）
        """
        task_plan = self.fusion_dir / "task_plan.md"
        if not task_plan.exists():
            return None

        try:
            graph = TaskGraph.from_task_plan(str(task_plan))
        except Exception:
            return None

        config = scheduler_config or SchedulerConfig()
        budget = BudgetManager(budget_config) if budget_config else BudgetManager()
        router = Router(budget_manager=budget, default_backend=default_backend)
        conflict = ConflictDetector()

        self._scheduler = Scheduler(
            graph=graph,
            config=config,
            conflict_detector=conflict,
            budget_manager=budget,
            router=router,
        )

        # 更新上下文
        self._context.scheduler_enabled = config.enabled

        return self._scheduler

    @property
    def scheduler(self) -> Optional[Scheduler]:
        """当前 Scheduler 实例（可能为 None）"""
        return getattr(self, '_scheduler', None)

    def get_next_batch(self) -> Optional[ScheduleDecision]:
        """
        获取下一批可执行任务。

        委托 Scheduler.pick_next_batch()。
        如果 Scheduler 未初始化，返回 None。
        """
        sched = self.scheduler
        if sched is None:
            return None
        return sched.pick_next_batch()

    def _sync_scheduler_snapshot(self) -> None:
        """将 scheduler 状态同步到 sessions.json"""
        sched = self.scheduler
        if not sched:
            return
        try:
            scheduler_data = {
                "enabled": sched.config.enabled,
                "current_batch_id": sched.get_progress().get("batches_done", 0),
                "parallel_tasks": len(
                    sched.graph.get_ready_tasks()
                ) if sched.config.enabled else 0,
            }
            # 通过 extra 将 scheduler 数据合并到 _runtime
            sessions_file = self.fusion_dir / "sessions.json"
            if sessions_file.exists():
                import json as _json
                with open(sessions_file, "r", encoding="utf-8") as f:
                    data = _json.load(f)
                runtime = data.get("_runtime", {})
                runtime["scheduler"] = scheduler_data
                data["_runtime"] = runtime
                fd, tmp_path = tempfile.mkstemp(
                    dir=str(sessions_file.parent), suffix=".tmp"
                )
                try:
                    with os.fdopen(fd, "w", encoding="utf-8") as f:
                        _json.dump(data, f, indent=2, ensure_ascii=False)
                    os.replace(tmp_path, str(sessions_file))
                except BaseException:
                    os.unlink(tmp_path)
                    raise
        except (IOError, OSError, ValueError):
            pass  # 故障安全: 文件 I/O 或 JSON 解析失败不阻塞主流程

    def complete_task(
        self,
        task_id: str,
        tokens_used: int = 0,
        latency_ms: int = 0,
    ) -> None:
        """
        标记任务完成并更新 Scheduler 状态。

        同时更新 Context 的任务计数。
        """
        sched = self.scheduler
        if sched:
            sched.on_task_done(task_id, tokens_used, latency_ms)
            # 同步上下文
            progress = sched.get_progress()
            self._context.completed_tasks = progress["completed"]
            self._context.pending_tasks = progress["pending"]
            self._context.failed_tasks = progress["failed"]
            self._context.current_batch_id = progress.get("batches_done", 0)
            self._sync_scheduler_snapshot()

    def fail_task(
        self,
        task_id: str,
        tokens_used: int = 0,
        latency_ms: int = 0,
    ) -> None:
        """标记任务失败并更新 Scheduler 状态。"""
        sched = self.scheduler
        if sched:
            sched.on_task_failed(task_id, tokens_used, latency_ms)
            progress = sched.get_progress()
            self._context.completed_tasks = progress["completed"]
            self._context.pending_tasks = progress["pending"]
            self._context.failed_tasks = progress["failed"]
            self._sync_scheduler_snapshot()

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
        status = {
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

        # v2.5.0 调度器信息
        sched = self.scheduler
        if sched:
            status["scheduler"] = {
                "enabled": sched.config.enabled,
                "progress": sched.get_progress(),
            }

        return status


def create_kernel(fusion_dir: str = ".fusion") -> FusionKernel:
    """
    创建并初始化内核实例。

    行为：
    1. 加载 sessions 快照状态
    2. 按 config.yaml 自动初始化 scheduler/budget（若 task_plan.md 存在）
    """
    kernel = FusionKernel(fusion_dir=fusion_dir)
    kernel.load_state()

    # 自动接入 v2.5.0 调度器配置（故障安全，不阻塞调用方）
    try:
        cfg = load_fusion_config(fusion_dir)
        max_parallel = int(cfg.get("scheduler_max_parallel", 2))
        if max_parallel < 1:
            max_parallel = 1

        scheduler_config = SchedulerConfig(
            enabled=bool(cfg.get("scheduler_enabled", False)),
            max_parallel=max_parallel,
            fail_fast=bool(cfg.get("scheduler_fail_fast", False)),
        )
        budget_config = BudgetConfig(
            global_token_limit=int(cfg.get("budget_global_token_limit", 100_000)),
            global_latency_limit_ms=int(cfg.get("budget_global_latency_limit_ms", 7_200_000)),
            warning_threshold=float(cfg.get("budget_warning_threshold", 0.8)),
            hard_limit_action=str(cfg.get("budget_hard_limit_action", "serial")),
        )
        kernel.init_scheduler(
            scheduler_config=scheduler_config,
            budget_config=budget_config,
            default_backend=str(cfg.get("backend_primary", "codex")),
        )
    except Exception:
        pass

    return kernel
