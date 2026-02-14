# Fusion Progress Log

## Session Info
- Goal: 全仓扫描缺口 + 可执行优先级计划 + TDD执行批次1
- Started: 2026-02-11
- Branch: current working tree

## Timeline

| Time | Phase | Event | Status | Details |
|------|-------|-------|--------|---------|
| 2026-02-11 | SCAN | 初始化 planning 文件 | OK | 创建 task_plan.md/findings.md/progress.md |

## Current Status

```
Phase: COMPLETE_ROUND37 (5/5)
Task: docs schema-version and artifact consistency hardening + freshness guards
Backend: codex-cli
Progress: 100%
```

## Errors

| Time | Task | Error | Attempt | Resolution |
|------|------|-------|---------|------------|
| - | - | - | - | - |

## Command Output Log

| Step | Command | Exit | Key Output |
|------|---------|------|------------|
| 1 | create planning files | 0 | files created |
| 2026-02-11 | SCAN | git status + TODO/PENDING 扫描 | OK | 发现文档陈旧计数、脚本测试覆盖不均 |
| 2026-02-11 | SCAN | syntax + pytest baseline | OK | shell_syntax:OK, 326 passed |
| 2026-02-11 | PLAN | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round1.md |
| 2026-02-11 | EXECUTE(Task1) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 1 failed |
| 2026-02-11 | EXECUTE(Task1) | GREEN | OK | README.zh-CN 去除硬编码通过数，测试变为 1 passed |
| 2026-02-11 | EXECUTE(Task1) | REGRESSION | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_fusion_status_script.py` -> 6 passed |
| 2026-02-11 | EXECUTE(Task2) | RED | OK | `pytest -q scripts/runtime/tests/test_fusion_hook_doctor_script.py` -> 2 failed |
| 2026-02-11 | EXECUTE(Task2) | GREEN | OK | 新增 `--json` 参数与机器可读输出，测试变为 2 passed |
| 2026-02-11 | EXECUTE(Task2) | CHECK | OK | `bash scripts/fusion-hook-doctor.sh --json .` 输出 JSON |
| 2026-02-11 | EXECUTE(Task3) | RED | OK | `pytest -q ...::test_status_can_disable_leaderboard` -> 1 failed |
| 2026-02-11 | EXECUTE(Task3) | GREEN | OK | 增加 `FUSION_STATUS_SHOW_LEADERBOARD` 开关，测试变为 1 passed |
| 2026-02-11 | VERIFY | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_docs_freshness.py` -> 9 passed |
| 2026-02-11 | VERIFY | FULL | OK | `pytest -q` -> 330 passed |
| 2026-02-11 | REFACTOR(Task2) | TEST CLEANUP | OK | 简化 test env 注入，targeted+full 仍全部通过 |
| 2026-02-11 | SCAN(Round2) | 全仓复扫 | OK | baseline 330 passed；start/init 直接测试缺口确认 |
| 2026-02-11 | PLAN(Round2) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round2.md |
| 2026-02-11 | EXECUTE(TaskC2) | RED-recheck | OK | `pytest -q ...json_success ...json_error_on_invalid_engine` -> 2 passed（先前 RED 已在上一轮完成） |
| 2026-02-11 | EXECUTE(TaskC2.1) | RED | OK | `pytest -q ...::test_fusion_init_json_fallback_without_jq_or_python3` -> 1 failed(JSONDecodeError) |
| 2026-02-11 | EXECUTE(TaskC2.1) | GREEN | OK | 修复 `fusion-init.sh` fallback 输出为合法 JSON；`bash -n` 通过 |
| 2026-02-11 | EXECUTE(TaskC2.1) | VERIFY | OK | `pytest -q ...json_success ...json_error_on_invalid_engine ...json_fallback` -> 3 passed |
| 2026-02-11 | VERIFY(Round2) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_status_script.py` -> 15 passed |
| 2026-02-11 | VERIFY(Round2) | FULL | OK | `pytest -q` -> 336 passed |
| 2026-02-11 | TRANSITION | Round2->Round3 | OK | task_plan 状态切换到 SCAN_ROUND3 |
| 2026-02-11 | SCAN(Round3) | baseline + syntax + todo | OK | `pytest -q` -> 336 passed; `bash -n scripts/*.sh` -> shell_syntax:OK |
| 2026-02-11 | SCAN(Round3) | script/test coverage map | OK | 缺少直接引用脚本: cancel/continue/git/logs/pause/resume/loop-guardian |
| 2026-02-11 | PLAN(Round3) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round3.md |
| 2026-02-11 | EXECUTE(Round3-TaskA3) | RED | OK | `pytest -q ...::test_resume_rejects_unknown_option` -> 1 failed (returncode 0) |
| 2026-02-11 | EXECUTE(Round3-TaskA3) | GREEN | OK | `fusion-resume.sh` 新增 `-h/--help` + unknown option 拒绝 |
| 2026-02-11 | EXECUTE(Round3-TaskA3) | VERIFY | OK | `bash -n scripts/fusion-resume.sh && pytest -q ...::test_resume_rejects_unknown_option` -> 1 passed |
| 2026-02-11 | EXECUTE(Round3-TaskB3) | RED | OK | `pytest -q ...::test_git_rejects_unknown_action` -> 1 failed (returncode 0) |
| 2026-02-11 | EXECUTE(Round3-TaskB3) | GREEN | OK | `fusion-git.sh` case 默认分支改为 Unknown action + usage + exit 1 |
| 2026-02-11 | EXECUTE(Round3-TaskB3) | VERIFY | OK | `bash -n scripts/fusion-git.sh && pytest -q ...::test_git_rejects_unknown_action` -> 1 passed |
| 2026-02-11 | EXECUTE(Round3-TaskC3) | RED | OK | `pytest -q ...::test_logs_rejects_non_numeric_lines` -> 1 failed (缺少自定义错误文案) |
| 2026-02-11 | EXECUTE(Round3-TaskC3) | GREEN | OK | `fusion-logs.sh` 增加 `LINES` 正整数校验 + usage |
| 2026-02-11 | EXECUTE(Round3-TaskC3) | VERIFY | OK | `bash -n scripts/fusion-logs.sh && pytest -q ...::test_logs_rejects_non_numeric_lines` -> 1 passed |
| 2026-02-11 | VERIFY(Round3) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_status_script.py` -> 18 passed |
| 2026-02-11 | VERIFY(Round3) | FULL | OK | `pytest -q` -> 339 passed |
| 2026-02-11 | REFACTOR(Round3) | test file structure cleanup | OK | 将 `__main__` 入口移动到文件末尾，不改变行为 |
| 2026-02-11 | VERIFY(Round3) | FULL(recheck) | OK | `pytest -q` -> 339 passed |
| 2026-02-11 | SCAN(Round4) | baseline + syntax + mapping | OK | `pytest -q` -> 339 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; NO_REF: pause/cancel/continue/loop-guardian |
| 2026-02-11 | PLAN(Round4) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round4.md |
| 2026-02-11 | EXECUTE(Round4-TaskA4) | RED | OK | `pytest -q ...::test_pause_rejects_unknown_option` -> 1 failed (returncode 0) |
| 2026-02-11 | EXECUTE(Round4-TaskA4) | GREEN | OK | `fusion-pause.sh` 新增 `-h/--help` + unknown option 拒绝 |
| 2026-02-11 | EXECUTE(Round4-TaskA4) | VERIFY | OK | `bash -n scripts/fusion-pause.sh && pytest -q ...::test_pause_rejects_unknown_option` -> 1 passed |
| 2026-02-11 | EXECUTE(Round4-TaskB4) | RED | OK | `pytest -q ...::test_cancel_rejects_unknown_option` -> 1 failed (returncode 0) |
| 2026-02-11 | EXECUTE(Round4-TaskB4) | GREEN | OK | `fusion-cancel.sh` 新增 `-h/--help` + unknown option 拒绝 |
| 2026-02-11 | EXECUTE(Round4-TaskB4) | VERIFY | OK | `bash -n scripts/fusion-cancel.sh && pytest -q ...::test_cancel_rejects_unknown_option` -> 1 passed |
| 2026-02-11 | EXECUTE(Round4-TaskC4) | RED | OK | `pytest -q ...::test_continue_rejects_unknown_option` -> 1 failed (returncode 0) |
| 2026-02-11 | EXECUTE(Round4-TaskC4) | GREEN | OK | `fusion-continue.sh` 新增 `-h/--help` + unknown option 拒绝 |
| 2026-02-11 | EXECUTE(Round4-TaskC4) | VERIFY | OK | `bash -n scripts/fusion-continue.sh && pytest -q ...::test_continue_rejects_unknown_option` -> 1 passed |
| 2026-02-11 | VERIFY(Round4) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_status_script.py` -> 21 passed |
| 2026-02-11 | VERIFY(Round4) | FULL | OK | `pytest -q` -> 342 passed |
| 2026-02-11 | SCAN(Round5-pre) | script/test mapping refresh | OK | 当前仅 `loop-guardian` 无直接测试引用 |
| 2026-02-11 | SCAN(Round5) | baseline + syntax + mapping | OK | `pytest -q` -> 342 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; only NO_REF=loop-guardian |
| 2026-02-11 | PLAN(Round5) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round5.md |
| 2026-02-11 | EXECUTE(Round5-TaskA5) | RED | OK | `pytest -q ...::test_status_uses_loaded_config_thresholds` -> 1 failed (status 显示默认阈值) |
| 2026-02-11 | EXECUTE(Round5-TaskA5) | GREEN | OK | `guardian_status` 改为使用 shell 阈值变量注入（非 jq env） |
| 2026-02-11 | EXECUTE(Round5-TaskA5) | VERIFY | OK | `bash -n scripts/loop-guardian.sh && pytest -q ...::test_status_uses_loaded_config_thresholds` -> 1 passed |
| 2026-02-11 | EXECUTE(Round5-TaskB5) | RED | OK | `pytest -q ...::test_init_creates_fusion_dir_when_missing` -> 1 failed |
| 2026-02-11 | EXECUTE(Round5-TaskB5) | GREEN | OK | `guardian_init` 增加 `mkdir -p "$FUSION_DIR"` |
| 2026-02-11 | EXECUTE(Round5-TaskB5) | VERIFY | OK | `bash -n scripts/loop-guardian.sh && pytest -q ...::test_init_creates_fusion_dir_when_missing` -> 1 passed |
| 2026-02-11 | EXECUTE(Round5-TaskC5) | RED | OK | `pytest -q ...::test_status_includes_state_and_walltime_thresholds` -> 1 failed |
| 2026-02-11 | EXECUTE(Round5-TaskC5) | GREEN | OK | `guardian_status` 增加 State Visits / Wall Time 阈值输出 |
| 2026-02-11 | EXECUTE(Round5-TaskC5) | VERIFY | OK | `bash -n scripts/loop-guardian.sh && pytest -q ...status... ...init...` -> 3 passed |
| 2026-02-11 | VERIFY(Round5) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_loop_guardian_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_status_script.py` -> 24 passed |
| 2026-02-11 | VERIFY(Round5) | FULL | OK | `pytest -q` -> 345 passed |
| 2026-02-11 | SCAN(Round6) | fusion-start usage path probe | OK | `bash scripts/fusion-start.sh --bad` -> `goal: No such file or directory`; `bash scripts/fusion-start.sh -h` -> exit 1 + 同错误 |
| 2026-02-11 | PLAN(Round6) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round6.md |
| 2026-02-11 | EXECUTE(Round6-TaskA6) | RED | OK | `pytest -q ...::test_help_exits_zero_and_shows_usage` -> 1 failed (exit 1) |
| 2026-02-11 | EXECUTE(Round6-TaskA6) | GREEN | OK | `fusion-start.sh` usage 统一为安全字符串 `Usage: fusion-start.sh <goal> [--force]` |
| 2026-02-11 | EXECUTE(Round6-TaskA6) | VERIFY | OK | `bash -n scripts/fusion-start.sh && pytest -q ...::test_help_exits_zero_and_shows_usage` -> 1 passed |
| 2026-02-11 | EXECUTE(Round6-TaskB6) | RED | OK | 预扫描证据：`bash scripts/fusion-start.sh --bad` 出现 `goal: No such file or directory` |
| 2026-02-11 | EXECUTE(Round6-TaskB6) | VERIFY | OK | `pytest -q ...::test_unknown_option_reports_usage_without_shell_redirection_error` -> 1 passed |
| 2026-02-11 | EXECUTE(Round6-TaskC6) | RED | OK | 预扫描证据：`bash scripts/fusion-start.sh -h` 旧行为 exit 1 + 重定向错误 |
| 2026-02-11 | EXECUTE(Round6-TaskC6) | VERIFY | OK | `pytest -q ...::test_missing_goal_reports_usage_without_shell_redirection_error` -> 1 passed |
| 2026-02-11 | VERIFY(Round6) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_loop_guardian_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_status_script.py` -> 27 passed |
| 2026-02-11 | VERIFY(Round6) | FULL | OK | `pytest -q` -> 348 passed |
| 2026-02-11 | SCAN(Round7-pre) | achievements 参数探测 | OK | `--top abc` 返回0但stderr有 `head: invalid number of lines`; `--root` 缺失值仍返回0 |
| 2026-02-11 | REFACTOR(Round5) | loop-guardian test file cleanup | OK | `__main__` 入口移至文件末尾，不影响行为 |
| 2026-02-11 | VERIFY | FULL(recheck) | OK | `pytest -q` -> 348 passed |
| 2026-02-11 | SCAN(Round7) | baseline + syntax + mapping | OK | `pytest -q` -> 348 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; achievements 参数误用缺口确认 |
| 2026-02-11 | PLAN(Round7) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round7.md |
| 2026-02-11 | EXECUTE(Round7-TaskA7) | RED | OK | `pytest -q ...::test_rejects_non_numeric_top_value` -> 1 failed (returncode 0) |
| 2026-02-11 | EXECUTE(Round7-TaskA7) | GREEN | OK | `fusion-achievements.sh` 增加 `TOP_N` 正整数校验 |
| 2026-02-11 | EXECUTE(Round7-TaskA7) | VERIFY | OK | `bash -n scripts/fusion-achievements.sh && pytest -q ...::test_rejects_non_numeric_top_value` -> 1 passed |
| 2026-02-11 | EXECUTE(Round7-TaskB7) | RED | OK | `pytest -q ...::test_rejects_missing_root_value` -> 1 failed (returncode 0) |
| 2026-02-11 | EXECUTE(Round7-TaskB7) | GREEN | OK | `--root` 分支增加缺失值校验 |
| 2026-02-11 | EXECUTE(Round7-TaskB7) | VERIFY | OK | `bash -n scripts/fusion-achievements.sh && pytest -q ...::test_rejects_missing_root_value` -> 1 passed |
| 2026-02-11 | EXECUTE(Round7-TaskC7) | RED | OK | `pytest -q ...::test_rejects_missing_top_value` -> 1 failed (returncode 0) |
| 2026-02-11 | EXECUTE(Round7-TaskC7) | GREEN | OK | `--top` 分支增加缺失值校验 |
| 2026-02-11 | EXECUTE(Round7-TaskC7) | VERIFY | OK | `bash -n scripts/fusion-achievements.sh && pytest -q ...::test_rejects_missing_top_value` -> 1 passed |
| 2026-02-11 | VERIFY(Round7) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_achievements_script.py scripts/runtime/tests/test_fusion_start_script.py scripts/runtime/tests/test_loop_guardian_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_status_script.py` -> 33 passed |
| 2026-02-11 | VERIFY(Round7) | FULL | OK | `pytest -q` -> 351 passed |
| 2026-02-11 | SCAN(Round8-pre) | achievements CLI 兼容性探测 | OK | 错误参数仍输出标题；`--top=<n>` / `--root=<path>` 当前不支持 |
| 2026-02-11 | SCAN(Round8) | BASELINE | OK | `pytest -q` -> 351 passed; `bash -n scripts/*.sh` -> shell_syntax:OK |
| 2026-02-11 | SCAN(Round8) | RUST_QGATE | WARN | `cargo fmt --all -- --check` failed (format diff in `fusion-runtime-io/src/lib.rs`) |
| 2026-02-11 | SCAN(Round8) | RUST_QGATE | WARN | `cargo clippy --workspace --all-targets -- -D warnings` failed (`too_many_arguments` in `fusion-cli/src/main.rs`) |
| 2026-02-11 | SCAN(Round8) | CLI_PROBE | OK | achievements: `--top=2` / `--root=.` -> Unknown option; invalid `--top abc` path仍打印横幅 |
| 2026-02-11 | PLAN(Round8) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round8.md |
| 2026-02-11 | EXECUTE(Round8-TaskA8) | RED | OK | `pytest -q ...::test_rejects_non_numeric_top_value` -> 1 failed (`=== Fusion Achievements ===` unexpectedly found) |
| 2026-02-11 | EXECUTE(Round8-TaskA8) | GREEN | OK | `fusion-achievements.sh` 将横幅输出下移至参数校验之后 |
| 2026-02-11 | EXECUTE(Round8-TaskA8) | VERIFY | OK | `bash -n scripts/fusion-achievements.sh && pytest -q ...::test_rejects_non_numeric_top_value` -> 1 passed |
| 2026-02-11 | EXECUTE(Round8-TaskB8) | RED | OK | `pytest -q ...::test_supports_top_equals_syntax` -> 1 failed (returncode 1) |
| 2026-02-11 | EXECUTE(Round8-TaskB8) | GREEN | OK | achievements 解析器新增 `--top=*` 与 `--root=*` 分支 |
| 2026-02-11 | EXECUTE(Round8-TaskB8) | VERIFY | OK | `bash -n scripts/fusion-achievements.sh && pytest -q ...::test_supports_top_equals_syntax` -> 1 passed |
| 2026-02-11 | EXECUTE(Round8-TaskC8) | VERIFY | OK | `pytest -q ...::test_supports_root_equals_syntax` -> 1 passed |
| 2026-02-11 | VERIFY(Round8) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_achievements_script.py` -> 8 passed |
| 2026-02-11 | VERIFY(Round8) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_achievements_script.py ... test_fusion_status_script.py` -> 35 passed |
| 2026-02-11 | VERIFY(Round8) | FULL | OK | `pytest -q` -> 353 passed |
| 2026-02-11 | SCAN(Round9) | BASELINE | OK | `pytest -q` -> 353 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; `(cd rust && cargo test -q)` -> all passed |
| 2026-02-11 | SCAN(Round9) | CLI_PROBE | OK | `fusion-logs --help` exit1; `fusion-git --help` exit1; `fusion-codeagent --help` timeout+route（缺口确认） |
| 2026-02-11 | PLAN(Round9) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round9.md |
| 2026-02-11 | EXECUTE(Round9-TaskA9) | RED | OK | `pytest -q ...::test_logs_help_exits_zero_and_shows_usage` -> 1 failed (returncode 1) |
| 2026-02-11 | EXECUTE(Round9-TaskA9) | GREEN | OK | `fusion-logs.sh` 新增 `usage()` + `-h/--help` 早返回 |
| 2026-02-11 | EXECUTE(Round9-TaskA9) | VERIFY | OK | `pytest -q ...::test_logs_help_exits_zero_and_shows_usage` -> 1 passed |
| 2026-02-11 | EXECUTE(Round9-TaskB9) | RED | OK | `pytest -q ...::test_git_help_exits_zero_and_shows_usage` -> 1 failed (returncode 1) |
| 2026-02-11 | EXECUTE(Round9-TaskB9) | GREEN | OK | `fusion-git.sh` 新增 `usage()` + `--help` 分支 |
| 2026-02-11 | EXECUTE(Round9-TaskB9) | VERIFY | OK | `pytest -q ...::test_git_help_exits_zero_and_shows_usage` -> 1 passed |
| 2026-02-11 | EXECUTE(Round9-TaskC9) | RED | OK | `pytest -q ...::test_help_exits_zero_without_routing` -> 1 failed（无 usage 且有 route） |
| 2026-02-11 | EXECUTE(Round9-TaskC9) | GREEN | OK | `fusion-codeagent.sh` `main()` 前置 `--help` 早返回，避免 `ensure_fusion/route` |
| 2026-02-11 | EXECUTE(Round9-TaskC9) | VERIFY | OK | `pytest -q ...::test_help_exits_zero_without_routing` -> 1 passed |
| 2026-02-11 | VERIFY(Round9) | SYNTAX | OK | `bash -n scripts/fusion-logs.sh scripts/fusion-git.sh scripts/fusion-codeagent.sh` -> pass |
| 2026-02-11 | VERIFY(Round9) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_codeagent_script.py` -> 14 passed |
| 2026-02-11 | VERIFY(Round9) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_achievements_script.py ... test_fusion_status_script.py` -> 42 passed |
| 2026-02-11 | VERIFY(Round9) | FULL | OK | `pytest -q` -> 356 passed |
| 2026-02-11 | SCAN(Round10) | BASELINE | OK | `pytest -q` -> 356 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; `(cd rust && cargo test -q)` -> all passed |
| 2026-02-11 | SCAN(Round10) | RUST_QGATE | WARN | `cd rust && cargo fmt --all -- --check` -> failed (diff in `fusion-runtime-io/src/lib.rs`) |
| 2026-02-11 | SCAN(Round10) | RUST_QGATE | WARN | `cd rust && cargo clippy --workspace --all-targets -- -D warnings` -> failed (`too_many_arguments` in `try_inject_safe_backlog`) |
| 2026-02-11 | PLAN(Round10) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round10.md |
| 2026-02-11 | EXECUTE(Round10-TaskC10) | RED | OK | `pytest -q ...::test_status_help_exits_zero_without_fusion_dir` -> 1 failed (returncode 1) |
| 2026-02-11 | EXECUTE(Round10-TaskC10) | GREEN | OK | `fusion-status.sh` 新增 `usage()` + `-h/--help` 早返回 |
| 2026-02-11 | EXECUTE(Round10-TaskC10) | VERIFY | OK | `bash -n scripts/fusion-status.sh && pytest -q ...::test_status_help_exits_zero_without_fusion_dir` -> 1 passed |
| 2026-02-11 | EXECUTE(Round10-TaskA10) | RED | OK | 复用扫描证据：`cargo clippy ... -D warnings` 报 `too_many_arguments (9/7)` |
| 2026-02-11 | EXECUTE(Round10-TaskA10) | GREEN | OK | `fusion-cli` 引入 `SafeBacklogTrigger`，收敛 `try_inject_safe_backlog` 参数 |
| 2026-02-11 | EXECUTE(Round10-TaskA10) | VERIFY | OK | `cd rust && cargo clippy --workspace --all-targets -- -D warnings` -> pass |
| 2026-02-11 | EXECUTE(Round10-TaskB10) | RED | OK | 复用扫描证据：`cargo fmt --all -- --check` failed |
| 2026-02-11 | EXECUTE(Round10-TaskB10) | GREEN | OK | `cd rust && cargo fmt --all` |
| 2026-02-11 | EXECUTE(Round10-TaskB10) | VERIFY | OK | `cd rust && cargo fmt --all -- --check` -> pass |
| 2026-02-11 | VERIFY(Round10) | SYNTAX | OK | `bash -n scripts/fusion-status.sh scripts/fusion-logs.sh scripts/fusion-git.sh scripts/fusion-codeagent.sh` -> pass |
| 2026-02-11 | VERIFY(Round10) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_codeagent_script.py` -> 21 passed |
| 2026-02-11 | VERIFY(Round10) | RUST | OK | `cd rust && cargo test -q && cargo clippy ... && cargo fmt --check` -> pass |
| 2026-02-11 | VERIFY(Round10) | FULL | OK | `pytest -q` -> 357 passed |
| 2026-02-11 | SCAN(Round11) | BASELINE | OK | `pytest -q` -> 357 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; rust test/clippy/fmt 全通过 |
| 2026-02-11 | SCAN(Round11) | CLI_PROBE | OK | `fusion-status --json` 仍输出人类横幅；achievements `--root=`/`--top=` 已拒绝 |
| 2026-02-11 | PLAN(Round11) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round11.md |
| 2026-02-11 | EXECUTE(Round11-TaskA11) | RED | OK | `pytest -q ...::test_status_json_mode_outputs_machine_readable_summary` -> 1 failed (JSONDecodeError) |
| 2026-02-11 | EXECUTE(Round11-TaskB11) | RED | OK | `pytest -q ...::test_status_json_mode_reports_missing_fusion_dir` -> 1 failed (JSONDecodeError) |
| 2026-02-11 | EXECUTE(Round11-TaskC11) | RED | OK | `pytest -q ...::test_status_json_mode_omits_human_banner` -> 1 failed (banner present) |
| 2026-02-11 | EXECUTE(Round11-TaskABC) | GREEN | OK | `fusion-status.sh` 增加 `--json` 参数解析与 `emit_json_status` 成功/失败分支 |
| 2026-02-11 | EXECUTE(Round11-TaskA11) | VERIFY | OK | `pytest -q ...::test_status_json_mode_outputs_machine_readable_summary` -> 1 passed |
| 2026-02-11 | EXECUTE(Round11-TaskB11) | VERIFY | OK | `pytest -q ...::test_status_json_mode_reports_missing_fusion_dir` -> 1 passed |
| 2026-02-11 | EXECUTE(Round11-TaskC11) | VERIFY | OK | `pytest -q ...::test_status_json_mode_omits_human_banner` -> 1 passed |
| 2026-02-11 | VERIFY(Round11) | SYNTAX | OK | `bash -n scripts/fusion-status.sh` -> pass |
| 2026-02-11 | VERIFY(Round11) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py` -> 10 passed |
| 2026-02-11 | VERIFY(Round11) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py ... test_fusion_hook_doctor_script.py` -> 46 passed |
| 2026-02-11 | VERIFY(Round11) | FULL | OK | `pytest -q` -> 360 passed |
| 2026-02-11 | VERIFY(Round11) | PROBE | OK | `fusion-status.sh --json` -> `{"result":"ok",...}`; missing `.fusion` -> `{"result":"error",...}` |
| 2026-02-11 | SCAN(Round12) | BASELINE | OK | `pytest -q` -> 360 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; rust test/clippy/fmt 全通过 |
| 2026-02-11 | SCAN(Round12) | CLI_PROBE | OK | `fusion-status --json` 仅基础字段，缺 task/dependency/achievement counters |
| 2026-02-11 | PLAN(Round12) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round12.md |
| 2026-02-11 | EXECUTE(Round12-TaskA12) | RED | OK | `pytest -q ...::test_status_json_includes_task_counts` -> 1 failed (missing task_* fields) |
| 2026-02-11 | EXECUTE(Round12-TaskB12) | RED | OK | `pytest -q ...::test_status_json_includes_dependency_summary` -> 1 failed (missing dependency_* fields) |
| 2026-02-11 | EXECUTE(Round12-TaskC12) | RED | OK | `pytest -q ...::test_status_json_includes_achievement_counters` -> 1 failed (missing achievement_* fields) |
| 2026-02-11 | EXECUTE(Round12-TaskABC) | GREEN | OK | `fusion-status.sh` JSON 摘要新增 task/dependency/achievement 计数字段 |
| 2026-02-11 | EXECUTE(Round12-TaskABC) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py -k "json_includes_task_counts or json_includes_dependency_summary or json_includes_achievement_counters"` -> 3 passed |
| 2026-02-11 | VERIFY(Round12) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py` -> 13 passed |
| 2026-02-11 | VERIFY(Round12) | FULL | OK | `pytest -q` -> 363 passed |
| 2026-02-11 | TRANSITION | Round12->Round13 | OK | 开始新一轮全仓扫描、50项缺口与优先级计划 |
| 2026-02-11 | SCAN(Round13) | BASELINE | OK | `pytest -q` -> 363 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; rust test/clippy/fmt 全通过 |
| 2026-02-11 | SCAN(Round13) | CLI_PROBE | WARN | `fusion-codeagent.sh --bad` timeout(124) 且误路由；`fusion-hook-doctor.sh --bad` 出现 `cd: --` 不可读错误 |
| 2026-02-11 | PLAN(Round13) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round13.md |
| 2026-02-11 | EXECUTE(Round13-TaskA13) | RED | OK | `pytest -q ...::test_unknown_option_exits_nonzero_without_routing` -> 1 failed（returncode 0） |
| 2026-02-11 | EXECUTE(Round13-TaskA13) | GREEN | OK | `fusion-codeagent.sh` 前置未知选项校验，避免误路由 |
| 2026-02-11 | EXECUTE(Round13-TaskA13) | VERIFY | OK | `pytest -q ...::test_unknown_option_exits_nonzero_without_routing` -> 1 passed |
| 2026-02-11 | EXECUTE(Round13-TaskB13) | RED | OK | `pytest -q ...::test_json_mode_rejects_unknown_option` -> 1 failed（result=warn） |
| 2026-02-11 | EXECUTE(Round13-TaskB13) | GREEN | OK | `fusion-hook-doctor.sh` 参数解析拒绝未知选项；JSON 模式输出 error 对象 |
| 2026-02-11 | EXECUTE(Round13-TaskB13) | VERIFY | OK | `pytest -q ...::test_json_mode_rejects_unknown_option` -> 1 passed |
| 2026-02-11 | EXECUTE(Round13-TaskC13) | RED | OK | `pytest -q ...::test_json_mode_rejects_invalid_project_root` -> 1 failed（result=warn） |
| 2026-02-11 | EXECUTE(Round13-TaskC13) | GREEN | OK | `fusion-hook-doctor.sh` 增加 `project_root` 存在性校验并失败快返 |
| 2026-02-11 | EXECUTE(Round13-TaskC13) | VERIFY | OK | `pytest -q ...::test_json_mode_rejects_invalid_project_root` -> 1 passed |
| 2026-02-11 | VERIFY(Round13) | SYNTAX | OK | `bash -n scripts/fusion-codeagent.sh scripts/fusion-hook-doctor.sh` -> pass |
| 2026-02-11 | VERIFY(Round13) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_codeagent_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py` -> 11 passed |
| 2026-02-11 | VERIFY(Round13) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py ... test_loop_guardian_script.py` -> 52 passed |
| 2026-02-11 | VERIFY(Round13) | FULL | OK | `pytest -q` -> 366 passed |
| 2026-02-11 | VERIFY(Round13) | PROBE | OK | `fusion-codeagent.sh --bad` -> rc1 + usage；`fusion-hook-doctor.sh --json --bad` / invalid_root -> error JSON |
| 2026-02-11 | SCAN(Round14) | BASELINE | OK | `pytest -q` -> 366 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; rust test/clippy/fmt 全通过 |
| 2026-02-11 | SCAN(Round14) | CLI_PROBE | WARN | `fusion-hook-doctor.sh --json --fix .` -> `{"result":"error","reason":"Unknown option: --fix"}` |
| 2026-02-11 | PLAN(Round14) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round14.md |
| 2026-02-11 | EXECUTE(Round14-TaskA14) | RED | OK | `pytest -q ...::test_json_mode_fix_writes_project_settings` -> 1 failed（`--fix` unknown） |
| 2026-02-11 | EXECUTE(Round14-TaskA14) | GREEN | OK | `fusion-hook-doctor.sh` 增加 `--fix`，自动写入 `.claude/settings.local.json` |
| 2026-02-11 | EXECUTE(Round14-TaskA14) | VERIFY | OK | `pytest -q ...::test_json_mode_fix_writes_project_settings` -> 1 passed |
| 2026-02-11 | EXECUTE(Round14-TaskB14) | RED | OK | `pytest -q ...::test_json_mode_reports_fixed_flag` -> 1 failed（缺少 `fixed` 字段） |
| 2026-02-11 | EXECUTE(Round14-TaskB14) | GREEN | OK | hook-doctor JSON summary 增加 `fixed` bool 字段 |
| 2026-02-11 | EXECUTE(Round14-TaskB14) | VERIFY | OK | `pytest -q ...::test_json_mode_reports_fixed_flag` -> 1 passed |
| 2026-02-11 | EXECUTE(Round14-TaskC14) | RED | OK | `pytest -q ...::test_hooks_setup_mentions_fix_flow` -> 1 failed（文档缺少 `--fix`） |
| 2026-02-11 | EXECUTE(Round14-TaskC14) | GREEN | OK | `docs/HOOKS_SETUP.md` 增补 doctor + auto-fix 指引 |
| 2026-02-11 | EXECUTE(Round14-TaskC14) | VERIFY | OK | `pytest -q ...::test_hooks_setup_mentions_fix_flow` -> 1 passed |
| 2026-02-11 | VERIFY(Round14) | SYNTAX | OK | `bash -n scripts/fusion-hook-doctor.sh` -> pass |
| 2026-02-11 | VERIFY(Round14) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_docs_freshness.py` -> 8 passed |
| 2026-02-11 | VERIFY(Round14) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py ... test_docs_freshness.py` -> 56 passed |
| 2026-02-11 | VERIFY(Round14) | FULL | OK | `pytest -q` -> 369 passed |
| 2026-02-11 | VERIFY(Round14) | PROBE | OK | `fusion-hook-doctor.sh --json --fix .` -> `{"result":"ok",...,"fixed":true}` |
| 2026-02-11 | SCAN(Round15) | BASELINE | OK | `pytest -q` -> 369 passed; `bash -n scripts/*.sh` -> shell_syntax:OK; rust test/clippy/fmt 全通过 |
| 2026-02-11 | SCAN(Round15) | CLI_PROBE | WARN | stop-guard lock 竞争在 structured 模式仍 rc=2 且无 JSON；README 中英文缺少 hook-doctor `--json --fix` 快速恢复指引 |
| 2026-02-11 | PLAN(Round15) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round15.md |
| 2026-02-11 | EXECUTE(Round15-TaskA15) | RED | OK | `pytest -q scripts/runtime/tests/test_fusion_stop_guard_script.py::...::test_structured_lock_contention_returns_json_block` -> 1 failed（rc=2） |
| 2026-02-11 | EXECUTE(Round15-TaskA15) | GREEN | OK | `fusion-stop-guard.sh` 锁竞争分支改为 `emit_block_response`（structured JSON / legacy exit2） |
| 2026-02-11 | EXECUTE(Round15-TaskA15) | VERIFY | OK | `pytest -q ...::test_structured_lock_contention_returns_json_block` -> 1 passed |
| 2026-02-11 | EXECUTE(Round15-TaskB15) | GREEN | OK | 新增 `test_fusion_stop_guard_script.py`（structured/legacy/allow/lock 场景） |
| 2026-02-11 | EXECUTE(Round15-TaskB15) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_fusion_stop_guard_script.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round15-TaskC15) | RED | OK | docs freshness 新增 README/README.zh-CN hook-doctor fix 测试后均失败 |
| 2026-02-11 | EXECUTE(Round15-TaskC15) | GREEN | OK | `README.md` + `README.zh-CN.md` 增补 `fusion-hook-doctor.sh --json --fix` 快速修复章节 |
| 2026-02-11 | EXECUTE(Round15-TaskC15) | VERIFY | OK | 两个 docs freshness 新增测试均 1 passed |
| 2026-02-11 | EXECUTE(Round15-Extra) | VERIFY | OK | achievements 契约补测：`--root=` / `--top=` / unknown option 共 3 passed |
| 2026-02-11 | VERIFY(Round15) | SYNTAX | OK | `bash -n scripts/fusion-stop-guard.sh` -> pass |
| 2026-02-11 | VERIFY(Round15) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_fusion_achievements_script.py` -> 20 passed |
| 2026-02-11 | VERIFY(Round15) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py ... test_docs_freshness.py` -> 66 passed |
| 2026-02-11 | VERIFY(Round15) | FULL | OK | `pytest -q` -> 379 passed |
| 2026-02-11 | VERIFY(Round15) | PROBE | OK | structured 锁竞争返回 JSON block rc0；legacy 仍 rc2；hook-doctor `--json --fix` 返回 `fixed:true` |
| 2026-02-11 | SCAN(Round16) | BASELINE | OK | `pytest -q` -> 379 passed；shell/rust 门禁持续通过 |
| 2026-02-11 | SCAN(Round16) | CLI_PROBE | WARN | `fusion-logs --bad` 缺少 unknown-option 契约；`fusion-git --bad` 错误输出在 stdout；stop structured 空 stdin 缺专测 |
| 2026-02-11 | PLAN(Round16) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round16.md |
| 2026-02-11 | EXECUTE(Round16-TaskA16) | RED | OK | `pytest -q ...::test_logs_rejects_unknown_option` / `...::test_git_unknown_action_reports_to_stderr_with_usage` 均 failed |
| 2026-02-11 | EXECUTE(Round16-TaskA16) | GREEN | OK | `fusion-logs.sh` 增加 unknown-option/参数个数校验；`fusion-git.sh` 错误统一写 stderr + usage |
| 2026-02-11 | EXECUTE(Round16-TaskA16) | VERIFY | OK | 两个新测试均 1 passed；`pytest -q scripts/runtime/tests/test_fusion_control_script_validation.py` -> 10 passed |
| 2026-02-11 | EXECUTE(Round16-TaskB16) | VERIFY | OK | `pytest -q ...::test_structured_blocks_with_empty_stdin` -> 1 passed |
| 2026-02-11 | EXECUTE(Round16-TaskC16) | VERIFY | OK | `pytest -q ...::test_stop_guard_structured_without_stdin_uses_runtime_adapter` -> 1 passed |
| 2026-02-11 | VERIFY(Round16) | SYNTAX | OK | `bash -n scripts/fusion-logs.sh scripts/fusion-git.sh scripts/fusion-stop-guard.sh` -> pass |
| 2026-02-11 | VERIFY(Round16) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_control_script_validation.py scripts/runtime/tests/test_fusion_stop_guard_script.py scripts/runtime/tests/test_hook_shell_runtime_path.py` -> 23 passed |
| 2026-02-11 | VERIFY(Round16) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py ... test_docs_freshness.py` -> 76 passed |
| 2026-02-11 | VERIFY(Round16) | FULL | OK | `pytest -q` -> 383 passed |
| 2026-02-11 | VERIFY(Round16) | PROBE | OK | `fusion-logs --bad` / `fusion-git --bad` 输出契约命中；structured 空 stdin 仍返回 JSON block |
| 2026-02-11 | SCAN(Round17) | BASELINE | OK | `pytest -q` -> 383 passed；门禁持续通过 |
| 2026-02-11 | SCAN(Round17) | CLI_PROBE | WARN | status JSON 参数组合/ hook-doctor fix-failure / logs 多参数边界缺专门契约测试 |
| 2026-02-11 | PLAN(Round17) | 写入执行计划 | OK | docs/plans/2026-02-11-repo-gap-priority-round17.md |
| 2026-02-11 | EXECUTE(Round17-TaskA17) | RED-recheck | OK | 新增 status JSON 参数契约测试后首跑即通过（现有实现已满足） |
| 2026-02-11 | EXECUTE(Round17-TaskA17) | VERIFY | OK | `pytest -q ...::test_status_json_unknown_option_reports_error_object` + `...::test_status_json_help_still_shows_usage_and_exits_zero` -> 2 passed |
| 2026-02-11 | EXECUTE(Round17-TaskB17) | RED-recheck | OK | 新增 hook-doctor fix-failure 测试后首跑通过（当前实现满足 warn+fixed=false） |
| 2026-02-11 | EXECUTE(Round17-TaskB17) | VERIFY | OK | `pytest -q ...::test_json_mode_fix_failure_reports_warn_and_fixed_false` -> 1 passed |
| 2026-02-11 | EXECUTE(Round17-TaskC17) | RED-recheck | OK | 新增 logs 多参数测试后首跑通过（当前实现满足） |
| 2026-02-11 | EXECUTE(Round17-TaskC17) | VERIFY | OK | `pytest -q ...::test_logs_rejects_too_many_arguments` -> 1 passed |
| 2026-02-11 | VERIFY(Round17) | SYNTAX | OK | `bash -n scripts/fusion-status.sh scripts/fusion-hook-doctor.sh scripts/fusion-logs.sh` -> pass |
| 2026-02-11 | VERIFY(Round17) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py scripts/runtime/tests/test_fusion_hook_doctor_script.py scripts/runtime/tests/test_fusion_control_script_validation.py` -> 33 passed |
| 2026-02-11 | VERIFY(Round17) | TARGETED | OK | `pytest -q scripts/runtime/tests/test_fusion_status_script.py ... test_docs_freshness.py` -> 80 passed |
| 2026-02-11 | VERIFY(Round17) | FULL | OK | `pytest -q` -> 387 passed |
| 2026-02-11 | VERIFY(Round17) | PROBE | OK | `status --json --bad` 输出错误对象；`status --json --help` rc0；`hook-doctor --json --fix` 失败场景 `warn+fixed:false`；`logs 10 extra` 报 Too many arguments |
| 2026-02-11 | SCAN(Round18) | BASELINE | OK | `pytest -q` -> 387 passed；`bash -n scripts/*.sh` -> shell_syntax:OK；rust clippy/fmt 均通过 |
| 2026-02-11 | SCAN(Round18) | CLI_PROBE | OK | `fusion-start/status/hook-doctor/logs/git/achievements/stop-guard` 探针完成，锁定 CI+文档+审计三项缺口 |
| 2026-02-11 | PLAN(Round18) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round18.md` |
| 2026-02-11 | EXECUTE(Round18-TaskA18) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 2 failed（workflow 缺失） |
| 2026-02-11 | EXECUTE(Round18-TaskA18) | GREEN | OK | 新增 `.github/workflows/ci-contract-gates.yml`（shell/pytest/rust gates） |
| 2026-02-11 | EXECUTE(Round18-TaskA18) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 2 passed |
| 2026-02-11 | EXECUTE(Round18-TaskB18) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 2 failed（CLI_CONTRACT_MATRIX 缺失） |
| 2026-02-11 | EXECUTE(Round18-TaskB18) | GREEN | OK | 新增 `docs/CLI_CONTRACT_MATRIX.md`，覆盖 13 个脚本契约矩阵 |
| 2026-02-11 | EXECUTE(Round18-TaskB18) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 6 passed |
| 2026-02-11 | EXECUTE(Round18-TaskC18) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 3 failed（脚本不存在） |
| 2026-02-11 | EXECUTE(Round18-TaskC18) | GREEN | OK | 新增 `scripts/release-contract-audit.sh`，实现 `--help`/`--dry-run`/默认全门禁 |
| 2026-02-11 | EXECUTE(Round18-TaskC18) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 3 passed |
| 2026-02-11 | VERIFY(Round18) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> pass |
| 2026-02-11 | VERIFY(Round18) | TARGET_SCRIPT | OK | `pytest -q ...test_ci_contract_gates.py ...test_release_contract_audit_script.py ...test_docs_freshness.py` -> 11 passed, 22 subtests |
| 2026-02-11 | VERIFY(Round18) | TARGETED | OK | `pytest -q`(13-file targeted suite) -> 87 passed, 22 subtests |
| 2026-02-11 | VERIFY(Round18) | FULL | OK | `pytest -q` -> 394 passed, 22 subtests |
| 2026-02-11 | VERIFY(Round18) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | VERIFY(Round18) | DRY_RUN | OK | `./scripts/release-contract-audit.sh --dry-run` 输出完整命令清单 |
| 2026-02-11 | VERIFY(Round18) | RELEASE_AUDIT | OK | `./scripts/release-contract-audit.sh` 全门禁通过 |
| 2026-02-11 | VERIFY(Round18) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_docs_freshness.py` -> 11 passed, 22 subtests |
| 2026-02-11 | VERIFY(Round18) | FINAL_FULL | OK | `pytest -q` -> 394 passed, 22 subtests |
| 2026-02-11 | SCAN(Round19) | BASELINE | OK | `pytest -q` -> 394 passed, 22 subtests；`bash -n scripts/*.sh` + rust clippy/fmt 均通过 |
| 2026-02-11 | SCAN(Round19) | PROBE | WARN | `release-contract-audit --dry-run --fast/--skip-*` 不支持；runner `--suite contract` 误走 full；未知 suite 未报错 |
| 2026-02-11 | PLAN(Round19) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round19.md` |
| 2026-02-11 | EXECUTE(Round19-TaskA19) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 4 failed |
| 2026-02-11 | EXECUTE(Round19-TaskA19) | GREEN | OK | `release-contract-audit.sh` 增加 `--fast`/`--skip-rust`/`--skip-python` + step failure summary |
| 2026-02-11 | EXECUTE(Round19-TaskA19) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 7 passed |
| 2026-02-11 | EXECUTE(Round19-TaskB19) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 2 failed |
| 2026-02-11 | EXECUTE(Round19-TaskB19) | GREEN | OK | `regression_runner.py` 新增 contract suite + unknown suite reject |
| 2026-02-11 | EXECUTE(Round19-TaskB19) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 2 passed |
| 2026-02-11 | EXECUTE(Round19-TaskC19) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 4 failed（help exit code + CI/release 文档缺失） |
| 2026-02-11 | EXECUTE(Round19-TaskC19) | GREEN | OK | 更新 `CLI_CONTRACT_MATRIX` + `HOOKS_SETUP` + `README`(EN/ZH) CI/release 契约章节 |
| 2026-02-11 | EXECUTE(Round19-TaskC19) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 9 passed |
| 2026-02-11 | VERIFY(Round19) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> pass |
| 2026-02-11 | VERIFY(Round19) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_docs_freshness.py` -> 18 passed, 19 subtests |
| 2026-02-11 | VERIFY(Round19) | TARGETED | OK | `pytest -q`(14-file targeted suite) -> 96 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round19) | FULL | OK | `pytest -q` -> 403 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round19) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | VERIFY(Round19) | PROBE | OK | `release-contract-audit --dry-run --fast --skip-rust` & `--dry-run --skip-python` 输出符合预期 |
| 2026-02-11 | VERIFY(Round19) | CONTRACT_SUITE | OK | `python3 scripts/runtime/regression_runner.py --suite contract --min-pass-rate 0.99` -> 8/8 passed |
| 2026-02-11 | VERIFY(Round19) | RELEASE_AUDIT | OK | `./scripts/release-contract-audit.sh --fast` 全门禁通过 |
| 2026-02-11 | VERIFY(Round19) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_docs_freshness.py` -> 18 passed, 19 subtests |
| 2026-02-11 | VERIFY(Round19) | FINAL_FULL | OK | `pytest -q` -> 403 passed, 23 subtests |
| 2026-02-11 | SCAN(Round20) | BASELINE | OK | `pytest -q` -> 403 passed, 23 subtests；shell/rust 门禁通过 |
| 2026-02-11 | SCAN(Round20) | PROBE | WARN | `release-audit --json` 不支持；runner `--list-suites` 不支持；CI workflow 无 cache 步骤 |
| 2026-02-11 | PLAN(Round20) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round20.md` |
| 2026-02-11 | EXECUTE(Round20-TaskA20) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 1 failed (`--json`) |
| 2026-02-11 | EXECUTE(Round20-TaskA20) | GREEN | OK | release-audit 增加 `--json` 与 JSON summary 输出 |
| 2026-02-11 | EXECUTE(Round20-TaskA20) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 8 passed |
| 2026-02-11 | EXECUTE(Round20-TaskB20) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed (`--list-suites`) |
| 2026-02-11 | EXECUTE(Round20-TaskB20) | GREEN | OK | regression_runner 增加 `--list-suites` 分支 |
| 2026-02-11 | EXECUTE(Round20-TaskB20) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 3 passed |
| 2026-02-11 | EXECUTE(Round20-TaskC20) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（缺 cache 步骤） |
| 2026-02-11 | EXECUTE(Round20-TaskC20) | GREEN | OK | CI workflow 增加 pip cache + rust cache |
| 2026-02-11 | EXECUTE(Round20-TaskC20) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 3 passed |
| 2026-02-11 | VERIFY(Round20) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> pass |
| 2026-02-11 | VERIFY(Round20) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 14 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round20) | TARGETED | OK | `pytest -q`(14-file targeted suite) -> 99 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round20) | FULL | OK | `pytest -q` -> 406 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round20) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | VERIFY(Round20) | JSON_DRY_RUN | OK | `./scripts/release-contract-audit.sh --dry-run --json --fast --skip-rust` 输出 JSON 摘要 |
| 2026-02-11 | VERIFY(Round20) | LIST_SUITES | OK | `python3 scripts/runtime/regression_runner.py --list-suites` 输出 `phase1/phase2/contract/all` |
| 2026-02-11 | VERIFY(Round20) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 14 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round20) | FINAL_FULL | OK | `pytest -q` -> 406 passed, 23 subtests |
| 2026-02-11 | SCAN(Round21) | BASELINE | OK | `pytest -q` -> 406 passed, 23 subtests；shell/rust 门禁通过 |
| 2026-02-11 | SCAN(Round21) | PROBE | WARN | `release-audit --json-pretty` 不支持；runner `--list-suites --json` 不支持；文档缺失新命令 |
| 2026-02-11 | PLAN(Round21) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round21.md` |
| 2026-02-11 | EXECUTE(Round21-TaskA21) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 1 failed (`--json-pretty`) |
| 2026-02-11 | EXECUTE(Round21-TaskA21) | GREEN | OK | release-audit 增加 `--json-pretty` + JSON payload `json_pretty` flag |
| 2026-02-11 | EXECUTE(Round21-TaskA21) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 9 passed |
| 2026-02-11 | EXECUTE(Round21-TaskB21) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed (`--list-suites --json`) |
| 2026-02-11 | EXECUTE(Round21-TaskB21) | GREEN | OK | regression_runner 增加 `--json` 元数据输出，list-suites 支持 JSON |
| 2026-02-11 | EXECUTE(Round21-TaskB21) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 4 passed |
| 2026-02-11 | EXECUTE(Round21-TaskC21) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 2 failed（json-pretty/list-suites-json 文档缺失） |
| 2026-02-11 | EXECUTE(Round21-TaskC21) | GREEN | OK | 更新 `HOOKS_SETUP` + `README`(EN/ZH) + `CLI_CONTRACT_MATRIX` 机器模式命令 |
| 2026-02-11 | EXECUTE(Round21-TaskC21) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 11 passed |
| 2026-02-11 | VERIFY(Round21) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> pass |
| 2026-02-11 | VERIFY(Round21) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_docs_freshness.py` -> 24 passed, 19 subtests |
| 2026-02-11 | VERIFY(Round21) | TARGETED | OK | `pytest -q`(14-file targeted suite) -> 103 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round21) | FULL | OK | `pytest -q` -> 410 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round21) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | VERIFY(Round21) | JSON_PRETTY | OK | `./scripts/release-contract-audit.sh --dry-run --json --json-pretty --fast --skip-rust` 输出格式化 JSON |
| 2026-02-11 | VERIFY(Round21) | LIST_SUITES_JSON | OK | `python3 scripts/runtime/regression_runner.py --list-suites --json` 输出机器 payload |
| 2026-02-11 | VERIFY(Round21) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_docs_freshness.py` -> 24 passed, 19 subtests |
| 2026-02-11 | VERIFY(Round21) | FINAL_FULL | OK | `pytest -q` -> 410 passed, 23 subtests |
| 2026-02-11 | SCAN(Round22) | BASELINE | OK | `pytest -q` -> 410 passed, 23 subtests；shell/rust 门禁通过 |
| 2026-02-11 | SCAN(Round22) | PROBE | WARN | run JSON 缺少 steps/timing；runner suite JSON 缺失；CI 缺 machine smoke gate |
| 2026-02-11 | PLAN(Round22) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round22.md` |
| 2026-02-11 | EXECUTE(Round22-TaskA22) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 1 failed（缺 steps_executed） |
| 2026-02-11 | EXECUTE(Round22-TaskA22) | GREEN | OK | release-audit JSON 增加 `steps_executed`/`step_results`/`total_duration_ms` |
| 2026-02-11 | EXECUTE(Round22-TaskA22) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 10 passed |
| 2026-02-11 | EXECUTE(Round22-TaskB22) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（suite --json 输出文本） |
| 2026-02-11 | EXECUTE(Round22-TaskB22) | GREEN | OK | runner suite 执行路径支持 `--json` 汇总输出 |
| 2026-02-11 | EXECUTE(Round22-TaskB22) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round22-TaskC22) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（machine smoke command 缺失） |
| 2026-02-11 | EXECUTE(Round22-TaskC22) | GREEN | OK | workflow 新增 machine mode smoke step |
| 2026-02-11 | EXECUTE(Round22-TaskC22) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 4 passed |
| 2026-02-11 | VERIFY(Round22) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> pass |
| 2026-02-11 | VERIFY(Round22) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 19 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round22) | TARGETED | OK | `pytest -q`(14-file targeted suite) -> 106 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round22) | FULL | OK | `pytest -q` -> 413 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round22) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | VERIFY(Round22) | JSON_RUN | OK | `./scripts/release-contract-audit.sh --json --skip-python --skip-rust` 输出 steps/timing |
| 2026-02-11 | VERIFY(Round22) | SUITE_JSON | OK | `python3 scripts/runtime/regression_runner.py --suite contract --json --min-pass-rate 0.99` 输出 JSON 汇总 |
| 2026-02-11 | VERIFY(Round22) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 19 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round22) | FINAL_FULL | OK | `pytest -q` -> 413 passed, 23 subtests |
| 2026-02-11 | SCAN(Round23) | BASELINE | OK | `pytest -q` -> 413 passed, 23 subtests；shell/rust 门禁通过 |
| 2026-02-11 | SCAN(Round23) | PROBE | WARN | step_results 缺 step/timestamps；runner suite json 缺 scenario details；CI 未跑 suite json smoke |
| 2026-02-11 | PLAN(Round23) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round23.md` |
| 2026-02-11 | EXECUTE(Round23-TaskA23) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 1 failed（缺 `step`） |
| 2026-02-11 | EXECUTE(Round23-TaskA23) | GREEN | OK | release-audit step_results 增加 `step/started_at_ms/finished_at_ms` |
| 2026-02-11 | EXECUTE(Round23-TaskA23) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 10 passed |
| 2026-02-11 | EXECUTE(Round23-TaskB23) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（缺 scenario_results） |
| 2026-02-11 | EXECUTE(Round23-TaskB23) | GREEN | OK | runner suite JSON 增加 `scenario_results` 与 `failed_scenarios` |
| 2026-02-11 | EXECUTE(Round23-TaskB23) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round23-TaskC23) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（缺 suite json smoke） |
| 2026-02-11 | EXECUTE(Round23-TaskC23) | GREEN | OK | CI workflow machine smoke 增加 `--suite contract --json` |
| 2026-02-11 | EXECUTE(Round23-TaskC23) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 4 passed |
| 2026-02-11 | VERIFY(Round23) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> pass |
| 2026-02-11 | VERIFY(Round23) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 19 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round23) | TARGETED | OK | `pytest -q`(14-file targeted suite) -> 106 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round23) | FULL | OK | `pytest -q` -> 413 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round23) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | VERIFY(Round23) | JSON_RUN | OK | `./scripts/release-contract-audit.sh --json --skip-python --skip-rust` 输出 step/time 扩展字段 |
| 2026-02-11 | VERIFY(Round23) | SUITE_JSON | OK | `python3 scripts/runtime/regression_runner.py --suite contract --json --min-pass-rate 0.99` 输出 scenario details |
| 2026-02-11 | VERIFY(Round23) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 19 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round23) | FINAL_FULL | OK | `pytest -q` -> 413 passed, 23 subtests |
| 2026-02-11 | EXECUTE(Round23-TaskC23) | ALIGN | OK | machine smoke 追加 `regression_runner.py --suite contract --json --min-pass-rate 0.99` 并同步测试断言 |
| 2026-02-11 | VERIFY(Round23) | FINAL_RECHECK | OK | `pytest -q` -> 413 passed, 23 subtests |

| 2026-02-11 | SCAN(Round24) | BASELINE | OK | `pytest -q` -> 413 passed, 23 subtests；shell/rust 门禁通过 |
| 2026-02-11 | SCAN(Round24) | PROBE | WARN | release-audit 缺 step exit_code；runner json 缺 longest_scenario；CI 缺 json artifacts |
| 2026-02-11 | PLAN(Round24) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round24.md` |
| 2026-02-11 | EXECUTE(Round24-TaskA24) | CARRY | OK | 已在前序批次完成并被 round24 target suite 覆盖验证 |
| 2026-02-11 | EXECUTE(Round24-TaskC24) | CARRY | OK | 已在前序批次完成并被 round24 target suite 覆盖验证 |
| 2026-02-11 | EXECUTE(Round24-TaskB24) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py -q` -> 1 failed（缺 `longest_scenario`） |
| 2026-02-11 | EXECUTE(Round24-TaskB24) | GREEN | OK | `regression_runner.py` JSON payload 增加 `longest_scenario{name,duration_ms}` |
| 2026-02-11 | EXECUTE(Round24-TaskB24) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py -q` -> 5 passed |
| 2026-02-11 | VERIFY(Round24) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> bash -n OK |
| 2026-02-11 | VERIFY(Round24) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 21 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round24) | FULL | OK | `pytest -q` -> 415 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round24) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-11 | VERIFY(Round24) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 21 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round24) | FINAL_FULL | OK | `pytest -q` -> 415 passed, 23 subtests |

| 2026-02-11 | SCAN(Round25) | BASELINE | OK | `pytest -q` -> 415 passed, 23 subtests |
| 2026-02-11 | SCAN(Round25) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> shell-syntax:OK |
| 2026-02-11 | SCAN(Round25) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | SCAN(Round25) | PROBE | WARN | release-audit 缺 failed_steps；runner 缺 fastest_scenario；CI 缺 schema smoke |
| 2026-02-11 | PLAN(Round25) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round25.md` |
| 2026-02-11 | EXECUTE(Round25-TaskA25) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 2 failed（缺 failed_steps 字段） |
| 2026-02-11 | EXECUTE(Round25-TaskA25) | GREEN | OK | `release-contract-audit.sh` payload 增加 `failed_steps` 与 `failed_steps_count` |
| 2026-02-11 | EXECUTE(Round25-TaskA25) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 11 passed |
| 2026-02-11 | EXECUTE(Round25-TaskB25) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（缺 fastest_scenario） |
| 2026-02-11 | EXECUTE(Round25-TaskB25) | GREEN | OK | `regression_runner.py` payload 增加 `fastest_scenario{name,duration_ms}` |
| 2026-02-11 | EXECUTE(Round25-TaskB25) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round25-TaskC25) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（缺 schema smoke） |
| 2026-02-11 | EXECUTE(Round25-TaskC25) | GREEN | OK | CI workflow machine smoke 增加 `python3 - <<'PY'` schema 校验 |
| 2026-02-11 | EXECUTE(Round25-TaskC25) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 6 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round25) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> bash -n OK |
| 2026-02-11 | VERIFY(Round25) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round25) | FULL | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round25) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-11 | VERIFY(Round25) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round25) | FINAL_FULL | OK | `pytest -q` -> 416 passed, 23 subtests |

| 2026-02-11 | SCAN(Round26) | BASELINE | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | SCAN(Round26) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> shell-syntax:OK |
| 2026-02-11 | SCAN(Round26) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | SCAN(Round26) | PROBE | WARN | release-audit 缺 failed_commands；runner 缺 scenario_count_by_result；CI 缺 multi-schema smoke |
| 2026-02-11 | PLAN(Round26) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round26.md` |
| 2026-02-11 | EXECUTE(Round26-TaskA26) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 2 failed（缺 failed_commands） |
| 2026-02-11 | EXECUTE(Round26-TaskA26) | GREEN | OK | `release-contract-audit.sh` payload 增加 `failed_commands` |
| 2026-02-11 | EXECUTE(Round26-TaskA26) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 11 passed |
| 2026-02-11 | EXECUTE(Round26-TaskB26) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（缺 scenario_count_by_result） |
| 2026-02-11 | EXECUTE(Round26-TaskB26) | GREEN | OK | `regression_runner.py` payload 增加 `scenario_count_by_result` |
| 2026-02-11 | EXECUTE(Round26-TaskB26) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round26-TaskC26) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（缺 multi-schema smoke） |
| 2026-02-11 | EXECUTE(Round26-TaskC26) | GREEN | OK | workflow machine smoke 增加 release/suites/contract JSON schema 校验 |
| 2026-02-11 | EXECUTE(Round26-TaskC26) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 6 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round26) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> bash -n OK |
| 2026-02-11 | VERIFY(Round26) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round26) | FULL | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round26) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-11 | VERIFY(Round26) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round26) | FINAL_FULL | OK | `pytest -q` -> 416 passed, 23 subtests |

| 2026-02-11 | SCAN(Round27) | BASELINE | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | SCAN(Round27) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> shell-syntax:OK |
| 2026-02-11 | SCAN(Round27) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | SCAN(Round27) | PROBE | WARN | release-audit 缺 failed_commands_count；runner 缺 duration_stats；CI required keys 未同步 |
| 2026-02-11 | PLAN(Round27) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round27.md` |
| 2026-02-11 | EXECUTE(Round27-TaskA27) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 2 failed（缺 failed_commands_count） |
| 2026-02-11 | EXECUTE(Round27-TaskA27) | GREEN | OK | `release-contract-audit.sh` payload 增加 `failed_commands_count` |
| 2026-02-11 | EXECUTE(Round27-TaskA27) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 11 passed |
| 2026-02-11 | EXECUTE(Round27-TaskB27) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（缺 duration_stats） |
| 2026-02-11 | EXECUTE(Round27-TaskB27) | GREEN | OK | `regression_runner.py` payload 增加 `duration_stats` |
| 2026-02-11 | EXECUTE(Round27-TaskB27) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round27-TaskC27) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（缺 duration_stats/failed_commands_count required key） |
| 2026-02-11 | EXECUTE(Round27-TaskC27) | GREEN | OK | workflow schema smoke required keys 增加 `duration_stats` 与 `failed_commands_count` |
| 2026-02-11 | EXECUTE(Round27-TaskC27) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 6 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round27) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> bash -n OK |
| 2026-02-11 | VERIFY(Round27) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round27) | FULL | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round27) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-11 | VERIFY(Round27) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round27) | FINAL_FULL | OK | `pytest -q` -> 416 passed, 23 subtests |

| 2026-02-11 | SCAN(Round28) | BASELINE | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | SCAN(Round28) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> shell-syntax:OK |
| 2026-02-11 | SCAN(Round28) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | SCAN(Round28) | PROBE | WARN | release-audit 缺 success/commands count；runner 缺 failed_rate；CI required keys 未同步 |
| 2026-02-11 | PLAN(Round28) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round28.md` |
| 2026-02-11 | EXECUTE(Round28-TaskA28) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 2 failed（缺 success_steps_count/commands_count） |
| 2026-02-11 | EXECUTE(Round28-TaskA28) | GREEN | OK | `release-contract-audit.sh` payload 增加 `success_steps_count` 与 `commands_count` |
| 2026-02-11 | EXECUTE(Round28-TaskA28) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 11 passed |
| 2026-02-11 | EXECUTE(Round28-TaskB28) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（缺 failed_rate） |
| 2026-02-11 | EXECUTE(Round28-TaskB28) | GREEN | OK | `regression_runner.py` payload 增加 `failed_rate` |
| 2026-02-11 | EXECUTE(Round28-TaskB28) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round28-TaskC28) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（缺 failed_rate/success_steps_count/commands_count required key） |
| 2026-02-11 | EXECUTE(Round28-TaskC28) | GREEN | OK | workflow schema smoke required keys 增加 `failed_rate/success_steps_count/commands_count` |
| 2026-02-11 | EXECUTE(Round28-TaskC28) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 6 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round28) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> bash -n OK |
| 2026-02-11 | VERIFY(Round28) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round28) | FULL | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round28) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-11 | VERIFY(Round28) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round28) | FINAL_FULL | OK | `pytest -q` -> 416 passed, 23 subtests |

| 2026-02-11 | SCAN(Round29) | BASELINE | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | SCAN(Round29) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> shell-syntax:OK |
| 2026-02-11 | SCAN(Round29) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | SCAN(Round29) | PROBE | WARN | release-audit 缺 success/failed rates；runner 缺 success_rate；CI required keys 未同步 |
| 2026-02-11 | PLAN(Round29) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round29.md` |
| 2026-02-11 | EXECUTE(Round29-TaskA29) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 2 failed（缺 success_rate/failed_rate） |
| 2026-02-11 | EXECUTE(Round29-TaskA29) | GREEN | OK | `release-contract-audit.sh` payload 增加 `success_rate/failed_rate` |
| 2026-02-11 | EXECUTE(Round29-TaskA29) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 11 passed |
| 2026-02-11 | EXECUTE(Round29-TaskB29) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（缺 success_rate） |
| 2026-02-11 | EXECUTE(Round29-TaskB29) | GREEN | OK | `regression_runner.py` payload 增加 `success_rate` |
| 2026-02-11 | EXECUTE(Round29-TaskB29) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round29-TaskC29) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（缺 success_rate required key） |
| 2026-02-11 | EXECUTE(Round29-TaskC29) | GREEN | OK | workflow schema smoke required keys 增加 release/runner rates |
| 2026-02-11 | EXECUTE(Round29-TaskC29) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 6 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round29) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> bash -n OK |
| 2026-02-11 | VERIFY(Round29) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round29) | FULL | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round29) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-11 | VERIFY(Round29) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round29) | FINAL_FULL | OK | `pytest -q` -> 416 passed, 23 subtests |

| 2026-02-11 | SCAN(Round30) | BASELINE | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | SCAN(Round30) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> shell-syntax:OK |
| 2026-02-11 | SCAN(Round30) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | SCAN(Round30) | PROBE | WARN | release-audit 缺 error_step_count；runner 缺 success/failure count；CI required keys 未同步 |
| 2026-02-11 | PLAN(Round30) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round30.md` |
| 2026-02-11 | EXECUTE(Round30-TaskA30) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 2 failed（缺 error_step_count） |
| 2026-02-11 | EXECUTE(Round30-TaskA30) | GREEN | OK | `release-contract-audit.sh` payload 增加 `error_step_count` |
| 2026-02-11 | EXECUTE(Round30-TaskA30) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 11 passed |
| 2026-02-11 | EXECUTE(Round30-TaskB30) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（缺 success_count/failure_count） |
| 2026-02-11 | EXECUTE(Round30-TaskB30) | GREEN | OK | `regression_runner.py` payload 增加 `success_count/failure_count` |
| 2026-02-11 | EXECUTE(Round30-TaskB30) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round30-TaskC30) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（缺 success_count/failure_count/error_step_count required key） |
| 2026-02-11 | EXECUTE(Round30-TaskC30) | GREEN | OK | workflow schema smoke required keys 增加 `error_step_count/success_count/failure_count` |
| 2026-02-11 | EXECUTE(Round30-TaskC30) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 6 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round30) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> bash -n OK |
| 2026-02-11 | VERIFY(Round30) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round30) | FULL | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round30) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-11 | VERIFY(Round30) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round30) | FINAL_FULL | OK | `pytest -q` -> 416 passed, 23 subtests |

| 2026-02-11 | SCAN(Round31) | BASELINE | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | SCAN(Round31) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> shell-syntax:OK |
| 2026-02-11 | SCAN(Round31) | RUST | OK | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> pass |
| 2026-02-11 | SCAN(Round31) | PROBE | WARN | release-audit 缺 command rates；runner 缺 total_scenarios；CI required keys 未同步 |
| 2026-02-11 | PLAN(Round31) | 写入执行计划 | OK | `docs/plans/2026-02-11-repo-gap-priority-round31.md` |
| 2026-02-11 | EXECUTE(Round31-TaskA31) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 2 failed（缺 success_command_rate/failed_command_rate） |
| 2026-02-11 | EXECUTE(Round31-TaskA31) | GREEN | OK | `release-contract-audit.sh` payload 增加 `success_command_rate/failed_command_rate` |
| 2026-02-11 | EXECUTE(Round31-TaskA31) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 11 passed |
| 2026-02-11 | EXECUTE(Round31-TaskB31) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（缺 total_scenarios） |
| 2026-02-11 | EXECUTE(Round31-TaskB31) | GREEN | OK | `regression_runner.py` payload 增加 `total_scenarios` |
| 2026-02-11 | EXECUTE(Round31-TaskB31) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-11 | EXECUTE(Round31-TaskC31) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（缺 total_scenarios/command_rates required key） |
| 2026-02-11 | EXECUTE(Round31-TaskC31) | GREEN | OK | workflow schema smoke required keys 增加 `total_scenarios/success_command_rate/failed_command_rate` |
| 2026-02-11 | EXECUTE(Round31-TaskC31) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 6 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round31) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> bash -n OK |
| 2026-02-11 | VERIFY(Round31) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round31) | FULL | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-11 | VERIFY(Round31) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-11 | VERIFY(Round31) | FINAL_TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests |
| 2026-02-11 | VERIFY(Round31) | FINAL_FULL | OK | `pytest -q` -> 416 passed, 23 subtests |
| 2026-02-12 | SCAN(Round32) | BASELINE | OK | `bash -n scripts/*.sh` -> shell-syntax:OK; `pytest -q` -> 446 passed, 23 subtests; rust clippy/fmt 均通过 |
| 2026-02-12 | SCAN(Round32) | PROBE | WARN | release JSON 缺 `schema_version/step_rate_basis/command_rate_basis`; runner JSON 缺 `schema_version/rate_basis`; CI workflow 未校验 basis 一致性 |
| 2026-02-12 | PLAN(Round32) | 写入执行计划 | OK | `docs/plans/2026-02-12-repo-gap-priority-round32.md` |
| 2026-02-12 | EXECUTE(Round32-TaskA32) | RED | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 2 failed（缺 schema/basis 字段） |
| 2026-02-12 | EXECUTE(Round32-TaskA32) | GREEN | OK | `release-contract-audit.sh` payload 增加 `schema_version/step_rate_basis/command_rate_basis` |
| 2026-02-12 | EXECUTE(Round32-TaskA32) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py` -> 11 passed |
| 2026-02-12 | EXECUTE(Round32-TaskB32) | RED | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 1 failed（缺 schema_version/rate_basis） |
| 2026-02-12 | EXECUTE(Round32-TaskB32) | GREEN | OK | `regression_runner.py` payload 增加 `schema_version/rate_basis` |
| 2026-02-12 | EXECUTE(Round32-TaskB32) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_regression_runner_contract_suite.py` -> 5 passed |
| 2026-02-12 | EXECUTE(Round32-TaskC32) | RED | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 1 failed（workflow 缺 schema/basis 校验） |
| 2026-02-12 | EXECUTE(Round32-TaskC32) | GREEN | OK | workflow required keys 增加 schema/basis，并新增 basis 一致性校验 |
| 2026-02-12 | EXECUTE(Round32-TaskC32) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_ci_contract_gates.py` -> 6 passed, 4 subtests passed |
| 2026-02-12 | VERIFY(Round32) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> shell-syntax:OK |
| 2026-02-12 | VERIFY(Round32) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 22 passed, 4 subtests passed |
| 2026-02-12 | VERIFY(Round32) | FULL | OK | `pytest -q` -> 446 passed, 23 subtests passed |
| 2026-02-12 | VERIFY(Round32) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-12 | VERIFY(Round32) | PROBE_RELEASE | OK | `release-contract-audit --dry-run --json` -> `v1 0 2` (`schema_version step_rate_basis command_rate_basis`) |
| 2026-02-12 | VERIFY(Round32) | PROBE_RUNNER | OK | `regression_runner --suite contract --json` -> `v1 8 8` (`schema_version rate_basis total_scenarios`) |
| 2026-02-13 | SCAN(Round33) | QUICK_SCAN | OK | `rg TODO/FIXME` 无真实待办；scripts 均有测试引用；contract probes keys 完整 |
| 2026-02-13 | SCAN(Round33) | BASELINE | OK | `bash -n scripts/*.sh` -> shell-syntax:OK；`pytest -q` -> 446 passed, 23 subtests passed；rust clippy/fmt 通过 |
| 2026-02-13 | SCAN(Round33) | PROBE | WARN | docs/CLI_CONTRACT_MATRIX + README(EN/ZH) 未提及 `schema_version/step_rate_basis/command_rate_basis/rate_basis` |
| 2026-02-13 | PLAN(Round33) | 写入执行计划 | OK | `docs/plans/2026-02-13-repo-gap-priority-round33.md` |
| 2026-02-13 | EXECUTE(Round33-TaskA33) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::...::test_cli_contract_matrix_mentions_schema_and_basis_fields` -> 1 failed（缺 schema/basis 文案） |
| 2026-02-13 | EXECUTE(Round33-TaskA33) | GREEN | OK | `docs/CLI_CONTRACT_MATRIX.md` 补齐 release/runner schema+basis 说明 |
| 2026-02-13 | EXECUTE(Round33-TaskA33) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::...::test_cli_contract_matrix_mentions_schema_and_basis_fields` -> 1 passed |
| 2026-02-13 | EXECUTE(Round33-TaskB33) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::...::test_readme_mentions_machine_schema_and_basis_fields` -> 1 failed |
| 2026-02-13 | EXECUTE(Round33-TaskB33) | GREEN | OK | `README.md` 增加 machine JSON key highlights（schema/basis） |
| 2026-02-13 | EXECUTE(Round33-TaskB33) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::...::test_readme_mentions_machine_schema_and_basis_fields` -> 1 passed |
| 2026-02-13 | EXECUTE(Round33-TaskC33) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::...::test_readme_zh_cn_mentions_machine_schema_and_basis_fields` -> 1 failed |
| 2026-02-13 | EXECUTE(Round33-TaskC33) | GREEN | OK | `README.zh-CN.md` 增加机器 JSON 字段要点（schema/basis） |
| 2026-02-13 | EXECUTE(Round33-TaskC33) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::...::test_readme_zh_cn_mentions_machine_schema_and_basis_fields` -> 1 passed |
| 2026-02-13 | VERIFY(Round33) | SYNTAX | OK | `bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> shell-syntax:OK |
| 2026-02-13 | VERIFY(Round33) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 36 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round33) | FULL | OK | `pytest -q` -> 449 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round33) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-13 | SCAN(Round34) | BASELINE | OK | `bash -n scripts/*.sh` -> shell-syntax:OK；`pytest -q` -> 449 passed, 23 subtests passed；rust clippy/fmt 通过 |
| 2026-02-13 | SCAN(Round34) | PROBE | WARN | HOOKS_SETUP 缺 schema/basis 字段文案；CLI matrix 缺 machine required keys 段落；README EN/ZH 缺 step/command basis 分母显式语义 |
| 2026-02-13 | SCAN(Round34) | COVERAGE_MAP | OK | `for f in scripts/*.sh ...` 无 MISSING_SCRIPT_REF，脚本测试引用完备 |
| 2026-02-13 | PLAN(Round34) | 写入执行计划 | OK | `docs/plans/2026-02-13-repo-gap-priority-round34.md` |
| 2026-02-13 | EXECUTE(Round34-TaskA34) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_machine_schema_and_basis_fields` -> 1 failed (`schema_version` missing in HOOKS_SETUP) |
| 2026-02-13 | EXECUTE(Round34-TaskA34) | GREEN | OK | `docs/HOOKS_SETUP.md` 增加 machine JSON key highlights（schema/basis） |
| 2026-02-13 | EXECUTE(Round34-TaskA34) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_machine_schema_and_basis_fields` -> 1 passed |
| 2026-02-13 | EXECUTE(Round34-TaskA34) | CHECKPOINT | OK | `bash -n scripts/*.sh` -> shell-syntax:OK |
| 2026-02-13 | EXECUTE(Round34-TaskB34) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_has_machine_required_keys_note` -> 1 failed（缺 Required machine JSON keys 段落） |
| 2026-02-13 | EXECUTE(Round34-TaskB34) | GREEN | OK | `docs/CLI_CONTRACT_MATRIX.md` 新增 Required machine JSON keys (minimum) |
| 2026-02-13 | EXECUTE(Round34-TaskB34) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_has_machine_required_keys_note` -> 1 passed |
| 2026-02-13 | EXECUTE(Round34-TaskB34) | CHECKPOINT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 16 passed, 19 subtests passed |
| 2026-02-13 | EXECUTE(Round34-TaskC34) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_explain_basis_denominators` -> 1 failed（README 缺 step/command basis 分母语义） |
| 2026-02-13 | EXECUTE(Round34-TaskC34) | GREEN | OK | `README.md` + `README.zh-CN.md` 增加 `step_rate_basis=total_steps`/`command_rate_basis=total_commands` 文案 |
| 2026-02-13 | EXECUTE(Round34-TaskC34) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_explain_basis_denominators` -> 1 passed |
| 2026-02-13 | EXECUTE(Round34-TaskC34) | CHECKPOINT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 39 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round34) | SYNTAX | OK | `bash -n scripts/*.sh` -> shell-syntax:OK |
| 2026-02-13 | VERIFY(Round34) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 39 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round34) | FULL | OK | `pytest -q` -> 452 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round34) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-13 | SCAN(Round35) | BASELINE | OK | `bash -n scripts/*.sh` -> shell-syntax:OK；`pytest -q` -> 452 passed, 23 subtests passed；rust clippy/fmt 通过 |
| 2026-02-13 | SCAN(Round35) | DOCS_PROBE | WARN | HOOKS_SETUP 缺 step/command basis 分母语义；CLI matrix 缺 `schema_version=v1`；README EN/ZH 缺 CI machine artifact 文件示例 |
| 2026-02-13 | SCAN(Round35) | DOCS_FRESHNESS | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 17 passed, 19 subtests passed |
| 2026-02-13 | PLAN(Round35) | 写入执行计划 | OK | `docs/plans/2026-02-13-repo-gap-priority-round35.md` |
| 2026-02-13 | EXECUTE(Round35-TaskA35) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_explains_basis_denominators` -> 1 failed（缺 step/command basis 分母语义） |
| 2026-02-13 | EXECUTE(Round35-TaskA35) | GREEN | OK | `docs/HOOKS_SETUP.md` 新增 denominator semantics 文案 |
| 2026-02-13 | EXECUTE(Round35-TaskA35) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_explains_basis_denominators` -> 1 passed |
| 2026-02-13 | EXECUTE(Round35-TaskA35) | CHECKPOINT | OK | `bash -n scripts/*.sh` -> shell-syntax:OK |
| 2026-02-13 | EXECUTE(Round35-TaskB35) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_mentions_schema_version_v1` -> 1 failed（缺 schema_version=v1 文案） |
| 2026-02-13 | EXECUTE(Round35-TaskB35) | GREEN | OK | `docs/CLI_CONTRACT_MATRIX.md` 新增 `Current schema contract: schema_version=v1` |
| 2026-02-13 | EXECUTE(Round35-TaskB35) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_mentions_schema_version_v1` -> 1 passed |
| 2026-02-13 | EXECUTE(Round35-TaskB35) | CHECKPOINT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 19 passed, 19 subtests passed |
| 2026-02-13 | EXECUTE(Round35-TaskC35) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_ci_machine_artifacts` -> 1 failed（README EN/ZH 缺 artifact 文件示例） |
| 2026-02-13 | EXECUTE(Round35-TaskC35) | GREEN | OK | `README.md` + `README.zh-CN.md` 新增 `/tmp/release-audit-dry-run.json` 与 `/tmp/runner-contract.json` 文案 |
| 2026-02-13 | EXECUTE(Round35-TaskC35) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_ci_machine_artifacts` -> 1 passed |
| 2026-02-13 | EXECUTE(Round35-TaskC35) | CHECKPOINT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 42 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round35) | SYNTAX | OK | `bash -n scripts/*.sh` -> shell-syntax:OK |
| 2026-02-13 | VERIFY(Round35) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 42 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round35) | FULL | OK | `pytest -q` -> 455 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round35) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-13 | SCAN(Round36) | BASELINE | OK | `bash -n scripts/*.sh` -> shell-syntax:OK；`pytest -q` -> 455 passed, 23 subtests passed；rust clippy/fmt 通过 |
| 2026-02-13 | SCAN(Round36) | DOCS_FRESHNESS | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 20 passed, 19 subtests passed |
| 2026-02-13 | SCAN(Round36) | DOCS_PROBE | WARN | HOOKS_SETUP 缺 artifact 文件说明；CLI matrix Notes 缺 artifact 文件说明；README EN/ZH 缺 `/tmp/runner-suites.json` |
| 2026-02-13 | PLAN(Round36) | 写入执行计划 | OK | `docs/plans/2026-02-13-repo-gap-priority-round36.md` |
| 2026-02-13 | EXECUTE(Round36-TaskA36) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_ci_machine_artifacts` -> 1 failed（HOOKS_SETUP 缺 artifact 文件示例） |
| 2026-02-13 | EXECUTE(Round36-TaskA36) | GREEN | OK | `docs/HOOKS_SETUP.md` 新增 CI machine artifact examples（release-audit/runner-contract） |
| 2026-02-13 | EXECUTE(Round36-TaskA36) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_ci_machine_artifacts` -> 1 passed |
| 2026-02-13 | EXECUTE(Round36-TaskA36) | CHECKPOINT | OK | `bash -n scripts/*.sh` -> shell-syntax:OK |
| 2026-02-13 | EXECUTE(Round36-TaskB36) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_notes_mention_ci_machine_artifacts` -> 1 failed（matrix Notes 缺 artifact 文件说明） |
| 2026-02-13 | EXECUTE(Round36-TaskB36) | GREEN | OK | `docs/CLI_CONTRACT_MATRIX.md` Notes 新增 artifact 三文件说明 |
| 2026-02-13 | EXECUTE(Round36-TaskB36) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_notes_mention_ci_machine_artifacts` -> 1 passed |
| 2026-02-13 | EXECUTE(Round36-TaskB36) | CHECKPOINT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 22 passed, 19 subtests passed |
| 2026-02-13 | EXECUTE(Round36-TaskC36) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_runner_suites_artifact` -> 1 failed（README EN/ZH 缺 runner-suites artifact） |
| 2026-02-13 | EXECUTE(Round36-TaskC36) | GREEN | OK | `README.md` + `README.zh-CN.md` 新增 `/tmp/runner-suites.json` |
| 2026-02-13 | EXECUTE(Round36-TaskC36) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_runner_suites_artifact` -> 1 passed |
| 2026-02-13 | EXECUTE(Round36-TaskC36) | CHECKPOINT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 45 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round36) | SYNTAX | OK | `bash -n scripts/*.sh` -> shell-syntax:OK |
| 2026-02-13 | VERIFY(Round36) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 45 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round36) | FULL | OK | `pytest -q` -> 458 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round36) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-13 | SCAN(Round37) | BASELINE | OK | `bash -n scripts/*.sh` -> shell-syntax:OK；`pytest -q` -> 455 passed, 23 subtests passed；rust clippy/fmt 通过 |
| 2026-02-13 | SCAN(Round37) | DOCS_FRESHNESS | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 23 passed, 19 subtests passed |
| 2026-02-13 | SCAN(Round37) | DOCS_PROBE | WARN | HOOKS_SETUP 缺 `/tmp/runner-suites.json` 与 `schema_version=v1`；README EN/ZH 缺 `schema_version=v1` |
| 2026-02-13 | PLAN(Round37) | 写入执行计划 | OK | `docs/plans/2026-02-13-repo-gap-priority-round37.md` |
| 2026-02-13 | EXECUTE(Round37-TaskA37) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_runner_suites_artifact` -> 1 failed（HOOKS_SETUP 缺 `/tmp/runner-suites.json`） |
| 2026-02-13 | EXECUTE(Round37-TaskA37) | GREEN | OK | `docs/HOOKS_SETUP.md` artifact examples 新增 `/tmp/runner-suites.json` |
| 2026-02-13 | EXECUTE(Round37-TaskA37) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_runner_suites_artifact` -> 1 passed |
| 2026-02-13 | EXECUTE(Round37-TaskA37) | CHECKPOINT | OK | `bash -n scripts/*.sh` -> shell-syntax:OK |
| 2026-02-13 | EXECUTE(Round37-TaskB37) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_schema_version_v1` -> 1 failed（HOOKS_SETUP 缺 schema_version=v1） |
| 2026-02-13 | EXECUTE(Round37-TaskB37) | GREEN | OK | `docs/HOOKS_SETUP.md` machine JSON key highlights 新增 `schema_version=v1` 契约行 |
| 2026-02-13 | EXECUTE(Round37-TaskB37) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_hooks_setup_mentions_schema_version_v1` -> 1 passed |
| 2026-02-13 | EXECUTE(Round37-TaskB37) | CHECKPOINT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 25 passed, 19 subtests passed |
| 2026-02-13 | EXECUTE(Round37-TaskC37) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_schema_version_v1` -> 1 failed（README EN/ZH 缺 schema_version=v1） |
| 2026-02-13 | EXECUTE(Round37-TaskC37) | GREEN | OK | `README.md` + `README.zh-CN.md` 新增 `schema_version=v1` 契约文案 |
| 2026-02-13 | EXECUTE(Round37-TaskC37) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_readme_en_zh_mention_schema_version_v1` -> 1 passed |
| 2026-02-13 | EXECUTE(Round37-TaskC37) | CHECKPOINT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 48 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round37) | SYNTAX | OK | `bash -n scripts/*.sh` -> shell-syntax:OK |
| 2026-02-13 | VERIFY(Round37) | TARGET_SCRIPT | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py scripts/runtime/tests/test_release_contract_audit_script.py scripts/runtime/tests/test_regression_runner_contract_suite.py scripts/runtime/tests/test_ci_contract_gates.py` -> 48 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round37) | FULL | OK | `pytest -q` -> 461 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round37) | RUST | OK | `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `(cd rust && cargo fmt --all -- --check)` -> pass |
| 2026-02-13 | SCAN(Round38) | BASELINE | OK | `bash -n scripts/*.sh` -> shell-syntax:OK; `pytest -q` -> 461 passed, 23 subtests passed; rust clippy/fmt 通过 |
| 2026-02-13 | SCAN(Round38) | DOCS_PROBE | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 26 passed, 19 subtests passed |
| 2026-02-13 | SCAN(Round38) | TEAM_STATE | WARN | `.fusion/task_plan.md` 当前 `DECOMPOSE (3/8)`；Task2/Task3 仍 `PENDING` |
| 2026-02-13 | PLAN(Round38) | WRITE_PLAN | OK | `docs/plans/2026-02-13-repo-gap-priority-round38.md` |
| 2026-02-13 | EXECUTE(Round38-TaskA38) | RED | IN_PROGRESS | 准备新增 double-backend failure 报告测试 |
| 2026-02-13 | EXECUTE(Round38-TaskA38) | RED | OK | `pytest -q ...test_double_backend_failure_writes_backend_failure_report ...test_success_clears_stale_backend_failure_report` -> 2 failed |
| 2026-02-13 | EXECUTE(Round38-TaskA38) | GREEN | OK | `fusion-codeagent.sh` 增加 `write_backend_failure_report` + 成功路径清理 stale backend report |
| 2026-02-13 | EXECUTE(Round38-TaskA38) | VERIFY | OK | 两个新增测试 -> 2 passed；`pytest -q scripts/runtime/tests/test_fusion_codeagent_script.py` -> 14 passed |
| 2026-02-13 | EXECUTE(Round38-TaskB38) | RED | OK | `pytest -q ...test_status_json_includes_backend_failure_summary ...test_status_prints_backend_failure_report` -> 2 failed |
| 2026-02-13 | EXECUTE(Round38-TaskB38) | GREEN | OK | `fusion-status.sh` JSON 增加 `backend_*` 字段 + human 增加 `## Backend Failure Report` |
| 2026-02-13 | EXECUTE(Round38-TaskB38) | VERIFY | OK | 新增两测 -> 2 passed；`pytest -q scripts/runtime/tests/test_fusion_status_script.py` -> 20 passed |
| 2026-02-13 | EXECUTE(Round38-TaskC38) | RED | OK | `pytest -q ...test_readme_en_zh_mention_backend_failure_report` -> 1 failed |
| 2026-02-13 | EXECUTE(Round38-TaskC38) | GREEN | OK | `README.md` / `README.zh-CN.md` 补齐 `.fusion/backend_failure_report.json` 与状态区块说明 |
| 2026-02-13 | EXECUTE(Round38-TaskC38) | VERIFY | OK | 单测 -> 1 passed；`pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 27 passed, 19 subtests passed |
| 2026-02-13 | VERIFY(Round38) | SYNTAX | OK | `bash -n scripts/*.sh` -> shell-syntax:OK |
| 2026-02-13 | VERIFY(Round38) | TARGETED | OK | `pytest -q ...codeagent...status...docs...release...runner...ci...` -> 83 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round38) | FULL | OK | `pytest -q` -> 466 passed, 23 subtests passed |
| 2026-02-13 | VERIFY(Round38) | RUST | OK | `cd rust && cargo clippy --workspace --all-targets -- -D warnings` -> pass; `cargo fmt --all -- --check` -> pass |
| 2026-02-13 | COMPLETE(Round38) | DELIVERY | OK | `task_plan.md/findings.md/progress.md` 已同步为 COMPLETE_ROUND38 (5/5) |
| 2026-02-13 | SCAN(Round39) | BASELINE | OK | `bash -n scripts/*.sh` -> shell-syntax:OK; `pytest -q` -> 466 passed, 23 subtests passed; rust clippy/fmt 通过 |
| 2026-02-13 | SCAN(Round39) | DOCS_FRESHNESS | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 27 passed, 19 subtests passed |
| 2026-02-13 | SCAN(Round39) | DOC_GAPS | WARN | `SKILL.md` 缺 `.fusion/backend_failure_report.json`; `docs/CLI_CONTRACT_MATRIX.md` 缺 backend_* JSON 字段契约 |
| 2026-02-13 | PLAN(Round39) | WRITE_PLAN | OK | `docs/plans/2026-02-13-repo-gap-priority-round39.md` |
| 2026-02-13 | EXECUTE(Round39-TaskA39) | RED | IN_PROGRESS | 缺依赖时清理 stale backend failure report |
| 2026-02-13 | EXECUTE(Round39-TaskA39) | GREEN | OK | `fusion-codeagent.sh` 缺依赖分支清理陈旧 `.fusion/backend_failure_report.json` |
| 2026-02-13 | EXECUTE(Round39-TaskA39) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_fusion_codeagent_script.py` -> 18 passed |
| 2026-02-13 | EXECUTE(Round39-TaskB39) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_skill_md_mentions_backend_failure_report` -> 1 failed |
| 2026-02-13 | EXECUTE(Round39-TaskB39) | GREEN | OK | `SKILL.md` 补齐 `.fusion/backend_failure_report.json` 说明 |
| 2026-02-13 | EXECUTE(Round39-TaskB39) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_skill_md_mentions_backend_failure_report` -> 1 passed |
| 2026-02-13 | EXECUTE(Round39-TaskC39) | RED | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py::TestDocsFreshness::test_cli_contract_matrix_mentions_backend_failure_report` -> 4 failed, 1 passed |
| 2026-02-13 | EXECUTE(Round39-TaskC39) | GREEN | OK | `docs/CLI_CONTRACT_MATRIX.md` 增补 `backend_status/backend_primary/backend_fallback` 与 `.fusion/backend_failure_report.json` 说明 |
| 2026-02-13 | EXECUTE(Round39-TaskC39) | VERIFY | OK | `pytest -q scripts/runtime/tests/test_docs_freshness.py` -> 29 passed, 23 subtests passed |
| 2026-02-13 | SCAN(Round40) | BACKEND_EVIDENCE | WARN | `codeagent-wrapper --backend codex` hang，持续输出 `state db missing rollout path...`，需 timeout 终止；`--backend claude` 正常并返回 `SESSION_ID: <uuid>` |
| 2026-02-13 | EXECUTE(Round40-TaskA40) | RED | OK | `pytest -q ...::test_execute_phase_stores_uuid_session_id` -> 1 failed（UUID 未写入 sessions.json） |
| 2026-02-13 | EXECUTE(Round40-TaskA40) | GREEN | OK | `extract_session_id` 改为解析 `SESSION_ID:` 行，支持 UUID |
| 2026-02-13 | EXECUTE(Round40-TaskA40) | VERIFY | OK | `pytest -q ...::test_execute_phase_stores_uuid_session_id` -> 1 passed |
| 2026-02-13 | EXECUTE(Round40-TaskB40) | RED | OK | `pytest -q ...::test_execute_phase_resume_failure_retries_without_resume` -> 1 failed（直接 fallback 到 codex） |
| 2026-02-13 | EXECUTE(Round40-TaskB40) | GREEN | OK | resume 失败后同后端无 resume 重试一次，成功则不 fallback |
| 2026-02-13 | EXECUTE(Round40-TaskB40) | VERIFY | OK | `pytest -q ...::test_execute_phase_resume_failure_retries_without_resume` -> 1 passed |
| 2026-02-13 | EXECUTE(Round40-TaskC40) | RED | OK | `pytest -q ...::test_timeout_falls_back_to_claude` -> 1 failed（无 timeout，不触发 fallback） |
| 2026-02-13 | EXECUTE(Round40-TaskC40) | GREEN | OK | 新增 `FUSION_CODEAGENT_TIMEOUT_SEC`，用 timeout/gtimeout 包裹 wrapper 调用，超时触发 fallback |
| 2026-02-13 | EXECUTE(Round40-TaskC40) | VERIFY | OK | `pytest -q ...::test_timeout_falls_back_to_claude` -> 1 passed |
| 2026-02-13 | VERIFY(Round40) | BUNDLE | OK | `bash -n scripts/*.sh` -> OK；`pytest -q scripts/runtime/tests/test_fusion_codeagent_script.py scripts/runtime/tests/test_docs_freshness.py` -> 47 passed, 23 subtests passed；`pytest -q` -> 472 passed, 27 subtests passed；rust clippy/fmt -> pass |
