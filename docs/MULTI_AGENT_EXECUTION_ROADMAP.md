# Multi-Agent Execution Roadmap

> Status: Active
> Role: post-GA execution source of truth for multi-agent maturation
> Scope: additive to the current Rust-primary control plane; not a v3.0 GA blocker

This document defines how Fusion should mature from the current agent-aware orchestration baseline into a true multi-agent execution system without giving up auditability, recovery, or contract stability.

If this document conflicts with the current GA release boundary, `docs/V3_GA_EXECUTION_ROADMAP.md` wins for GA decisions.

## Current position

The repository already has an agent spine, but it is not yet a full parallel worker system.

- `agents.enabled: false` remains the default production path
- `single_orchestrator` already computes a ready, non-conflicting batch from task metadata
- `role_handoff` already supports `planner -> coder -> reviewer` baton passing with a reviewer gate
- `fusion status --json` and runtime events already expose batch, role, review, and handoff summaries
- execution still selects one active task per `fusion-bridge codeagent` run; the current system is agent-aware orchestration, not full parallel multi-worker execution

## Recommended operating model

Fusion should support three modes with clear intent and promotion rules.

### Mode 1: `single_runner`

Use this as the default.

- One runner owns the current task and the current workspace
- This remains the safest path for small or ambiguous work
- Every advanced mode must be able to fall back here without data loss

### Mode 2: `role_handoff`

Use this for higher-risk single-task work.

- `planner` scopes and clarifies
- `coder` implements and updates tests
- `reviewer` approves or rejects with `Review-Status`
- This mode optimizes quality, traceability, and recovery, not throughput

### Mode 3: `parallel_workers`

Use this only for truly independent tasks.

- Each worker gets one leased `task_id`
- Each worker runs in an isolated workspace
- The orchestrator is the only component allowed to assign, cancel, or requeue work
- Reviewer or merge gates remain centralized

This is the only mode that should be called "multi-agent parallel execution" in live docs.

## Best-practice rules

These rules are the architectural guardrails for every agent mode.

1. The orchestrator is the only control plane.
2. `.fusion/task_plan.md` remains the canonical task contract.
3. Agents exchange structured state, not free-form negotiation transcripts.
4. Reviewer approval is a gate, not a suggestion, for gated tasks.
5. True parallel write work requires isolated workspaces; shared-workspace parallel editing is not a supported end state.
6. Every advanced mode must preserve auditable runtime state in `.fusion/events.jsonl` and `.fusion/sessions.json`.
7. Every advanced mode must preserve a hard fallback to the single-runner path.

## Non-goals

Fusion should explicitly avoid these anti-patterns.

- No open-ended swarm of peer agents negotiating by prompt alone
- No shared-workspace parallel coding with only best-effort conflict avoidance
- No GA requirement that multi-agent be enabled by default
- No new standalone explain command; policy remains on the existing status surface

## Execution phases

### Phase 0: Freeze the contract boundary

Goal: make current agent behavior explicit before expanding it.

- Keep `single_runner` as the default
- Keep `role_handoff` as the only promoted collaborative mode before worker isolation exists
- Align README, README.zh-CN, `PARALLEL_EXECUTION.md`, and active roadmap references to say that true parallel workers are post-GA
- Add tests that prevent live docs from claiming that current batch planning already means true parallel execution

Acceptance:

- live docs describe the same three-mode model
- repo/docs contract tests fail on stale or inflated claims

### Phase 1: Add worker isolation primitives

Goal: make parallel execution technically safe.

- Introduce worker lease state under `_runtime.agents.workers.*`
- Create isolated workspace semantics per worker, preferably via worktree-style directories
- Persist worker lifecycle events such as lease granted, worker started, worker heartbeat, worker cancelled, worker completed
- Keep one task per worker and one owner per worker

Acceptance:

- the orchestrator can allocate and reclaim leases deterministically
- a worker crash can be detected and requeued without corrupting the main workspace

### Phase 2: Ship `parallel_workers` MVP

Goal: run independent tasks concurrently under orchestrator control.

- Promote batch planning from selection-only to execution-aware leasing
- Allow multiple workers only when dependencies are satisfied and writes do not conflict
- Record per-worker workspace path, backend route, session identity, and lease timeout
- Keep `role_handoff` and reviewer gates available on top of worker outputs where needed

Acceptance:

- independent tasks complete concurrently with auditable results
- failed workers do not invalidate sibling workers
- blocked tasks are requeued with explicit reasons

### Phase 3: Centralize merge and review gates

Goal: prevent parallel throughput from degrading code quality.

- Add an orchestrator-controlled merge queue
- Require worker outputs to include test evidence and a patch or mergeable diff summary
- Route high-risk or policy-mandated tasks through reviewer approval before merge
- Keep final repository mutation under orchestrator control

Acceptance:

- worker completion does not imply repository integration
- review and merge decisions are visible in status and events

### Phase 4: Promote and narrow defaults

Goal: make multi-agent usable without making it surprising.

- keep `single_runner` as the global default unless evidence justifies a narrower default promotion
- optionally promote `role_handoff` by task policy for high-risk changes
- keep `parallel_workers` opt-in until conflict, recovery, and merge evidence are strong enough

Acceptance:

- promotion decisions are evidence-based
- fallback to single-runner remains fast and boring

## Implementation order

Implement these tracks in order:

1. Docs and contract convergence
2. Worker lease and workspace isolation
3. Parallel worker executor
4. Merge/review gates for worker outputs
5. Promotion criteria and operational guidance

## Evidence required before promotion

Do not promote `parallel_workers` in live defaults until the repository has all of the following:

- contract tests for worker lease, cancel, timeout, and requeue
- smoke coverage proving isolated workspaces are used
- status/reporting coverage for worker state and merge queue state
- recovery coverage showing that a dead worker does not strand the workflow
- docs that explain when to use `single_runner`, `role_handoff`, and `parallel_workers`

## Relationship to other docs

- `docs/V3_GA_EXECUTION_ROADMAP.md`: current GA release boundary
- `README.md` and `README.zh-CN.md`: live operator-facing summary
- `PARALLEL_EXECUTION.md`: parallel execution concepts and compatibility notes; if it conflicts with this roadmap, update it
- `docs/CLI_CONTRACT_MATRIX.md`: current command and status contract
