# Parallel Execution Protocol

Fusion 支持并行执行独立任务以提高效率。

---

## 并行策略

### 任务依赖图

任务以 DAG (有向无环图) 形式组织：

```
        ┌─────────────┐
        │ api_design  │
        └──────┬──────┘
               │
       ┌───────┴───────┐
       ▼               ▼
┌─────────────┐ ┌─────────────┐
│ db_schema   │ │ auth_types  │
└──────┬──────┘ └──────┬──────┘
       │               │
       └───────┬───────┘
               ▼
        ┌─────────────┐
        │ auth_impl   │
        └──────┬──────┘
               ▼
        ┌─────────────┐
        │   tests     │
        └─────────────┘
```

### 并行执行规则

1. **无依赖任务** - 可以并行执行
2. **有依赖任务** - 等待所有依赖完成后执行
3. **最大并行度** - 当前由 `execution.parallel` 与 `scheduler.max_parallel` 共同约束；模板默认值均为 `2`
4. **并行开关与冲突策略** - `parallel.enabled` 控制是否启用并行，`parallel.conflict_check` 与 `parallel.fail_fast` 控制冲突检测和失败策略

---

## Codex 并行调用

### 方式 1: 使用 codeagent-wrapper --parallel

```bash
codeagent-wrapper --backend codex --parallel <<'EOF'
---TASK---
id: task1_$(date +%s)
workdir: $PWD
---CONTENT---
<task 1 content>

---TASK---
id: task2_$(date +%s)
workdir: $PWD
---CONTENT---
<task 2 content>

---TASK---
id: task3_$(date +%s)
workdir: $PWD
dependencies: task1_xxx, task2_xxx
---CONTENT---
<task 3 content - depends on 1 and 2>
EOF
```

### 方式 2: 使用 Claude Task 工具

```
# 并行启动多个后台任务
Task({
  description: "Execute task 1",
  prompt: "<task 1>",
  subagent_type: "code",
  run_in_background: true
})

Task({
  description: "Execute task 2",
  prompt: "<task 2>",
  subagent_type: "code",
  run_in_background: true
})

# 等待所有任务完成
TaskOutput({ task_id: "task1_id", block: true, timeout: 600000 })
TaskOutput({ task_id: "task2_id", block: true, timeout: 600000 })
```

---

## 依赖解析算法

### 拓扑排序

```text
def topological_sort(tasks):
    """
    返回任务的执行顺序，考虑依赖关系。
    无依赖的任务在同一层，可以并行执行。
    """
    # 计算入度
    in_degree = {t.id: 0 for t in tasks}
    for task in tasks:
        for dep in task.dependencies:
            in_degree[task.id] += 1

    # 找到所有入度为 0 的任务（可并行执行）
    ready = [t for t in tasks if in_degree[t.id] == 0]

    result = []
    while ready:
        # 这一批可以并行执行
        batch = ready[:]
        result.append(batch)

        ready = []
        for task in batch:
            # 更新依赖此任务的其他任务的入度
            for other in tasks:
                if task.id in other.dependencies:
                    in_degree[other.id] -= 1
                    if in_degree[other.id] == 0:
                        ready.append(other)

    return result
```

### 执行示例

给定任务:

```yaml
tasks:
  - id: A, dependencies: []
  - id: B, dependencies: []
  - id: C, dependencies: [A]
  - id: D, dependencies: [A, B]
  - id: E, dependencies: [C, D]
```

执行顺序:

```
Batch 1 (parallel): [A, B]
Batch 2 (parallel): [C, D]  # 等待 A, B 完成
Batch 3: [E]                 # 等待 C, D 完成
```

---

## 并行执行的注意事项

### 文件冲突

并行任务不应修改同一文件。如果检测到冲突：

1. 将冲突任务改为串行执行
2. 或者拆分任务以避免冲突

### Session 管理

每个并行任务可能有自己的 Codex session：

```json
{
  "tasks": {
    "task_a": { "session": "019xxx-a" },
    "task_b": { "session": "019xxx-b" }
  }
}
```

### 错误隔离

- 一个任务失败不影响其他并行任务
- 但依赖失败任务的后续任务会被跳过

### 资源限制

- 设置 `CODEAGENT_MAX_PARALLEL_WORKERS` 限制并发
- 推荐: 2-4 个并行任务，避免 API 限流

---

## 并行执行状态跟踪

### progress.md 格式

```markdown
| Time  | Event            | Status | Details                |
| ----- | ---------------- | ------ | ---------------------- |
| 14:30 | Batch 1 started  | OK     | Tasks: A, B (parallel) |
| 14:35 | Task A completed | OK     | Duration: 5min         |
| 14:36 | Task B completed | OK     | Duration: 6min         |
| 14:36 | Batch 2 started  | OK     | Tasks: C, D (parallel) |
| 14:42 | Task C completed | OK     | Duration: 6min         |
| 14:43 | Task D completed | OK     | Duration: 7min         |
| 14:43 | Batch 3 started  | OK     | Tasks: E               |
| 14:48 | Task E completed | OK     | Duration: 5min         |
```

### sessions.json 格式

```json
{
  "parallel_batches": [
    {
      "batch_id": 1,
      "tasks": ["A", "B"],
      "status": "completed",
      "started_at": "...",
      "completed_at": "..."
    },
    {
      "batch_id": 2,
      "tasks": ["C", "D"],
      "status": "completed"
    }
  ]
}
```

---

## 配置

### `.fusion/config.yaml`

当前推荐基线请参考 `templates/config.yaml`。

如果这里描述的并行配置基线或相关仓库/runtime 契约发生变化，请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`。

```yaml
execution:
  parallel: 2 # 最大并行任务数
  batch_timeout: 1800 # 每批次超时（秒）

parallel:
  enabled: true
  conflict_check: true # 检查文件冲突
  fail_fast: false # 一个失败是否停止所有

scheduler:
  enabled: true
  max_parallel: 2 # 调度器批次并行度上限
  fail_fast: false
```
