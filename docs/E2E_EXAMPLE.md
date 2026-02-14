# Fusion E2E Example

This example shows a full `/fusion` run with the default routing strategy:

- Codex: planning/analysis/review
- Claude: implementation/verification execution

## Scenario

```bash
/fusion "add email verification for signup"
```

## 1) UNDERSTAND summary

```text
Goal: add email verification for signup
Context: FastAPI + PostgreSQL + pytest
Scope: model change + API update + tests + docs
Assumptions: token via signed URL, 24h expiry
Decision: score=8 >= 7, continue to INITIALIZE
```

## 2) DECOMPOSE output (task types)

```text
Task 1 [implementation] add verification_token fields
Task 2 [implementation] add POST /auth/verify-email
Task 3 [verification] add API tests for token flow
Task 4 [documentation] update signup flow docs
```

## 3) EXECUTE routing decisions

`fusion-codeagent.sh` emits route logs:

```text
[fusion] route: phase=EXECUTE task_type=implementation -> claude (fallback=codex, reason=task_type:implementation)
[fusion] route: phase=EXECUTE task_type=verification -> claude (fallback=codex, reason=task_type:verification)
[fusion] route: phase=EXECUTE task_type=documentation -> claude (fallback=codex, reason=task_type:documentation)
```

## 4) REVIEW routing decision

```text
[fusion] route: phase=REVIEW task_type=unknown -> codex (fallback=claude, reason=phase:REVIEW)
```

## 5) Stall fallback example

If no progress is detected for multiple rounds:

```text
Event: SAFE_BACKLOG_INJECTED
Category: documentation
Reason: no_progress_rounds=3
```

## 6) Completion artifacts

- `.fusion/task_plan.md`: all tasks marked `COMPLETED`
- `.fusion/progress.md`: timeline with phase transitions
- `.fusion/events.jsonl`: routing, backlog, supervisor events
- `.fusion/sessions.json`: latest phase and backend session IDs

## 7) Hook wiring reminder

Use host-level hooks (example file):

- `.claude/settings.example.json`
- `docs/HOOKS_SETUP.md`
