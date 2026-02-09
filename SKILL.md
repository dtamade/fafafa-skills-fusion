---
name: fusion
version: "2.0.0"
description: |
  自主工作流 Skill - 给目标后自主执行，只在必要时打扰用户。
  融合 Codex 执行、TDD 流程、Git 集成、3-Strike 降级策略。
  v2: Attention 注入 + 进度监控 + 会话恢复 + LoopGuardian 防死循环。
user-invocable: true
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
  - WebFetch
  - WebSearch
  - Task
  - TaskOutput
  - AskUserQuestion
  - mcp__ace-tool__search_context
  - mcp__ace-tool__enhance_prompt
hooks:
  PreToolUse:
    - matcher: "Write|Edit|Bash|Read|Glob|Grep"
      hooks:
        # Attention injection: keeps goal/task/progress in Claude's context window
        - type: command
          command: "bash ${CLAUDE_PLUGIN_ROOT}/scripts/fusion-pretool.sh"
  PostToolUse:
    - matcher: "Write|Edit"
      hooks:
        # Progress monitor: detects task status changes and provides guidance
        - type: command
          command: "bash ${CLAUDE_PLUGIN_ROOT}/scripts/fusion-posttool.sh"
  Stop:
    - hooks:
        # Loop engine with LoopGuardian anti-deadloop protection
        - type: command
          command: "bash ${CLAUDE_PLUGIN_ROOT}/scripts/fusion-stop-guard.sh"
---

# Fusion - 自主工作流

## ⚠️ 执行循环协议 (CRITICAL)

**当 /fusion 被激活后，你必须进入自主执行循环，直到所有任务完成。**

### 循环规则

```
LOOP:
  1. 读取 .fusion/task_plan.md 获取下一个待执行任务
  2. 如果没有待执行任务 → 进入 DELIVER 阶段 → 结束循环
  3. 执行当前任务（TDD 流程）
  4. 更新 task_plan.md 和 progress.md
  5. 返回步骤 1 继续下一个任务
```

### 关键约束

- **永远不要停下来问用户"是否继续"** - 除非遇到阻塞性问题
- **每个工具调用后立即检查进度** - 读取 task_plan.md 决定下一步
- **任务失败不是停止的理由** - 应用 3-Strike 协议，降级后继续
- **只有以下情况才询问用户**:
  - 3-Strike 全部失败
  - 需要用户做决策（如多个实现方案选择）
  - 遇到超出范围的问题

### 执行状态检查

每次工具调用后，执行以下检查：

```python
# 伪代码
def check_and_continue():
    tasks = read(".fusion/task_plan.md")

    pending = [t for t in tasks if t.status in ["PENDING", "IN_PROGRESS"]]

    if not pending:
        return DELIVER()  # 所有任务完成，交付

    current_task = pending[0]
    return execute_task(current_task)  # 继续执行
```

---

## 核心理念

**"给目标，自动执行，只在必要时打扰"**

融合多个优秀方案的精华：
- **Codex** - 深度代码分析和任务执行
- **TDD** - 强制测试驱动开发
- **Git** - 自动分支和提交管理
- **3-Strike** - 智能错误恢复和降级

---

## 快速开始

```bash
/fusion "实现用户认证系统"
```

就这么简单。Fusion 会自动：
1. 分析目标和代码库
2. 拆分为可执行的子任务
3. 按 TDD 流程逐个实现
4. 自动 commit 每个完成的任务
5. 最终汇报结果

---

## 命令参考

### 主命令

```bash
/fusion "<目标描述>"
```

启动自主工作流。Fusion 会：
- 调用 Codex 分析代码库并拆分任务
- 按依赖关系调度执行
- 持续写入进度到 `.fusion/progress.md`
- 只在阻塞时询问用户

### 子命令

| 命令 | 描述 |
|------|------|
| `/fusion status` | 查看当前任务状态和进度 |
| `/fusion resume` | 恢复上次中断的任务 |
| `/fusion pause` | 暂停当前执行 |
| `/fusion cancel` | 取消当前任务 |
| `/fusion logs` | 查看详细执行日志 |

### 选项

```bash
--backend codex|claude   # 指定后端 (默认: codex)
--parallel N             # 最大并行任务数 (默认: 2)
--dry-run                # 只生成计划不执行
--no-tdd                 # 跳过 TDD 流程
--no-git                 # 跳过 Git 集成
```

---

## 工作流

### 9 阶段流程

```
┌─────────────────────────────────────────────────────────────┐
│                     Fusion Workflow                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  0. UNDERSTAND  - 理解目标，评估清晰度，必要时追问 (新)     │
│  1. INITIALIZE  - 初始化 .fusion 目录和文件                 │
│  2. ANALYZE     - 分析目标，理解代码库上下文                │
│  3. DECOMPOSE   - Codex 拆分为可执行的原子任务              │
│  4. EXECUTE     - 对每个任务执行 TDD 循环:                  │
│                   RED → GREEN → REFACTOR                     │
│  5. VERIFY      - 运行完整测试套件                          │
│  6. REVIEW      - 代码质量自审查                            │
│  7. COMMIT      - Git 提交所有变更                          │
│  8. DELIVER     - 最终汇报                                  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Phase 0: UNDERSTAND (理解确认)

**目的**：在开始执行前确保 AI 正确理解用户意图

```
用户输入 /fusion "目标"
      │
      ▼
┌─────────────────────────────────────────────────────────────┐
│  1. 静默扫描 (5-10秒)                                       │
│     • 检测技术栈 (package.json/go.mod/pyproject.toml)       │
│     • 检测项目结构 (src/, tests/)                           │
│     • 语义搜索相关文件 (ace-tool)                           │
│                                                              │
│  2. 目标评分 (0-10)                                         │
│     • 目标明确性 (0-3)                                      │
│     • 预期结果 (0-3)                                        │
│     • 边界范围 (0-2)                                        │
│     • 约束条件 (0-2)                                        │
│                                                              │
│  3. 决策                                                     │
│     • ≥7 分 → 展示理解摘要，等待确认                        │
│     • <7 分 → 追问补充（最多 2 轮）                         │
│                                                              │
│  4. 用户响应                                                 │
│     • "ok" → 进入 INITIALIZE                                │
│     • "改动" → 调整后重新确认                               │
│     • "取消" → 退出                                         │
└─────────────────────────────────────────────────────────────┘
```

**理解确认卡片示例**：

```
┌─────────────────────────────────────────────────┐
│ 📋 Fusion 理解确认                              │
├─────────────────────────────────────────────────┤
│ 目标：实现用户认证系统                          │
│                                                 │
│ 上下文：                                        │
│ • 技术栈：Express + TypeScript + PostgreSQL    │
│ • 测试框架：Jest                                │
│ • 相关文件：src/middleware/auth.ts              │
│                                                 │
│ 计划范围：                                      │
│ • 用户模型 + 注册/登录 API + JWT 中间件        │
│ • 预计 5 个任务，约 20-30 分钟                 │
│                                                 │
│ 假设 ⚠️：                                       │
│ • 认证方式：JWT（如需 Session 请说明）          │
│ • 密码哈希：bcrypt                              │
└─────────────────────────────────────────────────┘

确认开始？(ok/修改/取消)
```

**追问示例**（一次一个问题，优先多选）：

```
● 认证方式？
  [1] JWT (推荐 - 项目已有 jsonwebtoken 依赖)
  [2] Session
  [3] OAuth2
  [4] 其他
```

**跳过机制**：
```bash
/fusion --force "目标"   # 跳过 UNDERSTAND，直接执行
/fusion --yolo "目标"    # 同上
```

详细提示词参考：[prompts/understand.md](prompts/understand.md)

### TDD 循环 (每个实现任务)

```
1. 分析需求 → 确定测试用例
2. 写失败测试 (RED)
3. 运行确认失败
4. 最小代码通过 (GREEN)
5. 运行确认通过
6. 重构优化 (REFACTOR)
7. 提交
```

---

## 用户打扰规则

| 场景 | 行为 |
|------|------|
| 正常执行 | 静默写入 `.fusion/progress.md` |
| 阶段完成 | 静默（完全自主模式）|
| 可恢复错误 | 自动重试/降级 |
| **需要决策** | 询问用户 |
| **3次连续失败** | 询问用户 |
| 最终完成 | 详细汇报 |

---

## 3-Strike 错误协议 + 降级

```
┌───────────────────────────────────────────────────────────┐
│                  3-Strike 错误协议                         │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Strike 1: Codex 针对性修复                               │
│  └─ 分析错误原因，应用修复，重试                          │
│                                                           │
│  Strike 2: Codex 换实现方案                               │
│  └─ 不重复已失败的路径，尝试替代方案                      │
│                                                           │
│  Strike 3: 降级到 Claude 本地                             │
│  └─ 使用 Claude 直接执行，跳过 Codex                      │
│                                                           │
│  3 Strikes 后: 升级给用户                                 │
│  └─ 详细说明尝试过什么，请求指导                          │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

---

## 后端路由

| 任务类型 | 后端 | 原因 |
|----------|------|------|
| 任务分解 | Codex | 需要深度代码库理解 |
| 复杂分析 | Codex | 擅长理解依赖和架构 |
| 算法实现 | Codex | 强大的逻辑推理 |
| 重构 | Codex | 需要跟踪多文件依赖 |
| 简单编辑 | Claude | 快速响应 |
| 文档生成 | Claude | 擅长自然语言 |
| 配置修改 | Claude | 低复杂度任务 |

---

## 进度文件 (.fusion/ 目录)

```
.fusion/
├── task_plan.md      # 任务计划（阶段、状态、依赖）
├── progress.md       # 进度日志（时间线）
├── findings.md       # 发现记录（研究、决策）
├── sessions.json     # 会话存储（Codex SESSION_ID）
└── config.yaml       # 运行时配置
```

### 查看进度

```bash
# 方式 1: 使用命令
/fusion status

# 方式 2: 直接读取
cat .fusion/progress.md

# 方式 3: 监听变化
tail -f .fusion/progress.md
```

---

## Git 集成

- **任务开始**: `git checkout -b fusion/<goal-slug>`
- **任务完成**: `git commit -m "<conventional commit>"`
- **全部完成**: 汇报分支状态，建议 merge/PR

### Commit Message 格式

遵循 Conventional Commits:
```
feat: add user authentication endpoint
fix: resolve password hashing issue
test: add integration tests for auth module
refactor: extract validation logic
docs: update API documentation
```

---

## 任务自动拆分

Codex 分析代码库后生成任务列表，特点：
- **粒度**: 每任务 5-15 分钟
- **依赖**: 自动识别任务间依赖
- **并行**: 无依赖任务可并行执行

### 示例

**输入**: "实现用户认证系统"

**拆分输出**:
```yaml
tasks:
  - id: auth_api_design
    type: design
    dependencies: []

  - id: db_user_schema
    type: implementation
    dependencies: []

  - id: auth_register
    type: implementation
    dependencies: [auth_api_design, db_user_schema]

  - id: auth_login
    type: implementation
    dependencies: [auth_api_design, db_user_schema]

  - id: jwt_middleware
    type: implementation
    dependencies: [auth_login]

  - id: integration_tests
    type: verification
    dependencies: [auth_register, auth_login, jwt_middleware]
```

---

## 与现有工具的关系

### 内部调用

Fusion 内部使用:
- `codeagent-wrapper` - 多后端 AI 代码执行（支持 Codex、Claude 等）

### 借鉴来源

| 来源 | 融入特性 |
|------|----------|
| codex skill | HEREDOC 语法、SESSION_ID、并行执行 |
| planning-with-files | 文件持久化、3-Strike 协议 |
| subagent-driven | 两阶段审查模式 |
| superpowers TDD | 红-绿-重构循环 |

---

## 故障排除

### Codex 超时
- 默认超时: 2 小时
- 触发降级到 Claude 本地

### 任务失败
- 查看 `.fusion/progress.md` 中的错误日志
- 使用 `/fusion logs` 查看详细信息
- 3-Strike 后会自动询问用户

### 会话恢复
- `/clear` 后运行 `/fusion resume`
- 读取 `.fusion/sessions.json` 恢复 Codex 会话

---

## 配置

### .fusion/config.yaml

```yaml
# 后端配置
backends:
  primary: codex
  fallback: claude

# 执行配置
execution:
  parallel: 2
  timeout: 7200000  # 2 小时

# TDD 配置
tdd:
  enabled: true
  test_command: "npm test"

# Git 配置
git:
  enabled: true
  branch_prefix: "fusion/"
  auto_commit: true
```

---

## 执行协议

详细的执行流程请参考 [EXECUTION_PROTOCOL.md](EXECUTION_PROTOCOL.md)

### 后端调用规范

**统一使用 codeagent-wrapper**（支持多后端，自动降级）：

```bash
# 主调用（HEREDOC 语法避免 shell 转义）
codeagent-wrapper --backend codex - "$PWD" <<'EOF'
<task content here>
EOF

# 恢复会话
codeagent-wrapper --backend codex resume <SESSION_ID> - "$PWD" <<'EOF'
<task content>
EOF

# 降级到 Claude
codeagent-wrapper --backend claude - "$PWD" <<'EOF'
<task content>
EOF
```

**Bash 工具参数**：
- `timeout: 7200000` (固定 2 小时)
- `description: "<简短描述>"`

### 会话复用

每次后端调用返回 SESSION_ID，后续调用使用 `resume`：

```bash
# 首次调用
codeagent-wrapper --backend codex - "$PWD" <<'EOF'
analyze codebase
EOF
# 返回 SESSION_ID: 019xxx

# 后续调用（复用上下文）
codeagent-wrapper --backend codex resume 019xxx - "$PWD" <<'EOF'
implement based on analysis
EOF
```

### 降级执行

当 Codex 失败时，降级到 Claude 本地：

```markdown
[CODEX_FALLBACK] Task: <task_id>
Reason: <failure reason>
Action: Executing with Claude directly
```

然后直接使用 Edit/Write 工具完成任务。

---

## 并行执行

详细的并行策略请参考 [PARALLEL_EXECUTION.md](PARALLEL_EXECUTION.md)

### 基本原理

- 无依赖的任务可以并行执行
- 有依赖的任务等待依赖完成后执行
- 最大并行度由 `config.yaml` 中的 `parallel` 控制

### 依赖示例

```yaml
tasks:
  - id: A, dependencies: []      # 第一批
  - id: B, dependencies: []      # 第一批 (与 A 并行)
  - id: C, dependencies: [A, B]  # 第二批 (等待 A, B)
```

执行顺序：
```
Batch 1: [A, B] (并行)
Batch 2: [C]    (等待 Batch 1 完成)
```

---

## Git 集成

### 自动分支

工作流开始时自动创建分支：
```bash
git checkout -b fusion/<goal-slug>
```

### 自动提交

每个任务完成后自动提交：
```bash
git add -A
git commit -m "feat(<scope>): <description>"
```

### 脚本

使用 `scripts/fusion-git.sh` 进行 Git 操作：
```bash
# 创建分支
./scripts/fusion-git.sh create-branch "auth-system"

# 提交变更
./scripts/fusion-git.sh commit "feat(auth): add login endpoint"

# 查看状态
./scripts/fusion-git.sh status
```

---

## 内部实现细节

### 文件引用

本 Skill 使用以下内部文件：

| 文件 | 用途 |
|------|------|
| `EXECUTION_PROTOCOL.md` | 详细执行流程 |
| `PARALLEL_EXECUTION.md` | 并行执行策略 |
| `prompts/decompose.md` | Codex 任务分解 prompt |
| `prompts/tdd.md` | TDD 实现 prompt |
| `prompts/error_recovery.md` | 错误恢复 prompt |
| `prompts/code_review.md` | 代码审查 prompt |
| `prompts/commit_message.md` | Commit 消息生成 |
| `templates/*.md` | 文件模板 |
| `scripts/*.sh` | 辅助脚本 |
