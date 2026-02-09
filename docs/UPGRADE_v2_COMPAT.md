# v2.0 → v2.1.0 升级与兼容指南

> 版本: v2.1.0
> 更新日期: 2026-02-09

## 概述

v2.1.0 在 Shell 脚本内部引入了 Python FSM 运行时内核，采用 **Strangler Fig Pattern** 逐步替换原有逻辑。对用户而言，所有命令和文件格式 **100% 向后兼容**。

## 架构变化

```
v2.0 (纯 Shell)              v2.1.0 (Shell + Python Runtime)

fusion-stop-guard.sh          fusion-stop-guard.sh
  └─ Shell 逻辑                 ├─ runtime.enabled? ──→ compat_v2.py ──→ Kernel
                                └─ fallback ──→ 原 Shell 逻辑

fusion-pretool.sh             fusion-pretool.sh
  └─ Shell 逻辑                 ├─ runtime.enabled? ──→ compat_v2.py
                                └─ fallback ──→ 原 Shell 逻辑

fusion-posttool.sh            fusion-posttool.sh
  └─ Shell 逻辑                 ├─ runtime.enabled? ──→ compat_v2.py
                                └─ fallback ──→ 原 Shell 逻辑
```

## 启用 Runtime

Runtime 默认**关闭**。通过 `.fusion/config.yaml` 控制：

```yaml
# .fusion/config.yaml
runtime:
  enabled: true       # 启用 Python Runtime 内核
  compat_mode: true   # v2 兼容模式 (保持 Shell fallback)
```

### 启用后的行为

| Hook | runtime.enabled=false (默认) | runtime.enabled=true |
|------|------|------|
| stop-guard | Shell 逻辑 | Python → Shell fallback |
| pretool | Shell 逻辑 | Python → Shell fallback |
| posttool | Shell 逻辑 | Python → Shell fallback |

当 Python 适配层发生异常时，自动降级到原 Shell 逻辑，**不会中断工作流**。

## 文件格式变化

### sessions.json

v2.1.0 新增 `_runtime` 字段（带下划线前缀，不影响 v2.0 解析）：

```json
{
  "goal": "实现用户认证",
  "status": "in_progress",
  "current_phase": "EXECUTE",

  "_runtime": {
    "version": "2.1.0",
    "state": "EXECUTE",
    "last_event_counter": 42
  }
}
```

v2.0 脚本中的 `grep`/`jq` 解析不会受到 `_runtime` 字段的影响。

### 新增文件

启用 Runtime 后，`.fusion/` 目录会新增：

```
.fusion/
├── sessions.json         # 扩展了 _runtime 字段
├── events.jsonl          # [新] 事件溯源日志 (append-only)
├── task_plan.md          # 不变
├── config.yaml           # 扩展了 runtime 区段
├── .progress_snapshot    # 不变
└── .state.lock           # [新] 状态锁 (自动清理)
```

## 命令兼容性

| 命令 | v2.0 | v2.1.0 | 备注 |
|------|------|--------|------|
| `/fusion` | ✅ | ✅ | 启动工作流 |
| `/fusion status` | ✅ | ✅ | 查看状态 |
| `/fusion resume` | ✅ | ✅ | 恢复中断的工作流 |
| `/fusion pause` | ✅ | ✅ | 暂停工作流 |
| `/fusion cancel` | ✅ | ✅ | 取消工作流 |

## 回退方案

如果 Runtime 导致问题，可以立即关闭：

```yaml
# .fusion/config.yaml
runtime:
  enabled: false  # 回退到纯 Shell 模式
```

回退后：
- Shell 脚本恢复原有行为
- `events.jsonl` 和 `_runtime` 字段保留但不再写入
- **无需清理任何文件**，工作流可继续执行

## 阶段自动纠正

v2.1.0 新增了阶段一致性检查（仅 runtime 启用时生效）：

| 检测条件 | 自动纠正 |
|----------|----------|
| EXECUTE + 所有任务完成 | 派发 `ALL_TASKS_DONE` → 进入 VERIFY |
| VERIFY + 有 PENDING 任务 | 派发 `VERIFY_FAIL` → 回退 EXECUTE |

这避免了 v2.0 中偶发的"阶段卡住"问题。

## 性能影响

| Hook | v2.0 (纯 Shell) | v2.1.0 (Python) | 变化 |
|------|------------------|------------------|------|
| pretool | ~5ms | ~0.3ms (内存调用) | 更快 |
| posttool | ~5ms | ~0.2ms (内存调用) | 更快 |
| stop-guard | ~10ms | ~0.4ms (内存调用) | 更快 |

> 注: Python 路径的延迟来自进程内函数调用，不含 Python 启动开销。Shell 脚本通过 `python3 -m runtime.compat_v2` 子进程调用时，额外增加 ~50-80ms 启动开销，仍在阈值内。

## 依赖要求

| 依赖 | v2.0 | v2.1.0 | 备注 |
|------|------|--------|------|
| Bash 4.0+ | 必需 | 必需 | 不变 |
| Python 3.8+ | 可选 | 推荐 | Runtime 需要 |
| PyYAML | 不需要 | 推荐 | 读取 config.yaml |
| jq | 推荐 | 推荐 | 不变 |

如果 Python 不可用，Runtime 自动禁用，完全回退到 Shell 模式。

## 验证升级

升级后运行以下命令验证：

```bash
# 1. 单元测试 (139 tests)
cd scripts && python3 -m pytest runtime/tests/ -v

# 2. 回归测试 (35 scenarios)
python3 scripts/runtime/regression_runner.py --suite phase1

# 3. 性能基准
python3 scripts/runtime/bench_hook_latency.py --runs 300

# 4. Shell 脚本语法检查
bash -n scripts/fusion-stop-guard.sh
bash -n scripts/fusion-pretool.sh
bash -n scripts/fusion-posttool.sh
```

## 常见问题

### Q: 我需要修改任何现有文件吗？
不需要。v2.1.0 完全向后兼容。只需要在 `config.yaml` 中设置 `runtime.enabled: true` 即可启用新特性。

### Q: Runtime 崩溃会影响工作流吗？
不会。所有 Shell 脚本都有 fallback 路径：Python 失败后自动降级到原 Shell 逻辑。

### Q: events.jsonl 会无限增长吗？
当前版本不会自动清理。每个事件约 200 字节，1000 个事件约 200KB。v2.5.0 将加入 retention 策略。

### Q: 可以在启用 Runtime 后再关闭吗？
可以。修改 `config.yaml` 中 `runtime.enabled: false` 即可。不需要清理任何文件。
