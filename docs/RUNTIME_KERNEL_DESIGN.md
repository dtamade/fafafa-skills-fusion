# Fusion Runtime Kernel v2.1.0 设计文档

> 状态: 设计阶段（历史设计稿）
> 版本: v2.1.0
> 目标: 把 8 阶段从 Prompt 约束升级为可执行 FSM
> 说明：本文描述的是历史 runtime kernel 的设计基线，不代表当前控制面入口都仍由该实现承担；当前控制面与 hook 契约请以 Rust bridge 和实时脚本行为为准。
> 现状补充：仓库中的旧 reference helper 已移除。下文保留的 `_FusionKernel`、`_dispatch()`、`_load_state()` 等符号仅用于说明当时的历史设计草案，不对应当前仓库中的可执行实现或测试夹具。

## 1. 架构概览

```
┌─────────────────────────────────────────────────────────┐
│                    Fusion Runtime                        │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ Kernel      │  │ StateMachine│  │ SessionStore    │  │
│  │ (执行器)    │◄─│ (状态定义)  │  │ (事件溯源)      │  │
│  └──────┬──────┘  └─────────────┘  └────────┬────────┘  │
│         │                                    │           │
│         ▼                                    ▼           │
│  ┌─────────────┐                    ┌─────────────────┐  │
│  │ EventBus    │                    │ .fusion/        │  │
│  │ (事件总线)  │                    │ sessions.json   │  │
│  └─────────────┘                    │ events.jsonl    │  │
│                                     └─────────────────┘  │
├─────────────────────────────────────────────────────────┤
│     [historical] legacy compat adapter (removed from repo)│
│   (former legacy parity/reference layer for hook path)   │
└─────────────────────────────────────────────────────────┘
         ▲                    ▲                    ▲
         │                    │                    │
    ┌────┴────┐         ┌────┴────┐         ┌────┴────┐
    │ pretool │         │ posttool│         │stop-guard│
    │   .sh   │         │   .sh   │         │   .sh   │
    └─────────┘         └─────────┘         └─────────┘
```

## 2. 状态机定义（历史设计草案）

### 2.1 状态枚举

```text
from enum import Enum, auto

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
```

### 2.2 事件枚举

> 历史口径说明：本小节展示的是 v2.1.0 设计草案里的事件全集。相关 legacy reference helper 已从仓库删除；未实际使用的历史事件面（如 `UNDERSTAND_DONE`、`TIMEOUT`、`LOOP_DETECTED`）也不属于当前 live/runtime 契约。

```text
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

    # 系统事件
    ERROR_OCCURRED = auto() # 发生错误
    TIMEOUT = auto()        # 超时
    LOOP_DETECTED = auto()  # 检测到循环
```

### 2.3 转移规则

```text
@dataclass
class Transition:
    """状态转移规则"""
    from_state: State
    event: Event
    to_state: State
    guard: Optional[Callable[[], bool]] = None  # 守卫条件
    action: Optional[Callable[[], None]] = None  # 转移动作

# 转移表
TRANSITIONS: List[Transition] = [
    # 正常流程
    Transition(State.IDLE, Event.START, State.INITIALIZE),
    Transition(State.INITIALIZE, Event.INIT_DONE, State.ANALYZE),
    Transition(State.ANALYZE, Event.ANALYZE_DONE, State.DECOMPOSE),
    Transition(State.DECOMPOSE, Event.DECOMPOSE_DONE, State.EXECUTE),
    Transition(State.EXECUTE, Event.TASK_DONE, State.EXECUTE),  # 继续执行
    Transition(State.EXECUTE, Event.ALL_TASKS_DONE, State.VERIFY),
    Transition(State.VERIFY, Event.VERIFY_PASS, State.REVIEW),
    Transition(State.VERIFY, Event.VERIFY_FAIL, State.EXECUTE),  # 回退修复
    Transition(State.REVIEW, Event.REVIEW_PASS, State.COMMIT),
    Transition(State.REVIEW, Event.REVIEW_FAIL, State.EXECUTE),  # 回退修复
    Transition(State.COMMIT, Event.COMMIT_DONE, State.DELIVER),
    Transition(State.DELIVER, Event.TASK_DONE, State.COMPLETED),

    # 暂停/恢复
    Transition(State.EXECUTE, Event.PAUSE, State.PAUSED),
    Transition(State.PAUSED, Event.RESUME, State.EXECUTE),

    # 取消 (从任何状态)
    # ... 动态注册

    # 错误处理 (从任何状态)
    # ... 动态注册
]
```

### 2.4 守卫条件示例

```text
def guard_can_commit() -> bool:
    """检查是否可以提交"""
    # 所有任务完成
    # 测试通过
    # 审查通过
    return all_tasks_completed() and tests_passed() and review_passed()

def guard_has_pending_tasks() -> bool:
    """检查是否有待执行任务"""
    return get_pending_task_count() > 0
```

## 3. 内核执行器（历史设计草案）

> 现状注记：本节保留设计/参考用途。对应的 legacy helper 已从仓库移除，不通过 `runtime` 包导出，也不是公开 CLI/API。
> 下方代码块只用于解释结构，符号名已对齐当前仓库私有命名，避免与历史公开草案混淆。

### 3.1 类定义

```text
class _FusionKernel:
    """Fusion 运行时内核"""

    def __init__(
        self,
        fusion_dir: str = ".fusion",
    ):
        self.fusion_dir = fusion_dir
        self._state_machine = StateMachine()
        self._persistence = SessionStore(fusion_dir)
        self._event_bus = EventBus()
        self._current_state: State = State.IDLE

    def _dispatch(self, event: Event, payload: dict = None) -> _TransitionResult:
        """
        派发事件，触发状态转移

        Returns:
            _TransitionResult: 转移结果
        """
        pass

    def _can_transition(self, event: Event) -> bool:
        """检查是否可以进行转移"""
        pass

    def _get_valid_events(self) -> List[Event]:
        """获取当前状态可接受的事件列表"""
        pass

    def _load_state(self) -> State:
        """从 sessions.json 加载状态"""
        pass

    def _sync_runtime_snapshot(self) -> None:
        """同步状态到 sessions.json"""
        pass
```

### 3.2 核心算法

```text
def _dispatch(self, event: Event, payload: dict = None) -> _TransitionResult:
    """派发事件，触发状态转移"""

    # 1. 查找匹配的转移规则
    transition = self._state_machine.find_transition(
        self._current_state, event
    )

    if transition is None:
        self._event_bus.emit("invalid_event", {
            "state": self._current_state,
            "event": event
        })
        return _TransitionResult(success=False, ...)

    # 2. 检查守卫条件
    if transition.guard and not transition.guard():
        self._event_bus.emit("guard_failed", {
            "transition": transition
        })
        return _TransitionResult(success=False, ...)

    # 3. 记录事件 (事件溯源)
    stored = self._append_runtime_event(
        event_type=event.name,
        from_state=self._current_state.name,
        to_state=transition.to_state.name,
        payload=payload,
    )
    event_id = stored.id if stored else None

    # 4. 执行转移动作 (如果有)
    if transition.action:
        try:
            transition.action()
        except Exception as e:
            self._dispatch(Event.ERROR_OCCURRED, {"error": str(e)})
            return _TransitionResult(success=False, ...)

    # 5. 更新状态
    old_state = self._current_state
    self._current_state = transition.to_state

    # 6. 持久化状态
    self._sync_runtime_snapshot(self._current_state)

    # 7. 发布状态变更事件
    self._event_bus.emit("state_changed", {
        "from": old_state,
        "to": self._current_state,
        "event": event,
        "event_id": event_id
    })

    return _TransitionResult(success=True, event_id=event_id, ...)
```

## 4. 事件溯源存储（历史设计草案）

### 4.1 事件格式

```json
{
  "id": "evt_001",
  "idempotency_key": "unique_key_for_dedup",
  "type": "TASK_DONE",
  "from_state": "EXECUTE",
  "to_state": "EXECUTE",
  "payload": {
    "task_id": "task_003",
    "result": "success"
  },
  "timestamp": 1707465600.123
}
```

### 4.2 存储结构

```
.fusion/
├── sessions.json      # 当前状态快照
├── events.jsonl       # 事件日志 (append-only)
└── .state.lock        # 状态锁
```

### 4.3 幂等性保证

```text
def append_event(self, event: dict, idempotency_key: str = None) -> str:
    """追加事件，支持幂等重试"""

    # 生成或使用提供的幂等键
    key = idempotency_key or self._generate_idempotency_key(event)

    # 检查是否已处理
    if self._is_event_processed(key):
        return self._get_existing_event_id(key)

    # 追加事件
    event_id = self._write_event(event, key)

    return event_id
```

## 5. 与现有系统集成

### 5.1 sessions.json 扩展

```json
{
  "goal": "实现用户认证",
  "status": "in_progress",
  "current_phase": "EXECUTE",

  "_runtime": {
    "version": "2.1.0",
    "state": "EXECUTE",
    "last_event_id": "evt_042",
    "iteration": 15
  }
}
```

### 5.2 历史对照适配层（历史设计项，legacy compat adapter 已移除）

> 下述片段仅保留为历史思路示例。仓库中已不存在该适配层，也不存在任何面向 hook live path 的旧适配 API。

```text
def adapt_stop_guard_call(context: dict) -> dict:
    """
    将 fusion-stop-guard.sh 的调用适配为 Kernel 事件

    Args:
        context: Shell 脚本传入的上下文

    Returns:
        dict: 处理结果
    """
    kernel = _FusionKernel()
    kernel._load_state()

    # 根据上下文决定事件
    if context.get("task_completed"):
        kernel._dispatch(Event.TASK_DONE, context)
    elif context.get("all_tasks_done"):
        kernel._dispatch(Event.ALL_TASKS_DONE)

    return {
        "state": kernel._current_state.name,
        "should_continue": kernel._current_state == State.EXECUTE
    }
```

## 6. 测试要点

### 6.1 状态机测试

```text
class TestStateMachine(HistoricalTestCase):

    def test_valid_transitions(self):
        """测试有效的状态转移"""
        sm = StateMachine()
        assert sm.can_transition(State.IDLE, Event.START)
        assert sm.can_transition(State.EXECUTE, Event.TASK_DONE)

    def test_invalid_transitions(self):
        """测试无效的状态转移"""
        sm = StateMachine()
        assert not sm.can_transition(State.IDLE, Event.TASK_DONE)
        assert not sm.can_transition(State.COMPLETED, Event.START)

    def test_guard_conditions(self):
        """测试守卫条件"""
        pass
```

### 6.2 内核测试

```text
class TestKernel(HistoricalTestCase):

    def test_full_workflow(self):
        """测试完整工作流"""
        kernel = _FusionKernel()
        assert kernel._dispatch(Event.START).success
        assert kernel._current_state == State.INITIALIZE
        # ...

    def test_resume_from_crash(self):
        """测试崩溃恢复"""
        # 模拟崩溃
        # 重新加载状态
        # 验证可以继续
        pass

    def test_idempotent_events(self):
        """测试事件幂等性"""
        pass
```

## 7. 配置扩展 (config.yaml)

> 历史口径说明：下面这段 `runtime.enabled` / `runtime.compat_mode` 配置块用于保留当时设计假设，不是当前主控制面契约定义。当前 live 行为请以 [docs/HOOKS_SETUP.md](./HOOKS_SETUP.md) 和 Rust bridge 实现为准。

```yaml
runtime:
  enabled: false          # 是否启用当时设计中的新内核 (历史默认)
  compat_mode: true       # v2 兼容模式（历史设计口径）
  event_store:
    max_events: 1000      # 最大事件数
    retention_days: 7     # 事件保留天数
  timeouts:
    state_lock_ms: 5000   # 状态锁超时
    transition_ms: 30000  # 转移超时
```

## 8. 实现计划

### Week 1
- [ ] State/Event 枚举定义
- [ ] Transition 数据结构
- [ ] 基础 StateMachine 类
- [ ] 单元测试框架

### Week 2
- [ ] 私有 SessionStore 实现
- [ ] EventBus 实现
- [ ] Kernel 核心逻辑
- [ ] 事件溯源测试

### Week 3
- [ ] 历史兼容适配层（历史项；仓库现状已删除）
- [ ] Shell 脚本改造
- [ ] 兼容性测试

### Week 4
- [ ] 性能优化
- [ ] 全量回归
- [ ] 文档完善
- [ ] v2.1.0 发布
