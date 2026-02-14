# Fusion Execution Protocol

当用户调用 `/fusion "<目标>"` 时，严格遵循此协议执行。

---

## 命令规范

### 后端调用工具

**统一使用 `codeagent-wrapper`**（支持多后端，按 phase/task-type 路由，失败自动降级）：

```bash
# 主调用（由路由决定）
codeagent-wrapper --backend <ROUTED_BACKEND> - "$PWD" <<'EOF'
<task content>
EOF

# 恢复会话（沿用当前任务路由）
codeagent-wrapper --backend <ROUTED_BACKEND> resume <SESSION_ID> - "$PWD" <<'EOF'
<task content>
EOF

# 备用后端（主后端失败时）
codeagent-wrapper --backend <FALLBACK_BACKEND> - "$PWD" <<'EOF'
<task content>
EOF
```

**Bash 调用参数**：
- `timeout: 7200000` (固定 2 小时)
- `description: "<简短描述>"`

### 任务类型与执行流程

| 任务类型 | 执行流程 | 说明 |
|----------|----------|------|
| `implementation` | 完整 TDD (RED→GREEN→REFACTOR) | 需要写测试的功能实现 |
| `verification` | 完整 TDD | 测试相关任务 |
| `design` | 直接执行 | API 设计、架构设计 |
| `documentation` | 直接执行 | 文档生成 |
| `configuration` | 直接执行 | 配置修改 |
| `research` | 直接执行 | 代码库研究 |

### 默认路由（v2.6.3）

| 路由维度 | 默认 `codex` | 默认 `claude` |
|----------|---------------|----------------|
| 阶段路由 | UNDERSTAND / INITIALIZE / ANALYZE / DECOMPOSE / VERIFY / REVIEW | EXECUTE / COMMIT / DELIVER |
| 任务类型路由（EXECUTE） | design / research | implementation / verification / documentation / configuration |

可在 `.fusion/config.yaml` 的 `backend_routing` 中覆盖。

---

## Phase 0: UNDERSTAND (理解确认)

**核心理念**：先理解后执行，降低目标漂移，但默认不打断自治循环。

### 0.1 检查跳过标志

```python
# 伪代码
if "--force" in args or "--yolo" in args:
    log("⚠️ 跳过理解确认（--force）")
    goto Phase_1_INITIALIZE
```

### 0.2 静默扫描项目

```python
context = {
    "tech_stack": detect_tech_stack(),
    "test_framework": detect_test_framework(),
    "project_structure": scan_directories(),
}
```

### 0.3 目标评分

评分维度（总分 0-10）：

- 目标明确性 (0-3)
- 预期结果 (0-3)
- 边界范围 (0-2)
- 约束条件 (0-2)

### 0.4 阈值决策（与配置一致）

| 条件 | 默认行为 (`require_confirmation=false`) | 严格模式 (`require_confirmation=true`) |
|------|------------------------------------------|----------------------------------------|
| `score >= pass_threshold` | 自动推进到 `INITIALIZE` | 自动推进到 `INITIALIZE` |
| `score < pass_threshold` | 记录缺失项与假设后继续 | 保持在 `UNDERSTAND`，等待澄清 |

### 0.5 配置项

```yaml
understand:
  pass_threshold: 7
  require_confirmation: false
  max_questions: 2
```

### 0.6 输出与持久化

- 将理解摘要、缺失项、假设写入 `.fusion/findings.md`
- 当满足推进条件时触发 `UNDERSTAND_DONE`
- 若严格模式低分阻塞，输出澄清提示并保持 `UNDERSTAND`

## Phase 1: INITIALIZE (初始化)

### 1.1 创建工作目录

```bash
# 初始化 .fusion 目录
mkdir -p .fusion
```

### 1.2 初始化文件

创建以下文件（使用 templates/ 中的模板）：
- `.fusion/task_plan.md` - 任务计划
- `.fusion/progress.md` - 进度日志
- `.fusion/findings.md` - 发现记录
- `.fusion/sessions.json` - 会话存储
- `.fusion/config.yaml` - 配置

### 1.3 记录开始

在 `progress.md` 中记录：
```markdown
| <timestamp> | INIT | Workflow started | OK | Goal: <用户目标> |
```

---

## Phase 2: ANALYZE (分析)

### 2.1 收集代码库上下文

使用 MCP 工具获取相关上下文：

```
mcp__ace-tool__search_context({
  query: "<基于目标的语义查询>",
  project_root_path: "$PWD"
})
```

### 2.2 识别关键文件

- 入口文件
- 相关模块
- 测试目录结构
- 配置文件

### 2.3 更新 findings.md

记录分析结果到 `.fusion/findings.md`

---

## Phase 3: DECOMPOSE (任务分解)

### 3.1 调用后端进行任务分解

**首选 Codex 后端**（复杂分析能力强）：

```bash
codeagent-wrapper --backend codex - "$PWD" <<'EOF'
ROLE_FILE: <prompts/decompose.md 的内容>

## Goal
<用户目标>

## Codebase Context
<Phase 2 收集的上下文>

## Output
生成 YAML 格式的任务列表，遵循 prompts/decompose.md 中的规范。
每个任务必须指定:
- type: implementation|verification|design|documentation|configuration|research
- owner: planner|coder|reviewer
EOF
```

### 3.1.1 DECOMPOSE 降级策略

如果 Codex 调用失败（超时或错误），降级到 Claude 本地：

```markdown
[DECOMPOSE_FALLBACK] Codex unavailable
Reason: <timeout/error>
Action: Using Claude local for task decomposition
```

然后 Claude 直接分析代码库并生成任务列表（无需调用外部后端）。

### 3.2 解析后端输出

从后端响应中提取：
1. YAML 任务列表（包含每个任务的 type）
2. SESSION_ID（保存到 sessions.json）

### 3.3 更新 task_plan.md

将解析的任务写入 `.fusion/task_plan.md`

### 3.4 记录进度

```markdown
| <timestamp> | DECOMPOSE | Created N tasks | OK | Tasks: task1, task2, ... |
```

---

## Phase 4: EXECUTE (执行)

### 4.1 任务调度

按依赖顺序执行任务：
1. 无依赖任务可并行执行（最多 parallel 个）
2. 有依赖任务等待依赖完成后执行

### 4.2 单任务执行流程

根据任务类型选择执行流程：

#### 4.2.0 检查任务类型

```python
# 伪代码
if task.type in ['implementation', 'verification']:
    execute_tdd_flow(task)  # 完整 TDD 流程
else:
    execute_direct_flow(task)  # 直接执行
```

#### 4.2.1 直接执行流程 (design/documentation/configuration/research)

```bash
codeagent-wrapper --backend <ROUTED_BACKEND> resume <SESSION_ID> - "$PWD" <<'EOF'
## Task: <task_id>
## Type: <task_type>

<task description>

要求：
1. 完成指定任务
2. 输出相关文件变更

输出：
- 变更的文件路径和内容
- 执行结果
EOF
```

#### 4.2.2 TDD 流程 (implementation/verification)

##### TDD - RED (写失败测试)

```bash
codeagent-wrapper --backend <ROUTED_BACKEND> resume <SESSION_ID> - "$PWD" <<'EOF'
## Task: <task_id>
## Phase: RED (Write Failing Test)

<task description>

要求：
1. 分析需求，确定测试用例
2. 编写测试代码（测试应该会失败）
3. 运行测试确认失败

输出：
- 测试文件路径和内容
- 测试运行结果（应显示 FAIL）
EOF
```

#### 4.2.2 TDD - GREEN (最小实现)

```bash
codeagent-wrapper --backend <ROUTED_BACKEND> resume <SESSION_ID> - "$PWD" <<'EOF'
## Task: <task_id>
## Phase: GREEN (Minimal Implementation)

要求：
1. 编写最小代码使测试通过
2. 不要添加额外功能
3. 运行测试确认通过

输出：
- 实现文件路径和内容
- 测试运行结果（应显示 PASS）
EOF
```

#### 4.2.3 TDD - REFACTOR (重构)

```bash
codeagent-wrapper --backend <ROUTED_BACKEND> resume <SESSION_ID> - "$PWD" <<'EOF'
## Task: <task_id>
## Phase: REFACTOR

要求：
1. 审查代码质量
2. 消除重复、改进命名
3. 保持测试通过
4. 运行测试确认仍然通过

输出：
- 重构后的代码（如有变更）
- 测试运行结果（应显示 PASS）
EOF
```

### 4.3 错误处理 (3-Strike 协议)

当任务执行失败时：

#### Strike 1: 针对性修复

```bash
codeagent-wrapper --backend <ROUTED_BACKEND> resume <SESSION_ID> - "$PWD" <<'EOF'
## Error Recovery - Strike 1

上一步失败：
<错误信息>

要求：
1. 分析错误原因
2. 应用针对性修复
3. 重试执行

输出：
- 错误分析
- 修复方案
- 重试结果
EOF
```

#### Strike 2: 换方案

```bash
codeagent-wrapper --backend <ROUTED_BACKEND> resume <SESSION_ID> - "$PWD" <<'EOF'
## Error Recovery - Strike 2

第一次修复仍然失败。

要求：
1. 不要重复之前的方法
2. 尝试完全不同的实现方案
3. 重试执行

已尝试过的方法：
<列出之前的尝试>

输出：
- 新方案描述
- 实现结果
EOF
```

#### Strike 3: 切换到备用后端

如果当前路由后端两次失败，切换到备用后端执行：

```markdown
[BACKEND_FALLBACK] Task: <task_id>
Reason: routed backend failed twice consecutively
Action: Switching to fallback backend
```

然后备用后端直接使用 Edit/Write 工具完成任务。

#### 3 Strikes 后: 询问用户

```
使用 AskUserQuestion 工具：
"任务 <task_id> 连续失败 3 次。

已尝试：
1. <方法1>
2. <方法2>
3. <方法3>

请选择：
- 提供指导
- 跳过此任务
- 取消工作流"
```

### 4.4 记录每个任务

每个任务完成后更新：

**task_plan.md**:
```markdown
### Task N: <task_id> [COMPLETED]
- Duration: Xmin
- Session: <session_id>
- Output: <files created/modified>
```

**progress.md**:
```markdown
| <timestamp> | EXECUTE | Task <task_id> completed | OK | TDD: RED→GREEN→REFACTOR |
```

---

## Phase 5: VERIFY (验证)

### 5.1 运行完整测试套件

```bash
# 自动检测测试命令
if [ -f "package.json" ]; then
  npm test
elif [ -f "pytest.ini" ] || [ -d "tests" ]; then
  pytest
elif [ -f "go.mod" ]; then
  go test ./...
fi
```

### 5.2 检查测试覆盖率

如果可用，运行覆盖率报告。

### 5.3 记录验证结果

```markdown
| <timestamp> | VERIFY | All tests passed | OK | X passed, 0 failed |
```

---

## Phase 6: REVIEW (审查)

### 6.1 代码质量自审查

```bash
codeagent-wrapper --backend <ROUTED_BACKEND> resume <SESSION_ID> - "$PWD" <<'EOF'
## Code Review

审查本次工作流中所有变更的文件：
<git diff 或文件列表>

检查项：
1. 代码风格一致性
2. 错误处理完整性
3. 安全性问题
4. 性能考虑
5. 文档完整性

输出：
- 发现的问题（如有）
- 建议的改进（如有）
- 最终评估：APPROVED / NEEDS_WORK
EOF
```

### 6.2 处理审查反馈

如果审查发现问题，创建修复任务并执行。

---

## Phase 7: COMMIT (提交)

### 7.1 Git 操作

```bash
# 检查是否有未提交的变更
git status

# 如果有变更
git add -A
git commit -m "<conventional commit message>"
```

### 7.2 Commit Message 生成

基于完成的任务生成 commit message：

```
feat(<scope>): <summary>

- <task1 description>
- <task2 description>
...

Fusion workflow completed.
```

---

## Phase 8: DELIVER (交付)

### 8.1 生成最终报告

```markdown
## Fusion Workflow Complete ✅

### Goal
<原始目标>

### Summary
- Duration: X minutes
- Tasks completed: N/N
- Tests: X passed
- Commits: N

### Changes Made
| File | Action | Description |
|------|--------|-------------|
| ... | ... | ... |

### Verification
- All tests passing: ✅
- Code review: APPROVED

### Git
- Branch: fusion/<goal-slug>
- Commits: N
- Ready for: merge / PR

### Recommendations
1. <建议1>
2. <建议2>
```

### 8.2 更新最终状态

**task_plan.md**:
```markdown
## Status
- Current Phase: DELIVER (8/8) ✅
- All tasks completed
```

**progress.md**:
```markdown
| <timestamp> | DELIVER | Workflow completed | OK | All N tasks done |
```

---

## 错误恢复总结

```
┌─────────────────────────────────────────────────────────────┐
│                    Error Handling Flow                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Task Fails                                                  │
│      │                                                       │
│      ▼                                                       │
│  Strike 1: 当前路由后端针对性修复                               │
│      │                                                       │
│      ▼ (仍失败)                                              │
│  Strike 2: 当前路由后端换方案                                   │
│      │                                                       │
│      ▼ (仍失败)                                              │
│  Strike 3: 切换到备用后端                                      │
│      │                                                       │
│      ▼ (仍失败)                                              │
│  询问用户：提供指导 / 跳过 / 取消                            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 会话管理

### sessions.json 格式

```json
{
  "workflow_id": "fusion_<timestamp>",
  "started_at": "<ISO timestamp>",
  "codex_session": "<SESSION_ID from Codex>",
  "tasks": {
    "task_id_1": {
      "status": "completed",
      "session": "<session_id>",
      "strikes": 0
    },
    "task_id_2": {
      "status": "in_progress",
      "session": "<session_id>",
      "strikes": 1
    }
  },
  "last_checkpoint": "<ISO timestamp>"
}
```

### 会话恢复 (/fusion resume)

1. 读取 `.fusion/sessions.json`
2. 找到最后一个 `in_progress` 任务
3. 使用 `codeagent-wrapper --backend codex resume <session_id>` 继续
4. 从断点继续执行

---

## 关键原则

1. **始终使用 HEREDOC** - 避免 shell 转义问题
2. **前台执行** - 不使用 background 模式
3. **保存 SESSION_ID** - 每次后端调用后更新 sessions.json
4. **记录一切** - 所有动作写入 progress.md
5. **3-Strike 不放弃** - 失败不是终点，是换方案的机会
6. **最少打扰** - 只在真正阻塞时询问用户
