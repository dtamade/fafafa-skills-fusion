# 持续自主开发机制 - 限制分析

**分析日期**: 2026-02-14
**分析人**: reviewer
**问题**: Stop hook 在任务耗尽后无法继续注入新任务

---

## 问题描述

在测试持续自主开发机制时发现：
- Stop hook 成功实现了自动注入 safe_backlog 任务
- 但在注入 3 次后，无法继续注入新任务
- 系统最终允许停止，无法实现真正的"永不停止"

---

## 根本原因

### 1. 候选任务池固定且有限

Safe backlog 的候选任务池只有 **3 种固定任务**：

```text
def _candidate_tasks(project_root: Path) -> List[Dict[str, str]]:
    candidates = []

    # 1. README 更新
    if readme.exists():
        candidates.append({
            "title": "更新 README 快速开始说明",
            "category": "documentation",
        })

    # 2. Runtime 测试
    if runtime_tests.exists():
        candidates.append({
            "title": "补充 runtime 回归测试清单",
            "category": "quality",
        })

    # 3. Runtime 优化
    if runtime_dir.exists():
        candidates.append({
            "title": "优化 runtime 热路径扫描开销",
            "category": "optimization",
        })

    return candidates
```

### 2. Novelty Window 防重复机制

Safe backlog 使用 fingerprint + novelty window 机制防止重复注入：

```text
novelty_window = 12  # 默认值
seen = set(seen_list[-novelty_window:])  # 最近 12 个任务的 fingerprint

for candidate in candidates:
    fingerprint = _fingerprint(candidate)
    if fingerprint in seen:
        continue  # 跳过已注入的任务
```

### 3. 任务耗尽后的行为

当所有候选任务都在 novelty window 内时：
1. `selected = []`（没有可注入的任务）
2. 触发 backoff 机制（增加 consecutive_failures）
3. 返回 `added: 0`
4. Stop hook 检测到注入失败，允许停止

---

## 测试验证

### 测试 1: 初始注入

```bash
$ echo '{}' | bash scripts/fusion-stop-guard.sh
# 结果: 成功注入 2 个任务
# - 补充 runtime 回归测试清单
# - 优化 runtime 热路径扫描开销
```

### 测试 2: 任务完成后再次注入

```bash
$ echo '{}' | bash scripts/fusion-stop-guard.sh
# 结果: decision=allow（允许停止）
# 原因: 候选任务池已耗尽
```

### 测试 3: 清除状态后重试

```bash
$ rm -f .fusion/safe_backlog.json
$ echo '{}' | bash scripts/fusion-stop-guard.sh
# 结果: 仍然 decision=allow
# 原因: 候选任务池仍然只有 3 种，很快耗尽
```

---

## 影响分析

### 当前行为

1. **第 1-3 轮**: 成功注入 safe_backlog 任务，系统持续运行
2. **第 4 轮开始**: 候选任务池耗尽，系统允许停止
3. **结果**: 系统最多运行 3 轮 safe_backlog 任务后停止

### 实际效果

- ✅ 短期持续开发：可以运行 3 轮额外任务
- ❌ 长期持续开发：无法实现真正的"永不停止"
- ❌ 无限自主运行：受限于固定的候选任务池

---

## 解决方案

### 方案 1: 扩展候选任务池（推荐）

**实现**: 动态生成更多候选任务

```text
def _candidate_tasks(project_root: Path) -> List[Dict[str, str]]:
    candidates = []

    # 现有的 3 种任务
    # ...

    # 新增：代码质量任务
    candidates.append({
        "title": "代码质量检查与改进",
        "category": "quality",
    })

    # 新增：文档完善任务
    candidates.append({
        "title": "补充 API 文档",
        "category": "documentation",
    })

    # 新增：性能分析任务
    candidates.append({
        "title": "性能瓶颈分析",
        "category": "optimization",
    })

    # 新增：技术债务清理
    candidates.append({
        "title": "技术债务清理",
        "category": "quality",
    })

    # 新增：依赖更新
    candidates.append({
        "title": "依赖版本更新",
        "category": "optimization",
    })

    return candidates
```

**优点**:
- 简单直接
- 立即可用
- 向后兼容

**缺点**:
- 仍然是固定任务池
- 最终还是会耗尽

### 方案 2: 动态任务生成

**实现**: 基于代码库分析动态生成任务

```text
def _generate_dynamic_tasks(project_root: Path) -> List[Dict[str, str]]:
    tasks = []

    # 分析代码库
    # - 查找 TODO/FIXME 注释
    # - 检测代码复杂度
    # - 分析测试覆盖率
    # - 检查文档完整性

    # 生成对应任务
    for todo in find_todos(project_root):
        tasks.append({
            "title": f"处理 TODO: {todo.description}",
            "category": "quality",
        })

    return tasks
```

**优点**:
- 真正的动态生成
- 任务池理论上无限
- 更有价值的任务

**缺点**:
- 实现复杂
- 需要代码分析能力
- 可能生成低质量任务

### 方案 3: 调整 Novelty Window

**实现**: 减小 novelty window，允许任务重复

```yaml
safe_backlog:
  novelty_window: 3  # 从 12 减小到 3
```

**优点**:
- 配置简单
- 立即生效
- 允许任务循环

**缺点**:
- 任务会重复执行
- 可能产生无意义的工作
- 不是真正的解决方案

### 方案 4: 混合策略（最佳）

**实现**: 结合方案 1 和方案 3

1. 扩展候选任务池到 10-15 种
2. 设置合理的 novelty window（6-8）
3. 允许任务在一定周期后重复

**优点**:
- 平衡了多样性和可持续性
- 实现简单
- 效果好

**缺点**:
- 仍然不是完美的无限生成

---

## 建议

### 短期（立即执行）

1. **扩展候选任务池到 10 种**
   - 添加 7 种新的候选任务
   - 覆盖更多场景

2. **调整 novelty window 到 6**
   - 允许任务在 6 轮后重复
   - 平衡多样性和可持续性

### 中期（1-2 周）

1. **实现基础的动态任务生成**
   - 扫描 TODO/FIXME 注释
   - 生成对应的清理任务

2. **添加任务优先级机制**
   - 根据代码库状态调整任务优先级
   - 优先执行更有价值的任务

### 长期（1-2 月）

1. **实现智能任务生成**
   - 基于代码分析生成任务
   - 学习用户偏好
   - 动态调整任务类型

2. **添加任务效果评估**
   - 跟踪任务执行效果
   - 淘汰低价值任务
   - 优化任务生成策略

---

## 结论

当前的持续自主开发机制**部分成功**：
- ✅ 成功实现了自动注入机制
- ✅ 可以运行 3 轮额外任务
- ❌ 无法实现真正的"永不停止"
- ❌ 受限于固定的候选任务池

**核心问题**: 候选任务池太小（只有 3 种），无法支持长期持续开发。

**推荐方案**: 混合策略（扩展任务池 + 调整 novelty window）

---

**分析人签名**: reviewer
**分析时间**: 2026-02-14T15:30:00Z

> Archive note: this review keeps its historical context. For current behavior, use the Rust and shell contracts.

