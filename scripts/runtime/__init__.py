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
    # Kernel
    "FusionKernel",
    "KernelConfig",
    "TransitionResult",
    "create_kernel",
]
