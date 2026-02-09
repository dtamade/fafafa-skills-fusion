# Fusion Skill

**自主工作流 Skill** - 给目标后自主执行，只在必要时打扰用户。

## 特性

- 🤖 **自主执行** - 给定目标后自动完成，无需频繁确认
- 🧪 **TDD 驱动** - 强制测试驱动开发流程
- 🔄 **智能降级** - 3-Strike 错误协议，自动降级到备用方案
- 📊 **进度追踪** - 持久化进度到文件，随时可恢复
- 🌳 **Git 集成** - 自动分支管理和提交
- ⚡ **并行执行** - 独立任务并行处理

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

## 工作流

```
INITIALIZE → ANALYZE → DECOMPOSE → EXECUTE → VERIFY → REVIEW → COMMIT → DELIVER
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
└── config.yaml       # 运行时配置
```

## 配置

编辑 `.fusion/config.yaml`：

```yaml
backends:
  primary: codex
  fallback: claude

execution:
  parallel: 2
  timeout: 7200000

tdd:
  enabled: true

git:
  enabled: true
  branch_prefix: "fusion/"
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
