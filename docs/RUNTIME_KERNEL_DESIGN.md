# Fusion Runtime Kernel v2.1.0 设计文档

> 状态: 设计阶段
> 版本: v2.1.0
> 目标: 把 8 阶段从 Prompt 约束升级为可执行 FSM

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
│                    compat_v2.py                          │
│              (v2 Shell 脚本适配层)                        │
└─────────────────────────────────────────────────────────┘
         ▲                    ▲                    ▲
         │                    │                    │
    ┌────┴────┐         ┌────┴────┐         ┌────┴────┐
    │ pretool │         │ posttool│         │stop-guard│
    │   .sh   │         │   .sh   │         │   .sh   │
    └─────────┘         └─────────┘         └─────────┘
```

## 2. 状态机定义 (state_machine.py)

### 2.1 状态枚举

```python
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

```python
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

```python
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

```python
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

## 3. 内核执行器 (kernel.py)

### 3.1 类定义

```python
class FusionKernel:
    """Fusion 运行时内核"""

    def __init__(
        self,
        fusion_dir: str = ".fusion",
        compat_mode: bool = True
    ):
        self.fusion_dir = fusion_dir
        self.compat_mode = compat_mode
        self.state_machine = StateMachine()
        self.session_store = SessionStore(fusion_dir)
        self.event_bus = EventBus()
        self._current_state: State = State.IDLE

    @property
    def current_state(self) -> State:
        """当前状态"""
        return self._current_state

    def dispatch(self, event: Event, payload: dict = None) -> bool:
        """
        派发事件，触发状态转移

        Returns:
            bool: 转移是否成功
        """
        pass

    def can_transition(self, event: Event) -> bool:
        """检查是否可以进行转移"""
        pass

    def get_valid_events(self) -> List[Event]:
        """获取当前状态可接受的事件列表"""
        pass

    def load_state(self) -> State:
        """从 sessions.json 加载状态"""
        pass

    def save_state(self) -> None:
        """保存状态到 sessions.json"""
        pass
```

### 3.2 核心算法

```python
def dispatch(self, event: Event, payload: dict = None) -> bool:
    """派发事件，触发状态转移"""

    # 1. 查找匹配的转移规则
    transition = self.state_machine.find_transition(
        self._current_state, event
    )

    if transition is None:
        self.event_bus.emit("invalid_event", {
            "state": self._current_state,
            "event": event
        })
        return False

    # 2. 检查守卫条件
    if transition.guard and not transition.guard():
        self.event_bus.emit("guard_failed", {
            "transition": transition
        })
        return False

    # 3. 记录事件 (事件溯源)
    event_id = self.session_store.append_event({
        "type": event.name,
        "from_state": self._current_state.name,
        "to_state": transition.to_state.name,
        "payload": payload,
        "timestamp": time.time()
    })

    # 4. 执行转移动作 (如果有)
    if transition.action:
        try:
            transition.action()
        except Exception as e:
            self.dispatch(Event.ERROR_OCCURRED, {"error": str(e)})
            return False

    # 5. 更新状态
    old_state = self._current_state
    self._current_state = transition.to_state

    # 6. 持久化状态
    self.save_state()

    # 7. 发布状态变更事件
    self.event_bus.emit("state_changed", {
        "from": old_state,
        "to": self._current_state,
        "event": event,
        "event_id": event_id
    })

    return True
```

## 4. 事件溯源存储 (session_store.py)

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

```python
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

### 5.2 Shell 脚本适配 (compat_v2.py)

```python
def adapt_stop_guard_call(context: dict) -> dict:
    """
    将 fusion-stop-guard.sh 的调用适配为 Kernel 事件

    Args:
        context: Shell 脚本传入的上下文

    Returns:
        dict: 处理结果
    """
    kernel = FusionKernel(compat_mode=True)
    kernel.load_state()

    # 根据上下文决定事件
    if context.get("task_completed"):
        kernel.dispatch(Event.TASK_DONE, context)
    elif context.get("all_tasks_done"):
        kernel.dispatch(Event.ALL_TASKS_DONE)

    return {
        "state": kernel.current_state.name,
        "should_continue": kernel.current_state == State.EXECUTE
    }
```

## 6. 测试要点

### 6.1 状态机测试

```python
class TestStateMachine(unittest.TestCase):

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

```python
class TestKernel(unittest.TestCase):

    def test_full_workflow(self):
        """测试完整工作流"""
        kernel = FusionKernel()
        assert kernel.dispatch(Event.START)
        assert kernel.current_state == State.INITIALIZE
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

```yaml
runtime:
  enabled: false          # 是否启用新内核 (默认关闭)
  compat_mode: true       # v2 兼容模式
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
- [ ] SessionStore 实现
- [ ] EventBus 实现
- [ ] Kernel 核心逻辑
- [ ] 事件溯源测试

### Week 3
- [ ] compat_v2 适配层
- [ ] Shell 脚本改造
- [ ] 兼容性测试

### Week 4
- [ ] 性能优化
- [ ] 全量回归
- [ ] 文档完善
- [ ] v2.1.0 发布
