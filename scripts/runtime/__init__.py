"""Fusion Runtime Package"""

from .state_machine import (
    State,
    Event,
    Transition,
    StateMachine,
    StateMachineContext,
    phase_to_state,
    state_to_phase,
)

from .event_bus import (
    EventBus,
    Subscription,
)

from .session_store import (
    SessionStore,
    StoredEvent,
)

from .kernel import (
    FusionKernel,
    KernelConfig,
    TransitionResult,
    create_kernel,
)

from .task_graph import (
    TaskNode,
    Batch,
    TaskGraph,
)

from .conflict_detector import (
    ConflictDetector,
    ConflictResult,
)

from .budget_manager import (
    BudgetManager,
    BudgetConfig,
    BudgetStatus,
)

from .router import (
    Router,
    RouteDecision,
)

from .scheduler import (
    Scheduler,
    SchedulerConfig,
    ScheduleDecision,
)

from .config import (
    load_raw_config,
    load_fusion_config,
)

from .safe_backlog import (
    generate_safe_backlog,
)

__version__ = "2.1.0"
__all__ = [
    # State Machine
    "State",
    "Event",
    "Transition",
    "StateMachine",
    "StateMachineContext",
    "phase_to_state",
    "state_to_phase",
    # Event Bus
    "EventBus",
    "Subscription",
    # Session Store
    "SessionStore",
    "StoredEvent",
    # Kernel
    "FusionKernel",
    "KernelConfig",
    "TransitionResult",
    "create_kernel",
    # Task Graph (v2.5.0)
    "TaskNode",
    "Batch",
    "TaskGraph",
    # Conflict Detector (v2.5.0)
    "ConflictDetector",
    "ConflictResult",
    # Budget Manager (v2.5.0)
    "BudgetManager",
    "BudgetConfig",
    "BudgetStatus",
    # Router (v2.5.0)
    "Router",
    "RouteDecision",
    # Scheduler (v2.5.0)
    "Scheduler",
    "SchedulerConfig",
    "ScheduleDecision",
    # Config
    "load_raw_config",
    "load_fusion_config",
    # Safe backlog
    "generate_safe_backlog",
]
