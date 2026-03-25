# v2.0 → v2.1.0 升级与兼容指南

> 版本: v2.1.0
> 最后校准: 2026-03-21

## 概述

v2.1.0 开始把控制面和 hook 路径逐步迁移到 Rust bridge / runtime 内核，采用 **Strangler Fig Pattern** 收敛旧实现。对用户而言，命令入口和文件格式保持兼容，但实现边界已调整：hook 保留最小 Shell fallback，控制面薄包装脚本则直接委托 Rust。

## 架构变化

```
v2.0 (纯 Shell)              v2.1.0 (Rust bridge 主线)

fusion-stop-guard.sh          fusion-stop-guard.sh
  └─ Shell 逻辑                 ├─ fusion-bridge hook stop-guard
                                └─ fallback ──→ 最小 Shell hook 路径

fusion-pretool.sh             fusion-pretool.sh
  └─ Shell 逻辑                 ├─ fusion-bridge hook pretool
                                └─ fallback ──→ 最小 Shell hook 路径

fusion-posttool.sh            fusion-posttool.sh
  └─ Shell 逻辑                 ├─ fusion-bridge hook posttool
                                └─ fallback ──→ 最小 Shell hook 路径

fusion-{status,resume,pause,cancel,continue,achievements}.sh
  └─ 各自 Shell 逻辑             └─ thin wrapper ──→ fusion-bridge <command>
```

## 启用 Runtime

在 v2.1.0 当时的升级语境里，下面的对比以 `runtime.enabled=false` 的升级前基线为起点；当前仓库模板默认值已经切到 `runtime.enabled=true`。通过由 `templates/config.yaml` 生成的 `.fusion/config.yaml` 控制：

```yaml
# .fusion/config.yaml
runtime:
  enabled: true # 启用 Rust bridge / runtime 主路径
  compat_mode: true # v2 兼容模式 (保留最小 Shell 回退)
```

### 启用后的行为

| 路径                                                       | runtime.enabled=false (历史升级前基线) | runtime.enabled=true         |
| ---------------------------------------------------------- | -------------------------------------- | ---------------------------- |
| stop-guard                                                 | Shell 逻辑                             | Rust bridge → Shell fallback |
| pretool                                                    | Shell 逻辑                             | Rust bridge → Shell fallback |
| posttool                                                   | Shell 逻辑                             | Rust bridge → Shell fallback |
| status / achievements / pause / resume / cancel / continue | 历史 Shell 实现                        | thin wrapper → Rust bridge   |

当前只有 live hook 路径在 Rust bridge 不可用、被禁用或返回失败时才会自动降级到最小 Shell 逻辑；控制面薄包装脚本会直接返回依赖错误。旧 runtime/reference 层已从仓库移除，不再保留任何公开或内部的旧 hook 适配入口。

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
├── task_plan.md          # 运行中任务计划
├── progress.md           # 运行中进度时间线
├── findings.md           # 运行中研究发现
├── config.yaml           # 由模板生成的工作区配置（扩展了 runtime 区段）
├── .progress_snapshot    # 不变
└── .state.lock           # [新] 状态锁 (自动清理)
```

## 命令兼容性

| 命令             | v2.0 | v2.1.0 | 备注             |
| ---------------- | ---- | ------ | ---------------- |
| `/fusion`        | ✅   | ✅     | 启动工作流       |
| `/fusion status` | ✅   | ✅     | 查看状态         |
| `/fusion resume` | ✅   | ✅     | 恢复中断的工作流 |
| `/fusion pause`  | ✅   | ✅     | 暂停工作流       |
| `/fusion cancel` | ✅   | ✅     | 取消工作流       |

## 回退方案

如果 hook/runtime 主路径导致问题，可以立即关闭：

```yaml
# .fusion/config.yaml
runtime:
  enabled: false # 回退 hook/runtime 到 Shell 路径
```

回退后：

- hook/runtime 入口恢复到 Shell 路径
- 控制面薄包装脚本若未提供 bridge，则仍会按依赖缺失处理
- `events.jsonl` 和 `_runtime` 字段保留但不再写入
- **无需清理任何文件**，工作流可继续执行

## 阶段自动纠正

v2.1.0 新增了阶段一致性检查（仅 runtime 启用时生效）：

| 检测条件                 | 自动纠正                            |
| ------------------------ | ----------------------------------- |
| EXECUTE + 所有任务完成   | 派发 `ALL_TASKS_DONE` → 进入 VERIFY |
| VERIFY + 有 PENDING 任务 | 派发 `VERIFY_FAIL` → 回退 EXECUTE   |

这避免了 v2.0 中偶发的"阶段卡住"问题。

## 性能影响

| Hook       | v2.0 (纯 Shell)   | v2.1.0 (Rust 主线)           | 变化 |
| ---------- | ----------------- | ---------------------------- | ---- |
| pretool    | 纯 Shell 历史基线 | Rust bridge / Shell fallback | 更快 |
| posttool   | 纯 Shell 历史基线 | Rust bridge / Shell fallback | 更快 |
| stop-guard | 纯 Shell 历史基线 | Rust bridge / Shell fallback | 更快 |

> 注: 当前 live hook 路径的主延迟来自 Rust bridge 或 Shell fallback；仓库内已不再保留单独的 hook parity 层。

## 依赖要求

| 依赖                  | v2.0 | v2.1.0 | 备注                                                           |
| --------------------- | ---- | ------ | -------------------------------------------------------------- |
| Bash 4.0+             | 必需 | 必需   | 不变                                                           |
| Rust stable toolchain | 可选 | 推荐   | 从源码构建 `fusion-bridge` 或使用 `cargo run --release` 时需要 |
| jq                    | 可选 | 可选   | 仅用于 machine JSON smoke 或人工 JSON 检查                     |

仓库已不再依赖旧 runtime helper；当前所需能力集中在 Rust bridge、Shell thin wrapper，以及可选的 machine JSON smoke / 人工 JSON 检查工具。`jq` 不是运行时必需依赖。Hook 是否可完整回退，取决于当前 bridge / Shell fallback 配置。

## 验证升级

升级后运行以下命令验证：

```bash
# 1. Rust integration / contract tests
cd rust
cargo test --release -p fusion-cli --test repo_contract
cargo test --release -p fusion-cli --test shell_contract
cargo test --release -p fusion-cli --test cli_smoke

# 2. 回归测试 (35 scenarios)
fusion-bridge regression --suite phase1

# 3. Rust contract / parity checks
fusion-bridge regression --suite phase2

# 4. Shell wrapper smoke
bash scripts/ci-machine-mode-smoke.sh
bash scripts/ci-cross-platform-smoke.sh

# 5. Shell 脚本语法检查
bash -n scripts/fusion-stop-guard.sh
bash -n scripts/fusion-pretool.sh
bash -n scripts/fusion-posttool.sh
```

如果升级文档里的命令面契约、兼容性边界，或相关验证基线发生变化，请同步更新受影响的活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`。

## 常见问题

### Q: 我需要修改任何现有文件吗？

不需要。v2.1.0 完全向后兼容。只需要在由 `templates/config.yaml` 生成的 `.fusion/config.yaml` 中设置 `runtime.enabled: true` 即可启用新特性。

### Q: Runtime 崩溃会影响工作流吗？

通常不会。hook 路径保留最小 Shell fallback；但 `status`、`achievements`、`pause`、`resume`、`cancel`、`continue` 这些控制面薄包装脚本要求 `fusion-bridge` 可用，不再回退到旧 Shell 主实现。

### Q: events.jsonl 会无限增长吗？

当前版本不会自动清理。每个事件约 200 字节，1000 个事件约 200KB。v2.5.0 将加入 retention 策略。

### Q: 可以在启用 Runtime 后再关闭吗？

可以。修改 `.fusion/config.yaml` 中 `runtime.enabled: false` 即可。不需要清理任何文件。
