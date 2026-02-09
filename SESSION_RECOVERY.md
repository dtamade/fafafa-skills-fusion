# Session Recovery Protocol

Fusion 支持会话恢复，允许在中断后继续工作。

---

## 恢复场景

### 1. `/clear` 后恢复

用户使用 `/clear` 清除上下文后：
```bash
/fusion resume
```

### 2. 会话超时恢复

Codex 会话超时或断开后恢复。

### 3. 手动暂停后恢复

使用 `/fusion pause` 暂停后恢复。

---

## 恢复流程

### Step 1: 读取 sessions.json

```json
{
  "workflow_id": "fusion_1707500000",
  "goal": "实现用户认证系统",
  "status": "in_progress",
  "current_phase": "EXECUTE",
  "codex_session": "019a7247-ac9d-71f3-89e2-xxx",
  "tasks": {
    "auth_api_design": { "status": "completed" },
    "db_schema": { "status": "completed" },
    "auth_login": { "status": "in_progress", "session": "019xxx" }
  }
}
```

### Step 2: 读取 task_plan.md

找到当前正在执行或待执行的任务。

### Step 3: 读取 progress.md

了解最后的执行状态和任何错误。

### Step 4: 恢复 Codex 会话

```bash
codeagent-wrapper --backend codex resume <session_id> - "$PWD" <<'EOF'
## Session Recovery

上次执行到：<任务名>
状态：<状态>

请继续执行...
EOF
```

### Step 5: 继续工作流

从断点继续执行剩余任务。

---

## 5-Question Reboot Test

恢复后，确保能回答这 5 个问题：

| 问题 | 来源 |
|------|------|
| 我在哪？ | `sessions.json` → current_phase |
| 要去哪？ | `task_plan.md` → 剩余任务 |
| 目标是什么？ | `sessions.json` → goal |
| 学到了什么？ | `findings.md` |
| 做了什么？ | `progress.md` |

如果任何问题无法回答，需要重新分析。

---

## 恢复命令实现

### /fusion resume

```markdown
当用户调用 /fusion resume 时：

1. 检查 .fusion/sessions.json 是否存在
2. 读取 sessions.json 获取：
   - workflow_id
   - codex_session
   - 当前任务状态
3. 读取 task_plan.md 获取未完成任务
4. 读取 progress.md 获取最后状态
5. 输出恢复摘要给用户
6. 使用 codeagent-wrapper --backend codex resume 继续执行
```

### 恢复摘要格式

```markdown
## Fusion Resume

### Workflow
- ID: fusion_1707500000
- Goal: 实现用户认证系统
- Started: 2026-02-09 14:30

### Progress
- Completed: 3/6 tasks
- Current: auth_login (in_progress)
- Remaining: 3 tasks

### Last Activity
- Time: 2026-02-09 15:15
- Action: auth_login - TDD GREEN phase
- Status: OK

### Codex Session
- Session ID: 019xxx
- Can resume: Yes

Continuing from auth_login...
```

---

## 检查点机制

### 自动检查点

在以下时机自动保存检查点：
1. 每个任务完成后
2. 每个 TDD 阶段完成后
3. 发生错误时
4. 会话结束时（Stop hook）

### 检查点内容

```json
{
  "checkpoint_time": "2026-02-09T15:15:00Z",
  "phase": "EXECUTE",
  "task": "auth_login",
  "tdd_phase": "GREEN",
  "codex_session": "019xxx",
  "files_modified": ["src/auth/login.ts"],
  "git_status": "uncommitted changes"
}
```

### Hook 配置

```yaml
# SKILL.md frontmatter
hooks:
  Stop:
    - hooks:
        # Single stop hook with safety features
        - type: command
          command: "bash ${CLAUDE_PLUGIN_ROOT}/scripts/fusion-stop-guard.sh"
```

---

## 错误恢复

### 检测未完成的工作

```bash
# 检查 git 状态
git status

# 检查未提交的变更
git diff

# 检查任务状态
grep "IN_PROGRESS\|PENDING" .fusion/task_plan.md
```

### 恢复策略

| 状态 | 策略 |
|------|------|
| 任务未开始 | 正常开始 |
| 任务进行中 | 从 TDD 当前阶段继续 |
| 任务失败 | 应用 3-Strike 协议 |
| 变更未提交 | 先提交后继续 |

---

## sessions.json 完整结构

```json
{
  "workflow_id": "fusion_<timestamp>",
  "goal": "<user goal>",
  "started_at": "<ISO timestamp>",
  "status": "not_started|in_progress|paused|completed|cancelled|stuck|waiting_user",
  "current_phase": "INITIALIZE|ANALYZE|DECOMPOSE|EXECUTE|VERIFY|REVIEW|COMMIT|DELIVER",

  "codex_session": "<main session ID>",

  "tasks": {
    "<task_id>": {
      "status": "pending|in_progress|completed|skipped",
      "session": "<task-specific session ID>",
      "tdd_phase": "RED|GREEN|REFACTOR|null",
      "strikes": 0,
      "started_at": "<timestamp>",
      "completed_at": "<timestamp>",
      "output": ["<file1>", "<file2>"]
    }
  },

  "strikes": {
    "current_task": "<task_id>",
    "count": 0,
    "history": [
      {
        "task": "<task_id>",
        "attempt": 1,
        "error": "<error message>",
        "resolution": "<what was tried>"
      }
    ]
  },

  "git": {
    "original_branch": "main",
    "working_branch": "fusion/auth-system",
    "commits": [
      {
        "hash": "abc123",
        "message": "feat(auth): add login endpoint",
        "task": "auth_login"
      }
    ]
  },

  "checkpoints": [
    {
      "time": "<timestamp>",
      "phase": "EXECUTE",
      "task": "auth_login",
      "details": "TDD GREEN completed"
    }
  ],

  "last_checkpoint": "<ISO timestamp>",

  "config": {
    "tdd_enabled": true,
    "git_enabled": true,
    "parallel": 2,
    "backend": "codex"
  }
}
```

---

## 恢复脚本

### scripts/fusion-resume.sh

详见 `scripts/fusion-resume.sh`，主要功能：
1. 读取 sessions.json
2. 显示恢复摘要
3. 指导 Claude 如何继续

### 使用方式

```bash
# 查看恢复信息
./scripts/fusion-resume.sh

# Claude 根据输出继续执行
# 使用 codeagent-wrapper --backend codex resume <SESSION_ID>
```
