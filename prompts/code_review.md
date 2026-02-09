# Code Review Prompt

You are performing a code review on the changes made during a Fusion workflow.

## Changes to Review
{{GIT_DIFF}}

## Files Modified
{{FILES_LIST}}

## Review Checklist

### 1. Code Quality
- [ ] Consistent code style
- [ ] Clear naming conventions
- [ ] No unnecessary complexity
- [ ] DRY principle followed

### 2. Error Handling
- [ ] All error cases handled
- [ ] Meaningful error messages
- [ ] No silent failures
- [ ] Graceful degradation where appropriate

### 3. Security
- [ ] No hardcoded secrets
- [ ] Input validation present
- [ ] No SQL injection vulnerabilities
- [ ] No XSS vulnerabilities
- [ ] Proper authentication/authorization

### 4. Performance
- [ ] No obvious N+1 queries
- [ ] Efficient algorithms used
- [ ] No unnecessary loops
- [ ] Proper caching where needed

### 5. Testing
- [ ] Tests cover main functionality
- [ ] Edge cases tested
- [ ] Tests are readable
- [ ] No flaky tests

### 6. Documentation
- [ ] Complex logic is commented
- [ ] Public APIs are documented
- [ ] README updated if needed

## Output Format

```yaml
review_result: APPROVED | NEEDS_WORK

summary: <one line summary>

findings:
  critical: []  # Must fix before merge
  high: []      # Should fix
  medium: []    # Nice to fix
  low: []       # Minor suggestions

details:
  - file: <path>
    line: <number>
    severity: critical | high | medium | low
    issue: <description>
    suggestion: <how to fix>

recommendations:
  - <suggestion 1>
  - <suggestion 2>
```

## Now review:

{{GIT_DIFF}}
