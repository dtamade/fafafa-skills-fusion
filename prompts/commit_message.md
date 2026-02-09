# Commit Message Generator Prompt

Generate a conventional commit message for the changes made.

## Changes Summary
{{CHANGES_SUMMARY}}

## Tasks Completed
{{TASKS_LIST}}

## Files Modified
{{FILES_LIST}}

## Conventional Commit Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting, semicolons, etc)
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

### Rules
1. Subject line max 50 characters
2. Body wrapped at 72 characters
3. Use imperative mood ("add" not "added")
4. No period at end of subject

## Output

Generate ONLY the commit message, nothing else:

```
<type>(<scope>): <subject>

- <bullet point 1>
- <bullet point 2>
...

Fusion workflow: <workflow_id>
```
