# Error Recovery Prompt

You are helping recover from a failed task execution.

## Context
- **Task ID**: {{TASK_ID}}
- **Strike Number**: {{STRIKE_NUMBER}}
- **Error Message**: {{ERROR_MESSAGE}}
- **Previous Attempts**: {{PREVIOUS_ATTEMPTS}}

## Recovery Strategy

### Strike 1: Targeted Fix
Focus on the specific error:
1. Analyze the error message carefully
2. Identify the root cause
3. Apply a minimal, targeted fix
4. Retry the exact same approach with the fix

### Strike 2: Alternative Approach
The previous fix didn't work. Try a different method:
1. Do NOT repeat any previous approaches
2. Consider completely different implementation strategies
3. Maybe use different libraries, patterns, or algorithms
4. Document why this approach might work better

### Strike 3: Fallback to Claude Local
Codex has failed twice. Now fallback to Claude local execution:
1. Claude will execute directly using Edit/Write tools
2. Skip external backend calls
3. Use simpler, more direct implementation
4. Focus on getting the task done, not elegance

## Output Format

```yaml
recovery_analysis:
  error_type: <classification of the error>
  root_cause: <identified cause>

previous_attempts:
  - attempt: 1
    approach: <what was tried>
    why_failed: <reason>

new_approach:
  strategy: <description>
  rationale: <why this should work>
  implementation: |
    <code or steps>

expected_outcome: <what success looks like>
```

## Now recover from:

Task: {{TASK_ID}}
Strike: {{STRIKE_NUMBER}}
Error: {{ERROR_MESSAGE}}

Previous attempts:
{{PREVIOUS_ATTEMPTS}}
