# Fusion Skill - 设计与实现文档

## 概述

Fusion 是一个自主工作流 Skill，融合多个优秀方案的精华，实现"给目标后自主执行"。

## 目录结构

```
fafafa-skills-fusion/
├── SKILL.md              # 主技能文件（Claude Code 入口）
├── DESIGN.md             # 设计文档（本文件）
├── templates/            # 文件模板
│   ├── task_plan.md      # 任务计划模板
│   ├── progress.md       # 进度日志模板
│   ├── findings.md       # 发现记录模板
│   └── config.yaml       # 配置模板
├── scripts/              # 辅助脚本
│   ├── fusion-init.sh       # 初始化 .fusion 目录
│   ├── fusion-status.sh     # 显示状态
│   ├── fusion-stop-guard.sh # Stop Hook (核心循环控制)
│   ├── fusion-pause.sh      # 暂停工作流
│   ├── fusion-cancel.sh     # 取消工作流
│   ├── fusion-resume.sh     # 恢复工作流
│   ├── fusion-logs.sh       # 查看日志
│   ├── fusion-continue.sh   # PostToolUse 继续提醒
│   └── fusion-git.sh        # Git 操作
└── prompts/              # Codex/Claude prompts
    ├── decompose.md      # 任务分解 prompt
    └── tdd.md            # TDD 实现 prompt
```

## 运行时目录 (.fusion/)

Fusion 在项目根目录创建 `.fusion/` 目录存储工作状态：

```
.fusion/
├── task_plan.md      # 当前任务计划和状态
├── progress.md       # 进度时间线
├── findings.md       # 研究和决策记录
├── sessions.json     # Codex SESSION_ID 存储
└── config.yaml       # 运行时配置
```
