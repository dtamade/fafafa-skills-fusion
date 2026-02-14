---
name: fusion
version: "2.6.3"
description: |
  自主工作流 Skill - 给目标后自主执行，只在必要时打扰用户。
  融合 Codex 规划/审查、Claude 执行、TDD、Git 与 3-Strike 降级策略。
user-invocable: true
hooks:
  PreToolUse:
    - matcher: "Write|Edit|Bash|Read|Glob|Grep"
      hooks:
        - type: command
          command: |
            if [ -f "${CLAUDE_PLUGIN_ROOT}/scripts/fusion-pretool.sh" ]; then
              bash "${CLAUDE_PLUGIN_ROOT}/scripts/fusion-pretool.sh"
            elif [ -f "scripts/fusion-pretool.sh" ]; then
              bash "scripts/fusion-pretool.sh"
            fi
  PostToolUse:
    - matcher: "Write|Edit"
      hooks:
        - type: command
          command: |
            if [ -f "${CLAUDE_PLUGIN_ROOT}/scripts/fusion-posttool.sh" ]; then
              bash "${CLAUDE_PLUGIN_ROOT}/scripts/fusion-posttool.sh"
            elif [ -f "scripts/fusion-posttool.sh" ]; then
              bash "scripts/fusion-posttool.sh"
            fi
  Stop:
    - hooks:
        - type: command
          command: |
            MODE="${FUSION_STOP_HOOK_MODE:-legacy}"
            if [ -f "${CLAUDE_PLUGIN_ROOT}/scripts/fusion-stop-guard.sh" ]; then
              FUSION_STOP_HOOK_MODE="$MODE" bash "${CLAUDE_PLUGIN_ROOT}/scripts/fusion-stop-guard.sh"
            elif [ -f "scripts/fusion-stop-guard.sh" ]; then
              FUSION_STOP_HOOK_MODE="$MODE" bash "scripts/fusion-stop-guard.sh"
            fi
---

# Fusion - 自主工作流

> Hook 配置请使用宿主标准设置文件（如 `.claude/settings.json`），见 `docs/HOOKS_SETUP.md`。


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
- **Codex** - 规划、复杂分析与审查
- **Claude** - 执行实现与本地改写
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
- 按阶段与任务类型路由 Codex/Claude（默认：Codex 规划审查，Claude 执行写码）
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
| `/fusion achievements` | 查看成就汇总与排行榜 |

### 选项

```bash
--backend codex|claude   # 强制指定后端 (默认: auto 路由)
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
│  3. DECOMPOSE   - Codex 拆分原子任务（可降级 Claude）       │
│  4. EXECUTE     - 默认 Claude 执行，按任务类型路由:          │
│                   RED → GREEN → REFACTOR                     │
│  5. VERIFY      - 运行完整测试套件                          │
│  6. REVIEW      - 代码质量自审查                            │
│  7. COMMIT      - Git 提交所有变更                          │
│  8. DELIVER     - 最终汇报                                  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Phase 0: UNDERSTAND (理解确认)

**目的**：在开始执行前确保 AI 尽量正确理解用户意图。

```
用户输入 /fusion "目标"
      │
      ▼
┌─────────────────────────────────────────────────────────────┐
│  1. 静默扫描（技术栈/目录/相关文件）                        │
│  2. 目标评分（0-10）                                       │
│  3. 输出理解摘要与假设                                     │
│  4. 阈值决策：                                              │
│     • 默认（require_confirmation=false）：继续执行         │
│     • 严格模式（require_confirmation=true）：低分阻塞      │
└─────────────────────────────────────────────────────────────┘
```

**当前默认行为（v2.6.x）**：

- `score >= pass_threshold`：自动推进到 `INITIALIZE`
- `score < pass_threshold`：记录缺失项与假设后继续（不阻塞）
- 可通过配置切换为严格模式：

```yaml
understand:
  pass_threshold: 7
  require_confirmation: false   # true: 低分阻塞，等待澄清
  max_questions: 2
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
│  Strike 1: 当前后端针对性修复                              │
│  └─ 分析错误原因，应用修复，重试                          │
│                                                           │
│  Strike 2: 当前后端换实现方案                              │
│  └─ 不重复已失败的路径，尝试替代方案                      │
│                                                           │
│  Strike 3: 切换到备用后端                                  │
│  └─ 避免重复失败路径，继续推进                              │
│                                                           │
│  3 Strikes 后: 升级给用户                                 │
│  └─ 详细说明尝试过什么，请求指导                          │
│                                                           │
└───────────────────────────────────────────────────────────┘
## 后端路由（摘要）

### Phase 默认路由

| 阶段 | 默认后端 |
|------|----------|
| UNDERSTAND / INITIALIZE / ANALYZE / DECOMPOSE / VERIFY / REVIEW | Codex |
| EXECUTE / COMMIT / DELIVER | Claude |

### Task Type 默认路由（EXECUTE 阶段）

| 任务类型 | 默认后端 | 说明 |
|----------|----------|------|
| implementation / verification | Claude | 执行编码与测试修复 |
| design / research | Codex | 深度分析、方案推导 |
| documentation / configuration | Claude | 低风险快速落地 |

详细路由与降级策略请看 `EXECUTION_PROTOCOL.md` 与 `templates/config.yaml`。

---

## 进度文件（.fusion）

```
.fusion/
├── task_plan.md
├── progress.md
├── findings.md
├── sessions.json
├── config.yaml
├── events.jsonl
├── backend_failure_report.json  # 后端阻塞（primary+fallback 失败）时生成
└── dependency_report.json   # 依赖阻塞时生成
```

常用查看方式：

```bash
/fusion status
cat .fusion/progress.md
tail -f .fusion/progress.md
```

---

## 依赖与自动修复

Fusion 会先自动处理关键依赖，再决定是否阻塞：

- 自动识别 Python：`python3` → `python`
- 自动定位 `codeagent-wrapper`：
  - `CODEAGENT_WRAPPER_BIN`
  - `PATH`
  - `./node_modules/.bin/codeagent-wrapper`
  - `~/.local/bin/codeagent-wrapper`
  - `~/.npm-global/bin/codeagent-wrapper`
- 仍无法处理时：
  - 写入 `.fusion/dependency_report.json`
  - 在 `/fusion status` 的 `Dependency Report` 展示修复建议
- 当后端执行阻塞（primary+fallback 都失败）时，会写入 `.fusion/backend_failure_report.json` 并在 `/fusion status` 的 `Backend Failure Report` 展示摘要
- 如果某个后端出现 hang/无响应，可设置 `FUSION_CODEAGENT_TIMEOUT_SEC=<seconds>` 让 `fusion-codeagent.sh` 超时后自动触发 fallback

---

## 配置（关键项）

```yaml
runtime:
  enabled: true

backends:
  primary: codex
  fallback: claude

backend_routing:
  phase_routing:
    EXECUTE: claude
  task_type_routing:
    implementation: claude
    design: codex

understand:
  pass_threshold: 7
  require_confirmation: false  # true: 低分阻塞并等待澄清

scheduler:
  enabled: true
  max_parallel: 2
  fail_fast: false

safe_backlog:
  enabled: true
  trigger_no_progress_rounds: 3

supervisor:
  enabled: false
  mode: advisory
```

完整配置见：`templates/config.yaml`。

---

## 执行规范与实现索引

- 执行协议：`EXECUTION_PROTOCOL.md`
- 并行策略：`PARALLEL_EXECUTION.md`
- 会话恢复：`SESSION_RECOVERY.md`
- Hook 挂载说明：`docs/HOOKS_SETUP.md`
- 提示词模板：`prompts/*.md`
- 运行脚本：`scripts/*.sh`
