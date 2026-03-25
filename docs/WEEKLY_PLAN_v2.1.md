# Fusion v2.1.0 每周执行清单

> Phase 1: 状态机内核 (0-30 天)
> 工作量估算: 每周 10-15 小时
> 说明：本文是历史执行清单，保留了当时对 `compat_mode` / 旧 runtime 路径的阶段性假设；当前实现请以仓库现状和契约文档为准。

---

## Week 1 (10-12h) — FSM 骨架可运行

### 本周目标
把 8 阶段流程从"文档约定"落到"可执行状态机骨架"

### 具体任务
- [ ] 定义状态/事件/转移模型（含守卫条件与错误转移）
- [ ] 实现最小 `kernel` 执行循环 (`dispatch -> transition -> emit`)
- [ ] 加入 `runtime.enabled`、`runtime.compat_mode` 配置开关
- [ ] 建立最小单测框架（覆盖状态迁移与非法事件）
- [ ] 提供本地 smoke runner

### 交付物
```
scripts/runtime/
├── state_machine
├── kernel
└── tests/
    ├── test_state_machine
    └── test_kernel_smoke

templates/config.yaml  # 新增 runtime 配置段
```

### 当时记录的验收范围
```text
测试记录： scripts/runtime/tests/test_state_machine -v
测试记录： scripts/runtime/tests/test_kernel_smoke -v
场景记录： scripts/runtime/dev_smoke --scenario basic_flow
```

### 风险检查点
| 风险 | 应对 |
|------|------|
| 状态定义膨胀 (>8 主状态) | 冻结 scope，不加新状态 |
| 工时超 12h | 延后文档，保核心代码 |
| 兼容风险不明 | `runtime.enabled` 默认 `false` |

---

## Week 2 (12-15h) — 事件溯源与恢复能力

### 本周目标
实现可重放的事件流，让 `resume` 具备稳定恢复基础

### 具体任务
- [ ] 实现私有持久化层 (append/replay/idempotency key)
- [ ] 实现进程内 `event_bus` (发布、订阅、错误隔离)
- [ ] 将 `kernel` 与 `session_store/event_bus` 接通
- [ ] 增加"中断→恢复"集成测试
- [ ] 做一次故障注入并验证回退

### 交付物
```
scripts/runtime/
├── _session_store
├── event_bus
└── tests/
    ├── test_session_store
    ├── test_event_bus
    └── test_resume_replay
```

### 当时记录的验收范围
```text
测试记录： scripts/runtime/tests/test_session_store -v
测试记录： scripts/runtime/tests/test_event_bus -v
测试记录： scripts/runtime/tests/test_resume_replay -v
场景记录： scripts/runtime/dev_smoke --scenario interrupt_resume
```

### 风险检查点
| 风险 | 应对 |
|------|------|
| 事件重复执行 | 强制 idempotency key 校验 |
| 文件锁竞争 | 统一复用 `.fusion/.state.lock` |
| jq 缺失 | 测试 jq 与非 jq 双路径 |

---

## Week 3 (10-12h) — Hook 薄适配 + v2 兼容

### 本周目标
让现有命令入口接入新内核，但用户命令体验不变

### 具体任务
- [ ] 实现 `compat_v2` 适配层 (旧命令语义 -> FSM 事件)
- [ ] 改造 `fusion-stop-guard.sh` 调 runtime adapter
- [ ] 改造 `fusion-pretool.sh`、`fusion-posttool.sh` 为薄适配层
- [ ] 增加兼容回归测试
- [ ] 做一次"runtime 关闭"回退演练

### 交付物
```
scripts/runtime/
├── compat_v2
└── tests/
    ├── test_compat_v2
    └── test_hook_adapter

scripts/
├── fusion-stop-guard.sh  # 适配调用
├── fusion-pretool.sh     # 适配调用
└── fusion-posttool.sh    # 适配调用
```

### 当时记录的验收范围
```text
测试记录： scripts/runtime/tests/test_compat_v2 -v
测试记录： scripts/runtime/tests/test_hook_adapter -v
Shell syntax check: bash -n scripts/fusion-stop-guard.sh scripts/fusion-pretool.sh scripts/fusion-posttool.sh
场景记录： scripts/runtime/dev_smoke --scenario v2_command_compat
```

### 风险检查点
| 风险 | 应对 |
|------|------|
| Hook 性能恶化 (旧 runtime 启动) | 保留 `compat_mode=true` 作为 hook/兼容层缓冲，而非恢复控制面旧主实现 |
| 兼容回归阻塞 | 保持旧逻辑可切回 |
| 工时不足 | 暂缓文档，先保回归测试 |

---

## Week 4 (12-15h) — 稳定性压测与 v2.1.0 发布

### 本周目标
达到发布门槛并产出可发布的 `v2.1.0`

### 具体任务
- [ ] 完成 Phase 1 全量回归 (≥30 场景)
- [ ] 跑性能基准：PreToolUse p95 < 80ms, StopHook p95 < 150ms
- [ ] 跑恢复可靠性测试：成功率 ≥95% (20 次)
- [ ] 整理最小发布文档
- [ ] 版本发布：更新版本号、CHANGELOG、发布标签

### 交付物
```
scripts/runtime/
├── regression_runner
├── bench_hook_latency
└── tests/  # 完整 Phase 1 测试集

docs/
├── RUNTIME_KERNEL.md
└── UPGRADE_v2_COMPAT.md

CHANGELOG.md  # 新增 v2.1.0 条目
```

### 当时记录的验收范围
```text
全量测试记录：discover -s scripts/runtime/tests -p 'test_*' -v

回归运行记录（≥99% 通过率）：
scripts/runtime/regression_runner --suite phase1 --min-pass-rate 0.99

性能基准记录：
scripts/runtime/bench_hook_latency --runs 300 --pretool-p95-ms 80 --stop-p95-ms 150

回归运行记录（≥95% 通过率）：
scripts/runtime/regression_runner --scenario resume_reliability --runs 20 --min-pass-rate 0.95
```

### 风险检查点
| 风险 | 应对 |
|------|------|
| 指标未达标 | 不发布，先发 `v2.1.0-rc` |
| 回归修复引入新问题 | 冻结功能，只做缺陷修复 |
| 发布不可回滚 | 必须保留安全开关 |

---

## 发布检查清单

### v2.1.0 发布门槛
- [ ] 状态迁移正确率 ≥ 99%
- [ ] PreToolUse p95 < 80ms
- [ ] StopHook p95 < 150ms
- [ ] `/fusion resume` 恢复成功率 ≥ 95%
- [ ] v2.0 命令兼容率 100%
- [ ] 所有 P0/P1 问题已修复
- [ ] CHANGELOG.md 已更新
- [ ] 发布标签已创建

### 回滚准备
- [ ] `runtime.enabled=false` 可一键关闭
- [ ] `compat_mode=true` 保留当时定义的 hook/兼容层行为（非要求控制面继续维持旧 Shell 主实现）
- [ ] 回滚文档已准备

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
