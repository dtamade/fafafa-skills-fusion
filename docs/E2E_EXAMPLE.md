# Fusion E2E Example

This example shows a full `/fusion` run with the default routing strategy:

- Codex: planning/analysis/review
- Claude: implementation/verification execution

The routing shown below reflects the checked-in `templates/config.yaml` baseline. Actual runs consume the generated `.fusion/config.yaml` initialized from that baseline. If maintainers change `backend_routing`, update this example together with the template.
If this example's active routing baseline or repository/runtime contract changes, update the affected active docs together with `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`.

## Scenario

```bash
/fusion "add email verification for signup"
```

## 1) UNDERSTAND handoff

```text
[fusion] UNDERSTAND runner currently minimal; proceed to INITIALIZE
[fusion] Current state: in_progress @ INITIALIZE
[fusion] Next action: Initialize workspace files and proceed to ANALYZE
[FUSION] Workflow initialized.
Goal: add email verification for signup
```

Current live start path also records this handoff in `.fusion/sessions.json`:

```text
_runtime.state=INITIALIZE
_runtime.understand.mode=minimal
_runtime.understand.forced=false
_runtime.understand.decision=auto_continue
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

- `.claude/settings.example.json` (checked-in template)
- `docs/HOOKS_SETUP.md`

The checked-in `.claude/settings.example.json` file is just the template. The actual `.claude/settings.json` and `.claude/settings.local.json` files are host-local hook configuration, not tracked workflow artifacts or repository-structure evidence.
