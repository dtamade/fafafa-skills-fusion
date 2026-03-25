# Fusion Skill v3 路线图

> 生成日期: 2026-02-09
> 来源: Codex 架构分析
> 状态: 历史阶段路线图（保留当时目标与文件命名）
> 当前执行真源：当前 live 执行顺序与 GA 收口请以 `docs/V3_GA_EXECUTION_ROADMAP.md` 为准。
> 说明：本文保留路线图撰写时的阶段性假设；若与当前仓库实现冲突，应以 `README.md`、`README.zh-CN.md`、`docs/CLI_CONTRACT_MATRIX.md`、`rust/README.md` 和实际脚本行为为准。
> 当前实现注记：Rust / `fusion-bridge` 已是主控制面；Shell 主要保留在 thin wrapper 与 hook 接线；旧 runtime/reference 层已从仓库移除。下文如出现历史 runtime 模块名时，应按历史阶段性口径理解。

## 总览

| 阶段 | 版本 | 时间 | 核心目标 | 验收指标 |
|------|------|------|----------|----------|
| Phase 1 | **v2.1.0** | 0-30 天 | 状态机内核 (Kernel Beta) | 状态迁移 ≥99%，Hook p95 <80ms |
| Phase 2 | **v2.5.0** | 31-60 天 | 并行调度 + Token 治理 | 加速比 ≥1.4x，超支率 ≤10% |
| Phase 3 | **v3.0.0** | 61-90 天 | 产品化 + 模型总线 GA | 端到端成功率 ≥90% |

---

## Phase 1: v2.1.0 (0-30 天) - 立内核、不动外壳

### 目标
把 8 阶段从"提示词约束"升级为"可执行 FSM"，同时保持 v2.0 完全兼容。

### 核心交付物

```
历史 runtime 模块集（阶段性命名）
├── kernel             # 状态机内核（历史模块名）
├── state_machine      # FSM 定义 (状态、事件、转移、守卫条件)
├── session_store      # 事件溯源存储 (幂等执行键)
├── event_bus          # 事件总线
└── 历史兼容适配层      # v2 兼容能力（历史模块名）

维护文档
├── 内核设计文档（历史计划名）
└── UPGRADE_v2_COMPAT.md

templates/config.yaml  # 增加 runtime.enabled, runtime.compat_mode
```

### 改造点
- `fusion-stop-guard.sh` → thin wrapper / `fusion-bridge hook stop-guard`
- `fusion-pretool.sh` → thin wrapper / hook pretool 入口
- `fusion-posttool.sh` → thin wrapper / hook posttool 入口

### 最小可验收能力
- 用户继续用原命令 (`/fusion`、`status`、`resume`)
- 状态流转更稳定、恢复更可靠
- 异常时给出明确"当前状态→下一动作"提示

### 验收指标
| 指标 | 目标 | 测试方法 |
|------|------|----------|
| 状态迁移正确率 | ≥ 99% | ≥30 回归场景 |
| PreToolUse p95 | < 80ms | 性能测试 |
| Stop Hook p95 | < 150ms | 性能测试 |
| `/fusion resume` 恢复成功率 | ≥ 95% | ≥20 次中断测试 |
| v2.0 命令兼容率 | 100% | 兼容性测试 |

### 风险与缓解
| 风险 | 缓解措施 |
|------|----------|
| 旧实现启动开销导致 Hook 变慢 | 默认 `compat_mode=true`，runtime 可开关灰度 |
| 状态 schema 漂移 | 保留文本工具兜底路径 |

---

## Phase 2: v2.5.0 (31-60 天) - 并行与治理同上 ✅ 2026-02-09

### 目标
实现真并行调度和 Token/时延治理，让执行可控可预测。

### 核心交付物

```
历史 runtime 模块集（阶段性命名）
├── task_graph          # DAG 任务图编译器 (Kahn 拓扑排序)
├── scheduler           # 并行调度器 (决策管道)
├── conflict_detector   # 文件冲突检测 (writeset 交集)
├── budget_manager      # Token/时延预算管理
├── router              # 模型路由 (类型+预算分级)
├── 并行模拟验收        # 并行验收测试（历史模块名）
└── Hook 延迟基准       # Hook+Scheduler 性能基准（历史模块名）

templates/config.yaml    # 新增 scheduler, budget 配置段
```

### 最小可验收能力
- ✅ 用户可感知"真并行"：有依赖任务自动排队，无依赖任务并行
- ✅ 看到 token/时延预算进度
- ✅ 超预算自动降速或回退串行，不会卡死
- ✅ `scheduler.enabled=false` 完全退化为 v2.1.0 串行行为

### 验收指标
| 指标 | 目标 | 实际 | 状态 |
|------|------|------|------|
| DAG 依赖违规数 | = 0 | 历史记录达标 | ✅ |
| 中位加速比 | ≥ 1.4x | 历史记录达标 | ✅ |
| 冲突回滚率 | ≤ 5% | 历史记录达标 | ✅ |
| Token 超支率 | ≤ 10% | 历史记录达标 | ✅ |
| 硬上限突破 | = 0 | 历史记录达标 | ✅ |
| 调度决策 p95 | < 200ms | 历史记录达标 | ✅ |
| v2.1.0 回归 | 全量通过 | 历史记录通过 | ✅ |

### 测试覆盖
当时的测试覆盖统计已记录为全绿；当前仓库验证请以 Rust 契约测试与 release 门禁为准。

当时记录中，回归与恢复可靠性均达标。

### 风险与缓解
| 风险 | 缓解措施 | 结果 |
|------|----------|------|
| 并行导致非确定性 | 先 `scheduler.enabled=false` 灰度 | ✅ 默认串行，灰度可控 |
| 冲突检测误报 | 支持"一键回退串行" | ✅ 历史记录达标，风险受控 |

---

## Phase 3: v3.0.0 (61-90 天) - 产品化 GA

### 目标
把能力产品化，提供可解释、可诊断、可协作的工作流体验。

### 核心交付物

```
历史 runtime 模块集（阶段性命名）
├── model_bus          # 多模型编排总线（历史模块名）
├── policy_engine      # 策略引擎（历史模块名）
└── telemetry          # 可观测性/遥测（历史模块名）

历史命令目标
├── explain 入口       # /fusion explain（历史目标）
└── doctor 入口        # /fusion doctor（历史目标）

维护文档
├── 模型总线文档（历史计划名）
├── 运维手册（历史计划名）
└── v2→v3 升级指南（历史计划名）
```

### 最小可验收能力
- 可解释的调度/模型决策
- 可诊断的失败原因
- 恢复后一致继续
- 高风险任务可开启双模型协作审阅

### 验收指标
| 指标 | 目标 | 测试方法 |
|------|------|----------|
| 端到端成功交付率 | ≥ 90% | ≥50 workflow |
| 恢复后继续成功率 | ≥ 97% | 中断恢复测试 |
| 无效用户打扰 | ≤ 1 次/工作流 | UX 测试 |
| 多模型策略误路由率 | ≤ 10% | 路由测试 |
| v2.0 兼容命令通过率 | 100% | 兼容性测试 |

### 风险与缓解
| 风险 | 缓解措施 |
|------|----------|
| 模型编排复杂度上升 | 先以影子模式旁路观测 |
| 策略漂移 | 保留单模型硬回退 |

---

## 兼容策略

### 原则
1. **增量交付** - 每阶段发可用版本
2. **向后兼容** - 全程保留 `compat_mode` 与串行回退
3. **灰度发布** - 新能力先旁路/小流量观察，指标达标再默认开启

### 版本兼容矩阵
| 用户版本 | v2.1.0 | v2.5.0 | v3.0.0 |
|----------|--------|--------|--------|
| v2.0 命令 | ✅ | ✅ | ✅ |
| v2.0 文件格式 | ✅ | ✅ | ✅ |
| 串行回退 | ✅ | ✅ | ✅ |

---

## 里程碑

- [x] **Week 4**: v2.1.0 发布 (状态机内核) ✅ 2026-02-09
- [x] **Week 8**: v2.5.0 发布 (并行调度 + Token/时延治理) ✅ 2026-02-09
- [x] **Week 8.5**: v2.6.0 收尾发布（启动闭环 + UNDERSTAND + 桥接） ✅ 2026-02-09
- [ ] **Week 12**: v3.0.0 GA 发布

### v2.6.0 收尾发布清单（已完成）

- ✅ Hook Runtime 模块路径断层修复（外部 cwd 可用）
- ✅ UNDERSTAND 阶段最小执行器落地并接入 `fusion-start.sh`
- ✅ `codeagent-wrapper` 桥接脚本落地（主后端+回退+会话持久化）
- ✅ 启动接线自动读取 `scheduler/budget/backend` 配置
- ✅ `fusion-status.sh` 增加 runtime/scheduler 可观测摘要
- ✅ 当时全量测试记录通过

---

## 参考

- [DESIGN.md](./DESIGN.md) - v2.0 架构设计
- [EXECUTION_PROTOCOL.md](./EXECUTION_PROTOCOL.md) - 8 阶段执行协议
- [docs/COMPATIBILITY.md](./docs/COMPATIBILITY.md) - 跨平台兼容性报告
- [docs/RUNTIME_KERNEL_DESIGN.md](./docs/RUNTIME_KERNEL_DESIGN.md) - v2.1.0 内核设计
- [docs/UPGRADE_v2_COMPAT.md](./docs/UPGRADE_v2_COMPAT.md) - v2.0→v2.1.0 升级指南
- [CHANGELOG.md](./CHANGELOG.md) - 变更日志
