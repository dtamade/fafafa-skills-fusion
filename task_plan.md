# Repo Gap Scan & Priority Execution Task Plan

## Goal
- 全仓扫描未完成项与缺口，输出可执行优先级计划；随后按 TDD 严格执行首批任务并记录命令输出。

## Status
- Current Phase: COMPLETE_ROUND40 (5/5)
- Active Task: -
- Backend: claude primary, codex fallback (codex hang guarded)
- Started: 2026-02-11
- Elapsed: -

## Configuration
- TDD: enabled (strict)
- Git: disabled (no commit in this round)
- Parallel: 1

## Phases
- [COMPLETED] Phase 1: 全仓扫描（代码/测试/文档/配置缺口）
- [COMPLETED] Phase 2: 形成优先级计划（docs/plans）
- [COMPLETED] Phase 3: 执行批次 1（默认 3 个任务，严格 TDD）
- [COMPLETED] Phase 4: 回归验证（目标测试 + 全量）
- [COMPLETED] Phase 5: 结果汇报与下一轮计划

## Candidate Tasks (to refine after scan)
- Task A: docs freshness guard + README.zh-CN dynamic wording [COMPLETED]
- Task B: fusion-hook-doctor --json output + tests [COMPLETED]
- Task C: fusion-status leaderboard toggle + tests [COMPLETED]

## Errors Encountered

| Task | Error | Attempt | Resolution |
|------|-------|---------|------------|
| - | - | - | - |

## Decisions Made

| Decision | Rationale | Task |
|----------|-----------|------|
| 严格使用 TDD | 用户明确要求 | 全部 |


## Round 2 Phases
- [COMPLETED] Phase 1: Round2 全仓扫描
- [COMPLETED] Phase 2: Round2 计划文档
- [COMPLETED] Phase 3: Round2 Batch1 TDD执行
- [COMPLETED] Phase 4: Round2 回归验证
- [COMPLETED] Phase 5: Round2 汇报

## Round 2 Candidate Tasks
- Task A2: fusion-start 参数校验（unknown/multi-goal）[COMPLETED]
- Task B2: fusion-init 支持 --engine rust|python [COMPLETED]
- Task C2: fusion-init 支持 --json 机器可读输出 [COMPLETED]
- Task C2.1: fusion-init --json 在无 jq/python3 fallback 仍输出合法 JSON [COMPLETED]

## Round 3 Phases
- [COMPLETED] Phase 1: Round3 全仓扫描
- [COMPLETED] Phase 2: Round3 计划文档
- [COMPLETED] Phase 3: Round3 Batch1 TDD执行
- [COMPLETED] Phase 4: Round3 回归验证
- [COMPLETED] Phase 5: Round3 汇报

## Round 3 Candidate Tasks
- Task A3: fusion-resume 参数校验（拒绝未知选项）[COMPLETED]
- Task B3: fusion-git 未知 action 明确报错[COMPLETED]
- Task C3: fusion-logs 行数参数校验（正整数）[COMPLETED]

## Round 4 Phases
- [COMPLETED] Phase 1: Round4 全仓扫描
- [COMPLETED] Phase 2: Round4 计划文档
- [COMPLETED] Phase 3: Round4 Batch1 TDD执行
- [COMPLETED] Phase 4: Round4 回归验证
- [COMPLETED] Phase 5: Round4 汇报

## Round 4 Candidate Tasks
- Task A4: fusion-pause 参数校验（拒绝未知选项）[COMPLETED]
- Task B4: fusion-cancel 参数校验（拒绝未知选项）[COMPLETED]
- Task C4: fusion-continue 参数校验（拒绝未知选项）[COMPLETED]

## Round 5 Phases
- [COMPLETED] Phase 1: Round5 全仓扫描
- [COMPLETED] Phase 2: Round5 计划文档
- [COMPLETED] Phase 3: Round5 Batch1 TDD执行
- [COMPLETED] Phase 4: Round5 回归验证
- [COMPLETED] Phase 5: Round5 汇报

## Round 5 Candidate Tasks
- Task A5: loop-guardian guardian_status 阈值与配置一致 [COMPLETED]
- Task B5: loop-guardian guardian_init 自动创建 FUSION_DIR [COMPLETED]
- Task C5: loop-guardian guardian_status 增加 state/walltime 阈值可见性 [COMPLETED]

## Round 6 Phases
- [COMPLETED] Phase 1: Round6 全仓扫描
- [COMPLETED] Phase 2: Round6 计划文档
- [COMPLETED] Phase 3: Round6 Batch1 TDD执行
- [COMPLETED] Phase 4: Round6 回归验证
- [COMPLETED] Phase 5: Round6 汇报

## Round 6 Candidate Tasks
- Task A6: fusion-start usage 字符串修复（消除 `<goal>` 重定向误解析）[COMPLETED]
- Task B6: fusion-start `--help` 退出码/输出回归 [COMPLETED]
- Task C6: fusion-start 未知参数与无参数 usage 一致性回归 [COMPLETED]

## Round 7 Phases
- [COMPLETED] Phase 1: Round7 全仓扫描
- [COMPLETED] Phase 2: Round7 计划文档
- [COMPLETED] Phase 3: Round7 Batch1 TDD执行
- [COMPLETED] Phase 4: Round7 回归验证
- [COMPLETED] Phase 5: Round7 汇报

## Round 7 Candidate Tasks
- Task A7: fusion-achievements `--top` 参数数值校验 [COMPLETED]
- Task B7: fusion-achievements `--root` 参数缺失值校验 [COMPLETED]
- Task C7: fusion-achievements `--top` 参数缺失值校验 [COMPLETED]

## Round 8 Phases
- [COMPLETED] Phase 1: Round8 全仓扫描
- [COMPLETED] Phase 2: Round8 计划文档
- [COMPLETED] Phase 3: Round8 Batch1 TDD执行
- [COMPLETED] Phase 4: Round8 回归验证
- [COMPLETED] Phase 5: Round8 汇报

## Round 8 Candidate Tasks
- Task A8: achievements 错误参数时避免输出成功标题横幅 [COMPLETED]
- Task B8: achievements 支持 `--top=<n>` 参数格式 [COMPLETED]
- Task C8: achievements 支持 `--root=<path>` 参数格式 [COMPLETED]
- Round8 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 8)」

## Round 9 Phases
- [COMPLETED] Phase 1: Round9 全仓扫描
- [COMPLETED] Phase 2: Round9 计划文档
- [COMPLETED] Phase 3: Round9 Batch1 TDD执行
- [COMPLETED] Phase 4: Round9 回归验证
- [COMPLETED] Phase 5: Round9 汇报

## Round 9 Candidate Tasks
- Task A9: fusion-logs 支持 `--help` 且退出 0 [COMPLETED]
- Task B9: fusion-git 支持 `--help` 且退出 0 [COMPLETED]
- Task C9: fusion-codeagent 支持 `--help` 且不触发 route [COMPLETED]
- Round9 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 9)」

## Round 10 Phases
- [COMPLETED] Phase 1: Round10 全仓扫描
- [COMPLETED] Phase 2: Round10 计划文档
- [COMPLETED] Phase 3: Round10 Batch1 TDD执行
- [COMPLETED] Phase 4: Round10 回归验证
- [COMPLETED] Phase 5: Round10 汇报

## Round 10 Candidate Tasks
- Task A10: rust clippy too_many_arguments 修复 [COMPLETED]
- Task B10: rust fmt --check 清零 [COMPLETED]
- Task C10: fusion-status 支持 `--help` 且无 .fusion 返回 0 [COMPLETED]
- Round10 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 10)」

## Round 11 Phases
- [COMPLETED] Phase 1: Round11 全仓扫描
- [COMPLETED] Phase 2: Round11 计划文档
- [COMPLETED] Phase 3: Round11 Batch1 TDD执行
- [COMPLETED] Phase 4: Round11 回归验证
- [COMPLETED] Phase 5: Round11 汇报

## Round 11 Candidate Tasks
- Task A11: fusion-status 支持 `--json` 机器可读输出 [COMPLETED]
- Task B11: fusion-status 缺失 .fusion 时 `--json` 错误对象 [COMPLETED]
- Task C11: fusion-status `--json` 不输出人类横幅 [COMPLETED]
- Round11 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 11)」

## Round 12 Phases
- [COMPLETED] Phase 1: Round12 全仓扫描
- [COMPLETED] Phase 2: Round12 计划文档
- [COMPLETED] Phase 3: Round12 Batch1 TDD执行
- [COMPLETED] Phase 4: Round12 回归验证
- [COMPLETED] Phase 5: Round12 汇报

## Round 12 Candidate Tasks
- Task A12: fusion-status --json 增加 task counters [COMPLETED]
- Task B12: fusion-status --json 增加 dependency summary [COMPLETED]
- Task C12: fusion-status --json 增加 achievement counters [COMPLETED]
- Round12 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 12)」

## Round 13 Phases
- [COMPLETED] Phase 1: Round13 全仓扫描
- [COMPLETED] Phase 2: Round13 计划文档
- [COMPLETED] Phase 3: Round13 Batch1 TDD执行
- [COMPLETED] Phase 4: Round13 回归验证
- [COMPLETED] Phase 5: Round13 汇报

## Round 13 Candidate Tasks
- Task A13: fusion-codeagent 拒绝未知选项并避免误路由 [COMPLETED]
- Task B13: fusion-hook-doctor 拒绝未知选项并输出稳定错误 [COMPLETED]
- Task C13: fusion-hook-doctor 无效 project_root 失败快返 [COMPLETED]
- Round13 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 13)」

## Round 14 Phases
- [COMPLETED] Phase 1: Round14 全仓扫描
- [COMPLETED] Phase 2: Round14 计划文档
- [COMPLETED] Phase 3: Round14 Batch1 TDD执行
- [COMPLETED] Phase 4: Round14 回归验证
- [COMPLETED] Phase 5: Round14 汇报

## Round 14 Candidate Tasks
- Task A14: fusion-hook-doctor 实现 `--fix` 自动修复 [COMPLETED]
- Task B14: fusion-hook-doctor JSON 增加 `fixed` 字段 [COMPLETED]
- Task C14: docs/HOOKS_SETUP 增补 `--fix` 流程 [COMPLETED]
- Round14 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 14)」

## Round 15 Phases
- [COMPLETED] Phase 1: Round15 全仓扫描
- [COMPLETED] Phase 2: Round15 计划文档
- [COMPLETED] Phase 3: Round15 Batch1 TDD执行
- [COMPLETED] Phase 4: Round15 回归验证
- [COMPLETED] Phase 5: Round15 汇报

## Round 15 Candidate Tasks
- Task A15: stop-guard 锁竞争 structured JSON 阻断修复 [COMPLETED]
- Task B15: stop-guard 专项脚本测试补齐 [COMPLETED]
- Task C15: README 中英文补齐 hook-doctor quick-fix [COMPLETED]
- Round15 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 15)」

## Round 16 Phases
- [COMPLETED] Phase 1: Round16 全仓扫描
- [COMPLETED] Phase 2: Round16 计划文档
- [COMPLETED] Phase 3: Round16 Batch1 TDD执行
- [COMPLETED] Phase 4: Round16 回归验证
- [COMPLETED] Phase 5: Round16 汇报

## Round 16 Candidate Tasks
- Task A16: logs + git 参数/错误通道契约统一 [COMPLETED]
- Task B16: stop-guard structured 无 stdin 契约测试 [COMPLETED]
- Task C16: hook shell runtime path parity 测试补齐 [COMPLETED]
- Round16 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 16)」

## Round 17 Phases
- [COMPLETED] Phase 1: Round17 全仓扫描
- [COMPLETED] Phase 2: Round17 计划文档
- [COMPLETED] Phase 3: Round17 Batch1 TDD执行
- [COMPLETED] Phase 4: Round17 回归验证
- [COMPLETED] Phase 5: Round17 汇报

## Round 17 Candidate Tasks
- Task A17: status JSON 参数契约补齐 [COMPLETED]
- Task B17: hook-doctor fix 失败路径契约补齐 [COMPLETED]
- Task C17: logs 多参数边界契约补齐 [COMPLETED]
- Round17 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 17)」

## Round 18 Phases
- [COMPLETED] Phase 1: Round18 全仓扫描
- [COMPLETED] Phase 2: Round18 计划文档
- [COMPLETED] Phase 3: Round18 Batch1 TDD执行
- [COMPLETED] Phase 4: Round18 回归验证
- [COMPLETED] Phase 5: Round18 汇报

## Round 18 Candidate Tasks
- Task A18: CI 契约门禁 workflow（shell+pytest+rust）[COMPLETED]
- Task B18: CLI 参数契约矩阵文档化 [COMPLETED]
- Task C18: 发布前自动契约审计脚本 [COMPLETED]
- Round18 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 18)」

## Round 19 Phases
- [COMPLETED] Phase 1: Round19 全仓扫描
- [COMPLETED] Phase 2: Round19 计划文档
- [COMPLETED] Phase 3: Round19 Batch1 TDD执行
- [COMPLETED] Phase 4: Round19 回归验证
- [COMPLETED] Phase 5: Round19 汇报

## Round 19 Candidate Tasks
- Task A19: release-contract-audit 组合参数 + 失败汇总 [COMPLETED]
- Task B19: regression_runner 增加 contract suite [COMPLETED]
- Task C19: 文档契约强化（matrix/help/CI/release）[COMPLETED]
- Round19 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 19)」

## Round 20 Phases
- [COMPLETED] Phase 1: Round20 全仓扫描
- [COMPLETED] Phase 2: Round20 计划文档
- [COMPLETED] Phase 3: Round20 Batch1 TDD执行
- [COMPLETED] Phase 4: Round20 回归验证
- [COMPLETED] Phase 5: Round20 汇报

## Round 20 Candidate Tasks
- Task A20: release-contract-audit 增加 `--json` 机器摘要 [COMPLETED]
- Task B20: regression_runner 增加 `--list-suites` [COMPLETED]
- Task C20: CI workflow 增加 pip/rust cache 契约 [COMPLETED]
- Round20 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 20)」

## Round 21 Phases
- [COMPLETED] Phase 1: Round21 全仓扫描
- [COMPLETED] Phase 2: Round21 计划文档
- [COMPLETED] Phase 3: Round21 Batch1 TDD执行
- [COMPLETED] Phase 4: Round21 回归验证
- [COMPLETED] Phase 5: Round21 汇报

## Round 21 Candidate Tasks
- Task A21: release-audit 增加 `--json-pretty` [COMPLETED]
- Task B21: regression_runner 支持 `--list-suites --json` [COMPLETED]
- Task C21: 文档契约同步（json-pretty/list-suites-json）[COMPLETED]
- Round21 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 21)」

## Round 22 Phases
- [COMPLETED] Phase 1: Round22 全仓扫描
- [COMPLETED] Phase 2: Round22 计划文档
- [COMPLETED] Phase 3: Round22 Batch1 TDD执行
- [COMPLETED] Phase 4: Round22 回归验证
- [COMPLETED] Phase 5: Round22 汇报

## Round 22 Candidate Tasks
- Task A22: release-audit 运行态 JSON 指标（steps/timing）[COMPLETED]
- Task B22: regression_runner suite 执行 JSON 汇总 [COMPLETED]
- Task C22: CI workflow 机器模式 smoke gate [COMPLETED]
- Round22 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 22)」

## Round 23 Phases
- [COMPLETED] Phase 1: Round23 全仓扫描
- [COMPLETED] Phase 2: Round23 计划文档
- [COMPLETED] Phase 3: Round23 Batch1 TDD执行
- [COMPLETED] Phase 4: Round23 回归验证
- [COMPLETED] Phase 5: Round23 汇报

## Round 23 Candidate Tasks
- Task A23: release-audit step-level 时间戳与序号 [COMPLETED]
- Task B23: regression_runner suite JSON 场景详情 [COMPLETED]
- Task C23: CI machine smoke 增加 suite JSON 命令 [COMPLETED]
- Round23 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 23)」

## Round 24 Phases
- [COMPLETED] Phase 1: Round24 全仓扫描
- [COMPLETED] Phase 2: Round24 计划文档
- [COMPLETED] Phase 3: Round24 Batch1 TDD执行
- [COMPLETED] Phase 4: Round24 回归验证
- [COMPLETED] Phase 5: Round24 汇报

## Round 24 Candidate Tasks
- Task A24: release-audit step-level `exit_code` [COMPLETED]
- Task B24: runner JSON `longest_scenario` 聚合 [COMPLETED]
- Task C24: CI machine JSON artifacts 上传 [COMPLETED]
- Round24 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 24)」

## Round 25 Phases
- [COMPLETED] Phase 1: Round25 全仓扫描
- [COMPLETED] Phase 2: Round25 计划文档
- [COMPLETED] Phase 3: Round25 Batch1 TDD执行
- [COMPLETED] Phase 4: Round25 回归验证
- [COMPLETED] Phase 5: Round25 汇报

## Round 25 Candidate Tasks
- Task A25: release-audit JSON `failed_steps` 聚合 [COMPLETED]
- Task B25: runner JSON `fastest_scenario` 聚合 [COMPLETED]
- Task C25: CI machine smoke 增加 schema 校验 [COMPLETED]
- Round25 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 25)」

## Round 26 Phases
- [COMPLETED] Phase 1: Round26 全仓扫描
- [COMPLETED] Phase 2: Round26 计划文档
- [COMPLETED] Phase 3: Round26 Batch1 TDD执行
- [COMPLETED] Phase 4: Round26 回归验证
- [COMPLETED] Phase 5: Round26 汇报

## Round 26 Candidate Tasks
- Task A26: release-audit JSON `failed_commands` 聚合 [COMPLETED]
- Task B26: runner JSON `scenario_count_by_result` 聚合 [COMPLETED]
- Task C26: CI machine smoke 增加 release/suites schema 校验 [COMPLETED]
- Round26 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 26)」

## Round 27 Phases
- [COMPLETED] Phase 1: Round27 全仓扫描
- [COMPLETED] Phase 2: Round27 计划文档
- [COMPLETED] Phase 3: Round27 Batch1 TDD执行
- [COMPLETED] Phase 4: Round27 回归验证
- [COMPLETED] Phase 5: Round27 汇报

## Round 27 Candidate Tasks
- Task A27: release-audit JSON `failed_commands_count` 聚合 [COMPLETED]
- Task B27: runner JSON `duration_stats` 聚合 [COMPLETED]
- Task C27: CI machine smoke schema required keys 同步 [COMPLETED]
- Round27 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 27)」

## Round 28 Phases
- [COMPLETED] Phase 1: Round28 全仓扫描
- [COMPLETED] Phase 2: Round28 计划文档
- [COMPLETED] Phase 3: Round28 Batch1 TDD执行
- [COMPLETED] Phase 4: Round28 回归验证
- [COMPLETED] Phase 5: Round28 汇报

## Round 28 Candidate Tasks
- Task A28: release-audit JSON `success_steps_count/commands_count` 聚合 [COMPLETED]
- Task B28: runner JSON `failed_rate` 聚合 [COMPLETED]
- Task C28: CI machine smoke schema required keys 同步 [COMPLETED]
- Round28 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 28)」

## Round 29 Phases
- [COMPLETED] Phase 1: Round29 全仓扫描
- [COMPLETED] Phase 2: Round29 计划文档
- [COMPLETED] Phase 3: Round29 Batch1 TDD执行
- [COMPLETED] Phase 4: Round29 回归验证
- [COMPLETED] Phase 5: Round29 汇报

## Round 29 Candidate Tasks
- Task A29: release-audit JSON `success_rate/failed_rate` 聚合 [COMPLETED]
- Task B29: runner JSON `success_rate` 聚合 [COMPLETED]
- Task C29: CI machine smoke schema required keys 同步 [COMPLETED]
- Round29 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 29)」

## Round 30 Phases
- [COMPLETED] Phase 1: Round30 全仓扫描
- [COMPLETED] Phase 2: Round30 计划文档
- [COMPLETED] Phase 3: Round30 Batch1 TDD执行
- [COMPLETED] Phase 4: Round30 回归验证
- [COMPLETED] Phase 5: Round30 汇报

## Round 30 Candidate Tasks
- Task A30: release-audit JSON `error_step_count` 别名 [COMPLETED]
- Task B30: runner JSON `success_count/failure_count` 聚合 [COMPLETED]
- Task C30: CI machine smoke schema required keys 同步 [COMPLETED]
- Round30 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 30)」

## Round 31 Phases
- [COMPLETED] Phase 1: Round31 全仓扫描
- [COMPLETED] Phase 2: Round31 计划文档
- [COMPLETED] Phase 3: Round31 Batch1 TDD执行
- [COMPLETED] Phase 4: Round31 回归验证
- [COMPLETED] Phase 5: Round31 汇报

## Round 31 Candidate Tasks
- Task A31: release-audit JSON `success_command_rate/failed_command_rate` 聚合 [COMPLETED]
- Task B31: runner JSON `total_scenarios` 聚合 [COMPLETED]
- Task C31: CI machine smoke schema required keys 同步 [COMPLETED]
- Round31 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 31)」

## Round 32 Phases
- [COMPLETED] Phase 1: Round32 全仓扫描
- [COMPLETED] Phase 2: Round32 计划文档
- [COMPLETED] Phase 3: Round32 Batch1 TDD执行
- [COMPLETED] Phase 4: Round32 回归验证
- [COMPLETED] Phase 5: Round32 汇报

## Round 32 Candidate Tasks
- Task A32: release-audit JSON `schema_version/step_rate_basis/command_rate_basis` [COMPLETED]
- Task B32: runner JSON `schema_version/rate_basis` [COMPLETED]
- Task C32: CI machine smoke required keys + basis consistency 校验 [COMPLETED]

## Round 33 Phases
- [COMPLETED] Phase 1: Round33 全仓扫描
- [COMPLETED] Phase 2: Round33 计划文档
- [COMPLETED] Phase 3: Round33 Batch1 TDD执行
- [COMPLETED] Phase 4: Round33 回归验证
- [COMPLETED] Phase 5: Round33 汇报

## Round 33 Candidate Tasks
- Task A33: CLI_CONTRACT_MATRIX 补齐 schema/basis 契约说明 [COMPLETED]
- Task B33: README(EN) 补齐 machine JSON schema/basis 示例 [COMPLETED]
- Task C33: README(ZH) 补齐 machine JSON schema/basis 示例 [COMPLETED]

## Round 34 Phases
- [COMPLETED] Phase 1: Round34 全仓扫描
- [COMPLETED] Phase 2: Round34 计划文档
- [COMPLETED] Phase 3: Round34 Batch1 TDD执行
- [COMPLETED] Phase 4: Round34 回归验证
- [COMPLETED] Phase 5: Round34 汇报

## Round 34 Candidate Tasks
- Task A34: HOOKS_SETUP 补齐 machine schema/basis 语义说明 + docs freshness 守卫 [COMPLETED]
- Task B34: CLI_CONTRACT_MATRIX 增加 machine required keys 段落 + docs freshness 守卫 [COMPLETED]
- Task C34: README(EN/ZH) 补齐 basis 分母语义说明 + docs freshness 守卫 [COMPLETED]
- Round34 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 34)」

## Round 35 Phases
- [COMPLETED] Phase 1: Round35 全仓扫描
- [COMPLETED] Phase 2: Round35 计划文档
- [COMPLETED] Phase 3: Round35 Batch1 TDD执行
- [COMPLETED] Phase 4: Round35 回归验证
- [COMPLETED] Phase 5: Round35 汇报

## Round 35 Candidate Tasks
- Task A35: HOOKS_SETUP 补齐 basis 分母语义文案 + docs freshness 守卫 [COMPLETED]
- Task B35: CLI_CONTRACT_MATRIX 增加 `schema_version=v1` 约束说明 + docs freshness 守卫 [COMPLETED]
- Task C35: README(EN/ZH) 补齐 CI machine artifact 文件说明 + docs freshness 守卫 [COMPLETED]
- Round35 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 35)」

## Round 36 Phases
- [COMPLETED] Phase 1: Round36 全仓扫描
- [COMPLETED] Phase 2: Round36 计划文档
- [COMPLETED] Phase 3: Round36 Batch1 TDD执行
- [COMPLETED] Phase 4: Round36 回归验证
- [COMPLETED] Phase 5: Round36 汇报

## Round 36 Candidate Tasks
- Task A36: HOOKS_SETUP 增加 CI machine artifact 文件说明 + docs freshness 守卫 [COMPLETED]
- Task B36: CLI_CONTRACT_MATRIX Notes 增加 CI artifact 文件说明 + docs freshness 守卫 [COMPLETED]
- Task C36: README(EN/ZH) 增加 `runner-suites` artifact 文件说明 + docs freshness 守卫 [COMPLETED]
- Round36 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 36)」

## Round 37 Phases
- [COMPLETED] Phase 1: Round37 全仓扫描
- [COMPLETED] Phase 2: Round37 计划文档
- [COMPLETED] Phase 3: Round37 Batch1 TDD执行
- [COMPLETED] Phase 4: Round37 回归验证
- [COMPLETED] Phase 5: Round37 汇报

## Round 37 Candidate Tasks
- Task A37: HOOKS_SETUP 增加 `runner-suites` artifact 文案 + docs freshness 守卫 [COMPLETED]
- Task B37: HOOKS_SETUP 增加 `schema_version=v1` 契约文案 + docs freshness 守卫 [COMPLETED]
- Task C37: README(EN/ZH) 增加 `schema_version=v1` 契约文案 + docs freshness 守卫 [COMPLETED]
- Round37 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 37)」

## Round 38 Phases
- [COMPLETED] Phase 1: Round38 全仓扫描
- [COMPLETED] Phase 2: Round38 计划文档
- [COMPLETED] Phase 3: Round38 Batch1 TDD执行
- [COMPLETED] Phase 4: Round38 回归验证
- [COMPLETED] Phase 5: Round38 汇报

## Round 38 Candidate Tasks
- Task A38: codeagent 双后端失败写入 backend_failure_report.json [COMPLETED]
- Task B38: status 暴露 backend failure 摘要（JSON + human）[COMPLETED]
- Task C38: README(EN/ZH) 补齐 backend_failure_report 指引 + docs freshness 守卫 [COMPLETED]
- Round38 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 38)」

## Round 39 Phases
- [COMPLETED] Phase 1: Round39 全仓扫描
- [COMPLETED] Phase 2: Round39 计划文档
- [COMPLETED] Phase 3: Round39 Batch1 TDD执行
- [COMPLETED] Phase 4: Round39 回归验证
- [COMPLETED] Phase 5: Round39 汇报

## Round 39 Candidate Tasks
- Task A39: codeagent 缺依赖写 dependency_report 时清理 stale backend_failure_report [COMPLETED]
- Task B39: SKILL.md 补齐 backend_failure_report.json 并加 docs freshness 守卫 [COMPLETED]
- Task C39: CLI_CONTRACT_MATRIX 补齐 backend report + status backend_* JSON 字段说明（docs freshness 守卫）[COMPLETED]
- Round39 Backlog: 50 项缺口清单见 findings.md「Scan Results (Round 39)」

## Round 40 Phases
- [COMPLETED] Phase 1: Round40 全仓扫描（会话循环中断/后端 hang）
- [COMPLETED] Phase 2: Round40 计划文档
- [COMPLETED] Phase 3: Round40 Batch1 TDD执行
- [COMPLETED] Phase 4: Round40 回归验证
- [COMPLETED] Phase 5: Round40 汇报

## Round 40 Candidate Tasks
- Task A40: codeagent session id 提取支持 UUID（避免写入错误 resume id）[COMPLETED]
- Task B40: resume 失败时同后端无 resume 重试（避免无谓 fallback 到 codex）[COMPLETED]
- Task C40: 增加 `FUSION_CODEAGENT_TIMEOUT_SEC` 超时保护，timeout 后触发 fallback（避免 codex hang 卡死）[COMPLETED]
