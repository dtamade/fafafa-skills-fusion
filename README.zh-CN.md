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
- 🧠 **UNDERSTAND 自动执行器** - 启动即评分、写入假设并自动推进阶段
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
  2. `task_plan.md` 中任务元数据 `- Owner:`（或 `- Role:`）
  3. 阶段默认角色（`planner`/`coder`/`reviewer`）
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

| 命令 | 描述 |
|------|------|
| `/fusion "<目标>"` | 启动自主工作流 |
| `/fusion status` | 查看当前进度 |
| `/fusion resume` | 恢复中断的任务 |
| `/fusion pause` | 暂停执行 |
| `/fusion cancel` | 取消任务 |
| `/fusion logs` | 查看详细日志 |
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

| 文档 | 描述 |
|------|------|
| [SKILL.md](SKILL.md) | 主技能文件 |
| [EXECUTION_PROTOCOL.md](EXECUTION_PROTOCOL.md) | 详细执行协议 |
| [PARALLEL_EXECUTION.md](PARALLEL_EXECUTION.md) | 并行执行策略 |
| [SESSION_RECOVERY.md](SESSION_RECOVERY.md) | 会话恢复机制 |
| [DESIGN.md](DESIGN.md) | 设计文档 |
| [docs/HOOKS_SETUP.md](docs/HOOKS_SETUP.md) | Hook 挂载说明 |
| [docs/E2E_EXAMPLE.md](docs/E2E_EXAMPLE.md) | 端到端执行示例 |
| [`.claude/settings.example.json`](.claude/settings.example.json) | 标准 Hook 配置模板 |
| [rust/README.md](rust/README.md) | Rust Bridge MVP 使用说明 |
| [docs/RUST_FUSION_BRIDGE_ROADMAP.md](docs/RUST_FUSION_BRIDGE_ROADMAP.md) | Rust 二进制迁移路线图 |

## 文件结构

```
fafafa-skills-fusion/
├── SKILL.md                    # 主技能入口
├── EXECUTION_PROTOCOL.md       # 执行协议
├── PARALLEL_EXECUTION.md       # 并行执行
├── SESSION_RECOVERY.md         # 会话恢复
├── DESIGN.md                   # 设计文档
├── README.md                   # 本文件
├── templates/                  # 文件模板
│   ├── task_plan.md
│   ├── progress.md
│   ├── findings.md
│   ├── sessions.json
│   └── config.yaml
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
├── config.yaml       # 运行时配置
└── events.jsonl      # 事件溯源日志
```

## 配置

编辑 `.fusion/config.yaml`：

```yaml
runtime:
  enabled: true
  compat_mode: true
  engine: "python"  # python | rust
  version: "2.6.3"

backends:
  primary: codex
  fallback: claude

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
  enabled: false
  max_parallel: 2

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
- 自动识别 Python：优先 `python3`，回退 `python`
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

## 发布状态（2026-02-10）

- ✅ 启动入口：`scripts/fusion-start.sh`
- ✅ Shell/Python 桥接：`scripts/fusion-pretool.sh`、`scripts/fusion-posttool.sh`、`scripts/fusion-stop-guard.sh`
- ✅ 状态双写：`events.jsonl` + `sessions.json`
- ✅ runtime 默认启用：`scripts/fusion-init.sh`
- ✅ UNDERSTAND 自动执行：`scripts/runtime/understand.py`
- ✅ codeagent-wrapper 闭环脚本：`scripts/fusion-codeagent.sh`
- ✅ 调度器自动接线：`scripts/runtime/kernel.py` `create_kernel()`
- ✅ 全量测试：运行 `pytest -q` 查看最新通过数

## 验证

```bash
# 全量单元测试
pytest -q

# 关键脚本回归（Hook 路径 / UNDERSTAND / codeagent / status）
pytest -q \
  scripts/runtime/tests/test_hook_shell_runtime_path.py \
  scripts/runtime/tests/test_understand.py \
  scripts/runtime/tests/test_fusion_codeagent_script.py \
  scripts/runtime/tests/test_fusion_status_script.py
```

## 3-Strike 错误协议

```
Strike 1: Codex 针对性修复
Strike 2: Codex 换实现方案
Strike 3: 降级到 Claude 本地
3 Strikes: 询问用户
```

## 融合来源

| 来源 | 融入特性 |
|------|----------|
| codex skill | HEREDOC 语法、SESSION_ID、并行执行 |
| planning-with-files | 文件持久化、3-Strike 协议 |
| subagent-driven | 两阶段审查模式 |
| superpowers TDD | 红-绿-重构循环 |
| ccg workflow | 多阶段工作流（简化版）|

## 与 ccg 的区别

| 特性 | ccg | Fusion |
|------|-----|--------|
| Gemini 依赖 | ❌ 必须 | ✅ 不需要 |
| 用户确认 | ❌ 每阶段 | ✅ 只在阻塞时 |
| 自动降级 | ❌ 无 | ✅ 3-Strike |
| 会话恢复 | ⚠️ 手动 | ✅ 自动 |

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
- `--fast`：跳过全量 `pytest -q`
- `--skip-rust`：跳过 rust clippy/fmt
- `--skip-python`：跳过 pytest 门禁

机器可读示例：

```bash
bash scripts/release-contract-audit.sh --dry-run --json --json-pretty --fast --skip-rust
python3 scripts/runtime/regression_runner.py --list-suites --json
```

机器 JSON 字段要点：
- release audit payload：`schema_version`、`step_rate_basis`、`command_rate_basis`
- runner contract payload：`schema_version`、`rate_basis`（应等于 `total_scenarios`）
- 分母语义：`step_rate_basis=total_steps`、`command_rate_basis=total_commands`、`rate_basis=total_scenarios`
- 当前 schema 契约：`schema_version=v1`

CI 机器产物示例：
- `/tmp/release-audit-dry-run.json`
- `/tmp/runner-suites.json`
- `/tmp/runner-contract.json`
