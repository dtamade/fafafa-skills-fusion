# Fusion Findings

## Goal Analysis

### Original Request
- 使用 writing-plans + planning-with-files 思路，先全仓扫描未完成项与缺口，再产出可执行优先级计划。
- 然后用 executing-plans 执行计划，严格按 TDD，并回报每步命令输出。
- 该模式反复执行。

### Interpreted Requirements
- 必须先研究现状，再执行，不可直接改代码。
- 计划要可落地（明确文件、步骤、验证命令）。
- 执行阶段遵循 RED -> GREEN -> REFACTOR。

### Scope
- 当前仓库 `/home/dtamade/projects/fafafa-skills-fusion`
- 先做一轮“扫描 + 计划 + 执行批次1”，再进入下一轮。

### Constraints
- 保持改动聚焦，优先修复高影响缺口。
- 全程保留命令输出证据。

## Codebase Analysis

### Relevant Files
| File | Purpose | Relevance |
|------|---------|-----------|
| scripts/fusion-status.sh | 状态输出 | 近期新增排行榜整合点 |
| scripts/fusion-achievements.sh | 成就统计 | 新能力核心 |
| scripts/runtime/tests/* | 回归验证 | TDD/回归证据 |

### Dependencies
- Bash + jq/python3 (fallback)
- pytest

### Patterns
- shell 脚本 + python 测试驱动
- runtime 适配器双栈（shell/python/rust）

## Research Notes

### Decisions
| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| 先扫描再执行 | 直接实现 / 先审查 | 避免盲改，满足用户要求 |

### Learnings
- 待扫描补充。

## References
- /home/dtamade/.codex/skills/superpowers/skills/writing-plans/SKILL.md
- /home/dtamade/.codex/skills/planning-with-files/SKILL.md
- /home/dtamade/.codex/skills/superpowers/skills/executing-plans/SKILL.md

## Scan Results (Round 1)

### Repository Health Snapshot
- `pytest -q` 全量通过：`326 passed`
- shell 脚本语法检查通过：`shell_syntax:OK`
- 当前工作区存在大量未提交变更（M + ??），需持续控制变更范围

### Gaps & Incomplete Items
1. **文档测试数陈旧**
   - `README.zh-CN.md` 仍写 `317 passed`
   - `CHANGELOG.md` 多处保留历史全量测试数字，最新状态未同步
2. **脚本测试覆盖不均衡**
   - `fusion-achievements.sh`、`fusion-status.sh`、`fusion-codeagent.sh` 已有测试引用
   - 多个关键脚本暂缺直接测试引用：`fusion-hook-doctor.sh`, `fusion-init.sh`, `fusion-start.sh`, `fusion-stop-guard.sh` 等
3. **流程层面的未完成风险**
   - 当前仓库改动非常多，若继续叠加无计划改动，回归成本会迅速上升

### Priority Recommendation (for execution)
- P0: 给 `fusion-hook-doctor.sh` 增加脚本测试（新脚本高价值、当前无直接测试）
- P1: 同步文档中的测试结果表述，避免对外信息过时
- P1: 增加一个针对 `fusion-status.sh` 排行榜 fallback 行为的回归测试（防慢扫/超时退化）


## Plan Output
- `docs/plans/2026-02-11-repo-gap-priority-round1.md`


## Execution Results (Round 1)
- Task 1 完成：新增 docs freshness 测试并移除 README.zh-CN 硬编码通过数。
- Task 2 完成：`fusion-hook-doctor.sh` 支持 `--json` 输出机器可读结果。
- Task 3 完成：`fusion-status.sh` 支持 `FUSION_STATUS_SHOW_LEADERBOARD` 开关。
- 回归：targeted 9 passed；full 330 passed。


## Scan Results (Round 2)

### Repository Health Snapshot
- 全量回归基线：`pytest -q` -> `330 passed`
- 直接测试覆盖仍有缺口：`fusion-start.sh`、`fusion-init.sh` 缺少直接脚本测试。

### Gaps & Incomplete Items
1. `fusion-start.sh` 参数解析过宽（未知选项/多目标未显式拒绝）。
2. `fusion-init.sh` 固定 `engine: "python"`，不利于 Rust 工作流快速切换。
3. `fusion-init.sh` 缺少机器可读输出，不便 CI/自动化链路使用。

### Priority Recommendation (Round 2)
- P0: `fusion-start.sh` 参数校验加强 + 测试。
- P1: `fusion-init.sh` 增加 `--engine` 选项 + 测试。
- P1: `fusion-init.sh` 增加 `--json` 输出 + 测试。


## Plan Output (Round 2)
- `docs/plans/2026-02-11-repo-gap-priority-round2.md`

## Execution Results (Round 2 Finalization Addendum)
- Task C2 补强：发现并修复 `fusion-init.sh --json` 在无 `jq/python3` 环境下 fallback 输出非合法 JSON 的缺口。
- 新增回归测试：`test_fusion_init_json_fallback_without_jq_or_python3`，确保最小 PATH 下仍输出可解析 JSON。
- 验证结果：Round2 targeted `15 passed`，全量 `336 passed`。

## Scan Results (Round 3)

### Repository Health Snapshot
- 全量回归基线：`pytest -q` -> `336 passed`。
- shell 脚本语法：`bash -n scripts/*.sh` -> `shell_syntax:OK`。

### Gaps & Incomplete Items
1. `fusion-resume.sh` 当前不做参数校验，误传参数可能导致误恢复工作流。
2. `fusion-git.sh` 未知 action 会静默降级到 `status`，容易掩盖拼写错误。
3. `fusion-logs.sh` 未校验 `LINES` 参数，非法值会以底层 `tail` 错误退出，缺少可理解错误提示。

### Priority Recommendation (Round 3)
- P0: `fusion-resume.sh` 参数校验（防误操作）。
- P1: `fusion-git.sh` 未知 action 显式报错（提升 CLI 可预期性）。
- P1: `fusion-logs.sh` 参数校验（提升可用性与错误可诊断性）。

## Plan Output (Round 3)
- `docs/plans/2026-02-11-repo-gap-priority-round3.md`

## Execution Results (Round 3)
- Task A3 完成：`fusion-resume.sh` 增加参数校验，未知参数明确报错；新增 `test_resume_rejects_unknown_option`。
- Task B3 完成：`fusion-git.sh` 未知 action 改为报错退出；新增 `test_git_rejects_unknown_action`。
- Task C3 完成：`fusion-logs.sh` 增加行数参数正整数校验；新增 `test_logs_rejects_non_numeric_lines`。
- 回归：Round3 targeted `18 passed`；全量 `339 passed`。

## Scan Results (Round 4)

### Repository Health Snapshot
- 全量回归基线：`pytest -q` -> `339 passed`。
- shell 脚本语法：`bash -n scripts/*.sh` -> `shell_syntax:OK`。

### Gaps & Incomplete Items
1. `fusion-pause.sh` 缺少参数校验，误传参数会继续执行 pause 状态变更。
2. `fusion-cancel.sh` 缺少参数校验，误传参数会继续执行 cancel 状态变更。
3. `fusion-continue.sh` 缺少参数校验，误传参数会被静默忽略，不利于排错与自动化一致性。
4. 以上三者当前均无直接测试覆盖（script-test mapping: NO_REF）。

### Priority Recommendation (Round 4)
- P0: `fusion-pause.sh` 参数校验 + 测试。
- P0: `fusion-cancel.sh` 参数校验 + 测试。
- P1: `fusion-continue.sh` 参数校验 + 测试。

## Plan Output (Round 4)
- `docs/plans/2026-02-11-repo-gap-priority-round4.md`

## Execution Results (Round 4)
- Task A4 完成：`fusion-pause.sh` 增加参数校验，未知参数明确报错；新增 `test_pause_rejects_unknown_option`。
- Task B4 完成：`fusion-cancel.sh` 增加参数校验，未知参数明确报错；新增 `test_cancel_rejects_unknown_option`。
- Task C4 完成：`fusion-continue.sh` 增加参数校验，未知参数明确报错；新增 `test_continue_rejects_unknown_option`。
- 回归：Round4 targeted `21 passed`；全量 `342 passed`。

## Scan Results (Round 5 - pre)
- script-test mapping 刷新后，仅 `loop-guardian.sh` 仍无直接测试引用。
- 下一轮优先将缺口聚焦到 LoopGuardian 的可测试性与行为一致性。

## Scan Results (Round 5)

### Repository Health Snapshot
- 全量回归基线：`pytest -q` -> `342 passed`。
- shell 脚本语法：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- script-test mapping：仅 `loop-guardian.sh` 仍无直接测试覆盖。

### Gaps & Incomplete Items
1. `loop-guardian.sh` 缺少直接单测，关键函数（`guardian_init`/`guardian_status`）行为无独立回归保护。
2. `guardian_status` 阈值展示使用 jq `env.*`，与 `config.yaml` 动态加载值不一致（显示默认值）。
3. `guardian_init` 在 `.fusion` 目录不存在时会失败，降低脚本独立可用性与可测试性。
4. `guardian_status` 目前未展示 `max_state_visits` 与 `max_wall_time_ms` 阈值，可观测性不足。

### Priority Recommendation (Round 5)
- P0: 修复 `guardian_status` 阈值显示与配置一致性 + 测试。
- P0: 修复 `guardian_init` 自动建目录能力 + 测试。
- P1: 增强 `guardian_status` 展示 state/wall-time 阈值 + 测试。

## Plan Output (Round 5)
- `docs/plans/2026-02-11-repo-gap-priority-round5.md`

## Execution Results (Round 5)
- Task A5 完成：为 `loop-guardian` 新增直接测试并修复 `guardian_status` 阈值显示与配置不一致问题。
- Task B5 完成：`guardian_init` 在 `.fusion` 缺失时自动创建目录并成功初始化 `loop_context.json`。
- Task C5 完成：`guardian_status` 增加 `State Visits` 与 `Wall Time` 阈值可见性。
- 回归：Round5 targeted `24 passed`；全量 `345 passed`。

## Scan Results (Round 6)

### Repository Health Snapshot
- 全量回归基线：`pytest -q` -> `345 passed`。
- 脚本覆盖已扩展到 `loop-guardian`，核心控制脚本直接测试已全覆盖。

### Gaps & Incomplete Items
1. `fusion-start.sh` usage 文案写法存在 shell 重定向误解析：`"<goal>"` 被解释为 `<goal` 输入重定向，导致帮助/错误路径抛出 `goal: No such file or directory`。
2. `--help` 当前异常返回 1（应为 0），且无法正确打印 usage。
3. 无参数启动路径同样受 usage 写法影响，可读性与 CLI 一致性不足。

### Priority Recommendation (Round 6)
- P0: 修复 `fusion-start.sh` usage 字符串写法，消除重定向误解析。
- P0: 固化 `--help` 正确退出码与输出行为测试。
- P1: 固化无参数/未知参数路径的 usage 输出一致性测试。

## Plan Output (Round 6)
- `docs/plans/2026-02-11-repo-gap-priority-round6.md`

## Execution Results (Round 6)
- Task A6 完成：修复 `fusion-start.sh` usage 字符串导致的 shell 重定向误解析，`--help` 路径恢复正确。
- Task B6 完成：新增未知参数路径回归，保证输出 usage 且不再出现 `No such file or directory`。
- Task C6 完成：新增无参数路径回归，确保 usage 输出一致且无重定向错误。
- 回归：Round6 targeted `27 passed`；全量 `348 passed`。

## Scan Results (Round 7 - pre)
- `fusion-achievements.sh --leaderboard-only --top abc` 目前返回 0，但向 stderr 打印 `head: invalid number of lines`，说明缺少参数值校验。
- `fusion-achievements.sh --root` 缺失值时返回 0，`LEADERBOARD_ROOT` 为空字符串，错误路径未被显式拒绝。
- 下一轮优先补齐 achievements CLI 参数校验与错误路径回归测试。

## Scan Results (Round 7)

### Repository Health Snapshot
- 全量回归基线：`pytest -q` -> `348 passed`。
- shell 脚本语法：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- 脚本测试映射：控制脚本与 `loop-guardian` 已有直接测试覆盖。

### Gaps & Incomplete Items
1. `fusion-achievements.sh --top abc` 当前返回 0，并泄漏底层 `head: invalid number of lines` 错误。
2. `fusion-achievements.sh --root` 缺失值当前返回 0，参数误用未被拒绝。
3. `fusion-achievements.sh --top` 缺失值当前回落默认 10，无法及时暴露调用错误。

### Priority Recommendation (Round 7)
- P0: 增加 `--top` 非法值校验并拒绝。
- P0: 增加 `--root` 缺失值校验并拒绝。
- P1: 增加 `--top` 缺失值校验并拒绝。

## Plan Output (Round 7)
- `docs/plans/2026-02-11-repo-gap-priority-round7.md`

## Execution Results (Round 7)
- Task A7 完成：新增 `--top` 非法值回归并实现正整数校验，避免泄漏 `head` 底层错误。
- Task B7 完成：新增 `--root` 缺失值回归并实现参数缺失拒绝。
- Task C7 完成：新增 `--top` 缺失值回归并实现参数缺失拒绝。
- 回归：Round7 targeted `33 passed`；全量 `351 passed`。

## Scan Results (Round 8 - pre)
- `fusion-achievements.sh` 在错误参数（如 `--top abc`）时仍输出 `=== Fusion Achievements ===` 标题，错误路径输出不够一致。
- 目前不支持 `--top=<n>` 与 `--root=<path>` 形式（会报 Unknown option）。
- 下一轮可优先补齐错误路径输出一致性与等号参数格式兼容。

## Scan Results (Round 8)

### Repository Health Snapshot
- Python 全量回归：`pytest -q` -> `351 passed`。
- Shell 语法基线：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- Rust 全量测试：`cargo test -q`（`rust/`）-> all passed。
- Rust 质量门禁：`cargo fmt --all -- --check` FAIL（格式偏差，`fusion-runtime-io/src/lib.rs`）。
- Rust 质量门禁：`cargo clippy --workspace --all-targets -- -D warnings` FAIL（`try_inject_safe_backlog` 参数过多）。

### 50-Task Gap Backlog (Prioritized)
1. **R8-001 [P0]** `fusion-achievements.sh` 在参数错误路径禁止输出成功标题横幅。
2. **R8-002 [P0]** `fusion-achievements.sh` 支持 `--top=<n>` 语法。
3. **R8-003 [P0]** `fusion-achievements.sh` 支持 `--root=<path>` 语法。
4. **R8-004 [P0]** `fusion-achievements.sh` 对 `--root=` 空值给出明确错误。
5. **R8-005 [P0]** `fusion-achievements.sh` 对 `--top=` 空值给出明确错误。
6. **R8-006 [P1]** achievements 无效参数统一写入 stderr + usage。
7. **R8-007 [P1]** achievements 切换为 `set -euo pipefail` 严格模式并补回归。
8. **R8-008 [P0]** `fusion-logs.sh` 增加 `-h/--help` 支持并返回 0。
9. **R8-009 [P1]** `fusion-logs.sh` 未知选项与非法行数分离报错。
10. **R8-010 [P0]** `fusion-git.sh` 增加 `-h/--help` 动作。
11. **R8-011 [P1]** `fusion-git.sh` `--help` 退出码固定为 0。
12. **R8-012 [P0]** `fusion-codeagent.sh` 增加 `-h/--help`（不触发后端路由）。
13. **R8-013 [P1]** `fusion-codeagent.sh --help` 禁止输出 route 日志。
14. **R8-014 [P1]** `fusion-status.sh` 增加 `-h/--help` 无 `.fusion` 时可查看用法。
15. **R8-015 [P2]** `fusion-status.sh` 增加可选 `--json` 机器可读输出。
16. **R8-016 [P1]** `fusion-hook-doctor.sh` 对不存在 project_root 返回明确错误。
17. **R8-017 [P2]** `fusion-hook-doctor.sh` 补充 `--strict` 模式（有 warn 即非 0）。
18. **R8-018 [P1]** `fusion-pretool.sh` 在 `python3` 不可用且 runtime 启用时稳健回退。
19. **R8-019 [P1]** `fusion-posttool.sh` 在 `python3` 不可用时不中断状态推进。
20. **R8-020 [P2]** `fusion-stop-guard.sh` 强化无 stdin 情况的 structured/legacy 兼容。
21. **R8-021 [P1]** 新增 `scripts/runtime/tests/test_fusion_cancel_script.py`（直接覆盖 cancel）。
22. **R8-022 [P1]** 新增 `scripts/runtime/tests/test_fusion_continue_script.py`。
23. **R8-023 [P1]** 新增 `scripts/runtime/tests/test_fusion_git_script.py`。
24. **R8-024 [P1]** 新增 `scripts/runtime/tests/test_fusion_init_script.py`。
25. **R8-025 [P1]** 新增 `scripts/runtime/tests/test_fusion_logs_script.py`。
26. **R8-026 [P1]** 新增 `scripts/runtime/tests/test_fusion_pause_script.py`。
27. **R8-027 [P1]** 新增 `scripts/runtime/tests/test_fusion_posttool_script.py`。
28. **R8-028 [P1]** 新增 `scripts/runtime/tests/test_fusion_pretool_script.py`。
29. **R8-029 [P1]** 新增 `scripts/runtime/tests/test_fusion_resume_script.py`。
30. **R8-030 [P1]** 新增 `scripts/runtime/tests/test_fusion_stop_guard_script.py`。
31. **R8-031 [P1]** 增补 `fusion-git --help` 行为回归测试。
32. **R8-032 [P1]** 增补 `fusion-logs --help` 行为回归测试。
33. **R8-033 [P1]** 增补 `fusion-codeagent --help` 行为回归测试。
34. **R8-034 [P0]** 增补 achievements 等号参数语法回归测试。
35. **R8-035 [P0]** 增补 achievements 错误路径不输出成功横幅回归测试。
36. **R8-036 [P1]** `scripts/runtime/config.py` 增加 loader 边界测试。
37. **R8-037 [P1]** `scripts/runtime/kernel.py` 增加阶段流转与恢复边界测试。
38. **R8-038 [P2]** `scripts/runtime/regression_runner.py` 增加参数解析单测。
39. **R8-039 [P2]** bench 脚本执行入口隔离，避免被误当生产路径。
40. **R8-040 [P2]** 增加 bench 脚本 smoke 测试（可跳过）。
41. **R8-041 [P0]** Rust `fusion-cli` 修复 clippy `too_many_arguments`（`try_inject_safe_backlog`）。
42. **R8-042 [P1]** Rust 全仓 `cargo fmt` 清理并保持 `--check` 通过。
43. **R8-043 [P1]** Rust safe-backlog 相关回归用例补齐。
44. **R8-044 [P1]** Rust hook 子命令与 Shell 行为对齐测试。
45. **R8-045 [P1]** Rust stop-hook JSON block 兼容性测试补齐。
46. **R8-046 [P1]** 更新 `docs/HOOKS_SETUP.md`：会话提前结束排障矩阵。
47. **R8-047 [P1]** 更新 `README.md`：新增 `fusion-hook-doctor` 常见故障章节。
48. **R8-048 [P2]** 新增脚本 CLI 行为契约文档（usage/exit code matrix）。
49. **R8-049 [P2]** CI 增加 Rust `clippy -D warnings` 质量门禁。
50. **R8-050 [P2]** CI 增加 Shell lint（shellcheck 可选或容器化）。

### Priority Recommendation (Round 8)
- **Batch1 (P0):** R8-001 / R8-002 / R8-003（当前执行）。
- **Batch2 (P0):** R8-008 / R8-010 / R8-012（CLI help 一致性）。
- **Batch3 (P0):** R8-034 / R8-035 / R8-041（回归 + Rust 质量门禁）。

## Plan Output (Round 8)
- `docs/plans/2026-02-11-repo-gap-priority-round8.md`

## Execution Results (Round 8)
- Task A8 完成：`fusion-achievements.sh` 在参数校验失败前不再打印成功横幅，错误路径输出更一致。
- Task B8 完成：新增 `--top=<n>` 参数格式支持，并保留缺失值防护。
- Task C8 完成：新增 `--root=<path>` 参数格式支持，并保留缺失值防护。
- 回归：achievements 专项 `8 passed`；Round8 targeted `35 passed`；全量 `353 passed`。

## Scan Results (Round 9 - pre)
- 已完成 Round8 Batch1；下一轮建议切入 `R8-008/R8-010/R8-012`（`logs/git/codeagent` help 一致性）。
- Rust 质量门禁仍有缺口：`cargo fmt --check` 与 `cargo clippy -D warnings` 未通过。

## Scan Results (Round 9)

### Repository Health Snapshot
- Python 全量回归：`pytest -q` -> `353 passed`（扫描时基线）。
- Shell 语法基线：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- Rust 全量测试：`cargo test -q`（`rust/`）-> all passed。
- CLI 探针：
  - `fusion-logs.sh --help` 返回 1（误判为行数参数错误）。
  - `fusion-git.sh --help` 返回 1（unknown action）。
  - `fusion-codeagent.sh --help` 超时/误路由（进入 phase `--HELP`）。

### 50-Task Gap Backlog (Prioritized)
1. **R9-001 [P0]** `fusion-logs.sh` 增加 `-h/--help` 并返回 0。
2. **R9-002 [P0]** `fusion-git.sh` 增加 `-h/--help` 并返回 0。
3. **R9-003 [P0]** `fusion-codeagent.sh` 增加 `-h/--help` 并避免触发路由。
4. **R9-004 [P1]** achievements 增加 `--root=` 空值专用回归测试。
5. **R9-005 [P1]** achievements 增加 `--top=` 空值专用回归测试。
6. **R9-006 [P1]** achievements 非法参数路径统一 stderr + usage 行为。
7. **R9-007 [P1]** achievements 评估切换 `set -euo pipefail` 并补兼容测试。
8. **R9-008 [P1]** `fusion-status.sh` 增加独立 `--help` 用法输出。
9. **R9-009 [P2]** `fusion-status.sh` 增加可选 `--json` 机器可读模式。
10. **R9-010 [P1]** `fusion-hook-doctor.sh` 对无效 project_root 返回明确错误。
11. **R9-011 [P2]** `fusion-hook-doctor.sh` 增加 `--strict`（warn 即非 0）。
12. **R9-012 [P1]** `fusion-pretool.sh` 在缺失 python3 时稳定回退。
13. **R9-013 [P1]** `fusion-posttool.sh` 在缺失 python3 时稳定回退。
14. **R9-014 [P1]** `fusion-stop-guard.sh` 增强 structured/legacy 无 stdin 兼容。
15. **R9-015 [P1]** 新增 `scripts/runtime/tests/test_fusion_cancel_script.py`。
16. **R9-016 [P1]** 新增 `scripts/runtime/tests/test_fusion_continue_script.py`。
17. **R9-017 [P1]** 新增 `scripts/runtime/tests/test_fusion_git_script.py`。
18. **R9-018 [P1]** 新增 `scripts/runtime/tests/test_fusion_init_script.py`。
19. **R9-019 [P1]** 新增 `scripts/runtime/tests/test_fusion_logs_script.py`。
20. **R9-020 [P1]** 新增 `scripts/runtime/tests/test_fusion_pause_script.py`。
21. **R9-021 [P1]** 新增 `scripts/runtime/tests/test_fusion_posttool_script.py`。
22. **R9-022 [P1]** 新增 `scripts/runtime/tests/test_fusion_pretool_script.py`。
23. **R9-023 [P1]** 新增 `scripts/runtime/tests/test_fusion_resume_script.py`。
24. **R9-024 [P1]** 新增 `scripts/runtime/tests/test_fusion_stop_guard_script.py`。
25. **R9-025 [P1]** 新增 `fusion-logs --help` 契约测试（exit code + usage）。
26. **R9-026 [P1]** 新增 `fusion-codeagent --help` 契约测试（无路由副作用）。
27. **R9-027 [P1]** achievements 等号参数矩阵测试扩展（组合场景）。
28. **R9-028 [P1]** `scripts/runtime/config.py` loader 边界用例补齐。
29. **R9-029 [P1]** `scripts/runtime/kernel.py` 状态流转边界回归补齐。
30. **R9-030 [P2]** `scripts/runtime/regression_runner.py` 参数解析单测补齐。
31. **R9-031 [P2]** bench 脚本执行入口隔离（避免误触生产路径）。
32. **R9-032 [P2]** bench 脚本 smoke 测试（可 skip）。
33. **R9-033 [P0]** Rust `fusion-cli` 修复 clippy `too_many_arguments`。
34. **R9-034 [P1]** Rust 全仓 `cargo fmt --check` 清零。
35. **R9-035 [P1]** Rust safe-backlog 回归测试补齐。
36. **R9-036 [P1]** Rust hook 子命令与 shell 行为对齐测试补齐。
37. **R9-037 [P1]** Rust stop-hook JSON block 兼容性测试补齐。
38. **R9-038 [P1]** 更新 `docs/HOOKS_SETUP.md` 会话中断排障矩阵。
39. **R9-039 [P1]** 更新 `README.md` hook-doctor 常见故障章节。
40. **R9-040 [P2]** 新增 CLI 行为契约文档（usage/exit matrix）。
41. **R9-041 [P2]** CI 增加 Rust `clippy -D warnings` 门禁。
42. **R9-042 [P2]** CI 增加 shell lint 门禁（shellcheck 或容器化）。
43. **R9-043 [P2]** 统一脚本 usage 文案输出函数模式。
44. **R9-044 [P2]** 提炼通用 shell 参数解析辅助以减少漂移。
45. **R9-045 [P2]** 增加 timeout 缺失环境兼容测试。
46. **R9-046 [P2]** 增加 jq 缺失环境兼容测试矩阵。
47. **R9-047 [P2]** 增加 python 缺失环境兼容测试矩阵。
48. **R9-048 [P2]** dependency_report schema 校验测试补齐。
49. **R9-049 [P1]** 增加 hooks 持续会话 E2E 测试（防提前结束）。
50. **R9-050 [P2]** achievements leaderboard 遍历安全与性能测试。

### Priority Recommendation (Round 9)
- **Batch1 (P0):** R9-001 / R9-002 / R9-003（本轮执行）。
- **Batch2 (P0):** R9-033 / R9-034 / R9-049。
- **Batch3 (P1):** R9-004 / R9-005 / R9-008。

## Plan Output (Round 9)
- `docs/plans/2026-02-11-repo-gap-priority-round9.md`

## Execution Results (Round 9)
- Task A9 完成：`fusion-logs.sh` 支持 `--help`，退出码为 0。
- Task B9 完成：`fusion-git.sh` 支持 `--help`，退出码为 0。
- Task C9 完成：`fusion-codeagent.sh` 支持 `--help`，且不触发 route 分支。
- 回归：Round9 script-targeted `14 passed`；Round9 targeted `42 passed`；全量 `356 passed`。

## Scan Results (Round 10 - pre)
- 下轮建议优先执行 Rust 质量门禁任务：`R9-033`、`R9-034`。
- hooks 持续会话问题建议进入 E2E 级别验证（`R9-049`）。

## Scan Results (Round 10)

### Repository Health Snapshot
- Python 全量回归（扫描时）：`pytest -q` -> `356 passed`。
- Shell 语法基线：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- Rust 测试基线：`cargo test -q`（`rust/`）-> all passed。
- Rust 质量门禁缺口：
  - `cargo fmt --all -- --check` FAIL（`fusion-runtime-io/src/lib.rs`）。
  - `cargo clippy --workspace --all-targets -- -D warnings` FAIL（`try_inject_safe_backlog` 参数过多）。
- CLI 缺口：`fusion-status.sh --help` 在无 `.fusion` 场景返回非 0。

### 50-Task Gap Backlog (Prioritized)
1. **R10-001 [P0]** 修复 `fusion-cli` `clippy::too_many_arguments`（`try_inject_safe_backlog`）。
2. **R10-002 [P0]** 修复 Rust `cargo fmt --check` 失败并清零格式差异。
3. **R10-003 [P0]** `fusion-status.sh` 支持 `-h/--help` 且返回 0。
4. **R10-004 [P1]** `fusion-status.sh` 增加 `--json` 输出模式。
5. **R10-005 [P1]** achievements 增加 `--root=` 空值回归测试。
6. **R10-006 [P1]** achievements 增加 `--top=` 空值回归测试。
7. **R10-007 [P1]** achievements 非法参数路径统一 stderr/usage 契约。
8. **R10-008 [P1]** `fusion-logs.sh` 补充未知选项明确报错路径。
9. **R10-009 [P1]** `fusion-git.sh` 补充 help 分支回归矩阵（help + action 并存场景）。
10. **R10-010 [P1]** `fusion-codeagent.sh` 增加 help + prompt 参数组合回归。
11. **R10-011 [P1]** `fusion-hook-doctor.sh` 处理无效 project_root 错误码与提示。
12. **R10-012 [P2]** `fusion-hook-doctor.sh` 增加 `--strict` 模式。
13. **R10-013 [P1]** `fusion-pretool.sh` 缺失 python3 时回退路径测试。
14. **R10-014 [P1]** `fusion-posttool.sh` 缺失 python3 时回退路径测试。
15. **R10-015 [P1]** `fusion-stop-guard.sh` structured 模式无 stdin 行为测试。
16. **R10-016 [P1]** 新增 `scripts/runtime/tests/test_fusion_cancel_script.py`。
17. **R10-017 [P1]** 新增 `scripts/runtime/tests/test_fusion_continue_script.py`。
18. **R10-018 [P1]** 新增 `scripts/runtime/tests/test_fusion_git_script.py`。
19. **R10-019 [P1]** 新增 `scripts/runtime/tests/test_fusion_init_script.py`。
20. **R10-020 [P1]** 新增 `scripts/runtime/tests/test_fusion_logs_script.py`。
21. **R10-021 [P1]** 新增 `scripts/runtime/tests/test_fusion_pause_script.py`。
22. **R10-022 [P1]** 新增 `scripts/runtime/tests/test_fusion_posttool_script.py`。
23. **R10-023 [P1]** 新增 `scripts/runtime/tests/test_fusion_pretool_script.py`。
24. **R10-024 [P1]** 新增 `scripts/runtime/tests/test_fusion_resume_script.py`。
25. **R10-025 [P1]** 新增 `scripts/runtime/tests/test_fusion_stop_guard_script.py`。
26. **R10-026 [P1]** `scripts/runtime/config.py` 补充 loader 边界用例。
27. **R10-027 [P1]** `scripts/runtime/kernel.py` 补充状态机边界回归。
28. **R10-028 [P2]** `scripts/runtime/regression_runner.py` 参数解析单测。
29. **R10-029 [P2]** bench 脚本入口隔离（显式 opt-in）。
30. **R10-030 [P2]** bench 脚本 smoke 测试（可 skip）。
31. **R10-031 [P1]** Rust safe-backlog 关键路径回归补齐。
32. **R10-032 [P1]** Rust hook 子命令与 shell 行为对齐测试。
33. **R10-033 [P1]** Rust stop-hook JSON block 兼容性测试。
34. **R10-034 [P1]** Rust dependency_report 互通格式测试。
35. **R10-035 [P1]** hooks 持续会话 E2E（防提前结束）测试。
36. **R10-036 [P1]** 文档 `docs/HOOKS_SETUP.md` 增补排障矩阵。
37. **R10-037 [P1]** `README.md` 增补 hook-doctor 诊断与恢复流程。
38. **R10-038 [P1]** `README.zh-CN.md` 同步 hook 诊断章节。
39. **R10-039 [P2]** CLI 行为契约文档（usage/exit code matrix）。
40. **R10-040 [P2]** 统一 shell usage 输出风格（函数化）。
41. **R10-041 [P2]** 抽象共享参数解析 helper，降低脚本漂移。
42. **R10-042 [P2]** CI 增加 Rust `clippy -D warnings` 门禁。
43. **R10-043 [P2]** CI 增加 Rust `fmt --check` 门禁。
44. **R10-044 [P2]** CI 增加 shell lint 门禁（shellcheck/容器化）。
45. **R10-045 [P2]** `fusion-status` 时间格式跨平台回归（GNU/BSD date）。
46. **R10-046 [P2]** `fusion-status` leaderboard 大目录性能回归。
47. **R10-047 [P2]** achievements leaderboard 遍历安全（符号链接/权限）测试。
48. **R10-048 [P2]** hook 运行日志可观测性增强（可选 debug 开关）。
49. **R10-049 [P2]** dependency auto-heal 失败场景可读性优化。
50. **R10-050 [P2]** release checklist 增加 hooks/runtime/rust 三线校验。

### Priority Recommendation (Round 10)
- **Batch1 (P0):** R10-001 / R10-002 / R10-003（本轮执行）。
- **Batch2 (P1):** R10-005 / R10-006 / R10-035。
- **Batch3 (P1/P2):** R10-031 / R10-032 / R10-042。

## Plan Output (Round 10)
- `docs/plans/2026-02-11-repo-gap-priority-round10.md`

## Execution Results (Round 10)
- Task A10 完成：通过 `SafeBacklogTrigger` 收敛参数，Rust `clippy -D warnings` 通过。
- Task B10 完成：执行 `cargo fmt --all` 后 `fmt --check` 通过。
- Task C10 完成：`fusion-status.sh` 新增 `--help`，无 `.fusion` 时也返回 0 并输出 usage。
- 回归：Round10 targeted `21 passed`；Rust test+clippy+fmt 全通过；Python 全量 `357 passed`。

## Scan Results (Round 11 - pre)
- 下轮建议优先补 `fusion-status --json` 与 achievements 空值等号参数测试（R10-004/005/006）。
- 若继续强化“持续工作不结束”，建议直接执行 hooks E2E（R10-035）。

## Scan Results (Round 11)

### Repository Health Snapshot
- Python 全量回归：`pytest -q` -> `357 passed`。
- Shell 语法基线：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- Rust 质量门禁：`cargo test -q` / `cargo clippy -D warnings` / `cargo fmt --check` 全通过。
- CLI 探针：
  - `fusion-status.sh --json` 当前仍输出人类文本横幅，尚无机器可读模式。
  - achievements `--root=` / `--top=` 已拒绝并给出缺失值提示（行为可进一步固化测试）。

### 50-Task Gap Backlog (Prioritized)
1. **R11-001 [P0]** `fusion-status.sh` 增加 `--json` 机器可读输出。
2. **R11-002 [P0]** `fusion-status.sh --json` 在无 `.fusion` 时返回 JSON 错误对象。
3. **R11-003 [P0]** `fusion-status.sh --json` 禁止输出人类横幅文本。
4. **R11-004 [P1]** achievements 增加 `--root=` 空值回归测试。
5. **R11-005 [P1]** achievements 增加 `--top=` 空值回归测试。
6. **R11-006 [P1]** achievements 错误参数输出契约（stderr + usage）回归。
7. **R11-007 [P1]** `fusion-status.sh` 增加 `--json` + `--help` 组合语义测试。
8. **R11-008 [P1]** `fusion-status.sh` 未知参数处理与 usage 统一。
9. **R11-009 [P1]** `fusion-logs.sh` 未知选项明确报错路径补齐。
10. **R11-010 [P1]** `fusion-git.sh` help/action 组合输入回归。
11. **R11-011 [P1]** `fusion-codeagent.sh` help+prompt 组合输入回归。
12. **R11-012 [P1]** `fusion-hook-doctor.sh` 无效 project_root 行为规范化。
13. **R11-013 [P2]** `fusion-hook-doctor.sh` strict 模式落地。
14. **R11-014 [P1]** `fusion-pretool.sh` 缺失 python3 回退回归。
15. **R11-015 [P1]** `fusion-posttool.sh` 缺失 python3 回退回归。
16. **R11-016 [P1]** `fusion-stop-guard.sh` structured 模式 stdin 边界测试。
17. **R11-017 [P1]** 新增 `test_fusion_cancel_script.py`。
18. **R11-018 [P1]** 新增 `test_fusion_continue_script.py`。
19. **R11-019 [P1]** 新增 `test_fusion_git_script.py`。
20. **R11-020 [P1]** 新增 `test_fusion_init_script.py`。
21. **R11-021 [P1]** 新增 `test_fusion_logs_script.py`。
22. **R11-022 [P1]** 新增 `test_fusion_pause_script.py`。
23. **R11-023 [P1]** 新增 `test_fusion_posttool_script.py`。
24. **R11-024 [P1]** 新增 `test_fusion_pretool_script.py`。
25. **R11-025 [P1]** 新增 `test_fusion_resume_script.py`。
26. **R11-026 [P1]** 新增 `test_fusion_stop_guard_script.py`。
27. **R11-027 [P1]** `scripts/runtime/config.py` loader 边界测试补齐。
28. **R11-028 [P1]** `scripts/runtime/kernel.py` 状态流转边界回归。
29. **R11-029 [P2]** `scripts/runtime/regression_runner.py` 参数单测。
30. **R11-030 [P2]** bench 脚本入口隔离。
31. **R11-031 [P2]** bench 脚本 smoke 测试。
32. **R11-032 [P1]** Rust safe-backlog 回归补齐。
33. **R11-033 [P1]** Rust hook 子命令与 shell 对齐测试。
34. **R11-034 [P1]** Rust stop-hook JSON block 兼容性测试。
35. **R11-035 [P1]** hooks 持续会话 E2E（防提前结束）测试。
36. **R11-036 [P1]** 文档 `HOOKS_SETUP` 增补 JSON/status 诊断步骤。
37. **R11-037 [P1]** `README.md` 补充 `fusion-status --json` 用法。
38. **R11-038 [P1]** `README.zh-CN.md` 同步 JSON 状态用法。
39. **R11-039 [P2]** CLI 契约文档（usage/exit/json schema）。
40. **R11-040 [P2]** 统一 shell usage 输出函数模式。
41. **R11-041 [P2]** 参数解析 helper 抽象化。
42. **R11-042 [P2]** CI 增加 Rust clippy 门禁（确认常驻）。
43. **R11-043 [P2]** CI 增加 Rust fmt 门禁（确认常驻）。
44. **R11-044 [P2]** CI 增加 shell lint 门禁。
45. **R11-045 [P2]** status 时间格式跨平台回归。
46. **R11-046 [P2]** status leaderboard 大目录性能回归。
47. **R11-047 [P2]** achievements leaderboard 安全遍历测试。
48. **R11-048 [P2]** hook debug 可观测性增强。
49. **R11-049 [P2]** dependency_report schema 校验器。
50. **R11-050 [P2]** release checklist 纳入 JSON/status 契约校验。

### Priority Recommendation (Round 11)
- **Batch1 (P0):** R11-001 / R11-002 / R11-003（本轮执行）。
- **Batch2 (P1):** R11-004 / R11-005 / R11-008。
- **Batch3 (P1):** R11-032 / R11-033 / R11-035。

## Plan Output (Round 11)
- `docs/plans/2026-02-11-repo-gap-priority-round11.md`

## Execution Results (Round 11)
- Task A11 完成：`fusion-status.sh` 新增 `--json` 成功路径输出（`result/status/phase`）。
- Task B11 完成：无 `.fusion` 时 `--json` 返回机器可读错误对象并退出 1。
- Task C11 完成：`--json` 模式不再输出 `=== Fusion Status ===` 人类横幅。
- 回归：Round11 status 专项 `10 passed`；Round11 targeted `46 passed`；全量 `360 passed`。

## Scan Results (Round 12 - pre)
- 下一轮建议优先补 achievements 空值等号参数契约测试（`R11-004/005/006`）。
- hooks 持续会话提前结束问题建议进入 E2E 验证（`R11-035`）。

## Scan Results (Round 12)

### Repository Health Snapshot
- Python 全量回归（扫描时）：`pytest -q` -> `360 passed`。
- Shell 语法基线：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- Rust 质量门禁维持通过：`cargo test -q`、`cargo clippy -D warnings`、`cargo fmt --check` 全绿。
- 新缺口：`fusion-status --json` 仅输出基础字段，缺少 task/dependency/achievement 摘要字段。

### 50-Task Gap Backlog (Prioritized)
1. **R12-001 [P0]** `fusion-status --json` 增加 `task_*` 计数字段。
2. **R12-002 [P0]** `fusion-status --json` 增加 `dependency_*` 摘要字段。
3. **R12-003 [P0]** `fusion-status --json` 增加 `achievement_*` 计数字段。
4. **R12-004 [P1]** achievements `--root=` 空值回归测试。
5. **R12-005 [P1]** achievements `--top=` 空值回归测试。
6. **R12-006 [P1]** achievements 错误参数 stderr/usage 契约测试。
7. **R12-007 [P1]** `fusion-status --json --bad` 行为契约测试。
8. **R12-008 [P1]** `fusion-status --json --help` 语义测试。
9. **R12-009 [P1]** `fusion-status` 未知参数纯文本路径测试。
10. **R12-010 [P1]** `fusion-status` JSON schema 文档补齐。
11. **R12-011 [P1]** `fusion-logs` 未知参数错误契约测试。
12. **R12-012 [P1]** `fusion-git` help/action 组合测试。
13. **R12-013 [P1]** `fusion-codeagent` help+prompt 组合测试。
14. **R12-014 [P1]** `fusion-hook-doctor` 无效路径错误契约。
15. **R12-015 [P2]** `fusion-hook-doctor --strict` 落地。
16. **R12-016 [P1]** `fusion-pretool` 无 python3 回退测试。
17. **R12-017 [P1]** `fusion-posttool` 无 python3 回退测试。
18. **R12-018 [P1]** `fusion-stop-guard` structured stdin 边界测试。
19. **R12-019 [P1]** 新增 `test_fusion_cancel_script.py`。
20. **R12-020 [P1]** 新增 `test_fusion_continue_script.py`。
21. **R12-021 [P1]** 新增 `test_fusion_git_script.py`。
22. **R12-022 [P1]** 新增 `test_fusion_init_script.py`。
23. **R12-023 [P1]** 新增 `test_fusion_logs_script.py`。
24. **R12-024 [P1]** 新增 `test_fusion_pause_script.py`。
25. **R12-025 [P1]** 新增 `test_fusion_posttool_script.py`。
26. **R12-026 [P1]** 新增 `test_fusion_pretool_script.py`。
27. **R12-027 [P1]** 新增 `test_fusion_resume_script.py`。
28. **R12-028 [P1]** 新增 `test_fusion_stop_guard_script.py`。
29. **R12-029 [P1]** `runtime/config.py` loader 边界测试。
30. **R12-030 [P1]** `runtime/kernel.py` 状态流转边界测试。
31. **R12-031 [P2]** `runtime/regression_runner.py` 参数单测。
32. **R12-032 [P2]** bench 入口隔离。
33. **R12-033 [P2]** bench smoke 测试。
34. **R12-034 [P1]** Rust safe-backlog 回归补齐。
35. **R12-035 [P1]** Rust hook 命令对齐测试。
36. **R12-036 [P1]** Rust stop-hook JSON block 兼容测试。
37. **R12-037 [P1]** hooks 持续会话 E2E（防结束）测试。
38. **R12-038 [P1]** `docs/HOOKS_SETUP.md` JSON 模式诊断补齐。
39. **R12-039 [P1]** `README.md` 增加 status JSON 字段说明。
40. **R12-040 [P1]** `README.zh-CN.md` 同步字段说明。
41. **R12-041 [P2]** CLI 契约文档（JSON schema + exit code）。
42. **R12-042 [P2]** 统一 shell usage 输出风格。
43. **R12-043 [P2]** 参数解析 helper 提炼。
44. **R12-044 [P2]** CI shell lint 门禁。
45. **R12-045 [P2]** CI JSON schema 校验流程。
46. **R12-046 [P2]** status 时间格式跨平台回归。
47. **R12-047 [P2]** status 大目录性能回归。
48. **R12-048 [P2]** achievements leaderboard 安全回归。
49. **R12-049 [P2]** dependency_report schema 校验器。
50. **R12-050 [P2]** release checklist 增加 JSON/Hook 契约校验。

### Priority Recommendation (Round 12)
- **Batch1 (P0):** R12-001 / R12-002 / R12-003（本轮执行）。
- **Batch2 (P1):** R12-004 / R12-005 / R12-006。
- **Batch3 (P1):** R12-037 / R12-038 / R12-039。

## Plan Output (Round 12)
- `docs/plans/2026-02-11-repo-gap-priority-round12.md`

## Execution Results (Round 12)
- Task A12 完成：`--json` 增加 `task_completed/task_pending/task_in_progress/task_failed`。
- Task B12 完成：`--json` 增加 `dependency_status/dependency_missing`。
- Task C12 完成：`--json` 增加 `achievement_completed_tasks/achievement_safe_total/achievement_advisory_total`。
- 回归：Round12 status 专项 `13 passed`；Round12 targeted `49 passed`；全量 `363 passed`。

## Scan Results (Round 13 - pre)
- 下一轮建议优先执行 achievements 空值等号参数契约测试（R12-004/005/006）。
- 若要彻底解决“会话会结束”，建议直接做 hooks E2E（R12-037）。

## Scan Results (Round 13)

### Repository Health Snapshot
- Python 全量回归：`pytest -q` -> `363 passed in 7.33s`。
- Shell 语法基线：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- Rust 质量门禁：`cargo test -q` / `cargo clippy -D warnings` / `cargo fmt --check` 全通过。
- CLI 探针发现：
  - `fusion-codeagent.sh --bad` 会进入路由并超时（`timeout 5` 返回 `124`），未拒绝未知选项。
  - `fusion-hook-doctor.sh --bad` 会触发 `cd: --: invalid option`，错误不够可读。
  - achievements `--top=` 目前能正确返回 `Missing value for --top`（建议固化契约测试）。

### 50-Task Gap Backlog (Prioritized)
1. **R13-001 [P0]** `fusion-codeagent.sh` 拒绝未知选项，避免误路由/超时。
2. **R13-002 [P0]** `fusion-hook-doctor.sh` 拒绝未知选项并输出稳定错误。
3. **R13-003 [P0]** `fusion-hook-doctor.sh` 无效 `project_root` 失败快返（可读原因）。
4. **R13-004 [P0]** `fusion-hook-doctor.sh` 增加 `--fix` 自动修复项目 hooks 配置。
5. **R13-005 [P1]** `fusion-hook-doctor --json` 增加 `fixed` 字段，便于自动化判断。
6. **R13-006 [P1]** `docs/HOOKS_SETUP.md` 增补 `--fix` 一键修复流程。
7. **R13-007 [P1]** `README.md` 增补 hook 持续会话排障路径。
8. **R13-008 [P1]** `README.zh-CN.md` 同步 hook 排障路径。
9. **R13-009 [P1]** achievements `--root=` 空值契约测试。
10. **R13-010 [P1]** achievements `--top=` 空值契约测试。
11. **R13-011 [P1]** achievements 错误参数 `stderr + usage` 契约测试。
12. **R13-012 [P1]** `fusion-status --json --bad` 契约测试。
13. **R13-013 [P1]** `fusion-status --json --help` 语义测试。
14. **R13-014 [P1]** `fusion-status --bad` 文本错误路径测试。
15. **R13-015 [P1]** `fusion-logs --bad` 明确 unknown-option 契约测试。
16. **R13-016 [P1]** `fusion-git --bad` usage 输出一致性测试。
17. **R13-017 [P1]** `fusion-codeagent` phase 规范化与非法 phase 行为测试。
18. **R13-018 [P1]** `fusion-codeagent` 显式 prompt 保留空格/换行测试。
19. **R13-019 [P1]** `fusion-codeagent` 依赖恢复后 `dependency_report` 清理测试。
20. **R13-020 [P1]** `fusion-pretool` 无 python3 fallback 测试。
21. **R13-021 [P1]** `fusion-posttool` 无 python3 fallback 测试。
22. **R13-022 [P1]** `fusion-stop-guard` structured 模式无 stdin 行为测试。
23. **R13-023 [P1]** `fusion-stop-guard` JSON block 字段完整性测试。
24. **R13-024 [P1]** `fusion-stop-guard` legacy mode in_progress 阻断测试。
25. **R13-025 [P1]** Rust `hook stop-guard` 与 shell 行为对齐测试。
26. **R13-026 [P2]** `loop-guardian` 状态恢复边界测试。
27. **R13-027 [P2]** `loop-guardian` stale lock 清理边界测试。
28. **R13-028 [P1]** 新增 `test_fusion_stop_guard_script.py`。
29. **R13-029 [P1]** 新增 `test_fusion_pretool_script.py`。
30. **R13-030 [P1]** 新增 `test_fusion_posttool_script.py`。
31. **R13-031 [P1]** 新增 `test_fusion_pause_script.py`。
32. **R13-032 [P1]** 新增 `test_fusion_resume_script.py`。
33. **R13-033 [P1]** 新增 `test_fusion_cancel_script.py`。
34. **R13-034 [P1]** 新增 `test_fusion_continue_script.py`。
35. **R13-035 [P1]** 新增 `test_fusion_logs_script.py`。
36. **R13-036 [P1]** 新增 `test_fusion_git_script.py`。
37. **R13-037 [P1]** 新增 `test_fusion_init_script.py`。
38. **R13-038 [P1]** `runtime/config.py` loader 边界测试补齐。
39. **R13-039 [P1]** `runtime/kernel.py` 调度边界路径测试补齐。
40. **R13-040 [P2]** `runtime/regression_runner.py` 参数解析单测。
41. **R13-041 [P2]** `runtime/router.py` fallback 优先级边界测试。
42. **R13-042 [P2]** `runtime/task_graph.py` 环依赖检测回归。
43. **R13-043 [P2]** `runtime/session_store.py` 损坏文件恢复测试。
44. **R13-044 [P1]** Rust dependency_report schema 互通测试。
45. **R13-045 [P1]** Rust safe-backlog 事件字段对齐测试。
46. **R13-046 [P1]** Rust stop-hook structured JSON 契约测试。
47. **R13-047 [P2]** CI 增加 shellcheck 门禁。
48. **R13-048 [P2]** CI 固化 Rust clippy/fmt 门禁。
49. **R13-049 [P2]** CLI usage/exit code matrix 文档化。
50. **R13-050 [P2]** release checklist 纳入 hooks + JSON 契约校验。

### Priority Recommendation (Round 13)
- **Batch1 (P0):** R13-001 / R13-002 / R13-003（本轮执行）。
- **Batch2 (P0/P1):** R13-004 / R13-005 / R13-006。
- **Batch3 (P1):** R13-009 / R13-010 / R13-011。

## Plan Output (Round 13)
- `docs/plans/2026-02-11-repo-gap-priority-round13.md`

## Execution Results (Round 13)
- Task A13 完成：`fusion-codeagent.sh` 新增未知选项拒绝分支，`--bad` 不再进入 route/backend。
- Task B13 完成：`fusion-hook-doctor.sh` 对未知 `-` 参数统一返回错误，`--json` 下输出 `{result:error, reason:*}`。
- Task C13 完成：`fusion-hook-doctor.sh` 对无效 `project_root` 失败快返，错误语义稳定。
- 回归：Round13 `codeagent+hook-doctor` 专项 `11 passed`；Round13 targeted `52 passed`；全量 `366 passed`。

## Scan Results (Round 14 - pre)
- 下一轮建议优先执行 hooks 自动修复（`R13-004/005/006`）：`fusion-hook-doctor --fix` + JSON `fixed` 字段 + 文档补齐。
- 若继续强化成就系统，建议执行 achievements 参数契约测试补齐（`R13-009/010/011`）。

## Scan Results (Round 14)

### Repository Health Snapshot
- Python 全量回归：`pytest -q` -> `366 passed in 21.22s`。
- Shell 语法基线：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- Rust 质量门禁：`cargo test -q` / `cargo clippy -D warnings` / `cargo fmt --check` 全通过。
- CLI 探针发现：`fusion-hook-doctor.sh --json --fix .` 目前返回 `{"result":"error","reason":"Unknown option: --fix"}`，自动修复链路未落地。

### 50-Task Gap Backlog (Prioritized)
1. **R14-001 [P0]** `fusion-hook-doctor.sh` 实现 `--fix`，自动写入项目 hooks 配置。
2. **R14-002 [P0]** `fusion-hook-doctor --json` 增加 `fixed` 字段（本次是否执行修复）。
3. **R14-003 [P0]** `docs/HOOKS_SETUP.md` 增补 `--fix` 一键诊断修复流程。
4. **R14-004 [P1]** achievements `--root=` 空值契约测试。
5. **R14-005 [P1]** achievements `--top=` 空值契约测试。
6. **R14-006 [P1]** achievements 错误参数 `stderr + usage` 契约测试。
7. **R14-007 [P1]** `README.md` 增补 hook 持续会话排障路径。
8. **R14-008 [P1]** `README.zh-CN.md` 同步 hook 排障路径。
9. **R14-009 [P1]** `fusion-status --json --bad` 契约测试。
10. **R14-010 [P1]** `fusion-status --json --help` 语义测试。
11. **R14-011 [P1]** `fusion-status --bad` 文本错误路径测试。
12. **R14-012 [P1]** `fusion-logs --bad` unknown-option 契约测试。
13. **R14-013 [P1]** `fusion-git --bad` usage 一致性测试。
14. **R14-014 [P1]** `fusion-codeagent` 非法 phase 语义测试。
15. **R14-015 [P1]** `fusion-codeagent` 显式 prompt 保真测试。
16. **R14-016 [P1]** `fusion-codeagent` dependency_report 清理路径测试。
17. **R14-017 [P1]** `fusion-pretool` 无 python3 fallback 测试。
18. **R14-018 [P1]** `fusion-posttool` 无 python3 fallback 测试。
19. **R14-019 [P1]** `fusion-stop-guard` structured 无 stdin 测试。
20. **R14-020 [P1]** `fusion-stop-guard` JSON block 字段完整性测试。
21. **R14-021 [P1]** `fusion-stop-guard` legacy mode 阻断路径测试。
22. **R14-022 [P1]** Rust hook stop-guard parity 测试。
23. **R14-023 [P1]** Rust dependency_report schema parity。
24. **R14-024 [P1]** Rust safe-backlog 事件字段 parity。
25. **R14-025 [P2]** loop-guardian 状态恢复边界。
26. **R14-026 [P2]** loop-guardian stale lock 清理边界。
27. **R14-027 [P1]** 新增 `test_fusion_stop_guard_script.py`。
28. **R14-028 [P1]** 新增 `test_fusion_pretool_script.py`。
29. **R14-029 [P1]** 新增 `test_fusion_posttool_script.py`。
30. **R14-030 [P1]** 新增 `test_fusion_pause_script.py`。
31. **R14-031 [P1]** 新增 `test_fusion_resume_script.py`。
32. **R14-032 [P1]** 新增 `test_fusion_cancel_script.py`。
33. **R14-033 [P1]** 新增 `test_fusion_continue_script.py`。
34. **R14-034 [P1]** 新增 `test_fusion_logs_script.py`。
35. **R14-035 [P1]** 新增 `test_fusion_git_script.py`。
36. **R14-036 [P1]** 新增 `test_fusion_init_script.py`。
37. **R14-037 [P1]** `runtime/config.py` loader 边界测试。
38. **R14-038 [P1]** `runtime/kernel.py` 调度边界测试。
39. **R14-039 [P2]** `runtime/regression_runner.py` 参数解析单测。
40. **R14-040 [P2]** `runtime/router.py` fallback 边界测试。
41. **R14-041 [P2]** `runtime/task_graph.py` 环依赖回归。
42. **R14-042 [P2]** `runtime/session_store.py` 损坏恢复测试。
43. **R14-043 [P2]** CI shellcheck 门禁。
44. **R14-044 [P2]** CI 固化 Rust clippy/fmt 门禁。
45. **R14-045 [P2]** CLI usage/exit-code matrix 文档。
46. **R14-046 [P2]** release checklist 纳入 hooks/json 契约。
47. **R14-047 [P2]** hooks 调试日志开关（可选）。
48. **R14-048 [P2]** status 大目录性能回归。
49. **R14-049 [P2]** achievements leaderboard 安全遍历回归。
50. **R14-050 [P2]** dependency auto-heal 失败提示可读性优化。

### Priority Recommendation (Round 14)
- **Batch1 (P0):** R14-001 / R14-002 / R14-003（本轮执行）。
- **Batch2 (P1):** R14-004 / R14-005 / R14-006。
- **Batch3 (P1):** R14-007 / R14-008 / R14-009。

## Plan Output (Round 14)
- `docs/plans/2026-02-11-repo-gap-priority-round14.md`

## Execution Results (Round 14)
- Task A14 完成：`fusion-hook-doctor.sh` 支持 `--fix`，可自动生成项目 `.claude/settings.local.json` 完整 hooks。
- Task B14 完成：`fusion-hook-doctor --json` 增加 `fixed` 布尔字段，明确本次是否执行自动修复。
- Task C14 完成：`docs/HOOKS_SETUP.md` 增补 doctor + auto-fix 命令与验证流程。
- 回归：Round14 `hook-doctor+docs` 专项 `8 passed`；Round14 targeted `56 passed`；全量 `369 passed`。

## Scan Results (Round 15 - pre)
- 下一轮建议优先执行 achievements 参数契约测试补齐（`R14-004/005/006`）。
- 若继续推进持续会话稳定性，建议补 `stop-guard` 独立脚本测试（`R14-019/020/021/027`）。

## Scan Results (Round 15)

### Repository Health Snapshot
- Python 全量回归：`pytest -q` -> `369 passed in 20.01s`。
- Shell 语法基线：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- Rust 质量门禁：`cargo test -q` / `cargo clippy -D warnings` / `cargo fmt --check` 全通过。
- CLI 探针发现：
  - `fusion-stop-guard` 在 `.state.lock` 已存在时，无论结构化模式均直接 `exit 2`，缺少 JSON block（可能导致 Hook 会话中断）。
  - README / README.zh-CN 尚未显式给出 `fusion-hook-doctor --json --fix` 快速恢复命令。

### 50-Task Gap Backlog (Prioritized)
1. **R15-001 [P0]** `fusion-stop-guard` 锁竞争时 structured 模式输出 JSON block（不直接 exit 2）。
2. **R15-002 [P0]** 新增 `test_fusion_stop_guard_script.py` 覆盖 structured/legacy 阻断契约。
3. **R15-003 [P0]** README/README.zh-CN 增补 `hook-doctor --json --fix` 快速恢复路径。
4. **R15-004 [P1]** achievements `--root=` 空值契约测试。
5. **R15-005 [P1]** achievements `--top=` 空值契约测试。
6. **R15-006 [P1]** achievements unknown option 输出契约测试。
7. **R15-007 [P1]** stop-guard structured 无 stdin 契约测试。
8. **R15-008 [P1]** stop-guard legacy 文本阻断契约测试。
9. **R15-009 [P1]** stop-guard completed 状态自动放行测试。
10. **R15-010 [P1]** stop-guard 无 `.fusion` 放行测试。
11. **R15-011 [P1]** hook-doctor `--fix` 幂等性测试。
12. **R15-012 [P1]** hook-doctor `--fix` 失败路径错误契约。
13. **R15-013 [P1]** hook-doctor JSON schema 文档化。
14. **R15-014 [P1]** docs/HOOKS_SETUP 增补 lock contention 排障。
15. **R15-015 [P1]** `fusion-status` 显示 hook-doctor quick command。
16. **R15-016 [P1]** `fusion-status --json` 增加 `hook_health` 字段（可选）。
17. **R15-017 [P1]** `fusion-codeagent` 非法 phase 拒绝策略测试。
18. **R15-018 [P1]** `fusion-codeagent` prompt 保真测试。
19. **R15-019 [P1]** `fusion-logs --bad` unknown option 契约测试。
20. **R15-020 [P1]** `fusion-git --bad` usage 契约测试。
21. **R15-021 [P1]** `fusion-init --json` unknown option 契约测试。
22. **R15-022 [P1]** `fusion-start` multi-goal 契约测试强化。
23. **R15-023 [P1]** `fusion-pretool` 无 python3 回退测试。
24. **R15-024 [P1]** `fusion-posttool` 无 python3 回退测试。
25. **R15-025 [P1]** Rust stop-hook structured parity 测试。
26. **R15-026 [P1]** Rust hook doctor（未来）接口兼容占位测试。
27. **R15-027 [P1]** runtime compat_v2 stop-guard schema 对齐测试。
28. **R15-028 [P2]** loop-guardian state aging 边界测试。
29. **R15-029 [P2]** loop-guardian stale lock 边界测试。
30. **R15-030 [P2]** runtime/kernel stop guard integration 回归。
31. **R15-031 [P2]** runtime/router fallback 优先级边界。
32. **R15-032 [P2]** runtime/task_graph 环依赖回归。
33. **R15-033 [P2]** runtime/session_store 损坏恢复回归。
34. **R15-034 [P2]** runtime/regression_runner 参数测试。
35. **R15-035 [P2]** CI shellcheck 门禁。
36. **R15-036 [P2]** CI bash syntax + stop-hook 专项门禁。
37. **R15-037 [P2]** CI rust clippy/fmt 固化门禁。
38. **R15-038 [P2]** CLI usage/exit-code matrix 文档。
39. **R15-039 [P2]** release checklist 纳入 hook health 验收。
40. **R15-040 [P2]** README 增补 structured/legacy stop 模式说明。
41. **R15-041 [P2]** README.zh-CN 同步 stop 模式说明。
42. **R15-042 [P2]** achievements 输出可选 JSON 模式。
43. **R15-043 [P2]** achievements leaderboard 权限错误可读化。
44. **R15-044 [P2]** status leaderboard 超时可观测性增强。
45. **R15-045 [P2]** hook-doctor 增加 `--strict` 模式。
46. **R15-046 [P2]** hook-doctor 输出 remediation checklist。
47. **R15-047 [P2]** docs E2E 增加 stop-guard structured 示例。
48. **R15-048 [P2]** docs E2E 增加 hook-doctor auto-fix 示例。
49. **R15-049 [P2]** 统一脚本错误输出风格。
50. **R15-050 [P2]** 统一测试命名与分组规范。

### Priority Recommendation (Round 15)
- **Batch1 (P0):** R15-001 / R15-002 / R15-003（本轮执行）。
- **Batch2 (P1):** R15-004 / R15-005 / R15-006。
- **Batch3 (P1):** R15-007 / R15-008 / R15-009。

## Plan Output (Round 15)
- `docs/plans/2026-02-11-repo-gap-priority-round15.md`

## Execution Results (Round 15)
- Task A15 完成：`fusion-stop-guard.sh` 在 lock 竞争时改为 mode-aware 阻断：structured 返回 JSON block（rc=0），legacy 保持 exit2。
- Task B15 完成：新增 `scripts/runtime/tests/test_fusion_stop_guard_script.py`，覆盖 structured/legacy/allow/lock 关键行为。
- Task C15 完成：`README.md` 与 `README.zh-CN.md` 增补 `fusion-hook-doctor.sh --json --fix` 快速恢复流程。
- Extra 完成：achievements 参数契约测试补齐（`--root=`/`--top=`/unknown option）。
- 回归：Round15 `stop-guard+docs+achievements` 专项 `20 passed`；Round15 targeted `66 passed`；全量 `379 passed`。

## Scan Results (Round 16 - pre)
- 下一轮建议优先补 `fusion-logs` / `fusion-git` unknown-option 契约一致性（R15-019/020）。
- 可继续推进 stop-hook 扩展：structured 无 stdin 与 runtime parity（R15-007/025/027）。

## Scan Results (Round 16)

### Repository Health Snapshot
- Python 全量回归：`pytest -q` -> `379 passed in 10.74s`。
- Shell 语法基线：`bash -n scripts/*.sh` -> `shell_syntax:OK`。
- Rust 质量门禁继续通过。
- CLI 探针发现：
  - `fusion-logs.sh --bad` 当前报 `LINES must be a positive integer`，缺少明确 unknown-option 契约。
  - `fusion-git.sh --bad` 当前错误输出在 stdout，stderr 为空，错误通道不一致。
  - `fusion-stop-guard` structured 模式即便无 stdin 也可工作（已验证），但缺少专门测试保护。

### 50-Task Gap Backlog (Prioritized)
1. **R16-001 [P0]** `fusion-logs.sh` 对未知选项输出 `Unknown option` + usage（stderr）。
2. **R16-002 [P0]** `fusion-git.sh` 错误统一输出到 stderr，未知 action 附 usage。
3. **R16-003 [P0]** `fusion-stop-guard` structured 无 stdin 行为增加契约测试。
4. **R16-004 [P1]** `test_hook_shell_runtime_path.py` 增加 runtime 模式下 stop-hook 空 stdin 契约。
5. **R16-005 [P1]** `fusion-logs.sh` 参数个数边界（多参）契约测试。
6. **R16-006 [P1]** `fusion-git.sh` unknown action stderr-only 契约测试。
7. **R16-007 [P1]** achievements CLI 契约继续补齐（root/top/help 组合）。
8. **R16-008 [P1]** `fusion-status --json --help` 契约测试。
9. **R16-009 [P1]** `fusion-status --json --bad` 契约测试。
10. **R16-010 [P1]** hook-doctor `--fix` 幂等性扩展测试。
11. **R16-011 [P1]** hook-doctor 自动修复失败路径测试。
12. **R16-012 [P1]** README/docs 的 hook quick-fix 跨文档一致性测试。
13. **R16-013 [P1]** `fusion-init` unknown option JSON/text 契约。
14. **R16-014 [P1]** `fusion-start` option parsing 边界。
15. **R16-015 [P1]** `fusion-codeagent` phase 非法值约束测试。
16. **R16-016 [P1]** `fusion-codeagent` explicit prompt 保真测试。
17. **R16-017 [P1]** `fusion-continue` 参数契约加强。
18. **R16-018 [P1]** `fusion-cancel` 参数契约加强。
19. **R16-019 [P1]** `fusion-pause` 参数契约加强。
20. **R16-020 [P1]** `fusion-resume` 参数契约加强。
21. **R16-021 [P1]** `fusion-pretool` 无 python3 fallback 测试。
22. **R16-022 [P1]** `fusion-posttool` 无 python3 fallback 测试。
23. **R16-023 [P1]** `fusion-stop-guard` runtime parity with compat_v2（decision/reason/systemMessage）。
24. **R16-024 [P1]** `runtime.compat_v2` stop_guard schema 回归。
25. **R16-025 [P1]** `runtime.compat_v2` pretool progress 行格式回归。
26. **R16-026 [P1]** `runtime.compat_v2` posttool changed/no-change 边界。
27. **R16-027 [P2]** loop-guardian stale lock 更细颗粒测试。
28. **R16-028 [P2]** loop-guardian backoff 行为回归。
29. **R16-029 [P2]** scheduler 与 stop-hook 的集成边界。
30. **R16-030 [P2]** router fallback 顺序边界。
31. **R16-031 [P2]** task_graph cycle 路径加强。
32. **R16-032 [P2]** session_store 损坏恢复路径加强。
33. **R16-033 [P2]** regression_runner 参数边界。
34. **R16-034 [P2]** docs E2E stop-hook lock contention 示例。
35. **R16-035 [P2]** docs E2E runtime parity 示例。
36. **R16-036 [P2]** docs HOOKS_SETUP strict mode 说明。
37. **R16-037 [P2]** README EN runtime parity 说明。
38. **R16-038 [P2]** README ZH runtime parity 说明。
39. **R16-039 [P2]** CLI error message 统一风格整理。
40. **R16-040 [P2]** usage 文本统一模板化。
41. **R16-041 [P2]** shellcheck 门禁落地。
42. **R16-042 [P2]** bash syntax gate 常驻。
43. **R16-043 [P2]** rust clippy/fmt gate 常驻。
44. **R16-044 [P2]** achievements JSON mode feasibility研究。
45. **R16-045 [P2]** status JSON schema 文档。
46. **R16-046 [P2]** release checklist 增加 CLI 契约回归。
47. **R16-047 [P2]** release checklist 增加 hook parity 回归。
48. **R16-048 [P2]** release checklist 增加 docs freshness 回归。
49. **R16-049 [P2]** 长期：Rust bridge 覆盖 hook-doctor 子命令。
50. **R16-050 [P2]** 长期：stop-hook runtime-only 模式可切换。

### Priority Recommendation (Round 16)
- **Batch1 (P0):** R16-001 / R16-002 / R16-003（本轮执行）。
- **Batch2 (P1):** R16-004 / R16-005 / R16-006。
- **Batch3 (P1):** R16-008 / R16-009 / R16-010。

## Plan Output (Round 16)
- `docs/plans/2026-02-11-repo-gap-priority-round16.md`

## Execution Results (Round 16)
- Task A16 完成：`fusion-logs.sh` 新增 unknown-option/参数个数契约；`fusion-git.sh` 错误统一 stderr 且 unknown action 输出 usage。
- Task B16 完成：`test_fusion_stop_guard_script.py` 补充 structured 空 stdin 阻断契约（稳定 JSON block）。
- Task C16 完成：`test_hook_shell_runtime_path.py` 补充 runtime 模式下 structured 空 stdin parity 测试。
- 回归：Round16 `control+stop+runtime` 专项 `23 passed`；Round16 targeted `76 passed`；全量 `383 passed`。

## Scan Results (Round 17 - pre)
- 下一轮建议聚焦 `status` JSON 参数契约（`--json --help`/`--json --bad`）与 hook-doctor `--fix` 失败路径。
- 如需继续整包，可将 logs/git/init/start 的“多参数/组合参数”边界一次打齐。

## Scan Results (Round 17)

### Repository Health Snapshot
- Python 全量回归：`pytest -q` -> `383 passed in 13.93s`。
- Shell/Rust 门禁稳定通过。
- 方向剩余高优边界：
  - `fusion-status --json --help` / `--json --bad` 缺少显式契约测试保护。
  - `fusion-hook-doctor --json --fix` 缺少“修复失败”路径测试保护。
  - `fusion-logs.sh` 新增多参数校验尚缺显式测试。

### 50-Task Gap Backlog (Prioritized)
1. **R17-001 [P0]** `fusion-status --json --bad` JSON 错误对象契约测试。
2. **R17-002 [P0]** `fusion-status --json --help` 帮助优先语义测试。
3. **R17-003 [P0]** `fusion-hook-doctor --json --fix` 修复失败路径测试。
4. **R17-004 [P1]** `fusion-logs` 多参数（too many args）契约测试。
5. **R17-005 [P1]** `fusion-logs` unknown option 契约保持测试。
6. **R17-006 [P1]** `fusion-git` stderr-only 契约保持测试。
7. **R17-007 [P1]** `fusion-status` unknown option 文本路径测试。
8. **R17-008 [P1]** achievements 参数组合边界测试。
9. **R17-009 [P1]** hook-doctor `--fix` 幂等性测试。
10. **R17-010 [P1]** hook-doctor JSON schema 文档补充。
11. **R17-011 [P1]** hook shell runtime path stop-hook parity 继续扩展。
12. **R17-012 [P1]** stop-guard legacy/structured lock contention 文档化。
13. **R17-013 [P1]** status JSON schema 文档化。
14. **R17-014 [P2]** CLI usage 文本统一模板。
15. **R17-015 [P2]** shellcheck 门禁。
16. **R17-016 [P2]** bash syntax 门禁常驻。
17. **R17-017 [P2]** rust clippy/fmt 门禁常驻。
18. **R17-018 [P2]** release checklist 加入 status 契约。
19. **R17-019 [P2]** release checklist 加入 hook-doctor 契约。
20. **R17-020 [P2]** release checklist 加入 logs/git 契约。
21. **R17-021 [P2]** runtime compat_v2 stop-guard schema 回归。
22. **R17-022 [P2]** runtime hook_adapter CLI parity 回归。
23. **R17-023 [P2]** session_store 损坏恢复回归。
24. **R17-024 [P2]** router fallback 边界回归。
25. **R17-025 [P2]** scheduler/stop-hook 集成边界。
26. **R17-026 [P2]** loop-guardian stale lock 边界。
27. **R17-027 [P2]** loop-guardian backoff 边界。
28. **R17-028 [P2]** docs E2E 增加 logs/git 错误契约片段。
29. **R17-029 [P2]** docs E2E 增加 status JSON 错误样例。
30. **R17-030 [P2]** docs E2E 增加 hook-doctor fix-failure 样例。
31. **R17-031 [P2]** README EN 契约测试命令片段。
32. **R17-032 [P2]** README ZH 契约测试命令片段。
33. **R17-033 [P2]** achievements JSON mode 评估。
34. **R17-034 [P2]** status leaderboard timeout 可观测性。
35. **R17-035 [P2]** hook-doctor strict mode 评估。
36. **R17-036 [P2]** stop-guard runtime-only mode 评估。
37. **R17-037 [P2]** fusion-init args matrix。
38. **R17-038 [P2]** fusion-start args matrix。
39. **R17-039 [P2]** fusion-codeagent args matrix。
40. **R17-040 [P2]** fusion-continue args matrix。
41. **R17-041 [P2]** fusion-resume args matrix。
42. **R17-042 [P2]** fusion-pause args matrix。
43. **R17-043 [P2]** fusion-cancel args matrix。
44. **R17-044 [P2]** fusion-achievements args matrix。
45. **R17-045 [P2]** fusion-status args matrix。
46. **R17-046 [P2]** fusion-hook-doctor args matrix。
47. **R17-047 [P2]** docs freshness 覆盖扩展。
48. **R17-048 [P2]** regression runner 增加契约组。
49. **R17-049 [P2]** CI 增加契约组默认执行。
50. **R17-050 [P2]** 发布前自动契约审计脚本。

### Priority Recommendation (Round 17)
- **Batch1 (P0):** R17-001 / R17-002 / R17-003（本轮执行）。
- **Batch2 (P1):** R17-004 / R17-005 / R17-006。
- **Batch3 (P1):** R17-007 / R17-008 / R17-009。

## Plan Output (Round 17)
- `docs/plans/2026-02-11-repo-gap-priority-round17.md`

## Execution Results (Round 17)
- Task A17 完成：新增 `fusion-status --json --bad` 与 `--json --help` 契约测试，确认当前实现语义稳定。
- Task B17 完成：新增 `fusion-hook-doctor --json --fix` 失败路径测试，确认失败时 `result=warn` 且 `fixed=false`。
- Task C17 完成：新增 `fusion-logs` 多参数边界测试，确认返回 `Too many arguments` + usage。
- 回归：Round17 `status+hook-doctor+control` 专项 `33 passed`；Round17 targeted `80 passed`；全量 `387 passed`。

## Scan Results (Round 18 - pre)
- 当前方向的 P0/P1 核心契约已基本清空；后续可转向“统一参数矩阵 + 文档 schema + CI 契约门禁”收尾。

## Scan Results (Round 18)

### Repository Health Snapshot
- 全量回归：`pytest -q` -> `394 passed, 22 subtests passed`。
- Shell 语法：`bash -n scripts/*.sh` -> `OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` 与 `cargo fmt --all -- --check` 均通过。
- 结构化缺口：
  - `.github/workflows` 缺失 CI 门禁（本轮补齐）。
  - CLI 参数契约无统一矩阵文档（本轮补齐）。
  - 发布前缺少一键审计入口（本轮补齐）。

### 50-Task Gap Backlog (Prioritized)
1. **R18-001 [P0]** CI workflow 增加 shell/pytest/rust 门禁（已执行）。
2. **R18-002 [P0]** CI workflow 契约测试落地（已执行）。
3. **R18-003 [P0]** CLI_CONTRACT_MATRIX 文档落地（已执行）。
4. **R18-004 [P0]** docs freshness 覆盖 contract matrix 必填列（已执行）。
5. **R18-005 [P0]** 发布前自动审计脚本（已执行）。
6. **R18-006 [P0]** 审计脚本 `--dry-run` 契约（已执行）。
7. **R18-007 [P0]** 审计脚本 unknown option 契约（已执行）。
8. **R18-008 [P0]** CI workflow 在 PR 与 push 双触发。
9. **R18-009 [P0]** CI workflow Python 版本策略（固定/矩阵）评估。
10. **R18-010 [P0]** CI workflow Rust toolchain 固化策略评估。
11. **R18-011 [P1]** release script 支持 `--fast`（跳过全量 pytest）选项。
12. **R18-012 [P1]** release script 失败时输出汇总失败阶段。
13. **R18-013 [P1]** release script 支持 `--skip-rust` 选项。
14. **R18-014 [P1]** release script 支持 `--skip-python` 选项。
15. **R18-015 [P1]** release script 支持 contract-only 测试集。
16. **R18-016 [P1]** release script 输出 JSON summary 模式。
17. **R18-017 [P1]** regression_runner 集成 `contract` profile。
18. **R18-018 [P1]** docs/HOOKS_SETUP 增加 release-audit 章节。
19. **R18-019 [P1]** README 增加 CI gates 徽章与说明。
20. **R18-020 [P1]** README.zh-CN 增加 CI gates 与审计脚本说明。
21. **R18-021 [P1]** CLI matrix 增加 `--help` 退出码统一列。
22. **R18-022 [P1]** CLI matrix 增加 JSON schema 链接。
23. **R18-023 [P1]** CLI matrix 增加跨脚本错误码规范说明。
24. **R18-024 [P1]** `fusion-start` 参数组合矩阵补测（force/help/unknown）。
25. **R18-025 [P1]** `fusion-init` `--engine` 与 `--json` 组合补测。
26. **R18-026 [P1]** `fusion-status` human/json 双模式快照补测。
27. **R18-027 [P1]** `fusion-logs` 参数边界/错误通道补测。
28. **R18-028 [P1]** `fusion-git` action 矩阵（成功/失败）补测。
29. **R18-029 [P1]** `fusion-codeagent` phase 合法集矩阵补测。
30. **R18-030 [P1]** `fusion-hook-doctor` `--fix` 与 path 组合补测。
31. **R18-031 [P2]** `fusion-achievements` 本地/排行榜模式 JSON 化评估。
32. **R18-032 [P2]** `fusion-pause/resume/cancel/continue` 统一 usage 文本模板。
33. **R18-033 [P2]** `fusion-stop-guard` structured payload schema 文档化。
34. **R18-034 [P2]** hook runtime adapter 错误恢复回归增强。
35. **R18-035 [P2]** `.claude/settings.json` 示例自动校验脚本。
36. **R18-036 [P2]** docs E2E 增加 stop-hook structured 场景。
37. **R18-037 [P2]** docs E2E 增加 hook-doctor `--fix` 失败场景。
38. **R18-038 [P2]** docs E2E 增加 achievements 排行榜场景。
39. **R18-039 [P2]** docs E2E 增加 release-audit 场景。
40. **R18-040 [P2]** shell 脚本统一 ANSI 开关（CI 静默模式）评估。
41. **R18-041 [P2]** shell 脚本统一 `set -euo pipefail` 校验。
42. **R18-042 [P2]** shell 脚本统一 trap/cleanup 策略评估。
43. **R18-043 [P2]** Python runtime 对 shell 合约快照测试工具。
44. **R18-044 [P2]** docs 自动生成 CLI contract matrix 工具。
45. **R18-045 [P2]** matrix 与测试双向一致性校验器。
46. **R18-046 [P2]** CI 分层（quick/full/nightly）策略设计。
47. **R18-047 [P2]** CI 缓存（pip/cargo）优化。
48. **R18-048 [P2]** release 审计失败通知模板。
49. **R18-049 [P2]** 版本发布 checklist 与审计脚本联动。
50. **R18-050 [P2]** Round19 候选任务自动生成器。

### Priority Recommendation (Round 18)
- **Batch1 (P0, 本轮已执行):** R18-001 / R18-003 / R18-005。
- **Batch2 (P1):** R18-011 / R18-017 / R18-018。
- **Batch3 (P1):** R18-021 / R18-024 / R18-030。

## Plan Output (Round 18)
- `docs/plans/2026-02-11-repo-gap-priority-round18.md`

## Execution Results (Round 18)
- Task A18 完成：新增 `.github/workflows/ci-contract-gates.yml`，CI 集成 shell + pytest + rust 门禁。
- Task B18 完成：新增 `docs/CLI_CONTRACT_MATRIX.md`，覆盖 13 个核心 CLI/Hook 脚本参数契约。
- Task C18 完成：新增 `scripts/release-contract-audit.sh`，支持 `--dry-run` 并可执行一键审计。
- 回归结果：
  - 新增测试集 `11 passed, 22 subtests passed`。
  - Round18 targeted `87 passed, 22 subtests passed`。
  - 全量 `394 passed, 22 subtests passed`。
  - `./scripts/release-contract-audit.sh` 全门禁通过。

## Scan Results (Round 19)

### Repository Health Snapshot
- 全量回归：`pytest -q` -> `403 passed, 23 subtests passed`。
- Shell 语法门禁：`bash -n scripts/*.sh` -> `OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` 均通过。
- Round19 探针关键发现：
  - `release-contract-audit.sh` 不支持组合参数（`--dry-run --fast` 等）并缺少失败 step 汇总（已修复）。
  - `regression_runner.py --suite contract` 误落 full suite；未知 suite 未报错（已修复）。
  - 文档未覆盖 CI gate 与 release audit 入口；CLI matrix 缺 `help exit code` 列（已修复）。

### 50-Task Gap Backlog (Prioritized)
1. **R19-001 [P0]** release-audit 支持 `--fast`（已执行）。
2. **R19-002 [P0]** release-audit 支持 `--skip-rust`（已执行）。
3. **R19-003 [P0]** release-audit 支持 `--skip-python`（已执行）。
4. **R19-004 [P0]** release-audit 失败 step summary 输出（已执行）。
5. **R19-005 [P0]** release-audit 组合参数测试补齐（已执行）。
6. **R19-006 [P0]** regression_runner 新增 `contract` suite（已执行）。
7. **R19-007 [P0]** regression_runner unknown suite 非零退出（已执行）。
8. **R19-008 [P0]** docs freshness 守卫 CI/release 文档链接（已执行）。
9. **R19-009 [P0]** CLI matrix 增加 `help exit code` 列（已执行）。
10. **R19-010 [P0]** README 中英补齐 CI/release gate 文档（已执行）。
11. **R19-011 [P1]** release-audit 支持 `--json` summary。
12. **R19-012 [P1]** release-audit 输出失败阶段耗时统计。
13. **R19-013 [P1]** release-audit 支持 `--contract-only` 显式开关。
14. **R19-014 [P1]** release-audit 支持 `--no-color` 输出。
15. **R19-015 [P1]** release-audit 支持命令重试策略（可配置）。
16. **R19-016 [P1]** release-audit 支持自定义 pytest 参数透传。
17. **R19-017 [P1]** CI workflow 增加 pip/cargo cache。
18. **R19-018 [P1]** CI workflow 增加 matrix（python/rust 版本策略）。
19. **R19-019 [P1]** CI workflow 增加 quick/full 分层 job。
20. **R19-020 [P1]** CI workflow 增加 docs freshness 专项 job。
21. **R19-021 [P1]** regression_runner `contract` suite 与 shell contract tests 对齐检查。
22. **R19-022 [P1]** regression_runner 支持 suite 列表打印。
23. **R19-023 [P1]** regression_runner 输出 machine-readable JSON。
24. **R19-024 [P1]** regression_runner 将 suite 元数据写入 `.fusion/events.jsonl`。
25. **R19-025 [P1]** regression_runner contract suite 增加 hook-doctor 场景。
26. **R19-026 [P1]** regression_runner contract suite 增加 status-json 场景。
27. **R19-027 [P1]** regression_runner contract suite 增加 logs/git error-channel 场景。
28. **R19-028 [P1]** CLI matrix 增加每条命令样例（valid/invalid）。
29. **R19-029 [P1]** CLI matrix 增加 shell return-code 统一规范说明。
30. **R19-030 [P1]** HOOKS_SETUP 增加 release-audit fail triage 指南。
31. **R19-031 [P2]** README 增加 CI badge。
32. **R19-032 [P2]** README.zh-CN 增加 CI badge 对应说明。
33. **R19-033 [P2]** docs E2E 新增 release-audit --fast 示例。
34. **R19-034 [P2]** docs E2E 新增 release-audit --skip-python 示例。
35. **R19-035 [P2]** docs E2E 新增 regression_runner --suite contract 示例。
36. **R19-036 [P2]** docs E2E 新增 unknown suite 错误示例。
37. **R19-037 [P2]** test_release_contract_audit_script 增加 forced fail exit code 断言。
38. **R19-038 [P2]** test_release_contract_audit_script 增加 combined skip 行为断言。
39. **R19-039 [P2]** test_docs_freshness 增加 HOOKS_SETUP 命令块语法守卫。
40. **R19-040 [P2]** test_docs_freshness 增加 CLI matrix 行数守卫。
41. **R19-041 [P2]** 引入 shellcheck 并与 CI 集成。
42. **R19-042 [P2]** release-audit 脚本抽离共享命令表配置。
43. **R19-043 [P2]** release-audit 支持 `FUSION_RELEASE_AUDIT_PROFILE`。
44. **R19-044 [P2]** release-audit 与 regression_runner profile 对齐。
45. **R19-045 [P2]** CI workflow 对 release-audit 脚本进行 smoke run。
46. **R19-046 [P2]** scripts 目录新增 `contract-profile.env` 模板。
47. **R19-047 [P2]** 发布 checklist 文档整合 release-audit options。
48. **R19-048 [P2]** 指南增加“本地与 CI 差异排查”章节。
49. **R19-049 [P2]** Round20 automation：基于 findings 自动生成 plan 草稿。
50. **R19-050 [P2]** Round20 automation：基于 progress 自动回填执行摘要。

### Priority Recommendation (Round 19)
- **Batch1 (P0, 本轮已执行):** R19-001 / R19-006 / R19-008。
- **Batch2 (P1):** R19-011 / R19-017 / R19-021。
- **Batch3 (P1):** R19-023 / R19-028 / R19-030。

## Plan Output (Round 19)
- `docs/plans/2026-02-11-repo-gap-priority-round19.md`

## Execution Results (Round 19)
- Task A19 完成：`release-contract-audit.sh` 增加 `--fast`/`--skip-rust`/`--skip-python`，并实现 step-level 失败汇总。
- Task B19 完成：`regression_runner.py` 新增 `contract` 套件分支并拒绝未知 suite。
- Task C19 完成：文档体系补齐 CI/release 契约，并将 matrix 扩展到 `help exit code`。
- 回归结果：
  - 新增测试集 `18 passed, 19 subtests passed`。
  - Round19 targeted `96 passed, 23 subtests passed`。
  - 全量 `403 passed, 23 subtests passed`。
  - `./scripts/release-contract-audit.sh --fast` 全门禁通过。
  - `python3 scripts/runtime/regression_runner.py --suite contract --min-pass-rate 0.99` 通过。

## Scan Results (Round 20)

### Repository Health Snapshot
- 全量回归：`pytest -q` -> `406 passed, 23 subtests passed`。
- Shell 语法：`bash -n scripts/*.sh` -> `OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` 与 `cargo fmt --all -- --check` 均通过。
- Round20 探针结论：
  - `release-contract-audit.sh` 缺少 `--json`（已补齐）。
  - `regression_runner.py` 缺少 suite 列表能力（已补齐）。
  - CI workflow 缺少 pip/rust cache 步骤（已补齐）。

### 50-Task Gap Backlog (Prioritized)
1. **R20-001 [P0]** release-audit `--json` dry-run summary（已执行）。
2. **R20-002 [P0]** release-audit JSON flags 字段标准化（已执行）。
3. **R20-003 [P0]** release-audit JSON commands 清单输出（已执行）。
4. **R20-004 [P0]** regression_runner `--list-suites`（已执行）。
5. **R20-005 [P0]** regression_runner list 输出包含 contract（已执行）。
6. **R20-006 [P0]** CI workflow pip cache（已执行）。
7. **R20-007 [P0]** CI workflow rust cache（已执行）。
8. **R20-008 [P0]** cache 步骤契约测试补齐（已执行）。
9. **R20-009 [P0]** release/runner/ci 专项回归门禁（已执行）。
10. **R20-010 [P0]** 全量回归与 rust 门禁再验证（已执行）。
11. **R20-011 [P1]** release-audit JSON run-mode 失败详情（stderr 摘要）细化。
12. **R20-012 [P1]** release-audit JSON 增加耗时统计字段。
13. **R20-013 [P1]** release-audit JSON 增加步骤级结果数组。
14. **R20-014 [P1]** release-audit 增加 `--json-pretty`。
15. **R20-015 [P1]** release-audit JSON schema 文档化。
16. **R20-016 [P1]** regression_runner `--list-suites --json` 输出。
17. **R20-017 [P1]** regression_runner suite 元数据（scenario count）输出。
18. **R20-018 [P1]** regression_runner contract suite 与 shell contract 命令自动对账。
19. **R20-019 [P1]** regression_runner 增加 `--suite quick`。
20. **R20-020 [P1]** regression_runner unknown suite 返回码文档化。
21. **R20-021 [P1]** CI workflow 缓存 key 精细化（lockfile/hash）。
22. **R20-022 [P1]** CI workflow 分离 quick/full jobs。
23. **R20-023 [P1]** CI workflow 增加 release-audit smoke job。
24. **R20-024 [P1]** CI workflow 增加 regression_runner contract job。
25. **R20-025 [P1]** CI workflow docs freshness job。
26. **R20-026 [P1]** docs/HOOKS_SETUP 增加 `--json` 输出示例。
27. **R20-027 [P1]** README EN 增加 `--list-suites` 示例。
28. **R20-028 [P1]** README ZH 增加 `--list-suites` 示例。
29. **R20-029 [P1]** CLI matrix 增加 release-audit `--json` 行。
30. **R20-030 [P1]** CLI matrix 增加 regression_runner CLI 行。
31. **R20-031 [P2]** release-audit 增加 `--output <file>`。
32. **R20-032 [P2]** release-audit JSON + text 双通道日志模式。
33. **R20-033 [P2]** release-audit 集成 shellcheck 可选门禁。
34. **R20-034 [P2]** CI workflow 支持 shellcheck gate。
35. **R20-035 [P2]** CI workflow 支持 markdownlint gate。
36. **R20-036 [P2]** regression_runner 支持 `--suite from-file`。
37. **R20-037 [P2]** regression_runner 支持失败重试。
38. **R20-038 [P2]** regression_runner 失败场景快照输出。
39. **R20-039 [P2]** docs E2E 增加 release-audit JSON 示例。
40. **R20-040 [P2]** docs E2E 增加 list-suites 示例。
41. **R20-041 [P2]** docs freshness 增加 workflow cache 文案守卫。
42. **R20-042 [P2]** docs freshness 增加 release-audit JSON 文案守卫。
43. **R20-043 [P2]** .claude 模板增加 release gate 命令提示。
44. **R20-044 [P2]** CHANGELOG 自动化写入 release gate 变更。
45. **R20-045 [P2]** findings/progress 自动摘要脚本。
46. **R20-046 [P2]** 统一质量门禁入口（Makefile/justfile）评估。
47. **R20-047 [P2]** rust bridge 与 release audit 联动策略。
48. **R20-048 [P2]** contract matrix 与测试文件自动一致性校验。
49. **R20-049 [P2]** Round21 planner 自动挑选 top-3 P1 实施项。
50. **R20-050 [P2]** Round21 planner 自动生成 RED 测试模板。

### Priority Recommendation (Round 20)
- **Batch1 (P0, 本轮已执行):** R20-001 / R20-004 / R20-006。
- **Batch2 (P1):** R20-011 / R20-016 / R20-021。
- **Batch3 (P1):** R20-018 / R20-024 / R20-029。

## Plan Output (Round 20)
- `docs/plans/2026-02-11-repo-gap-priority-round20.md`

## Execution Results (Round 20)
- Task A20 完成：`release-contract-audit.sh` 增加 `--json`，可输出 dry-run 机器摘要。
- Task B20 完成：`regression_runner.py` 增加 `--list-suites`。
- Task C20 完成：CI workflow 增加 pip cache 与 rust cache，并补齐测试守卫。
- 回归结果：
  - 新增测试集 `14 passed, 4 subtests passed`。
  - Round20 targeted `99 passed, 23 subtests passed`。
  - 全量 `406 passed, 23 subtests passed`。
  - `release-contract-audit --dry-run --json --fast --skip-rust` 输出合法 JSON。
  - `regression_runner --list-suites` 输出 `phase1/phase2/contract/all`。

## Scan Results (Round 21)

### Repository Health Snapshot
- 全量回归：`pytest -q` -> `410 passed, 23 subtests passed`。
- Shell 语法门禁：`bash -n scripts/*.sh` -> `OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` 均通过。
- Round21 探针确认缺口：
  - `release-contract-audit.sh` 不支持 `--json-pretty`（已修复）。
  - `regression_runner.py` 不支持 `--list-suites --json`（已修复）。
  - 文档未同步新命令（已修复）。

### 50-Task Gap Backlog (Prioritized)
1. **R21-001 [P0]** release-audit `--json-pretty` 参数支持（已执行）。
2. **R21-002 [P0]** release-audit JSON payload 增加 `json_pretty` flag（已执行）。
3. **R21-003 [P0]** release-audit `--json-pretty` 依赖 `--json` 校验（已执行）。
4. **R21-004 [P0]** release-audit pretty 输出契约测试（已执行）。
5. **R21-005 [P0]** regression_runner `--list-suites --json`（已执行）。
6. **R21-006 [P0]** regression_runner list JSON 默认 suite 字段（已执行）。
7. **R21-007 [P0]** docs freshness 覆盖 `--json-pretty`（已执行）。
8. **R21-008 [P0]** docs freshness 覆盖 `--list-suites --json`（已执行）。
9. **R21-009 [P0]** CLI matrix 增加 release-audit/runner 行（已执行）。
10. **R21-010 [P0]** README/HOOKS_SETUP 同步机器模式示例（已执行）。
11. **R21-011 [P1]** release-audit JSON 增加步骤耗时数组。
12. **R21-012 [P1]** release-audit JSON 增加失败原因分类码。
13. **R21-013 [P1]** release-audit JSON 增加 host/runtime 元数据。
14. **R21-014 [P1]** release-audit `--json` run-mode 成功输出步骤统计。
15. **R21-015 [P1]** release-audit `--json` 对 stdout/stderr 分流策略文档化。
16. **R21-016 [P1]** regression_runner `--list-suites --json-pretty`。
17. **R21-017 [P1]** regression_runner `--json` for suite run summary。
18. **R21-018 [P1]** regression_runner 输出 schema 文档化。
19. **R21-019 [P1]** regression_runner list 输出包含 scenario counts。
20. **R21-020 [P1]** regression_runner list 输出包含 suite descriptions。
21. **R21-021 [P1]** CI workflow 增加 release-audit JSON smoke step。
22. **R21-022 [P1]** CI workflow 增加 regression_runner list-suites smoke step。
23. **R21-023 [P1]** CI workflow job summary 注入关键 gate 结果。
24. **R21-024 [P1]** CI workflow 失败时 artifact 上传 gate logs。
25. **R21-025 [P1]** docs/HOOKS_SETUP 增加 JSON triage 指南。
26. **R21-026 [P1]** README EN 增加 machine mode 场景说明。
27. **R21-027 [P1]** README ZH 增加 machine mode 场景说明。
28. **R21-028 [P1]** CLI matrix 增加 runner unknown suite 失败样例。
29. **R21-029 [P1]** CLI matrix 增加 release-audit forced-fail 样例。
30. **R21-030 [P1]** docs E2E 增加 JSON-pretty 实战输出片段。
31. **R21-031 [P2]** release-audit JSON 输出到文件开关。
32. **R21-032 [P2]** release-audit JSON schema 校验测试。
33. **R21-033 [P2]** release-audit 结构化日志适配 `jq` 示例。
34. **R21-034 [P2]** regression_runner 加入 `--suites-from-file`。
35. **R21-035 [P2]** regression_runner 加入 suite alias（c=contract）。
36. **R21-036 [P2]** regression_runner list 本地化（en/zh）选项。
37. **R21-037 [P2]** CI workflow 增加 fail-fast 策略调优。
38. **R21-038 [P2]** CI workflow 增加 concurrency group。
39. **R21-039 [P2]** CI workflow 仅变更 docs 时走轻量门禁。
40. **R21-040 [P2]** docs freshness 增加 matrix 行完整性守卫。
41. **R21-041 [P2]** docs freshness 增加 README 命令块可执行性守卫。
42. **R21-042 [P2]** release-audit 单元测试细分 run-mode error 分支。
43. **R21-043 [P2]** runner 单元测试覆盖 `--json` 与 `--scenario` 组合非法输入。
44. **R21-044 [P2]** findings/progress 与 release-audit 输出自动关联。
45. **R21-045 [P2]** quality dashboard 统计 round-level pass trend。
46. **R21-046 [P2]** Round22 自动挑选 top P1 backlog 实施。
47. **R21-047 [P2]** Round22 生成命令清单模板。
48. **R21-048 [P2]** Round22 docs freshness 追加 contract matrix schema section。
49. **R21-049 [P2]** Round22 CI 添加 release-audit full run nightly。
50. **R21-050 [P2]** Round22 CI 添加 regression_runner contract nightly。

### Priority Recommendation (Round 21)
- **Batch1 (P0, 本轮已执行):** R21-001 / R21-005 / R21-007。
- **Batch2 (P1):** R21-011 / R21-017 / R21-021。
- **Batch3 (P1):** R21-019 / R21-025 / R21-028。

## Plan Output (Round 21)
- `docs/plans/2026-02-11-repo-gap-priority-round21.md`

## Execution Results (Round 21)
- Task A21 完成：`release-contract-audit.sh` 增加 `--json-pretty` 并与 `--json` 绑定校验。
- Task B21 完成：`regression_runner.py` 支持 `--list-suites --json` 机器输出。
- Task C21 完成：README/README.zh-CN/HOOKS_SETUP/CLI matrix 完成机器模式命令同步。
- 回归结果：
  - 新增测试集 `24 passed, 19 subtests passed`。
  - Round21 targeted `103 passed, 23 subtests passed`。
  - 全量 `410 passed, 23 subtests passed`。
  - `release-contract-audit --dry-run --json --json-pretty --fast --skip-rust` 输出多行 JSON。
  - `regression_runner --list-suites --json` 输出机器 payload。

## Scan Results (Round 22)

### Repository Health Snapshot
- 全量回归：`pytest -q` -> `413 passed, 23 subtests passed`。
- Shell 语法：`bash -n scripts/*.sh` -> `OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` 通过。
- Round22 probe 发现并修复：
  - release-audit run JSON 缺少 `steps_executed/step_results/total_duration_ms`。
  - regression_runner `--suite contract --json` 未输出机器汇总。
  - CI workflow 缺少 machine-mode smoke gate。

### 50-Task Gap Backlog (Prioritized)
1. **R22-001 [P0]** release-audit run JSON 增加 `steps_executed`（已执行）。
2. **R22-002 [P0]** release-audit run JSON 增加 `step_results`（已执行）。
3. **R22-003 [P0]** release-audit run JSON 增加 `total_duration_ms`（已执行）。
4. **R22-004 [P0]** step_result 增加 `status/duration_ms/command`（已执行）。
5. **R22-005 [P0]** regression_runner suite run JSON summary（已执行）。
6. **R22-006 [P0]** regression_runner JSON summary 增加 `suite` 字段（已执行）。
7. **R22-007 [P0]** regression_runner JSON summary 增加 `pass_rate/min_pass_rate`（已执行）。
8. **R22-008 [P0]** CI workflow 增加 release-audit machine smoke（已执行）。
9. **R22-009 [P0]** CI workflow 增加 runner list-suites machine smoke（已执行）。
10. **R22-010 [P0]** CI machine smoke 契约测试补齐（已执行）。
11. **R22-011 [P1]** release-audit run JSON 输出每步开始/结束时间戳。
12. **R22-012 [P1]** release-audit run JSON 增加失败分类码。
13. **R22-013 [P1]** release-audit run JSON 增加命令重试次数。
14. **R22-014 [P1]** release-audit run JSON 增加 host/system metadata。
15. **R22-015 [P1]** release-audit 支持 `--json-output <file>`。
16. **R22-016 [P1]** release-audit 支持 `--json-output-pretty`。
17. **R22-017 [P1]** regression_runner suite run JSON 增加失败场景列表。
18. **R22-018 [P1]** regression_runner suite run JSON 增加 per-scenario duration。
19. **R22-019 [P1]** regression_runner suite run JSON 增加 args 回显。
20. **R22-020 [P1]** regression_runner suite run JSON schema 文档化。
21. **R22-021 [P1]** CI workflow 增加 suite-run JSON smoke step。
22. **R22-022 [P1]** CI workflow 保存 machine JSON 输出为 artifact。
23. **R22-023 [P1]** CI workflow 在 summary 中展示 machine payload 关键字段。
24. **R22-024 [P1]** CI workflow 按变更类型启停 smoke steps。
25. **R22-025 [P1]** docs/HOOKS_SETUP 增加 run-mode JSON 输出示例。
26. **R22-026 [P1]** README EN 增加 suite-run JSON 示例。
27. **R22-027 [P1]** README ZH 增加 suite-run JSON 示例。
28. **R22-028 [P1]** CLI matrix 增加 run-mode JSON 指标字段说明。
29. **R22-029 [P1]** CLI matrix 增加 runner JSON summary 字段说明。
30. **R22-030 [P1]** docs freshness 守卫新增 run-mode JSON 示例。
31. **R22-031 [P2]** release-audit 增加 `--profile quick/full`。
32. **R22-032 [P2]** release-audit 增加 gate 并行执行（可选）。
33. **R22-033 [P2]** release-audit 支持 step timeout。
34. **R22-034 [P2]** release-audit 输出兼容 ndjson。
35. **R22-035 [P2]** runner `--json-pretty` for suite summaries。
36. **R22-036 [P2]** runner `--json` output include git sha。
37. **R22-037 [P2]** runner `--json` output include python/rust versions。
38. **R22-038 [P2]** CI workflow matrix 运行 contract suite JSON。
39. **R22-039 [P2]** CI workflow 引入 junit 报告上传。
40. **R22-040 [P2]** CI workflow 机器输出一致性校验。
41. **R22-041 [P2]** docs E2E 增加 run JSON step_results 示例。
42. **R22-042 [P2]** docs E2E 增加 runner suite JSON 示例。
43. **R22-043 [P2]** docs freshness 验证 JSON key list 文本一致性。
44. **R22-044 [P2]** lint rule 防止 CLI 矩阵漏更新。
45. **R22-045 [P2]** 自动脚本从测试生成 CLI matrix 草稿。
46. **R22-046 [P2]** 自动脚本从 workflow 生成 gate 摘要。
47. **R22-047 [P2]** Round23 planner 自动挑选 top P1 with tests。
48. **R22-048 [P2]** Round23 planner 生成 RED checklist。
49. **R22-049 [P2]** Round23 planner 生成 VERIFY command bundle。
50. **R22-050 [P2]** Round23 planner 自动回写 findings/progress stub。

### Priority Recommendation (Round 22)
- **Batch1 (P0, 本轮已执行):** R22-001 / R22-005 / R22-008。
- **Batch2 (P1):** R22-011 / R22-017 / R22-021。
- **Batch3 (P1):** R22-025 / R22-028 / R22-030。

## Plan Output (Round 22)
- `docs/plans/2026-02-11-repo-gap-priority-round22.md`

## Execution Results (Round 22)
- Task A22 完成：`release-contract-audit.sh` 在 run-mode JSON 下输出步骤与耗时指标。
- Task B22 完成：`regression_runner.py --suite contract --json` 输出机器汇总。
- Task C22 完成：CI workflow 增加 machine-mode smoke gate。
- 回归结果：
  - 新增测试集 `19 passed, 4 subtests passed`。
  - Round22 targeted `106 passed, 23 subtests passed`。
  - 全量 `413 passed, 23 subtests passed`。
  - `release-contract-audit --json --skip-python --skip-rust` 输出 step metrics。
  - `regression_runner --suite contract --json` 输出 suite summary JSON。

## Scan Results (Round 23)

### Repository Health Snapshot
- 全量回归：`pytest -q` -> `413 passed, 23 subtests passed`。
- Shell 语法：`bash -n scripts/*.sh` -> `OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` 通过。
- Round23 关键缺口并已收口：
  - release-audit step_results 缺 `step/started_at_ms/finished_at_ms`。
  - runner suite JSON 缺 `scenario_results/failed_scenarios`。
  - CI machine smoke 未覆盖 suite JSON 执行路径。

### 50-Task Gap Backlog (Prioritized)
1. **R23-001 [P0]** release-audit step_results 增加 step index（已执行）。
2. **R23-002 [P0]** release-audit step_results 增加 started_at_ms（已执行）。
3. **R23-003 [P0]** release-audit step_results 增加 finished_at_ms（已执行）。
4. **R23-004 [P0]** release-audit step_results 时间顺序断言（已执行）。
5. **R23-005 [P0]** runner suite JSON 增加 scenario_results（已执行）。
6. **R23-006 [P0]** runner suite JSON 增加 failed_scenarios（已执行）。
7. **R23-007 [P0]** scenario_results 包含 name/passed/duration_ms/error（已执行）。
8. **R23-008 [P0]** CI machine smoke 增加 suite contract JSON 命令（已执行）。
9. **R23-009 [P0]** CI smoke 命令契约测试补齐（已执行）。
10. **R23-010 [P0]** Round23 全回归验证通过（已执行）。
11. **R23-011 [P1]** release-audit step_results 增加 exit_code 字段。
12. **R23-012 [P1]** release-audit step_results 增加 retry_count 字段。
13. **R23-013 [P1]** release-audit step_results 增加 stderr_line_count。
14. **R23-014 [P1]** release-audit JSON run 输出 aggregate stats（p50/p95）。
15. **R23-015 [P1]** release-audit JSON 输出 schema 文档化。
16. **R23-016 [P1]** runner JSON 增加 scenario_type 分组统计。
17. **R23-017 [P1]** runner JSON 增加 longest_scenario 字段。
18. **R23-018 [P1]** runner JSON 增加 fail_reasons 去重统计。
19. **R23-019 [P1]** runner JSON 增加 command metadata。
20. **R23-020 [P1]** runner JSON 输出 pretty 模式。
21. **R23-021 [P1]** CI workflow 对 suite JSON 输出做 schema smoke 校验。
22. **R23-022 [P1]** CI workflow 保存 suite JSON 为 artifact。
23. **R23-023 [P1]** CI workflow 保存 release-audit JSON 为 artifact。
24. **R23-024 [P1]** CI workflow step summary 展示 machine json 关键字段。
25. **R23-025 [P1]** docs/HOOKS_SETUP 增加 step_results 字段解释。
26. **R23-026 [P1]** README EN 增加 suite JSON 字段解释。
27. **R23-027 [P1]** README ZH 增加 suite JSON 字段解释。
28. **R23-028 [P1]** CLI matrix 增加 step_results schema 摘要。
29. **R23-029 [P1]** CLI matrix 增加 scenario_results schema 摘要。
30. **R23-030 [P1]** docs freshness 增加新字段文案守卫。
31. **R23-031 [P2]** release-audit 支持 step-level timeout 配置。
32. **R23-032 [P2]** release-audit 支持 step-level parallel group。
33. **R23-033 [P2]** release-audit 输出 ndjson stream mode。
34. **R23-034 [P2]** runner 支持 scenario filter（name regex）。
35. **R23-035 [P2]** runner 支持 failed-only rerun 输出 JSON。
36. **R23-036 [P2]** runner 支持 per-scenario stdout capture。
37. **R23-037 [P2]** CI workflow 增加 nightly full JSON capture。
38. **R23-038 [P2]** CI workflow 增加 flaky detection for scenario durations。
39. **R23-039 [P2]** docs E2E 增加 step timestamp 样例。
40. **R23-040 [P2]** docs E2E 增加 scenario_results 样例。
41. **R23-041 [P2]** docs freshness 增加 JSON key ordering guard。
42. **R23-042 [P2]** 自动脚本对比 findings backlog 与 tests 覆盖。
43. **R23-043 [P2]** 自动脚本生成 CI smoke contract checklist。
44. **R23-044 [P2]** 自动脚本按 round 汇总 test 增长趋势。
45. **R23-045 [P2]** Round24 planner 自动选 top3 P1 with smallest diff。
46. **R23-046 [P2]** Round24 planner 自动生成 RED-only test scaffold。
47. **R23-047 [P2]** Round24 planner 自动生成 VERIFY command bundle。
48. **R23-048 [P2]** Round24 planner 自动写 docs update checklist。
49. **R23-049 [P2]** Round24 planner 自动写 CI update checklist。
50. **R23-050 [P2]** Round24 planner 自动写 findings/progress stub。

### Priority Recommendation (Round 23)
- **Batch1 (P0, 本轮已执行):** R23-001 / R23-005 / R23-008。
- **Batch2 (P1):** R23-011 / R23-016 / R23-021。
- **Batch3 (P1):** R23-025 / R23-028 / R23-030。

## Plan Output (Round 23)
- `docs/plans/2026-02-11-repo-gap-priority-round23.md`

## Execution Results (Round 23)
- Task A23 完成：release-audit `step_results` 增加 `step/started_at_ms/finished_at_ms`。
- Task B23 完成：runner suite JSON 增加 `scenario_results` 与 `failed_scenarios`。
- Task C23 完成：CI machine smoke 新增 suite contract JSON 命令。
- 回归结果：
  - 新增测试集 `19 passed, 4 subtests passed`。
  - Round23 targeted `106 passed, 23 subtests passed`。
  - 全量 `413 passed, 23 subtests passed`。
  - run JSON / suite JSON 均已输出扩展字段。

## Scan Results (Round 24)

### Repository Health Snapshot
- 执行前基线：`pytest -q` -> `413 passed, 23 subtests passed`。
- Round24 完成后：`pytest -q` -> `415 passed, 23 subtests passed`。
- Shell 语法：`bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> `OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` 通过。
- Round24 关键缺口并收口：
  - release-audit `step_results` 缺 `exit_code`。
  - runner suite JSON 缺 `longest_scenario` 聚合。
  - CI machine mode 未上传 JSON artifacts。

### 50-Task Gap Backlog (Prioritized)
1. **R24-001 [P0]** release-audit `step_results` 增加 `exit_code`（已执行）。
2. **R24-002 [P0]** release-audit forced-fail 场景断言 step exit_code（已执行）。
3. **R24-003 [P0]** runner suite JSON 增加 `longest_scenario`（已执行）。
4. **R24-004 [P0]** contract suite JSON longest 字段契约测试（已执行）。
5. **R24-005 [P0]** CI workflow 上传 machine JSON artifacts（已执行）。
6. **R24-006 [P0]** CI artifact step 合约测试补齐（已执行）。
7. **R24-007 [P0]** Round24 目标测试三件套全绿（已执行）。
8. **R24-008 [P0]** Round24 全量 pytest 门禁全绿（已执行）。
9. **R24-009 [P0]** Round24 rust clippy/fmt 门禁全绿（已执行）。
10. **R24-010 [P0]** Round24 扫描/计划/执行/验证记录落盘（已执行）。
11. **R24-011 [P1]** release-audit `step_results` 增加 `stderr_lines` 计数。
12. **R24-012 [P1]** release-audit `step_results` 增加 `stdout_lines` 计数。
13. **R24-013 [P1]** release-audit 增加 `failed_steps` 摘要列表。
14. **R24-014 [P1]** release-audit 输出 `max_step_duration_ms`。
15. **R24-015 [P1]** release-audit 输出 `avg_step_duration_ms`。
16. **R24-016 [P1]** runner suite JSON 增加 `fastest_scenario`。
17. **R24-017 [P1]** runner suite JSON 增加 `median_duration_ms`。
18. **R24-018 [P1]** runner suite JSON 增加 `p95_duration_ms`。
19. **R24-019 [P1]** runner suite JSON 增加 `slow_scenarios`（阈值过滤）。
20. **R24-020 [P1]** runner suite JSON 增加 `scenario_count_by_result`。
21. **R24-021 [P1]** CI workflow `runner-contract.json` schema smoke。
22. **R24-022 [P1]** CI workflow `release-audit-dry-run.json` schema smoke。
23. **R24-023 [P1]** CI summary 展示 longest scenario。
24. **R24-024 [P1]** CI summary 展示 release step failure_count。
25. **R24-025 [P1]** docs/CLI_CONTRACT_MATRIX 增补 longest_scenario 字段。
26. **R24-026 [P1]** README EN machine JSON 示例更新。
27. **R24-027 [P1]** README ZH machine JSON 示例更新。
28. **R24-028 [P1]** docs/HOOKS_SETUP 增加 CI artifacts 说明。
29. **R24-029 [P1]** docs/E2E_EXAMPLE 增加 suite JSON 样例。
30. **R24-030 [P1]** docs freshness 新增 artifact 文案守卫。
31. **R24-031 [P2]** release-audit 支持 `--json-schema` 输出。
32. **R24-032 [P2]** release-audit 支持 NDJSON 流输出。
33. **R24-033 [P2]** runner suite 支持 `--json-pretty`。
34. **R24-034 [P2]** runner suite 支持 `--scenario-regex` 过滤。
35. **R24-035 [P2]** runner suite 支持仅输出失败场景 `--failed-only`。
36. **R24-036 [P2]** runner suite 输出兼容 v2 schema 版本号。
37. **R24-037 [P2]** CI workflow 增加 nightly machine JSON capture。
38. **R24-038 [P2]** CI workflow 对 JSON artifact 做 retention 策略。
39. **R24-039 [P2]** CI workflow 增加 flaky-duration 报警阈值。
40. **R24-040 [P2]** docs 增补 machine JSON 字段演进历史。
41. **R24-041 [P2]** docs 增补失败案例与排错手册。
42. **R24-042 [P2]** 自动脚本生成 contract JSON 字段差异报告。
43. **R24-043 [P2]** 自动脚本校验 workflow 与 tests 一致性。
44. **R24-044 [P2]** 自动脚本聚合 round-by-round 通过率趋势。
45. **R24-045 [P2]** Round25 planner 自动挑选最小改动 P1 任务。
46. **R24-046 [P2]** Round25 planner 自动生成 RED 命令模板。
47. **R24-047 [P2]** Round25 planner 自动生成 GREEN 验证命令模板。
48. **R24-048 [P2]** Round25 planner 自动生成 VERIFY 门禁命令包。
49. **R24-049 [P2]** Round25 planner 自动生成 docs/checklist 模板。
50. **R24-050 [P2]** Round25 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 24)
- **Batch1 (P0, 本轮已执行):** R24-001 / R24-003 / R24-005。
- **Batch2 (P1):** R24-011 / R24-016 / R24-021。
- **Batch3 (P1):** R24-025 / R24-028 / R24-030。

## Plan Output (Round 24)
- `docs/plans/2026-02-11-repo-gap-priority-round24.md`

## Execution Results (Round 24)
- Task A24 完成：release-audit `step_results` 增加 `exit_code`。
- Task B24 完成：runner suite JSON 增加 `longest_scenario` 聚合。
- Task C24 完成：CI machine mode 上传 JSON artifacts。
- 回归结果：
  - 目标测试 `21 passed, 4 subtests passed`。
  - 全量 `415 passed, 23 subtests passed`。
  - shell + rust 门禁均通过。

## Scan Results (Round 25)

### Repository Health Snapshot
- 执行前基线：`pytest -q` -> `415 passed, 23 subtests passed`。
- 执行后结果：`pytest -q` -> `416 passed, 23 subtests passed`。
- Shell 语法门禁：`bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh` -> `OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> 通过。
- Round25 核心缺口：
  - release-audit JSON 缺失败聚合字段（`failed_steps`）。
  - runner suite JSON 缺最短耗时摘要（`fastest_scenario`）。
  - CI machine smoke 缺对 runner JSON 结构的 schema smoke。

### 50-Task Gap Backlog (Prioritized)
1. **R25-001 [P0]** release-audit JSON 增加 `failed_steps`（已执行）。
2. **R25-002 [P0]** release-audit JSON 增加 `failed_steps_count`（已执行）。
3. **R25-003 [P0]** force-fail 场景断言 failed_steps 聚合正确（已执行）。
4. **R25-004 [P0]** runner suite JSON 增加 `fastest_scenario`（已执行）。
5. **R25-005 [P0]** contract suite JSON 测试覆盖 fastest/longest 关系（已执行）。
6. **R25-006 [P0]** CI machine smoke 增加 runner JSON schema smoke（已执行）。
7. **R25-007 [P0]** CI 测试覆盖 schema smoke 命令存在性（已执行）。
8. **R25-008 [P0]** Round25 目标测试集全绿（已执行）。
9. **R25-009 [P0]** Round25 全量 pytest 全绿（已执行）。
10. **R25-010 [P0]** Round25 rust/clippy/fmt 门禁全绿（已执行）。
11. **R25-011 [P1]** release-audit JSON 增加 `failed_commands` 列表。
12. **R25-012 [P1]** release-audit JSON 增加 `success_steps_count`。
13. **R25-013 [P1]** release-audit JSON 增加 `max_step_duration_ms`。
14. **R25-014 [P1]** release-audit JSON 增加 `avg_step_duration_ms`。
15. **R25-015 [P1]** release-audit JSON 增加 schema 版本号字段。
16. **R25-016 [P1]** runner JSON 增加 `scenario_count_by_result`。
17. **R25-017 [P1]** runner JSON 增加 `median_duration_ms`。
18. **R25-018 [P1]** runner JSON 增加 `p95_duration_ms`。
19. **R25-019 [P1]** runner JSON 增加 `slow_scenarios` 阈值过滤。
20. **R25-020 [P1]** runner JSON 增加 `empty_suite` 保护字段。
21. **R25-021 [P1]** CI smoke 对 `/tmp/release-audit-dry-run.json` 做 schema 校验。
22. **R25-022 [P1]** CI smoke 校验 `/tmp/runner-suites.json` 的 default/suites 一致性。
23. **R25-023 [P1]** CI summary 输出 longest/fastest 场景摘要。
24. **R25-024 [P1]** CI summary 输出 failed_steps_count 摘要。
25. **R25-025 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `fastest_scenario` 字段描述。
26. **R25-026 [P1]** README EN 增补 runner JSON 字段样例（longest/fastest）。
27. **R25-027 [P1]** README ZH 增补 runner JSON 字段样例（longest/fastest）。
28. **R25-028 [P1]** HOOKS_SETUP 增补 machine schema smoke 说明。
29. **R25-029 [P1]** E2E_EXAMPLE 增加 machine JSON 样例片段。
30. **R25-030 [P1]** docs freshness 新增 fastest_scenario 文案守卫。
31. **R25-031 [P2]** release-audit 支持 `--json-schema` 输出文件。
32. **R25-032 [P2]** release-audit 支持 `--ndjson` streaming。
33. **R25-033 [P2]** runner 支持 `--json-pretty`。
34. **R25-034 [P2]** runner 支持 `--scenario-regex`。
35. **R25-035 [P2]** runner 支持 `--failed-only` 输出。
36. **R25-036 [P2]** runner 支持导出 `--out <file>`。
37. **R25-037 [P2]** CI 增加 nightly machine-schema job。
38. **R25-038 [P2]** CI artifact retention 策略明确化。
39. **R25-039 [P2]** CI 增加 duration 回归报警阈值。
40. **R25-040 [P2]** CI 增加 flaky scenario 检测标记。
41. **R25-041 [P2]** docs 增加 machine JSON schema 演进记录。
42. **R25-042 [P2]** docs 增加 machine mode 排障 FAQ。
43. **R25-043 [P2]** 自动脚本对比 tests 与 workflow 合约一致性。
44. **R25-044 [P2]** 自动脚本汇总 round-by-round 通过数曲线。
45. **R25-045 [P2]** 自动脚本生成下一轮候选 Top3。
46. **R25-046 [P2]** Round26 planner 自动生成 RED 命令模板。
47. **R25-047 [P2]** Round26 planner 自动生成 GREEN 命令模板。
48. **R25-048 [P2]** Round26 planner 自动生成 VERIFY 命令包。
49. **R25-049 [P2]** Round26 planner 自动生成 findings/progress 初稿。
50. **R25-050 [P2]** Round26 planner 自动生成 docs 更新 checklist。

### Priority Recommendation (Round 25)
- **Batch1 (P0, 本轮已执行):** R25-001 / R25-004 / R25-006。
- **Batch2 (P1):** R25-011 / R25-016 / R25-021。
- **Batch3 (P1):** R25-025 / R25-028 / R25-030。

## Plan Output (Round 25)
- `docs/plans/2026-02-11-repo-gap-priority-round25.md`

## Execution Results (Round 25)
- Task A25 完成：release-audit JSON 增加 `failed_steps` 与 `failed_steps_count`。
- Task B25 完成：runner JSON 增加 `fastest_scenario`。
- Task C25 完成：CI machine smoke 增加 runner-contract schema smoke。
- 回归结果：
  - 目标测试 `22 passed, 4 subtests passed`。
  - 全量 `416 passed, 23 subtests passed`。
  - shell + rust 门禁通过。

## Scan Results (Round 26)

### Repository Health Snapshot
- 执行前基线：`pytest -q` -> `416 passed, 23 subtests passed`。
- Round26 执行后：`pytest -q` -> `416 passed, 23 subtests passed`。
- Shell 语法门禁：`bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> `shell-syntax:OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> 通过。
- Round26 核心缺口并收口：
  - release-audit JSON 缺 `failed_commands` 列表。
  - runner contract JSON 缺 `scenario_count_by_result`。
  - CI machine smoke 仅校验 runner-contract，未校验 release/suites JSON。

### 50-Task Gap Backlog (Prioritized)
1. **R26-001 [P0]** release-audit JSON 增加 `failed_commands`（已执行）。
2. **R26-002 [P0]** force-fail 场景断言 `failed_commands`（已执行）。
3. **R26-003 [P0]** runner JSON 增加 `scenario_count_by_result`（已执行）。
4. **R26-004 [P0]** contract suite 测试断言 passed/failed 聚合（已执行）。
5. **R26-005 [P0]** CI machine smoke 校验 `runner-contract.json` 新键（已执行）。
6. **R26-006 [P0]** CI machine smoke 校验 `release-audit-dry-run.json`（已执行）。
7. **R26-007 [P0]** CI machine smoke 校验 `runner-suites.json`（已执行）。
8. **R26-008 [P0]** Round26 目标测试集全绿（已执行）。
9. **R26-009 [P0]** Round26 全量 pytest 全绿（已执行）。
10. **R26-010 [P0]** Round26 rust/clippy/fmt 门禁全绿（已执行）。
11. **R26-011 [P1]** release-audit JSON 增加 `failed_commands_count`。
12. **R26-012 [P1]** release-audit JSON 增加 `success_commands_count`。
13. **R26-013 [P1]** release-audit JSON 增加 `max_step_duration_ms`。
14. **R26-014 [P1]** release-audit JSON 增加 `avg_step_duration_ms`。
15. **R26-015 [P1]** release-audit JSON 增加 schema version 字段。
16. **R26-016 [P1]** runner JSON 增加 `duration_stats` 汇总对象。
17. **R26-017 [P1]** runner JSON 增加 `median_duration_ms`。
18. **R26-018 [P1]** runner JSON 增加 `p95_duration_ms`。
19. **R26-019 [P1]** runner JSON 增加 `slow_scenarios`（阈值过滤）。
20. **R26-020 [P1]** runner JSON 增加 `error_count_by_reason`。
21. **R26-021 [P1]** CI schema smoke 校验 release flags consistency。
22. **R26-022 [P1]** CI schema smoke 校验 runner-suites default in list。
23. **R26-023 [P1]** CI summary 输出 scenario_count_by_result。
24. **R26-024 [P1]** CI summary 输出 failed_commands。
25. **R26-025 [P1]** docs/CLI_CONTRACT_MATRIX 增加 `scenario_count_by_result` 字段说明。
26. **R26-026 [P1]** docs/CLI_CONTRACT_MATRIX 增加 `failed_commands` 字段说明。
27. **R26-027 [P1]** README EN machine JSON 示例同步新增字段。
28. **R26-028 [P1]** README ZH machine JSON 示例同步新增字段。
29. **R26-029 [P1]** HOOKS_SETUP 增加 multi-schema smoke 段落。
30. **R26-030 [P1]** docs freshness 增加 `scenario_count_by_result` 文案守卫。
31. **R26-031 [P2]** release-audit 支持 `--json-schema` 文件输出。
32. **R26-032 [P2]** release-audit 支持 `--output <path>`。
33. **R26-033 [P2]** runner 支持 `--json-pretty`。
34. **R26-034 [P2]** runner 支持 `--failed-only`。
35. **R26-035 [P2]** runner 支持 `--scenario-regex`。
36. **R26-036 [P2]** runner 支持 `--sort-by duration`。
37. **R26-037 [P2]** CI 新增 nightly machine-schema regression。
38. **R26-038 [P2]** CI artifact retention policy 文档化。
39. **R26-039 [P2]** CI 增加 schema drift detection。
40. **R26-040 [P2]** CI 增加 JSON lint（schema-lite）步骤。
41. **R26-041 [P2]** docs 增加 machine JSON 演进历史表。
42. **R26-042 [P2]** docs 增加 machine smoke 常见失败 FAQ。
43. **R26-043 [P2]** 自动脚本生成 round diff 报告（JSON keys）。
44. **R26-044 [P2]** 自动脚本生成 test-to-workflow coverage map。
45. **R26-045 [P2]** 自动脚本提取 Top3 下一轮候选。
46. **R26-046 [P2]** Round27 planner 自动生成 RED 命令模板。
47. **R26-047 [P2]** Round27 planner 自动生成 GREEN 命令模板。
48. **R26-048 [P2]** Round27 planner 自动生成 VERIFY 命令包。
49. **R26-049 [P2]** Round27 planner 自动生成 findings/progress 初稿。
50. **R26-050 [P2]** Round27 planner 自动生成 docs 更新 checklist。

### Priority Recommendation (Round 26)
- **Batch1 (P0, 本轮已执行):** R26-001 / R26-003 / R26-005。
- **Batch2 (P1):** R26-011 / R26-016 / R26-021。
- **Batch3 (P1):** R26-025 / R26-029 / R26-030。

## Plan Output (Round 26)
- `docs/plans/2026-02-11-repo-gap-priority-round26.md`

## Execution Results (Round 26)
- Task A26 完成：release-audit JSON 增加 `failed_commands`。
- Task B26 完成：runner JSON 增加 `scenario_count_by_result`。
- Task C26 完成：CI machine smoke 增加 release/suites/contract 三文件 schema 校验。
- 回归结果：
  - 目标测试 `22 passed, 4 subtests passed`。
  - 全量 `416 passed, 23 subtests passed`。
  - shell + rust 门禁通过。

## Scan Results (Round 27)

### Repository Health Snapshot
- 执行前基线：`pytest -q` -> `416 passed, 23 subtests passed`。
- Round27 执行后：`pytest -q` -> `416 passed, 23 subtests passed`。
- Shell 语法门禁：`bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> `shell-syntax:OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> 通过。
- Round27 关键缺口并收口：
  - release-audit JSON 缺 `failed_commands_count`。
  - runner contract JSON 缺 `duration_stats`。
  - CI machine schema smoke required keys 与最新 JSON 契约未同步。

### 50-Task Gap Backlog (Prioritized)
1. **R27-001 [P0]** release-audit JSON 增加 `failed_commands_count`（已执行）。
2. **R27-002 [P0]** run-json 场景断言 `failed_commands_count == 0`（已执行）。
3. **R27-003 [P0]** force-fail 场景断言 `failed_commands_count == 1`（已执行）。
4. **R27-004 [P0]** runner JSON 增加 `duration_stats`（已执行）。
5. **R27-005 [P0]** runner 测试断言 min/max/avg 区间关系（已执行）。
6. **R27-006 [P0]** CI schema smoke 增加 `duration_stats` required key（已执行）。
7. **R27-007 [P0]** CI schema smoke 增加 `failed_commands_count` required key（已执行）。
8. **R27-008 [P0]** Round27 目标测试集全绿（已执行）。
9. **R27-009 [P0]** Round27 全量 pytest 全绿（已执行）。
10. **R27-010 [P0]** Round27 rust/clippy/fmt 门禁全绿（已执行）。
11. **R27-011 [P1]** release-audit JSON 增加 `success_steps_count`。
12. **R27-012 [P1]** release-audit JSON 增加 `commands_count`。
13. **R27-013 [P1]** release-audit JSON 增加 `max_step_duration_ms`。
14. **R27-014 [P1]** release-audit JSON 增加 `avg_step_duration_ms`。
15. **R27-015 [P1]** release-audit JSON 增加 schema version 字段。
16. **R27-016 [P1]** runner JSON 增加 `median_duration_ms`。
17. **R27-017 [P1]** runner JSON 增加 `p95_duration_ms`。
18. **R27-018 [P1]** runner JSON 增加 `total_scenarios` 明确计数。
19. **R27-019 [P1]** runner JSON 增加 `failed_rate`。
20. **R27-020 [P1]** runner JSON 增加 `duration_stats` 文档注释与示例。
21. **R27-021 [P1]** CI schema smoke 校验 runner `duration_stats` 子键完整性。
22. **R27-022 [P1]** CI schema smoke 校验 release `flags` 结构完整性。
23. **R27-023 [P1]** CI schema smoke 校验 suites default_suite 在 suites 列表内。
24. **R27-024 [P1]** CI summary 输出 duration_stats 摘要。
25. **R27-025 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `duration_stats` 字段说明。
26. **R27-026 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `failed_commands_count` 字段说明。
27. **R27-027 [P1]** README EN machine JSON 示例同步新增字段。
28. **R27-028 [P1]** README ZH machine JSON 示例同步新增字段。
29. **R27-029 [P1]** HOOKS_SETUP 增加 machine schema smoke required keys 说明。
30. **R27-030 [P1]** docs freshness 新增 `duration_stats` 文案守卫。
31. **R27-031 [P2]** release-audit 支持 `--output <json-file>`。
32. **R27-032 [P2]** release-audit 支持 `--json-schema` 模式。
33. **R27-033 [P2]** runner 支持 `--json-pretty`。
34. **R27-034 [P2]** runner 支持 `--scenario-regex`。
35. **R27-035 [P2]** runner 支持 `--failed-only` 输出。
36. **R27-036 [P2]** runner 支持 `--sort-by duration`。
37. **R27-037 [P2]** CI machine smoke 迁移为独立脚本复用。
38. **R27-038 [P2]** CI 新增 nightly contract-json drift 检查。
39. **R27-039 [P2]** CI artifact 增加 retention 策略说明。
40. **R27-040 [P2]** CI 增加 schema drift failure hints。
41. **R27-041 [P2]** docs 增加 machine JSON 演进 changelog。
42. **R27-042 [P2]** docs 增加 machine smoke 故障排查 FAQ。
43. **R27-043 [P2]** 自动脚本比较 workflow required keys 与测试断言。
44. **R27-044 [P2]** 自动脚本生成 round 间 JSON schema diff。
45. **R27-045 [P2]** 自动脚本生成 round 间 pass-rate diff 报告。
46. **R27-046 [P2]** Round28 planner 自动提取 top3 P1 任务。
47. **R27-047 [P2]** Round28 planner 自动生成 RED 命令模板。
48. **R27-048 [P2]** Round28 planner 自动生成 GREEN 命令模板。
49. **R27-049 [P2]** Round28 planner 自动生成 VERIFY 命令包。
50. **R27-050 [P2]** Round28 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 27)
- **Batch1 (P0, 本轮已执行):** R27-001 / R27-004 / R27-006。
- **Batch2 (P1):** R27-011 / R27-016 / R27-021。
- **Batch3 (P1):** R27-025 / R27-029 / R27-030。

## Plan Output (Round 27)
- `docs/plans/2026-02-11-repo-gap-priority-round27.md`

## Execution Results (Round 27)
- Task A27 完成：release-audit JSON 增加 `failed_commands_count`。
- Task B27 完成：runner JSON 增加 `duration_stats`。
- Task C27 完成：CI machine schema smoke 同步 required keys。
- 回归结果：
  - 目标测试 `22 passed, 4 subtests passed`。
  - 全量 `416 passed, 23 subtests passed`。
  - shell + rust 门禁通过。

## Scan Results (Round 28)

### Repository Health Snapshot
- 执行前基线：`pytest -q` -> `416 passed, 23 subtests passed`。
- Round28 执行后：`pytest -q` -> `416 passed, 23 subtests passed`。
- Shell 语法门禁：`bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> `shell-syntax:OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> 通过。
- Round28 关键缺口并收口：
  - release-audit JSON 缺 `success_steps_count` 与 `commands_count`。
  - runner contract JSON 缺 `failed_rate`。
  - CI machine schema smoke required keys 未同步上述字段。

### 50-Task Gap Backlog (Prioritized)
1. **R28-001 [P0]** release-audit JSON 增加 `success_steps_count`（已执行）。
2. **R28-002 [P0]** release-audit JSON 增加 `commands_count`（已执行）。
3. **R28-003 [P0]** force-fail 场景断言 success_steps_count（已执行）。
4. **R28-004 [P0]** run-json 场景断言 commands_count（已执行）。
5. **R28-005 [P0]** runner JSON 增加 `failed_rate`（已执行）。
6. **R28-006 [P0]** runner 测试断言 failed_rate 计算正确（已执行）。
7. **R28-007 [P0]** CI schema smoke required_runner 增加 `failed_rate`（已执行）。
8. **R28-008 [P0]** CI schema smoke required_release 增加 success/commands count（已执行）。
9. **R28-009 [P0]** Round28 目标测试全绿（已执行）。
10. **R28-010 [P0]** Round28 全量 pytest 与 Rust 门禁全绿（已执行）。
11. **R28-011 [P1]** release-audit JSON 增加 `success_rate`。
12. **R28-012 [P1]** release-audit JSON 增加 `failed_rate`。
13. **R28-013 [P1]** release-audit JSON 增加 `max_step_duration_ms`。
14. **R28-014 [P1]** release-audit JSON 增加 `avg_step_duration_ms`。
15. **R28-015 [P1]** release-audit JSON 增加 schema version。
16. **R28-016 [P1]** runner JSON 增加 `duration_stats.median_ms`。
17. **R28-017 [P1]** runner JSON 增加 `duration_stats.p95_ms`。
18. **R28-018 [P1]** runner JSON 增加 `error_count_by_reason`。
19. **R28-019 [P1]** runner JSON 增加 `total_scenarios`。
20. **R28-020 [P1]** runner JSON 增加 `success_rate`。
21. **R28-021 [P1]** CI schema smoke 校验 runner `duration_stats` 子键完整性。
22. **R28-022 [P1]** CI schema smoke 校验 release `flags` 子键完整性。
23. **R28-023 [P1]** CI schema smoke 校验 suites default 在 list 内。
24. **R28-024 [P1]** CI step summary 输出 failed_rate 与 success_rate。
25. **R28-025 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `failed_rate` 字段说明。
26. **R28-026 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `success_steps_count` 字段说明。
27. **R28-027 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `commands_count` 字段说明。
28. **R28-028 [P1]** README EN machine JSON 示例同步新增字段。
29. **R28-029 [P1]** README ZH machine JSON 示例同步新增字段。
30. **R28-030 [P1]** docs freshness 新增 failed_rate 文案守卫。
31. **R28-031 [P2]** release-audit 支持 `--output <path>`。
32. **R28-032 [P2]** release-audit 支持 `--json-schema`。
33. **R28-033 [P2]** runner 支持 `--json-pretty`。
34. **R28-034 [P2]** runner 支持 `--scenario-regex`。
35. **R28-035 [P2]** runner 支持 `--failed-only`。
36. **R28-036 [P2]** runner 支持 `--sort-by duration`。
37. **R28-037 [P2]** CI machine smoke 抽离为复用脚本。
38. **R28-038 [P2]** CI nightly 增加 schema drift 检测。
39. **R28-039 [P2]** CI artifacts 增加 retention policy 注释。
40. **R28-040 [P2]** CI failure message 增加 schema diff hint。
41. **R28-041 [P2]** docs 增加 machine JSON 演进图。
42. **R28-042 [P2]** docs 增加 machine smoke FAQ。
43. **R28-043 [P2]** 自动脚本比较 tests 与 workflow required keys。
44. **R28-044 [P2]** 自动脚本生成 round schema diff 报告。
45. **R28-045 [P2]** 自动脚本生成回归通过率趋势。
46. **R28-046 [P2]** Round29 planner 自动提取 top3 P1。
47. **R28-047 [P2]** Round29 planner 自动生成 RED 命令模板。
48. **R28-048 [P2]** Round29 planner 自动生成 GREEN 命令模板。
49. **R28-049 [P2]** Round29 planner 自动生成 VERIFY 命令包。
50. **R28-050 [P2]** Round29 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 28)
- **Batch1 (P0, 本轮已执行):** R28-001 / R28-005 / R28-007。
- **Batch2 (P1):** R28-011 / R28-016 / R28-021。
- **Batch3 (P1):** R28-025 / R28-028 / R28-030。

## Plan Output (Round 28)
- `docs/plans/2026-02-11-repo-gap-priority-round28.md`

## Execution Results (Round 28)
- Task A28 完成：release-audit JSON 增加 `success_steps_count` 与 `commands_count`。
- Task B28 完成：runner JSON 增加 `failed_rate`。
- Task C28 完成：CI machine schema smoke 同步 required keys。
- 回归结果：
  - 目标测试 `22 passed, 4 subtests passed`。
  - 全量 `416 passed, 23 subtests passed`。
  - shell + rust 门禁通过。

## Scan Results (Round 29)

### Repository Health Snapshot
- 执行前基线：`pytest -q` -> `416 passed, 23 subtests passed`。
- Round29 执行后：`pytest -q` -> `416 passed, 23 subtests passed`。
- Shell 语法门禁：`bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> `shell-syntax:OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> 通过。
- Round29 关键缺口并收口：
  - release-audit JSON 缺 `success_rate/failed_rate`。
  - runner contract JSON 缺 `success_rate`。
  - CI machine schema smoke required keys 未同步 rate 字段。

### 50-Task Gap Backlog (Prioritized)
1. **R29-001 [P0]** release-audit JSON 增加 `success_rate`（已执行）。
2. **R29-002 [P0]** release-audit JSON 增加 `failed_rate`（已执行）。
3. **R29-003 [P0]** run-json 场景断言 success_rate=1/failed_rate=0（已执行）。
4. **R29-004 [P0]** force-fail 场景断言 success_rate=0/failed_rate=1（已执行）。
5. **R29-005 [P0]** runner JSON 增加 `success_rate`（已执行）。
6. **R29-006 [P0]** runner 测试断言 success_rate=passed/total（已执行）。
7. **R29-007 [P0]** runner 测试断言 success_rate+failed_rate=1（已执行）。
8. **R29-008 [P0]** CI schema smoke required_runner 增加 `success_rate`（已执行）。
9. **R29-009 [P0]** CI schema smoke required_release 增加 `success_rate/failed_rate`（已执行）。
10. **R29-010 [P0]** Round29 目标测试 + 全量 + Rust 门禁全绿（已执行）。
11. **R29-011 [P1]** release-audit JSON 增加 `rate_basis` 字段（steps_executed）。
12. **R29-012 [P1]** release-audit JSON 增加 `error_step_count` 别名。
13. **R29-013 [P1]** release-audit JSON 增加 `success_command_rate`。
14. **R29-014 [P1]** release-audit JSON 增加 `failed_command_rate`。
15. **R29-015 [P1]** release-audit JSON 增加 schema version。
16. **R29-016 [P1]** runner JSON 增加 `success_rate` 与 `pass_rate` 一致性测试。
17. **R29-017 [P1]** runner JSON 增加 `failure_count` 字段。
18. **R29-018 [P1]** runner JSON 增加 `success_count` 字段。
19. **R29-019 [P1]** runner JSON 增加 `duration_stats.median_ms`。
20. **R29-020 [P1]** runner JSON 增加 `duration_stats.p95_ms`。
21. **R29-021 [P1]** CI schema smoke 校验 runner rate 字段取值范围 [0,1]。
22. **R29-022 [P1]** CI schema smoke 校验 release rate 字段取值范围 [0,1]。
23. **R29-023 [P1]** CI summary 输出 success/failed rates。
24. **R29-024 [P1]** CI summary 输出 commands/steps 计数。
25. **R29-025 [P1]** docs/CLI_CONTRACT_MATRIX 增补 `success_rate` 字段说明。
26. **R29-026 [P1]** docs/CLI_CONTRACT_MATRIX 增补 release `success_rate` 字段说明。
27. **R29-027 [P1]** docs/CLI_CONTRACT_MATRIX 增补 release `failed_rate` 字段说明。
28. **R29-028 [P1]** README EN machine JSON 示例同步新增 rate 字段。
29. **R29-029 [P1]** README ZH machine JSON 示例同步新增 rate 字段。
30. **R29-030 [P1]** docs freshness 新增 rate 字段文案守卫。
31. **R29-031 [P2]** release-audit 支持 `--output <json-file>`。
32. **R29-032 [P2]** release-audit 支持 `--json-schema` 导出。
33. **R29-033 [P2]** runner 支持 `--json-pretty`。
34. **R29-034 [P2]** runner 支持 `--scenario-regex`。
35. **R29-035 [P2]** runner 支持 `--failed-only`。
36. **R29-036 [P2]** runner 支持 `--sort-by duration`。
37. **R29-037 [P2]** CI machine smoke 抽离独立脚本复用。
38. **R29-038 [P2]** CI nightly 增加 schema drift 检测。
39. **R29-039 [P2]** CI artifacts retention policy 文档化。
40. **R29-040 [P2]** CI failure logs 增加 schema diff hints。
41. **R29-041 [P2]** docs 增加 machine JSON schema 演进图。
42. **R29-042 [P2]** docs 增加 machine smoke troubleshooting FAQ。
43. **R29-043 [P2]** 自动脚本比较测试断言与 workflow required keys。
44. **R29-044 [P2]** 自动脚本输出 round schema diff 报告。
45. **R29-045 [P2]** 自动脚本输出 round pass-rate 趋势。
46. **R29-046 [P2]** Round30 planner 自动提取 top3 P1 任务。
47. **R29-047 [P2]** Round30 planner 自动生成 RED 命令模板。
48. **R29-048 [P2]** Round30 planner 自动生成 GREEN 命令模板。
49. **R29-049 [P2]** Round30 planner 自动生成 VERIFY 命令包。
50. **R29-050 [P2]** Round30 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 29)
- **Batch1 (P0, 本轮已执行):** R29-001 / R29-005 / R29-008。
- **Batch2 (P1):** R29-011 / R29-016 / R29-021。
- **Batch3 (P1):** R29-025 / R29-028 / R29-030。

## Plan Output (Round 29)
- `docs/plans/2026-02-11-repo-gap-priority-round29.md`

## Execution Results (Round 29)
- Task A29 完成：release-audit JSON 增加 `success_rate/failed_rate`。
- Task B29 完成：runner JSON 增加 `success_rate`。
- Task C29 完成：CI machine schema smoke 同步 required keys（rates）。
- 回归结果：
  - 目标测试 `22 passed, 4 subtests passed`。
  - 全量 `416 passed, 23 subtests passed`。
  - shell + rust 门禁通过。

## Scan Results (Round 30)

### Repository Health Snapshot
- 执行前基线：`pytest -q` -> `416 passed, 23 subtests passed`。
- Round30 执行后：`pytest -q` -> `416 passed, 23 subtests passed`。
- Shell 语法门禁：`bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> `shell-syntax:OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> 通过。
- Round30 关键缺口并收口：
  - release-audit JSON 缺失败步数字段别名 `error_step_count`。
  - runner contract JSON 缺显式 `success_count/failure_count`。
  - CI machine schema smoke required keys 未同步上述字段。

### 50-Task Gap Backlog (Prioritized)
1. **R30-001 [P0]** release-audit JSON 增加 `error_step_count`（已执行）。
2. **R30-002 [P0]** run-json 场景断言 `error_step_count == 0`（已执行）。
3. **R30-003 [P0]** force-fail 场景断言 `error_step_count == 1`（已执行）。
4. **R30-004 [P0]** runner JSON 增加 `success_count`（已执行）。
5. **R30-005 [P0]** runner JSON 增加 `failure_count`（已执行）。
6. **R30-006 [P0]** runner 测试断言 success_count 与 passed 一致（已执行）。
7. **R30-007 [P0]** runner 测试断言 failure_count 与 failed_scenarios 长度一致（已执行）。
8. **R30-008 [P0]** CI schema smoke required_release 增加 `error_step_count`（已执行）。
9. **R30-009 [P0]** CI schema smoke required_runner 增加 `success_count/failure_count`（已执行）。
10. **R30-010 [P0]** Round30 目标测试 + 全量 + Rust 门禁全绿（已执行）。
11. **R30-011 [P1]** release-audit JSON 增加 `error_steps` 详细对象。
12. **R30-012 [P1]** release-audit JSON 增加 `success_commands_count`。
13. **R30-013 [P1]** release-audit JSON 增加 `success_command_rate`。
14. **R30-014 [P1]** release-audit JSON 增加 `failed_command_rate`。
15. **R30-015 [P1]** release-audit JSON 增加 schema version。
16. **R30-016 [P1]** runner JSON 增加 `success_count + failure_count == total` 断言。
17. **R30-017 [P1]** runner JSON 增加 `success_rate == success_count/total` 断言。
18. **R30-018 [P1]** runner JSON 增加 `failed_rate == failure_count/total` 断言。
19. **R30-019 [P1]** runner JSON 增加 `duration_stats.median_ms`。
20. **R30-020 [P1]** runner JSON 增加 `duration_stats.p95_ms`。
21. **R30-021 [P1]** CI schema smoke 校验 count 字段非负。
22. **R30-022 [P1]** CI schema smoke 校验 rate 字段范围 [0,1]。
23. **R30-023 [P1]** CI summary 输出 success/failure count。
24. **R30-024 [P1]** CI summary 输出 error_step_count。
25. **R30-025 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `error_step_count` 字段说明。
26. **R30-026 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `success_count` 字段说明。
27. **R30-027 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `failure_count` 字段说明。
28. **R30-028 [P1]** README EN machine JSON 示例同步新增 count 字段。
29. **R30-029 [P1]** README ZH machine JSON 示例同步新增 count 字段。
30. **R30-030 [P1]** docs freshness 新增 count 字段文案守卫。
31. **R30-031 [P2]** release-audit 支持 `--output <json-file>`。
32. **R30-032 [P2]** release-audit 支持 `--json-schema`。
33. **R30-033 [P2]** runner 支持 `--json-pretty`。
34. **R30-034 [P2]** runner 支持 `--scenario-regex`。
35. **R30-035 [P2]** runner 支持 `--failed-only`。
36. **R30-036 [P2]** runner 支持 `--sort-by duration`。
37. **R30-037 [P2]** CI machine smoke 抽离复用脚本。
38. **R30-038 [P2]** CI nightly 增加 schema drift 检测。
39. **R30-039 [P2]** CI artifacts retention policy 文档化。
40. **R30-040 [P2]** CI failure logs 增加 schema diff hint。
41. **R30-041 [P2]** docs 增加 machine JSON evolution 图表。
42. **R30-042 [P2]** docs 增加 machine smoke troubleshooting FAQ。
43. **R30-043 [P2]** 自动脚本比较 tests 与 workflow required keys。
44. **R30-044 [P2]** 自动脚本输出 round schema diff 报告。
45. **R30-045 [P2]** 自动脚本输出 round pass-rate trend。
46. **R30-046 [P2]** Round31 planner 自动提取 top3 P1。
47. **R30-047 [P2]** Round31 planner 自动生成 RED 命令模板。
48. **R30-048 [P2]** Round31 planner 自动生成 GREEN 命令模板。
49. **R30-049 [P2]** Round31 planner 自动生成 VERIFY 命令包。
50. **R30-050 [P2]** Round31 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 30)
- **Batch1 (P0, 本轮已执行):** R30-001 / R30-004 / R30-008。
- **Batch2 (P1):** R30-011 / R30-016 / R30-021。
- **Batch3 (P1):** R30-025 / R30-028 / R30-030。

## Plan Output (Round 30)
- `docs/plans/2026-02-11-repo-gap-priority-round30.md`

## Execution Results (Round 30)
- Task A30 完成：release-audit JSON 增加 `error_step_count`。
- Task B30 完成：runner JSON 增加 `success_count/failure_count`。
- Task C30 完成：CI machine schema smoke 同步 required keys（counts）。
- 回归结果：
  - 目标测试 `22 passed, 4 subtests passed`。
  - 全量 `416 passed, 23 subtests passed`。
  - shell + rust 门禁通过。

## Scan Results (Round 31)

### Repository Health Snapshot
- 执行前基线：`pytest -q` -> `416 passed, 23 subtests passed`。
- Round31 执行后：`pytest -q` -> `416 passed, 23 subtests passed`。
- Shell 语法门禁：`bash -n scripts/release-contract-audit.sh scripts/fusion-*.sh scripts/loop-guardian.sh` -> `shell-syntax:OK`。
- Rust 门禁：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> 通过。
- Round31 关键缺口并收口：
  - release-audit JSON 缺 command 级 rate 字段。
  - runner JSON 缺显式 `total_scenarios` 字段。
  - CI machine schema smoke required keys 未同步新字段。

### 50-Task Gap Backlog (Prioritized)
1. **R31-001 [P0]** release-audit JSON 增加 `success_command_rate`（已执行）。
2. **R31-002 [P0]** release-audit JSON 增加 `failed_command_rate`（已执行）。
3. **R31-003 [P0]** run-json 场景断言 command rates = 1/0（已执行）。
4. **R31-004 [P0]** force-fail 场景断言 command rates = 0/1（已执行）。
5. **R31-005 [P0]** runner JSON 增加 `total_scenarios`（已执行）。
6. **R31-006 [P0]** runner 测试断言 total_scenarios = total（已执行）。
7. **R31-007 [P0]** CI schema smoke required_release 增加 command rate keys（已执行）。
8. **R31-008 [P0]** CI schema smoke required_runner 增加 total_scenarios key（已执行）。
9. **R31-009 [P0]** Round31 目标测试全绿（已执行）。
10. **R31-010 [P0]** Round31 全量 pytest + Rust 门禁全绿（已执行）。
11. **R31-011 [P1]** release-audit JSON 增加 `command_rate_basis`。
12. **R31-012 [P1]** release-audit JSON 增加 `step_rate_basis`。
13. **R31-013 [P1]** release-audit JSON 增加 `successful_commands` 列表。
14. **R31-014 [P1]** release-audit JSON 增加 `failed_commands` 去重统计。
15. **R31-015 [P1]** release-audit JSON 增加 schema version。
16. **R31-016 [P1]** runner JSON 增加 `success_count + failure_count == total_scenarios` 断言。
17. **R31-017 [P1]** runner JSON 增加 `total_scenarios == len(scenario_results)` 断言。
18. **R31-018 [P1]** runner JSON 增加 `duration_stats.median_ms`。
19. **R31-019 [P1]** runner JSON 增加 `duration_stats.p95_ms`。
20. **R31-020 [P1]** runner JSON 增加 `failure_reasons` 聚合。
21. **R31-021 [P1]** CI schema smoke 校验 runner count/rate 一致性。
22. **R31-022 [P1]** CI schema smoke 校验 release step/command rates 一致性。
23. **R31-023 [P1]** CI summary 输出 command rates。
24. **R31-024 [P1]** CI summary 输出 total_scenarios。
25. **R31-025 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `success_command_rate` 字段说明。
26. **R31-026 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `failed_command_rate` 字段说明。
27. **R31-027 [P1]** docs/CLI_CONTRACT_MATRIX 补充 `total_scenarios` 字段说明。
28. **R31-028 [P1]** README EN machine JSON 示例同步新增字段。
29. **R31-029 [P1]** README ZH machine JSON 示例同步新增字段。
30. **R31-030 [P1]** docs freshness 新增 command_rate/total_scenarios 文案守卫。
31. **R31-031 [P2]** release-audit 支持 `--output <json-file>`。
32. **R31-032 [P2]** release-audit 支持 `--json-schema`。
33. **R31-033 [P2]** runner 支持 `--json-pretty`。
34. **R31-034 [P2]** runner 支持 `--scenario-regex`。
35. **R31-035 [P2]** runner 支持 `--failed-only`。
36. **R31-036 [P2]** runner 支持 `--sort-by duration`。
37. **R31-037 [P2]** CI machine smoke 抽离复用脚本。
38. **R31-038 [P2]** CI nightly 增加 schema drift 检测。
39. **R31-039 [P2]** CI artifacts retention policy 文档化。
40. **R31-040 [P2]** CI failure logs 增加 schema diff hint。
41. **R31-041 [P2]** docs 增加 machine JSON evolution 图表。
42. **R31-042 [P2]** docs 增加 machine smoke troubleshooting FAQ。
43. **R31-043 [P2]** 自动脚本比较 tests 与 workflow required keys。
44. **R31-044 [P2]** 自动脚本输出 round schema diff 报告。
45. **R31-045 [P2]** 自动脚本输出 round pass-rate trend。
46. **R31-046 [P2]** Round32 planner 自动提取 top3 P1。
47. **R31-047 [P2]** Round32 planner 自动生成 RED 命令模板。
48. **R31-048 [P2]** Round32 planner 自动生成 GREEN 命令模板。
49. **R31-049 [P2]** Round32 planner 自动生成 VERIFY 命令包。
50. **R31-050 [P2]** Round32 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 31)
- **Batch1 (P0, 本轮已执行):** R31-001 / R31-005 / R31-007。
- **Batch2 (P1):** R31-011 / R31-016 / R31-021。
- **Batch3 (P1):** R31-025 / R31-028 / R31-030。

## Plan Output (Round 31)
- `docs/plans/2026-02-11-repo-gap-priority-round31.md`

## Execution Results (Round 31)
- Task A31 完成：release-audit JSON 增加 `success_command_rate/failed_command_rate`。
- Task B31 完成：runner JSON 增加 `total_scenarios`。
- Task C31 完成：CI machine schema smoke 同步 required keys。
- 回归结果：
  - 目标测试 `22 passed, 4 subtests passed`。
  - 全量 `416 passed, 23 subtests passed`。
  - shell + rust 门禁通过。

## Scan Results (Round 32)

### Repository Health Snapshot
- 扫描基线：`bash -n scripts/*.sh` -> `shell-syntax:OK`。
- 扫描基线：`pytest -q` -> `446 passed, 23 subtests passed`。
- 扫描基线：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> 通过。
- 快速契约探针：
  - `release-contract-audit --dry-run --json` 缺 `schema_version/step_rate_basis/command_rate_basis`。
  - `regression_runner --suite contract --json` 缺 `schema_version/rate_basis`。
  - CI machine smoke workflow 仍未校验上述 basis/schema 字段一致性。

### 50-Task Gap Backlog (Prioritized)
1. **R32-001 [P0]** release-audit JSON 增加 `schema_version`（已计划）。
2. **R32-002 [P0]** release-audit JSON 增加 `step_rate_basis`（已计划）。
3. **R32-003 [P0]** release-audit JSON 增加 `command_rate_basis`（已计划）。
4. **R32-004 [P0]** release-audit 测试断言 schema/basis 字段（已计划）。
5. **R32-005 [P0]** runner JSON 增加 `schema_version`（已计划）。
6. **R32-006 [P0]** runner JSON 增加 `rate_basis`（已计划）。
7. **R32-007 [P0]** runner 测试断言 `rate_basis == total_scenarios`（已计划）。
8. **R32-008 [P0]** CI machine smoke required_runner 增加 `schema_version/rate_basis`（已计划）。
9. **R32-009 [P0]** CI machine smoke required_release 增加 `schema_version/step_rate_basis/command_rate_basis`（已计划）。
10. **R32-010 [P0]** CI machine smoke 增加 basis 一致性校验（已计划）。
11. **R32-011 [P1]** release-audit JSON 增加 `schema_version_minor`。
12. **R32-012 [P1]** release-audit JSON 增加 `command_rate_basis_label`。
13. **R32-013 [P1]** runner JSON 增加 `rate_basis_label`。
14. **R32-014 [P1]** runner JSON 增加 `schema_version` 兼容测试（旧版本容忍）。
15. **R32-015 [P1]** CI workflow summary 输出 basis 字段。
16. **R32-016 [P1]** CI workflow summary 输出 schema version。
17. **R32-017 [P1]** docs/CLI_CONTRACT_MATRIX 补充 release schema/basis 字段说明。
18. **R32-018 [P1]** docs/CLI_CONTRACT_MATRIX 补充 runner schema/basis 字段说明。
19. **R32-019 [P1]** README EN 机器 JSON 示例补充 schema/basis 字段。
20. **R32-020 [P1]** README ZH 机器 JSON 示例补充 schema/basis 字段。
21. **R32-021 [P1]** docs freshness 增加 schema/basis 文案守卫。
22. **R32-022 [P1]** CI schema smoke 校验 runner rate 和 count 一致性（强约束）。
23. **R32-023 [P1]** CI schema smoke 校验 release step/command rate 与 basis 一致性（强约束）。
24. **R32-024 [P1]** release-audit JSON 在 `--dry-run` 标注 rate 语义说明。
25. **R32-025 [P1]** runner JSON 增加 `scenario_results_count`。
26. **R32-026 [P1]** runner 测试断言 `scenario_results_count == len(scenario_results)`。
27. **R32-027 [P1]** CI schema smoke required_runner 增加 `scenario_results_count`。
28. **R32-028 [P1]** workflow artifact 增加 release/runner schema-check report。
29. **R32-029 [P1]** regression_runner `--json` 错误路径增加 schema version。
30. **R32-030 [P1]** release-audit `--json` 错误路径增加 schema version。
31. **R32-031 [P2]** release-audit 支持 `--json-schema` 文件输出。
32. **R32-032 [P2]** runner 支持 `--json-schema` 文件输出。
33. **R32-033 [P2]** CI machine smoke 校验 schema hash 漂移。
34. **R32-034 [P2]** nightly job 自动比较 schema 变更并评论 PR。
35. **R32-035 [P2]** docs 增加 schema 演进版本表。
36. **R32-036 [P2]** docs 增加 basis 字段解释图。
37. **R32-037 [P2]** release-audit 增加 `--schema-version <v>` 兼容开关。
38. **R32-038 [P2]** runner 增加 `--schema-version <v>` 兼容开关。
39. **R32-039 [P2]** CI smoke 抽离 python schema 校验脚本复用。
40. **R32-040 [P2]** schema 校验脚本单独单测。
41. **R32-041 [P2]** 将 required keys 列表提取到单一配置源。
42. **R32-042 [P2]** tests 自动对齐 workflow required keys（避免漂移）。
43. **R32-043 [P2]** release JSON contract 快照测试。
44. **R32-044 [P2]** runner JSON contract 快照测试。
45. **R32-045 [P2]** CI 失败输出 diff keys（missing/extra）。
46. **R32-046 [P2]** Round33 planner 自动提取 top3 P1。
47. **R32-047 [P2]** Round33 planner 自动生成 RED 命令模板。
48. **R32-048 [P2]** Round33 planner 自动生成 GREEN 命令模板。
49. **R32-049 [P2]** Round33 planner 自动生成 VERIFY 命令包。
50. **R32-050 [P2]** Round33 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 32)
- **Batch1 (P0, 本轮执行):** R32-001 / R32-005 / R32-008。
- **Batch2 (P1):** R32-017 / R32-019 / R32-021。
- **Batch3 (P1):** R32-022 / R32-023 / R32-028。

## Plan Output (Round 32)
- `docs/plans/2026-02-12-repo-gap-priority-round32.md`

## Execution Results (Round 32)
- Task A32 完成：`release-contract-audit.sh` JSON 增加 `schema_version`、`step_rate_basis`、`command_rate_basis`。
- Task B32 完成：`regression_runner.py` contract JSON 增加 `schema_version`、`rate_basis`。
- Task C32 完成：CI machine smoke schema required keys 同步新增字段，并新增 basis 一致性校验。
- 回归结果：
  - 目标测试：`22 passed, 4 subtests passed`。
  - 全量测试：`446 passed, 23 subtests passed`。
  - Shell 语法门禁：`shell-syntax:OK`。
  - Rust 门禁：clippy/fmt 全通过。

## Scan Results (Round 33)

### Repository Health Snapshot
- 扫描基线：`bash -n scripts/*.sh` -> `shell-syntax:OK`。
- 扫描基线：`pytest -q` -> `446 passed, 23 subtests passed`。
- 扫描基线：`cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all -- --check` -> 通过。
- 快速契约探针：
  - `release-contract-audit --dry-run --json` 字段完整（含 `schema_version/step_rate_basis/command_rate_basis`）。
  - `regression_runner --suite contract --json` 字段完整（含 `schema_version/rate_basis`）。
- 全仓文档缺口：
  - `docs/CLI_CONTRACT_MATRIX.md` 未显式说明 schema/basis 字段契约。
  - `README.md` 与 `README.zh-CN.md` 的 machine JSON 示例未提及 schema/basis 字段。
  - `test_docs_freshness.py` 尚无上述文档守卫断言。

### 50-Task Gap Backlog (Prioritized)
1. **R33-001 [P0]** docs freshness 新增 CLI matrix schema/basis 守卫（计划执行）。
2. **R33-002 [P0]** `docs/CLI_CONTRACT_MATRIX.md` 补齐 release schema/basis 字段说明（计划执行）。
3. **R33-003 [P0]** `docs/CLI_CONTRACT_MATRIX.md` 补齐 runner schema/basis 字段说明（计划执行）。
4. **R33-004 [P0]** docs freshness 新增 README EN schema/basis 守卫（计划执行）。
5. **R33-005 [P0]** `README.md` 机器 JSON 示例补齐 schema/basis 字段（计划执行）。
6. **R33-006 [P0]** docs freshness 新增 README ZH schema/basis 守卫（计划执行）。
7. **R33-007 [P0]** `README.zh-CN.md` 机器 JSON 示例补齐 schema/basis 字段（计划执行）。
8. **R33-008 [P0]** Round33 targeted docs+contract suite 回归全绿（计划执行）。
9. **R33-009 [P0]** Round33 全量 pytest 回归全绿（计划执行）。
10. **R33-010 [P0]** Round33 shell + rust 门禁全绿（计划执行）。
11. **R33-011 [P1]** docs/HOOKS_SETUP 增补 schema/basis 字段定位说明。
12. **R33-012 [P1]** CLI matrix 增加 machine JSON `required keys` 子段落。
13. **R33-013 [P1]** README EN 增加 `rate_basis` 与 `total_scenarios` 的关系示例。
14. **R33-014 [P1]** README ZH 增加 `rate_basis` 与 `total_scenarios` 的关系示例。
15. **R33-015 [P1]** docs freshness 守卫 `success_command_rate/failed_command_rate` 文案说明。
16. **R33-016 [P1]** docs freshness 守卫 `schema_version=v1` 文案说明。
17. **R33-017 [P1]** CI workflow 注释增加 schema/basis 目的说明。
18. **R33-018 [P1]** CI workflow summary 输出 schema 版本。
19. **R33-019 [P1]** CI workflow summary 输出 basis consistency 检查结果。
20. **R33-020 [P1]** release JSON 文档增加错误路径字段说明。
21. **R33-021 [P1]** runner JSON 文档增加错误路径字段说明。
22. **R33-022 [P1]** docs matrix 增加 `exit_code` 与 JSON `result` 对齐规则。
23. **R33-023 [P1]** docs matrix 增加 `rate` 字段范围约束说明。
24. **R33-024 [P1]** docs matrix 增加 `count` 字段非负约束说明。
25. **R33-025 [P1]** README EN 增加 CI artifact JSON 文件用途说明。
26. **R33-026 [P1]** README ZH 增加 CI artifact JSON 文件用途说明。
27. **R33-027 [P1]** docs freshness 增加 artifact 文件名守卫。
28. **R33-028 [P1]** docs freshness 增加 workflow schema 校验命令守卫。
29. **R33-029 [P1]** regression_runner 文档增加 suite contract 字段表。
30. **R33-030 [P1]** release-contract-audit 文档增加字段表。
31. **R33-031 [P2]** release JSON schema 导出功能（`--json-schema`）。
32. **R33-032 [P2]** runner JSON schema 导出功能（`--json-schema`）。
33. **R33-033 [P2]** CI 增加 schema drift 报告 artifact。
34. **R33-034 [P2]** nightly 自动 diff schema 并汇总。
35. **R33-035 [P2]** docs 增加 schema 版本演进表。
36. **R33-036 [P2]** docs 增加 basis 字段可视化示意。
37. **R33-037 [P2]** release JSON 快照测试。
38. **R33-038 [P2]** runner JSON 快照测试。
39. **R33-039 [P2]** tests 与 workflow required keys 自动对齐脚本。
40. **R33-040 [P2]** workflow required keys 单一来源配置化。
41. **R33-041 [P2]** release-audit 支持 `--output <file>`。
42. **R33-042 [P2]** runner 支持 `--json-pretty`。
43. **R33-043 [P2]** runner 支持 `--scenario-regex`。
44. **R33-044 [P2]** runner 支持 `--failed-only`。
45. **R33-045 [P2]** runner 支持 `--sort-by duration`。
46. **R33-046 [P2]** Round34 planner 自动提取 top3 P1。
47. **R33-047 [P2]** Round34 planner 自动生成 RED 命令模板。
48. **R33-048 [P2]** Round34 planner 自动生成 GREEN 命令模板。
49. **R33-049 [P2]** Round34 planner 自动生成 VERIFY 命令包。
50. **R33-050 [P2]** Round34 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 33)
- **Batch1 (P0, 本轮执行):** R33-001 / R33-004 / R33-006。
- **Batch2 (P1):** R33-011 / R33-013 / R33-015。
- **Batch3 (P1):** R33-017 / R33-019 / R33-027。

## Plan Output (Round 33)
- `docs/plans/2026-02-13-repo-gap-priority-round33.md`

## Execution Results (Round 33)
- Task A33 完成：`docs/CLI_CONTRACT_MATRIX.md` 补齐 `schema_version/step_rate_basis/command_rate_basis/rate_basis` 契约说明，并新增 docs freshness 守卫。
- Task B33 完成：`README.md` 机器 JSON 示例补齐 schema/basis 字段说明，并新增 docs freshness 守卫。
- Task C33 完成：`README.zh-CN.md` 机器 JSON 示例补齐 schema/basis 字段说明，并新增 docs freshness 守卫。
- 回归结果：
  - 目标测试：`36 passed, 23 subtests passed`。
  - 全量测试：`449 passed, 23 subtests passed`。
  - Shell 语法门禁：`shell-syntax:OK`。
  - Rust 门禁：clippy/fmt 全通过。

## Scan Results (Round 34)

### Repository Health Snapshot
- 扫描基线：`bash -n scripts/*.sh` -> `shell-syntax:OK`。
- 扫描基线：`pytest -q` -> `449 passed, 23 subtests passed`。
- 扫描基线：`(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `cargo fmt --all -- --check` -> 通过。
- 缺口探针：`for f in scripts/*.sh ...` 脚本均能在 `scripts/runtime/tests/` 找到测试引用。
- 缺口探针：`docs/HOOKS_SETUP.md` 尚未提及 `schema_version/step_rate_basis/command_rate_basis/rate_basis`。
- 缺口探针：`docs/CLI_CONTRACT_MATRIX.md` 尚无独立的 machine required keys 段落。
- 缺口探针：`README.md` / `README.zh-CN.md` 尚无 `step_rate_basis=total_steps` 与 `command_rate_basis=total_commands` 的显式语义文案。

### 50-Task Gap Backlog (Prioritized)
1. **R34-001 [P0]** docs freshness 新增 HOOKS_SETUP schema/basis 守卫（计划执行）。
2. **R34-002 [P0]** `docs/HOOKS_SETUP.md` 补齐 schema/basis 字段说明（计划执行）。
3. **R34-003 [P0]** docs freshness 新增 CLI matrix required keys 守卫（计划执行）。
4. **R34-004 [P0]** `docs/CLI_CONTRACT_MATRIX.md` 新增 machine required keys 段落（计划执行）。
5. **R34-005 [P0]** docs freshness 新增 README EN/ZH basis 分母语义守卫（计划执行）。
6. **R34-006 [P0]** `README.md` 增加 `step_rate_basis=total_steps` 与 `command_rate_basis=total_commands` 文案（计划执行）。
7. **R34-007 [P0]** `README.zh-CN.md` 增加 `step_rate_basis=total_steps` 与 `command_rate_basis=total_commands` 文案（计划执行）。
8. **R34-008 [P0]** Round34 targeted docs+contract suite 回归全绿（计划执行）。
9. **R34-009 [P0]** Round34 全量 pytest 回归全绿（计划执行）。
10. **R34-010 [P0]** Round34 shell + rust 门禁全绿（计划执行）。
11. **R34-011 [P1]** docs/HOOKS_SETUP 增加 `rate_basis=total_scenarios` 单行示例输出。
12. **R34-012 [P1]** docs/HOOKS_SETUP 增加 release/runner JSON 字段对照表。
13. **R34-013 [P1]** docs freshness 守卫 HOOKS_SETUP 的 `--json-pretty` 示例不回退。
14. **R34-014 [P1]** CLI matrix Notes 增加 machine artifact 文件名引用。
15. **R34-015 [P1]** CLI matrix Notes 增加 `ci-contract-gates.yml` schema smoke 说明。
16. **R34-016 [P1]** README EN 增加 release command rates 与 `command_rate_basis` 关系示例。
17. **R34-017 [P1]** README ZH 增加 release command rates 与 `command_rate_basis` 关系示例。
18. **R34-018 [P1]** docs freshness 守卫 README EN 的 basis 语义示例。
19. **R34-019 [P1]** docs freshness 守卫 README ZH 的 basis 语义示例。
20. **R34-020 [P1]** docs/CLI_CONTRACT_MATRIX 增加 `schema_version=v1` 约束说明。
21. **R34-021 [P1]** docs/CLI_CONTRACT_MATRIX 增加 rate 字段区间约束说明。
22. **R34-022 [P1]** docs/CLI_CONTRACT_MATRIX 增加 count 字段非负约束说明。
23. **R34-023 [P1]** docs freshness 守卫 matrix 的 `success_command_rate/failed_command_rate` 关键词。
24. **R34-024 [P1]** docs freshness 守卫 matrix 的 `total_scenarios` 关键词。
25. **R34-025 [P1]** HOOKS_SETUP 增补 release/runner JSON 命令和输出文件建议。
26. **R34-026 [P1]** README EN 增补 CI artifact JSON 用途说明。
27. **R34-027 [P1]** README ZH 增补 CI artifact JSON 用途说明。
28. **R34-028 [P1]** docs freshness 增加 artifact 文件名守卫。
29. **R34-029 [P1]** docs freshness 增加 workflow schema smoke 命令守卫。
30. **R34-030 [P1]** regression_runner 文档增加 contract suite 字段列表。
31. **R34-031 [P2]** release-contract-audit 增加 `--json-schema` 导出能力。
32. **R34-032 [P2]** regression_runner 增加 `--json-schema` 导出能力。
33. **R34-033 [P2]** CI workflow 增加 schema drift artifact。
34. **R34-034 [P2]** nightly 自动比较 schema 漂移。
35. **R34-035 [P2]** docs 增加 schema 演进版本表。
36. **R34-036 [P2]** docs 增加 basis 字段可视化示意。
37. **R34-037 [P2]** release JSON contract 快照测试。
38. **R34-038 [P2]** runner JSON contract 快照测试。
39. **R34-039 [P2]** CI required keys 提取到单一配置源。
40. **R34-040 [P2]** tests 自动对齐 workflow required keys。
41. **R34-041 [P2]** release-audit 支持 `--output <file>`。
42. **R34-042 [P2]** runner 支持 `--json-pretty`。
43. **R34-043 [P2]** runner 支持 `--scenario-regex`。
44. **R34-044 [P2]** runner 支持 `--failed-only`。
45. **R34-045 [P2]** runner 支持 `--sort-by duration`。
46. **R34-046 [P2]** Round35 planner 自动提取 top3 P1。
47. **R34-047 [P2]** Round35 planner 自动生成 RED 命令模板。
48. **R34-048 [P2]** Round35 planner 自动生成 GREEN 命令模板。
49. **R34-049 [P2]** Round35 planner 自动生成 VERIFY 命令包。
50. **R34-050 [P2]** Round35 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 34)
- **Batch1 (P0, 本轮执行):** R34-001/R34-002 + R34-003/R34-004 + R34-005/R34-006/R34-007。
- **Batch2 (P1):** R34-011 / R34-014 / R34-018。
- **Batch3 (P1):** R34-020 / R34-023 / R34-028。

## Plan Output (Round 34)
- `docs/plans/2026-02-13-repo-gap-priority-round34.md`

## Execution Results (Round 34)
- Task A34 完成：`docs/HOOKS_SETUP.md` 补齐 machine JSON `schema_version/step_rate_basis/command_rate_basis/rate_basis` 文案，并新增 docs freshness 守卫。
- Task B34 完成：`docs/CLI_CONTRACT_MATRIX.md` 新增 `Required machine JSON keys (minimum)` 段落，并新增 docs freshness 守卫。
- Task C34 完成：`README.md` 与 `README.zh-CN.md` 增补 basis 分母语义（`step_rate_basis=total_steps`、`command_rate_basis=total_commands`、`rate_basis=total_scenarios`）并新增 docs freshness 守卫。
- 回归结果：
  - 目标测试：`39 passed, 23 subtests passed`。
  - 全量测试：`452 passed, 23 subtests passed`。
  - Shell 语法门禁：`shell-syntax:OK`。
  - Rust 门禁：clippy/fmt 全通过。

## Scan Results (Round 35)

### Repository Health Snapshot
- 扫描基线：`bash -n scripts/*.sh` -> `shell-syntax:OK`。
- 扫描基线：`pytest -q` -> `452 passed, 23 subtests passed`。
- 扫描基线：`(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `cargo fmt --all -- --check` -> 通过。
- 缺口探针：`pytest -q scripts/runtime/tests/test_docs_freshness.py` -> `17 passed, 19 subtests passed`。
- 缺口探针：`docs/HOOKS_SETUP.md` 尚未出现 `step_rate_basis=total_steps` / `command_rate_basis=total_commands` 分母语义。
- 缺口探针：`docs/CLI_CONTRACT_MATRIX.md` 尚未出现 `schema_version=v1` 版本约束文案。
- 缺口探针：`README.md` / `README.zh-CN.md` 尚未出现 CI machine artifact 文件示例（如 `/tmp/release-audit-dry-run.json`、`/tmp/runner-contract.json`）。

### 50-Task Gap Backlog (Prioritized)
1. **R35-001 [P0]** docs freshness 新增 HOOKS_SETUP basis 分母语义守卫（计划执行）。
2. **R35-002 [P0]** `docs/HOOKS_SETUP.md` 新增 `step_rate_basis=total_steps` 与 `command_rate_basis=total_commands` 文案（计划执行）。
3. **R35-003 [P0]** docs freshness 新增 CLI matrix `schema_version=v1` 守卫（计划执行）。
4. **R35-004 [P0]** `docs/CLI_CONTRACT_MATRIX.md` 补齐 `schema_version=v1` 约束说明（计划执行）。
5. **R35-005 [P0]** docs freshness 新增 README EN/ZH artifact 文件名守卫（计划执行）。
6. **R35-006 [P0]** `README.md` 增补 CI machine artifact 文件示例（计划执行）。
7. **R35-007 [P0]** `README.zh-CN.md` 增补 CI machine artifact 文件示例（计划执行）。
8. **R35-008 [P0]** Round35 targeted docs+contract suite 回归全绿（计划执行）。
9. **R35-009 [P0]** Round35 全量 pytest 回归全绿（计划执行）。
10. **R35-010 [P0]** Round35 shell + rust 门禁全绿（计划执行）。
11. **R35-011 [P1]** HOOKS_SETUP 增加 `rate_basis=total_scenarios` 分母语义同行说明。
12. **R35-012 [P1]** HOOKS_SETUP 增加 release/runner 字段表格。
13. **R35-013 [P1]** docs freshness 守卫 HOOKS_SETUP 含 `--json-pretty` 示例。
14. **R35-014 [P1]** CLI matrix Notes 增加 CI artifact 文件名引用。
15. **R35-015 [P1]** CLI matrix Notes 增加 schema smoke 命令说明。
16. **R35-016 [P1]** README EN 增加 `success_command_rate` 与 `command_rate_basis` 对照示例。
17. **R35-017 [P1]** README ZH 增加 `success_command_rate` 与 `command_rate_basis` 对照示例。
18. **R35-018 [P1]** docs freshness 守卫 README EN 对照示例。
19. **R35-019 [P1]** docs freshness 守卫 README ZH 对照示例。
20. **R35-020 [P1]** docs/CLI_CONTRACT_MATRIX 增加 rate 字段范围约束说明。
21. **R35-021 [P1]** docs/CLI_CONTRACT_MATRIX 增加 count 字段非负约束说明。
22. **R35-022 [P1]** docs freshness 守卫 matrix 的 `success_command_rate/failed_command_rate`。
23. **R35-023 [P1]** docs freshness 守卫 matrix 的 `total_scenarios`。
24. **R35-024 [P1]** HOOKS_SETUP 补充 `/tmp/runner-suites.json` artifact 说明。
25. **R35-025 [P1]** README EN 增补 artifact 消费方式示例。
26. **R35-026 [P1]** README ZH 增补 artifact 消费方式示例。
27. **R35-027 [P1]** docs freshness 增加 artifact 三文件守卫。
28. **R35-028 [P1]** docs freshness 增加 workflow schema smoke 守卫。
29. **R35-029 [P1]** regression_runner 文档增加 contract 字段表。
30. **R35-030 [P1]** release-contract-audit 文档增加字段表。
31. **R35-031 [P2]** release-contract-audit 增加 `--json-schema` 导出能力。
32. **R35-032 [P2]** regression_runner 增加 `--json-schema` 导出能力。
33. **R35-033 [P2]** CI workflow 增加 schema drift artifact。
34. **R35-034 [P2]** nightly 自动比较 schema 漂移。
35. **R35-035 [P2]** docs 增加 schema 演进版本表。
36. **R35-036 [P2]** docs 增加 basis 可视化示意。
37. **R35-037 [P2]** release JSON contract 快照测试。
38. **R35-038 [P2]** runner JSON contract 快照测试。
39. **R35-039 [P2]** required keys 单一配置源。
40. **R35-040 [P2]** tests 自动对齐 workflow required keys。
41. **R35-041 [P2]** release-audit 支持 `--output <file>`。
42. **R35-042 [P2]** runner 支持 `--json-pretty`。
43. **R35-043 [P2]** runner 支持 `--scenario-regex`。
44. **R35-044 [P2]** runner 支持 `--failed-only`。
45. **R35-045 [P2]** runner 支持 `--sort-by duration`。
46. **R35-046 [P2]** Round36 planner 自动提取 top3 P1。
47. **R35-047 [P2]** Round36 planner 自动生成 RED 命令模板。
48. **R35-048 [P2]** Round36 planner 自动生成 GREEN 命令模板。
49. **R35-049 [P2]** Round36 planner 自动生成 VERIFY 命令包。
50. **R35-050 [P2]** Round36 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 35)
- **Batch1 (P0, 本轮执行):** R35-001/R35-002 + R35-003/R35-004 + R35-005/R35-006/R35-007。
- **Batch2 (P1):** R35-011 / R35-014 / R35-018。
- **Batch3 (P1):** R35-020 / R35-022 / R35-027。

## Plan Output (Round 35)
- `docs/plans/2026-02-13-repo-gap-priority-round35.md`

## Execution Results (Round 35)
- Task A35 完成：`docs/HOOKS_SETUP.md` 增补 basis 分母语义（`step_rate_basis=total_steps`、`command_rate_basis=total_commands`、`rate_basis=total_scenarios`），并新增 docs freshness 守卫。
- Task B35 完成：`docs/CLI_CONTRACT_MATRIX.md` 增补 `Current schema contract: schema_version=v1`，并新增 docs freshness 守卫。
- Task C35 完成：`README.md` 与 `README.zh-CN.md` 增补 CI machine artifact 文件示例（`/tmp/release-audit-dry-run.json`、`/tmp/runner-contract.json`），并新增 docs freshness 守卫。
- 回归结果：
  - 目标测试：`42 passed, 23 subtests passed`。
  - 全量测试：`455 passed, 23 subtests passed`。
  - Shell 语法门禁：`shell-syntax:OK`。
  - Rust 门禁：clippy/fmt 全通过。

## Scan Results (Round 36)

### Repository Health Snapshot
- 扫描基线：`bash -n scripts/*.sh` -> `shell-syntax:OK`。
- 扫描基线：`pytest -q` -> `455 passed, 23 subtests passed`。
- 扫描基线：`(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `cargo fmt --all -- --check` -> 通过。
- 文档守卫基线：`pytest -q scripts/runtime/tests/test_docs_freshness.py` -> `20 passed, 19 subtests passed`。
- 缺口探针：`docs/HOOKS_SETUP.md` 尚无 CI machine artifact 文件说明。
- 缺口探针：`docs/CLI_CONTRACT_MATRIX.md` Notes 尚无 artifact 文件说明。
- 缺口探针：`README.md` 与 `README.zh-CN.md` 尚无 `/tmp/runner-suites.json` 文件示例。

### 50-Task Gap Backlog (Prioritized)
1. **R36-001 [P0]** docs freshness 新增 HOOKS_SETUP artifact 文件守卫（计划执行）。
2. **R36-002 [P0]** `docs/HOOKS_SETUP.md` 增补 CI machine artifact 文件示例（计划执行）。
3. **R36-003 [P0]** docs freshness 新增 CLI matrix Notes artifact 守卫（计划执行）。
4. **R36-004 [P0]** `docs/CLI_CONTRACT_MATRIX.md` Notes 增补 artifact 文件说明（计划执行）。
5. **R36-005 [P0]** docs freshness 新增 README EN/ZH `runner-suites` artifact 守卫（计划执行）。
6. **R36-006 [P0]** `README.md` 增补 `/tmp/runner-suites.json` 文件示例（计划执行）。
7. **R36-007 [P0]** `README.zh-CN.md` 增补 `/tmp/runner-suites.json` 文件示例（计划执行）。
8. **R36-008 [P0]** Round36 targeted docs+contract suite 回归全绿（计划执行）。
9. **R36-009 [P0]** Round36 全量 pytest 回归全绿（计划执行）。
10. **R36-010 [P0]** Round36 shell + rust 门禁全绿（计划执行）。
11. **R36-011 [P1]** HOOKS_SETUP 增加 artifact 含义说明（release/runner-suites/runner-contract）。
12. **R36-012 [P1]** HOOKS_SETUP 增加 artifact 与命令映射表。
13. **R36-013 [P1]** docs freshness 守卫 HOOKS_SETUP artifact 三文件齐全。
14. **R36-014 [P1]** CLI matrix Notes 增加 artifact 消费路径提示。
15. **R36-015 [P1]** CLI matrix Notes 增加 workflow 上传 artifact 说明。
16. **R36-016 [P1]** README EN 增加 artifact 快速检查命令示例。
17. **R36-017 [P1]** README ZH 增加 artifact 快速检查命令示例。
18. **R36-018 [P1]** docs freshness 守卫 README EN artifact 快速检查命令。
19. **R36-019 [P1]** docs freshness 守卫 README ZH artifact 快速检查命令。
20. **R36-020 [P1]** docs/CLI_CONTRACT_MATRIX 增加 rate 字段范围约束说明。
21. **R36-021 [P1]** docs/CLI_CONTRACT_MATRIX 增加 count 字段非负约束说明。
22. **R36-022 [P1]** docs freshness 守卫 matrix `success_command_rate/failed_command_rate`。
23. **R36-023 [P1]** docs freshness 守卫 matrix `total_scenarios`。
24. **R36-024 [P1]** HOOKS_SETUP 补充 `schema_version=v1` 单行契约说明。
25. **R36-025 [P1]** README EN 增补 `schema_version=v1` 行。
26. **R36-026 [P1]** README ZH 增补 `schema_version=v1` 行。
27. **R36-027 [P1]** docs freshness 增加 README EN/ZH schema-version 守卫。
28. **R36-028 [P1]** docs freshness 增加 workflow schema smoke 命令守卫。
29. **R36-029 [P1]** regression_runner 文档增加 contract 字段表。
30. **R36-030 [P1]** release-contract-audit 文档增加字段表。
31. **R36-031 [P2]** release-contract-audit 增加 `--json-schema` 导出能力。
32. **R36-032 [P2]** regression_runner 增加 `--json-schema` 导出能力。
33. **R36-033 [P2]** CI workflow 增加 schema drift artifact。
34. **R36-034 [P2]** nightly 自动比较 schema 漂移。
35. **R36-035 [P2]** docs 增加 schema 演进版本表。
36. **R36-036 [P2]** docs 增加 basis 可视化示意。
37. **R36-037 [P2]** release JSON contract 快照测试。
38. **R36-038 [P2]** runner JSON contract 快照测试。
39. **R36-039 [P2]** required keys 单一配置源。
40. **R36-040 [P2]** tests 自动对齐 workflow required keys。
41. **R36-041 [P2]** release-audit 支持 `--output <file>`。
42. **R36-042 [P2]** runner 支持 `--json-pretty`。
43. **R36-043 [P2]** runner 支持 `--scenario-regex`。
44. **R36-044 [P2]** runner 支持 `--failed-only`。
45. **R36-045 [P2]** runner 支持 `--sort-by duration`。
46. **R36-046 [P2]** Round37 planner 自动提取 top3 P1。
47. **R36-047 [P2]** Round37 planner 自动生成 RED 命令模板。
48. **R36-048 [P2]** Round37 planner 自动生成 GREEN 命令模板。
49. **R36-049 [P2]** Round37 planner 自动生成 VERIFY 命令包。
50. **R36-050 [P2]** Round37 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 36)
- **Batch1 (P0, 本轮执行):** R36-001/R36-002 + R36-003/R36-004 + R36-005/R36-006/R36-007。
- **Batch2 (P1):** R36-011 / R36-014 / R36-018。
- **Batch3 (P1):** R36-020 / R36-022 / R36-027。

## Plan Output (Round 36)
- `docs/plans/2026-02-13-repo-gap-priority-round36.md`

## Execution Results (Round 36)
- Task A36 完成：`docs/HOOKS_SETUP.md` 增补 CI machine artifact 文件示例（`/tmp/release-audit-dry-run.json`、`/tmp/runner-contract.json`），并新增 docs freshness 守卫。
- Task B36 完成：`docs/CLI_CONTRACT_MATRIX.md` Notes 增补 artifact 文件说明（含 `/tmp/runner-suites.json`），并新增 docs freshness 守卫。
- Task C36 完成：`README.md` 与 `README.zh-CN.md` 增补 `/tmp/runner-suites.json` 文件示例，并新增 docs freshness 守卫。
- 回归结果：
  - 目标测试：`45 passed, 23 subtests passed`。
  - 全量测试：`458 passed, 23 subtests passed`。
  - Shell 语法门禁：`shell-syntax:OK`。
  - Rust 门禁：clippy/fmt 全通过。

## Scan Results (Round 37)

### Repository Health Snapshot
- 扫描基线：`bash -n scripts/*.sh` -> `shell-syntax:OK`。
- 扫描基线：`pytest -q` -> `455 passed, 23 subtests passed`。
- 扫描基线：`(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `cargo fmt --all -- --check` -> 通过。
- 文档守卫基线：`pytest -q scripts/runtime/tests/test_docs_freshness.py` -> `23 passed, 19 subtests passed`。
- 缺口探针：`docs/HOOKS_SETUP.md` 尚无 `/tmp/runner-suites.json`。
- 缺口探针：`docs/HOOKS_SETUP.md` 尚无 `schema_version=v1` 契约文案。
- 缺口探针：`README.md` 与 `README.zh-CN.md` 尚无显式 `schema_version=v1` 契约文案。

### 50-Task Gap Backlog (Prioritized)
1. **R37-001 [P0]** docs freshness 新增 HOOKS_SETUP `runner-suites` artifact 守卫（计划执行）。
2. **R37-002 [P0]** `docs/HOOKS_SETUP.md` 增补 `/tmp/runner-suites.json` 文案（计划执行）。
3. **R37-003 [P0]** docs freshness 新增 HOOKS_SETUP `schema_version=v1` 守卫（计划执行）。
4. **R37-004 [P0]** `docs/HOOKS_SETUP.md` 增补 `schema_version=v1` 契约文案（计划执行）。
5. **R37-005 [P0]** docs freshness 新增 README EN/ZH `schema_version=v1` 守卫（计划执行）。
6. **R37-006 [P0]** `README.md` 增补 `schema_version=v1` 文案（计划执行）。
7. **R37-007 [P0]** `README.zh-CN.md` 增补 `schema_version=v1` 文案（计划执行）。
8. **R37-008 [P0]** Round37 targeted docs+contract suite 回归全绿（计划执行）。
9. **R37-009 [P0]** Round37 全量 pytest 回归全绿（计划执行）。
10. **R37-010 [P0]** Round37 shell + rust 门禁全绿（计划执行）。
11. **R37-011 [P1]** HOOKS_SETUP 增加 artifact 含义说明（release/suites/contract）。
12. **R37-012 [P1]** HOOKS_SETUP 增加 artifact 与命令映射表。
13. **R37-013 [P1]** docs freshness 守卫 HOOKS_SETUP artifact 三文件齐全。
14. **R37-014 [P1]** CLI matrix Notes 增加 artifact 消费路径提示。
15. **R37-015 [P1]** CLI matrix Notes 增加 workflow 上传 artifact 说明。
16. **R37-016 [P1]** README EN 增加 artifact 快速检查命令示例。
17. **R37-017 [P1]** README ZH 增加 artifact 快速检查命令示例。
18. **R37-018 [P1]** docs freshness 守卫 README EN artifact 快速检查命令。
19. **R37-019 [P1]** docs freshness 守卫 README ZH artifact 快速检查命令。
20. **R37-020 [P1]** docs/CLI_CONTRACT_MATRIX 增加 rate 字段范围约束说明。
21. **R37-021 [P1]** docs/CLI_CONTRACT_MATRIX 增加 count 字段非负约束说明。
22. **R37-022 [P1]** docs freshness 守卫 matrix `success_command_rate/failed_command_rate`。
23. **R37-023 [P1]** docs freshness 守卫 matrix `total_scenarios`。
24. **R37-024 [P1]** HOOKS_SETUP 补充 `schema_version=v1` 与 CLI matrix 对齐说明。
25. **R37-025 [P1]** README EN 增补 `schema_version=v1` 与 CLI matrix 对齐说明。
26. **R37-026 [P1]** README ZH 增补 `schema_version=v1` 与 CLI matrix 对齐说明。
27. **R37-027 [P1]** docs freshness 增加 README EN/ZH schema-version 对齐守卫。
28. **R37-028 [P1]** docs freshness 增加 workflow schema smoke 命令守卫。
29. **R37-029 [P1]** regression_runner 文档增加 contract 字段表。
30. **R37-030 [P1]** release-contract-audit 文档增加字段表。
31. **R37-031 [P2]** release-contract-audit 增加 `--json-schema` 导出能力。
32. **R37-032 [P2]** regression_runner 增加 `--json-schema` 导出能力。
33. **R37-033 [P2]** CI workflow 增加 schema drift artifact。
34. **R37-034 [P2]** nightly 自动比较 schema 漂移。
35. **R37-035 [P2]** docs 增加 schema 演进版本表。
36. **R37-036 [P2]** docs 增加 basis 可视化示意。
37. **R37-037 [P2]** release JSON contract 快照测试。
38. **R37-038 [P2]** runner JSON contract 快照测试。
39. **R37-039 [P2]** required keys 单一配置源。
40. **R37-040 [P2]** tests 自动对齐 workflow required keys。
41. **R37-041 [P2]** release-audit 支持 `--output <file>`。
42. **R37-042 [P2]** runner 支持 `--json-pretty`。
43. **R37-043 [P2]** runner 支持 `--scenario-regex`。
44. **R37-044 [P2]** runner 支持 `--failed-only`。
45. **R37-045 [P2]** runner 支持 `--sort-by duration`。
46. **R37-046 [P2]** Round38 planner 自动提取 top3 P1。
47. **R37-047 [P2]** Round38 planner 自动生成 RED 命令模板。
48. **R37-048 [P2]** Round38 planner 自动生成 GREEN 命令模板。
49. **R37-049 [P2]** Round38 planner 自动生成 VERIFY 命令包。
50. **R37-050 [P2]** Round38 planner 自动生成 findings/progress 初稿。

### Priority Recommendation (Round 37)
- **Batch1 (P0, 本轮执行):** R37-001/R37-002 + R37-003/R37-004 + R37-005/R37-006/R37-007。
- **Batch2 (P1):** R37-011 / R37-014 / R37-018。
- **Batch3 (P1):** R37-020 / R37-022 / R37-027。

## Plan Output (Round 37)
- `docs/plans/2026-02-13-repo-gap-priority-round37.md`

## Execution Results (Round 37)
- Task A37 完成：`docs/HOOKS_SETUP.md` artifact 列表补齐 `/tmp/runner-suites.json`，并新增 docs freshness 守卫。
- Task B37 完成：`docs/HOOKS_SETUP.md` 增补 `current schema contract: schema_version=v1`，并新增 docs freshness 守卫。
- Task C37 完成：`README.md` 与 `README.zh-CN.md` 增补 `schema_version=v1` 契约文案，并新增 docs freshness 守卫。
- 回归结果：
  - 目标测试：`48 passed, 23 subtests passed`。
  - 全量测试：`461 passed, 23 subtests passed`。
  - Shell 语法门禁：`shell-syntax:OK`。
  - Rust 门禁：clippy/fmt 全通过。

## Scan Results (Round 38)

### Repository Health Snapshot
- 扫描基线：`bash -n scripts/*.sh` -> `shell-syntax:OK`。
- 扫描基线：`pytest -q` -> `461 passed, 23 subtests passed`。
- 扫描基线：`(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `cargo fmt --all -- --check` -> 通过。
- 文档守卫基线：`pytest -q scripts/runtime/tests/test_docs_freshness.py` -> `26 passed, 19 subtests passed`。
- 编排状态探针：`.fusion/task_plan.md` 仍处于 `DECOMPOSE (3/8)`，Task2/Task3 处于 `PENDING`。
- 阻塞线索：`.fusion/progress.md` 记录双后端调用失败（codex 断流 + claude 参数异常），当前缺少统一结构化失败报告入口。

### Gaps & Incomplete Items
1. `fusion-codeagent.sh` 在 primary+fallback 均失败时仅返回非零，未持久化 backend 级失败上下文（不可机器消费）。
2. `fusion-status.sh` 仅展示 `dependency_report.json`，无法直接读取/呈现 backend 失败摘要。
3. README 中英仅说明 `dependency_report.json`，缺少 backend 双失败报告文件说明，运维排障路径不完整。

### Priority Recommendation (Round 38)
- P0: `fusion-codeagent.sh` 增加 `.fusion/backend_failure_report.json` 写入/清理契约。
- P1: `fusion-status.sh` 暴露 backend failure 摘要（JSON + human）。
- P1: README(EN/ZH) 补齐 backend failure 报告说明并加 docs freshness 守卫。

## Plan Output (Round 38)
- `docs/plans/2026-02-13-repo-gap-priority-round38.md`

## Execution Results (Round 38)
- Task A38 完成：`fusion-codeagent.sh` 在双后端失败时写入 `.fusion/backend_failure_report.json`，并在成功调用后清理陈旧 backend failure 报告。
- Task B38 完成：`fusion-status.sh` 在 JSON 模式新增 `backend_status/backend_primary/backend_fallback`，并在人类输出新增 `## Backend Failure Report`。
- Task C38 完成：README 中英 `Dependency Auto-Heal` 段落补齐 `.fusion/backend_failure_report.json` 指引，并由 docs freshness 保护。
- 回归：targeted 83 passed, 23 subtests passed；full 466 passed, 23 subtests passed；shell/rust 门禁均通过。

## Scan Results (Round 39)

### Repository Health Snapshot
- 扫描基线：`bash -n scripts/*.sh` -> `shell-syntax:OK`。
- 扫描基线：`pytest -q` -> `466 passed, 23 subtests passed`。
- 扫描基线：`(cd rust && cargo clippy --workspace --all-targets -- -D warnings)` + `cargo fmt --all -- --check` -> 通过。
- 文档守卫基线：`pytest -q scripts/runtime/tests/test_docs_freshness.py` -> `27 passed, 19 subtests passed`。

### Gaps & Incomplete Items
1. `SKILL.md` 仍只提及 `.fusion/dependency_report.json`，未同步 `.fusion/backend_failure_report.json`（文档缺口）。
2. `docs/CLI_CONTRACT_MATRIX.md` 未声明 `fusion-status.sh --json` 的 `backend_status/backend_primary/backend_fallback` 字段与 `.fusion/backend_failure_report.json` 关联（合约缺口）。
3. `fusion-codeagent.sh` 在缺依赖写 `dependency_report.json` 时不会清理陈旧 `backend_failure_report.json`，可能导致状态页同时出现误导信息（行为缺口）。

### Priority Recommendation (Round 39)
- P0: codeagent 缺依赖时清理 stale backend failure report。
- P1: SKILL.md 补齐 backend failure report 文件说明（并加 docs freshness 守卫）。
- P1: CLI_CONTRACT_MATRIX 补齐 backend report 与 status backend_* JSON 字段契约（并加 docs freshness 守卫）。

## Plan Output (Round 39)
- `docs/plans/2026-02-13-repo-gap-priority-round39.md`

## Execution Results (Round 39)
- Task A39 完成：缺依赖时写 `dependency_report.json` 前清理陈旧 `.fusion/backend_failure_report.json`，避免状态页误导。
- Task B39 完成：`SKILL.md` 补齐 `.fusion/backend_failure_report.json` 并由 docs freshness 守卫保护。
- Task C39 完成：`docs/CLI_CONTRACT_MATRIX.md` 补齐 `.fusion/backend_failure_report.json` 与 `backend_status/backend_primary/backend_fallback` 说明，并由 docs freshness 守卫保护。
- 回归：targeted `47 passed, 23 subtests passed`；full `472 passed, 27 subtests passed`；shell/rust 门禁均通过。

## Scan Results (Round 40)

### Repository Health Snapshot
- 验证基线：`pytest -q` -> `472 passed, 27 subtests passed`。
- shell 语法门禁：`bash -n scripts/*.sh` -> OK。
- Rust 门禁：`cargo clippy -- -D warnings` + `cargo fmt --check` -> OK。

### Evidence: Loop “中断”更像是后端 hang / resume 失败，而不是 hook 失效
- `codeagent-wrapper --backend codex` 在本机环境下会 hang，并持续输出 `state db missing rollout path...`，需要外部 `timeout` 才能终止（否则无法触发 fallback）。
- `codeagent-wrapper --backend claude` 正常返回，并输出 `SESSION_ID: <uuid>`。
- 仓库内 `.fusion/sessions.json` 存在非 UUID 的 `claude_session`（如 `"1610419"`）。旧逻辑在 `resume` 失败后会直接 fallback 到 `codex`，叠加 `codex` hang 后表现为“循环断/不继续”。

### Gaps & Incomplete Items
1. `fusion-codeagent.sh` 的 session id 提取仅匹配“6 位以上数字”，导致 Claude UUID session 不会被持久化，随后 resume 可靠性下降。
2. `resume` 失败会立即 fallback 到备用后端，缺少“同后端无 resume 重试”自愈路径，容易触发不必要降级。
3. 缺少对 `codeagent-wrapper` 调用的 timeout 保护，遇到 `codex` hang 时无法自动走 fallback，表现为工作流停住。

### Priority Recommendation (Round 40)
- P0: session id 提取改为解析 `SESSION_ID:` 行（支持 UUID）。
- P0: resume 失败时同后端无 resume 重试一次（避免无谓 fallback）。
- P0: 增加 `FUSION_CODEAGENT_TIMEOUT_SEC` 超时保护以让 hang 可触发 fallback。

## Plan Output (Round 40)
- `docs/plans/2026-02-12-fusion-loop-resilience-round40.md`

## Execution Results (Round 40)
- Task A40 完成：`extract_session_id` 解析 `SESSION_ID:` 行，支持 UUID 并避免误抓 PID。
- Task B40 完成：primary resume 失败时同后端无 resume 重试一次，成功则不触发 fallback。
- Task C40 完成：新增 `FUSION_CODEAGENT_TIMEOUT_SEC`，当 `timeout/gtimeout` 可用时对 `codeagent-wrapper` 调用启用超时，超时后触发 fallback。
- 回归：`pytest -q scripts/runtime/tests/test_fusion_codeagent_script.py` -> `18 passed`；full `pytest -q` -> `472 passed, 27 subtests passed`；shell/rust 门禁均通过。
