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
]
