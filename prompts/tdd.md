# TDD Implementation Prompt

You are implementing a task following strict TDD (Test-Driven Development).

## Task Info
- **Task ID**: {{TASK_ID}}
- **Description**: {{DESCRIPTION}}
- **Files to modify**: {{FILES}}
- **Test files**: {{TEST_FILES}}

## TDD Protocol

### Phase 1: RED (Write Failing Test)

1. Analyze the task requirements
2. Identify test cases that cover the functionality
3. Write test code that will FAIL (because implementation doesn't exist yet)
4. Run tests to confirm they fail

**Output**: Test file content + confirmation of failure

### Phase 2: GREEN (Minimal Implementation)

1. Write the MINIMUM code needed to pass the tests
2. Do not add extra features or optimizations
3. Focus only on making tests pass
4. Run tests to confirm they pass

**Output**: Implementation file content + confirmation of passing

### Phase 3: REFACTOR (Clean Up)

1. Review the implementation for:
   - Code duplication
   - Unclear naming
   - Missing error handling
   - Performance issues
2. Refactor while keeping tests green
3. Run tests after each refactor step

**Output**: Refactored code + confirmation tests still pass

## Constraints

- Do NOT skip the RED phase
- Do NOT write more code than needed in GREEN phase
- Do NOT break tests in REFACTOR phase
- Each phase should be a separate commit

## Test Command Detection

Detect project test command:
- `package.json` with test script → `npm test`
- `pytest.ini` or `tests/` with `.py` → `pytest`
- `go.mod` → `go test ./...`
- `Cargo.toml` → `cargo test`
- Default → ask user

## Now execute TDD for:

Task: {{TASK_ID}}
Description: {{DESCRIPTION}}
