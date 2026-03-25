## Summary

Describe what this PR changes and why.

## Type of Change

- [ ] feat
- [ ] fix
- [ ] docs
- [ ] test
- [ ] refactor
- [ ] chore

## Validation

- [ ] `cd rust && cargo test --release` passed locally
- [ ] Added/updated tests for behavior changes
- [ ] Updated docs/changelog if needed

## Checklist

- [ ] Scope is focused and does not include unrelated changes
- [ ] No sensitive information included
- [ ] Backward compatibility considered
- [ ] If active docs or repository/runtime contracts changed, `rust/crates/fusion-cli/tests/repo_contract.rs` was updated too
- [ ] If init/config behavior changed, `.fusion/config.yaml` is still treated as generated workspace state from `templates/config.yaml`
- [ ] If hook wiring changed, `.claude/settings.example.json` remains the checked-in template and `.claude/settings.json` / `.claude/settings.local.json` remain host-local files
