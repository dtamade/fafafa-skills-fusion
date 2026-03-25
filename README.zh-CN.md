# Fusion Skill

[English](README.md) | 简体中文

**自主工作流 Skill** - 给目标后自主执行，只在必要时打扰用户。

## 特性

- 🤖 **自主执行** - 给定目标后自动完成，无需频繁确认
- 🧪 **TDD 驱动** - 强制测试驱动开发流程
- 🔄 **智能降级** - 3-Strike 错误协议，自动降级到备用方案
- 📊 **进度追踪** - 持久化进度到文件，随时可恢复
- 🌳 **Git 集成** - 自动分支管理和提交
- ⚡ **并行执行** - 独立任务并行处理
- 🧠 **UNDERSTAND 阶段** - 主流程仍保留该阶段语义，当前由 Rust 启动路径做最小推进
- 🔌 **codeagent-wrapper 桥接** - 统一 Codex/Claude 调用与会话复用

## 默认后端路由

Fusion 默认采用角色分工路由：

- 规划 / 分析 / 审查阶段 -> `codex`
- 执行 / 提交 / 交付阶段 -> `claude`
- 在 `EXECUTE` 阶段按任务类型细分：
  - `implementation`、`verification` -> `claude`
  - `design`、`research` -> `codex`
  - `documentation`、`configuration` -> `claude`

- 角色来源优先级：
  1. 环境变量 `FUSION_AGENT_ROLE`
  2. `.fusion/task_plan.md` 中任务元数据 `- Owner:`（或 `- Role:`）
  3. 阶段默认角色（`planner`/`coder`/`reviewer`）
- `.fusion/task_plan.md` 中的评审门禁元数据：
  - `- Review-Status: none|pending|approved|changes_requested`
- 角色到后端映射：
  - `planner` -> `codex`
  - `coder` -> `claude`
  - `reviewer` -> `codex`
- 会话按角色隔离保存，例如 `planner_codex_session`、`coder_claude_session`（同时保持旧会话键兼容）。

## 快速开始

```bash
/fusion "实现用户认证系统"
```

Fusion 会自动：

1. 分析代码库上下文
2. 拆分为可执行的子任务
3. 按 TDD 流程逐个实现
4. 自动 commit 每个完成的任务
5. 最终汇报结果

## 命令

| 命令                   | 描述                 |
| ---------------------- | -------------------- |
| `/fusion "<目标>"`     | 启动自主工作流       |
| `/fusion status`       | 查看当前进度         |
| `/fusion resume`       | 恢复中断的任务       |
| `/fusion pause`        | 暂停执行             |
| `/fusion cancel`       | 取消任务             |
| `/fusion logs`         | 查看详细日志         |
| `/fusion achievements` | 查看成就汇总与排行榜 |

脚本模式：`bash scripts/fusion-achievements.sh`

## 工作流

```
UNDERSTAND → INITIALIZE → ANALYZE → DECOMPOSE → EXECUTE → VERIFY → REVIEW → COMMIT → DELIVER
                                         ↓
                                  TDD 循环:
                                  RED → GREEN → REFACTOR
```

## 文档

| 文档                                                                     | 描述                                                               |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------ |
| [SKILL.md](SKILL.md)                                                     | 主技能文件                                                         |
| [EXECUTION_PROTOCOL.md](EXECUTION_PROTOCOL.md)                           | 详细执行协议                                                       |
| [PARALLEL_EXECUTION.md](PARALLEL_EXECUTION.md)                           | 并行执行策略                                                       |
| [SESSION_RECOVERY.md](SESSION_RECOVERY.md)                               | 会话恢复机制                                                       |
| [DESIGN.md](DESIGN.md)                                                   | 设计文档                                                           |
| [docs/HOOKS_SETUP.md](docs/HOOKS_SETUP.md)                               | Hook 挂载说明                                                      |
| [docs/E2E_EXAMPLE.md](docs/E2E_EXAMPLE.md)                               | 端到端执行示例                                                     |
| [`.claude/settings.example.json`](.claude/settings.example.json)         | 受版本控制的 Hook 模板；复制后生成宿主本地 `.claude/settings.json` |
| [docs/V3_GA_EXECUTION_ROADMAP.md](docs/V3_GA_EXECUTION_ROADMAP.md)       | 当前 v3 GA 执行路线图                                              |
| [rust/README.md](rust/README.md)                                         | Rust Bridge 当前使用说明                                           |
| [docs/RUST_FUSION_BRIDGE_ROADMAP.md](docs/RUST_FUSION_BRIDGE_ROADMAP.md) | 历史 Rust 二进制迁移路线图                                         |

## 文件结构

```
fafafa-skills-fusion/
├── SKILL.md                    # 主技能入口
├── EXECUTION_PROTOCOL.md       # 执行协议
├── PARALLEL_EXECUTION.md       # 并行执行
├── SESSION_RECOVERY.md         # 会话恢复
├── DESIGN.md                   # 设计文档
├── README.md                   # 本文件
├── .fusion/                    # 本地运行态工作区（不作为模板真源）
│   ├── task_plan.md
│   ├── progress.md
│   ├── findings.md
│   ├── sessions.json
│   └── config.yaml               # 生成后的工作区配置
├── examples/
│   └── root-session/
│       └── README.md           # 根目录运行产物布局示例
├── templates/                  # 文件模板
│   ├── task_plan.md
│   ├── progress.md
│   ├── findings.md
│   ├── sessions.json
│   └── config.yaml               # 受版本控制的配置模板
├── scripts/                    # 辅助脚本
│   ├── fusion-start.sh
│   ├── fusion-codeagent.sh
│   ├── fusion-init.sh
│   ├── fusion-status.sh
│   ├── fusion-resume.sh
│   ├── fusion-stop-guard.sh
│   ├── fusion-pause.sh
│   ├── fusion-cancel.sh
│   ├── fusion-logs.sh
│   ├── fusion-continue.sh
│   └── fusion-git.sh
└── prompts/                    # Codex/Claude prompts
    ├── decompose.md
    ├── tdd.md
    ├── error_recovery.md
    ├── code_review.md
    ├── commit_message.md
    └── two_phase_review.md
```

## 运行时目录

Fusion 在项目根目录创建 `.fusion/` 存储工作状态：

```
.fusion/
├── task_plan.md      # 当前任务计划
├── progress.md       # 进度时间线
├── findings.md       # 研究发现
├── sessions.json     # 会话状态
├── config.yaml       # 由模板生成的运行时配置
└── events.jsonl      # 事件溯源日志
```

## 仓库卫生约定

- 实时会话状态只应写入 `.fusion/`。
- 版本库内可复用的模板真源放在 `templates/`。
- 仓库根目录的 `task_plan.md`、`progress.md`、`findings.md` 不再视为 canonical 文件，并已加入忽略规则，避免误提交运行时产物。
- 如果需要查看一个受版本控制的示例布局，请参考 `examples/root-session/README.md`。

## 配置

先运行 `scripts/fusion-init.sh` 或 `fusion-bridge init`，从 `templates/config.yaml` 生成 `.fusion/config.yaml`，再按需修改生成后的配置文件。

编辑 `.fusion/config.yaml`：

```yaml
runtime:
  enabled: true
  compat_mode: true # 保留 Shell 兜底路径，便于排障
  engine: "rust" # rust 主线；live 路径不再提供其他 engine 选项
  version: "2.6.3"

backends:
  primary: codex
  fallback: claude

agents:
  enabled: false
  mode: single_orchestrator # 默认；如需 planner -> coder -> reviewer 交接可改为 role_handoff
  review_policy: high_risk
  explain_level: compact

backend_routing:
  phase_routing:
    EXECUTE: claude
    REVIEW: codex
  task_type_routing:
    implementation: claude
    verification: claude
    design: codex
    documentation: claude

understand:
  pass_threshold: 7
  require_confirmation: false
  max_questions: 2

execution:
  parallel: 2
  timeout: 7200000

scheduler:
  enabled: true
  max_parallel: 2
  fail_fast: false

safe_backlog:
  enabled: true
  trigger_no_progress_rounds: 3
  inject_on_task_exhausted: true
  max_tasks_per_run: 2
  allowed_categories: "quality,documentation,optimization"
  diversity_rotation: true
  novelty_window: 12
  backoff_enabled: true
  backoff_base_rounds: 1
  backoff_max_rounds: 32
  backoff_jitter: 0.2
  backoff_force_probe_rounds: 20

supervisor:
  enabled: false
  mode: "advisory"
  persona: "Guardian"
  trigger_no_progress_rounds: 2
  cadence_rounds: 2
  force_emit_rounds: 12
  max_suggestions: 2

tdd:
  enabled: true

git:
  enabled: true
  branch_prefix: "fusion/"
```

`agents.enabled: false` 仍是当前默认路径。启用后，`single_orchestrator` 会先按任务元数据规划一个“已就绪且无 writes 冲突”的 batch，并受 `execution.parallel` 与 `scheduler.max_parallel` 共同约束；同时把 batch 状态写入 `_runtime.agents`，但每次 codeagent 运行仍从配置的 primary backend 起跑。`role_handoff` 则是交棒模式：planner -> coder -> reviewer，且 reviewer 批准是需要评审任务的硬门禁。

`agents.explain_level` 只会增强现有命令面的 explain 信息：把策略原因镜像到 `_runtime.agents.policy`、`fusion status --json` 与人类可读的 `fusion status`，不会新增独立 explain 命令。启用 `role_handoff` 后，`fusion status --json` 还可能出现 `agent_collaboration_mode`、`agent_turn_role`、`agent_turn_task_id`、`agent_turn_kind`、`agent_pending_reviews`、`agent_blocked_handoff_reason` 这些协作摘要字段。

对于需要 reviewer 门禁的任务，`.fusion/task_plan.md` 是当前唯一 canonical 审批面：

- `Review-Status: none` 表示当前没有评审门禁。
- `Review-Status: pending` 表示实现已就绪，等待 reviewer 批准。
- `Review-Status: approved` 表示 reviewer 已通过该任务。
- `Review-Status: changes_requested` 表示 reviewer 要求修改，任务重新回到实现方。

完整推荐基线请参考 `templates/config.yaml`。

- `engine: "rust"` 是推荐默认值；当 `fusion-bridge` 可用时，hook/runtime 路径会优先走 Rust。
- 薄包装脚本只会自动发现已安装的 `fusion-bridge` 或本地 `rust/target/release/` 构建，不会静默回落到 `target/debug`。
- `FUSION_BRIDGE_DISABLE=1` 可让支持该模式的 hook/runtime 入口跳过 Rust bridge 以便排障；`fusion-status.sh`、`fusion-logs.sh`、`fusion-git.sh`、`fusion-achievements.sh`、`fusion-pause.sh`、`fusion-resume.sh`、`fusion-catchup.sh`、`fusion-cancel.sh`、`fusion-continue.sh` 这类薄包装现在都要求 `fusion-bridge` 可用，或在文档注明的场景下显式走 `cargo --release` 回退。
- live 控制路径已不再提供其他 runtime engine 这类运行时选择；旧 runtime/reference 层已从仓库移除。
- `compat_mode: true` 表示当 Rust bridge 被跳过或不可用时，hook 和非薄包装辅助路径会回退到 Shell 兜底路径；控制面薄包装脚本仍要求 `fusion-bridge`。
- CI 的 release 门禁运行在 `ubuntu-latest`，同时 workflow 已补充 `macos-latest` smoke job，以及 `windows-latest` 的 Git Bash smoke job，用于覆盖 shell helper、控制面薄包装、真实 hook 路径与 catchup 恢复包装的跨平台证据。
- 截至 2026-03-25，macOS 与 Windows (Git Bash) 已通过远端 CI promotion evidence 升级为已验证状态，对应 run 为 `23539348456`；详见 [docs/COMPATIBILITY.md](docs/COMPATIBILITY.md)。
- WSL 当前仍按 post-GA 证据跟踪，不是当前 GA 阻断项。

## Safe Backlog 托底（长期自治防停摆）

当主任务无法继续拆解、或者长期无进展时，Fusion 会自动注入低风险任务，避免循环停摆。

### 什么时候会触发

- 连续多轮无进展（`trigger_no_progress_rounds`）
- 任务池耗尽（`inject_on_task_exhausted: true`）

### 注入什么任务

- `quality`：测试覆盖与质量修复
- `documentation`：文档补全与说明更新
- `optimization`：低风险优化与性能整理

### 为什么不会变机械

- 类别轮转（`diversity_rotation`）避免重复同类任务
- 新颖窗口（`novelty_window`）避免短周期重复指纹
- 优先级评分（`priority_score`）按历史频次和类别价值排序

### 为什么不会无限注入

- 指数退避（`backoff_*`）控制注入频率
- 抖动（`backoff_jitter`）避免固定节奏重复
- 强制探测（`backoff_force_probe_rounds`）防止长期饥饿
- 出现真实进展时自动复位 backoff

### 如何观察托底效果

运行：

```bash
/fusion status
```

关注输出：

- `safe_backlog.last_added`
- `safe_backlog.last_injected_at`
- `safe_backlog.last_injected_at_iso`

并查看 `.fusion/events.jsonl` 中的 `SAFE_BACKLOG_INJECTED` 事件（含 `reason` 和 `stall_score`）。

## 虚拟监督官（可选增补）

Fusion 支持可选的虚拟监督官（默认关闭），用于在无进展时提供“像人一样”的提醒，但不接管主执行流程：

- 当前仅支持 `advisory` 建议模式
- 在无进展轮次达到阈值后输出建议
- 写入 `SUPERVISOR_ADVISORY` 事件到 `.fusion/events.jsonl`
- 不直接改动任务状态，真正托底仍由 safe backlog 执行

## 依赖自动修复（Dependency Auto-Heal）

Fusion 在关键路径上会先尝试自动处理依赖，再决定是否阻塞：

- 自动定位 `codeagent-wrapper`：
  - `CODEAGENT_WRAPPER_BIN` 显式路径
  - `PATH`
  - `./node_modules/.bin/codeagent-wrapper`
  - `~/.local/bin/codeagent-wrapper`
  - `~/.npm-global/bin/codeagent-wrapper`
- live 控制路径已不再做解释器自动探测；仓库内也不再保留旧 runtime/reference 文件
- 仍无法处理时，会写入 `.fusion/dependency_report.json`，给出可执行修复建议（可由用户或 agent 继续处理）
- 若 primary+fallback 后端调用都失败，会写入 `.fusion/backend_failure_report.json`，记录后端与错误上下文

可通过以下命令查看依赖阻塞摘要：

```bash
/fusion status
```

输出里会显示 `## Dependency Report`。
若后端连续失败，也会显示 `## Backend Failure Report`。

## Hook Doctor 快速修复

如果你发现 Hook 没有持续工作、会话异常结束，可先执行：

```bash
bash scripts/fusion-hook-doctor.sh --json --fix .
```

然后再执行一次健康检查：

```bash
bash scripts/fusion-hook-doctor.sh --json .
```

当输出 `result=ok` 且 `warn_count=0` 时，说明 Hook 挂载正常。

首次自动修复后，请在 Claude Code 中打开 `/hooks` 审核并确认变更，然后重开一次会话。

## Hook 调试可见性

如果你希望在 Claude Code 中直接看到 hook 是否触发，可开启调试：

```bash
# 开启 hook 调试（在当前项目持续生效）
touch .fusion/.hook_debug

# 关闭 hook 调试
rm -f .fusion/.hook_debug
```

开启后，hook 会输出类似以下 stderr 调试行：

- `[fusion][hook-debug][pretool] ...`
- `[fusion][hook-debug][posttool] ...`
- `[fusion][hook-debug][stop] ...`

可通过以下方式查看最近调试信息：

```bash
/fusion status
# 或
tail -n 50 .fusion/hook-debug.log
```

## 发布状态（2026-03-21）

- ✅ 启动入口：`scripts/fusion-start.sh`
- ✅ Shell Hook 入口：`scripts/fusion-pretool.sh`、`scripts/fusion-posttool.sh`、`scripts/fusion-stop-guard.sh`
- ✅ 状态双写：`events.jsonl` + `sessions.json`
- ✅ 默认配置生成：`scripts/fusion-init.sh` → `fusion-bridge init`（从 `templates/config.yaml` 生成 `.fusion/config.yaml`）
- ✅ UNDERSTAND 阶段当前由 Rust 启动路径最小推进，旧参考实现已移除
- ✅ codeagent-wrapper 闭环脚本：`scripts/fusion-codeagent.sh`
- ✅ 全量测试：运行 `cd rust && cargo test --release`

当前主线实现已经明确：

- Rust / `fusion-bridge`：主控制面与默认运行时
- Shell：thin wrapper / Hook 入口兼容层
- 旧 runtime/reference 层：已从仓库移除，不再作为 runtime 选择或并行主实现

运维类包装脚本也在继续向 Rust-only 收敛：

- `scripts/release-contract-audit.sh` 仅保留参数校验，实际执行委托给 `fusion-bridge audit`
- `scripts/fusion-hook-selfcheck.sh` 仅保留参数校验，实际执行委托给 `fusion-bridge selfcheck`
- 审计 live gate 现已收敛为 shell/Rust-only：当前由 shell 语法检查、machine-mode JSON smoke、wrapper smoke 与 release Rust 门禁组成；selfcheck 已改为直接走 Rust contract regression

仓库运行产物约定也已统一：

- 实时会话状态只放 `.fusion/`
- 版本库只保留 `templates/` 模板
- `rust/target/`、`rust/.cargo-codex/` 这类本地 Rust cache 属于机器生成状态，应继续保持忽略
- `.ace-tool/`、`.claude/settings.json`、`.claude/settings.local.json` 这类宿主本地设置同样不属于仓库结构证据
- `.claude/settings.example.json` 继续作为受版本控制的 Hook 模板；只有生成出来的 `.claude/settings.json` / `.claude/settings.local.json` 才属于宿主本地配置
- 如需提交演示态目录，请放到 `examples/`

维护者可进一步参考：`docs/REPO_HYGIENE.md`

本轮仓库收敛总结见：`docs/REPO_CONVERGENCE_SUMMARY_2026-03.md`

## 验证

```bash
# 全量 Rust 验证
cd rust
cargo test --release

# 关键契约套件
cargo test --release -p fusion-cli --test repo_contract
cargo test --release -p fusion-cli --test shell_contract
cargo test --release -p fusion-cli --test cli_smoke
```

## 3-Strike 错误协议

```
Strike 1: Codex 针对性修复
Strike 2: Codex 换实现方案
Strike 3: 降级到 Claude 本地
3 Strikes: 询问用户
```

## 融合来源

| 来源                | 融入特性                           |
| ------------------- | ---------------------------------- |
| codex skill         | HEREDOC 语法、SESSION_ID、并行执行 |
| planning-with-files | 文件持久化、3-Strike 协议          |
| subagent-driven     | 两阶段审查模式                     |
| superpowers TDD     | 红-绿-重构循环                     |
| ccg workflow        | 多阶段工作流（简化版）             |

## 与 ccg 的区别

| 特性        | ccg       | Fusion        |
| ----------- | --------- | ------------- |
| Gemini 依赖 | ❌ 必须   | ✅ 不需要     |
| 用户确认    | ❌ 每阶段 | ✅ 只在阻塞时 |
| 自动降级    | ❌ 无     | ✅ 3-Strike   |
| 会话恢复    | ⚠️ 手动   | ✅ 自动       |

## License

MIT

## 开源协作

- [Contributing Guide (EN)](CONTRIBUTING.md)
- [贡献指南 (ZH)](CONTRIBUTING.zh-CN.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md)

## 维护者与社区

- Maintainer：**dtamade**
- Studio：**fafafa studio**
- 邮箱：`dtamade@gmail.com`
- QQ 群：`685403987`

## CI 与发布契约门禁

Fusion 提供了 CI 门禁工作流：`.github/workflows/ci-contract-gates.yml`。

建议在合并前本地执行发布契约审计：

```bash
bash scripts/release-contract-audit.sh --dry-run
bash scripts/release-contract-audit.sh
```

常用参数：

- `--fast`：跳过 wrapper smoke，仅保留 machine-mode smoke
- `--skip-rust`：跳过 rust clippy/test/fmt 门禁

机器可读示例：

```bash
bash scripts/release-contract-audit.sh --dry-run --json --json-pretty --fast --skip-rust
bash scripts/ci-machine-mode-smoke.sh
fusion-bridge regression --list-suites --json
```

CI 还会为远端 `macos-latest` 与 `windows-latest` job 上传跨平台 smoke summary JSON；对应产物文件均为 `cross-platform-smoke-summary.json`，分别位于 `/tmp/cross-platform-smoke-macos/` 与 `/tmp/cross-platform-smoke-windows/`。

这些改动推到 GitHub 后，可运行 `bash scripts/ci-remote-evidence.sh --repo dtamade/fafafa-skills-fusion --branch main --json --artifacts-dir /tmp/remote-ci-evidence` 抓取最新远端 promotion evidence，并写出 `remote-ci-evidence-summary.json`。

机器 JSON 字段、产物路径和 CLI 精确契约请统一参考 [docs/CLI_CONTRACT_MATRIX.md](docs/CLI_CONTRACT_MATRIX.md)。

如果活文档或仓库/runtime 契约发生变化，请同步更新 `rust/crates/fusion-cli/tests/repo_contract.rs` 以及受影响的契约文档。

Rust 侧本地验证也统一采用 release 策略：

```bash
cd rust
cargo test --release
```
