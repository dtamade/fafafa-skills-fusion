# Changelog

All notable changes to Fusion Skill will be documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/).

## [v2.5.0] - 2026-02-09

> Phase 2: 并行调度 + Token/时延治理 — 从串行升级为可控并行

### Added

- **DAG 任务图编译器**: 从 task_plan.md 解析依赖关系，Kahn 拓扑排序产出并行批次
  - `scripts/runtime/task_graph.py`
  - 支持循环检测、悬空依赖验证、重复依赖去重
- **文件冲突检测器**: writeset 交集检测，贪心分区产出无冲突子集
  - `scripts/runtime/conflict_detector.py`
- **Token/时延预算管理**: 全局预算追踪、超预算检测、降级建议
  - `scripts/runtime/budget_manager.py`
- **模型路由**: 基于任务类型和预算状态的后端选择 (codex/claude)
  - `scripts/runtime/router.py`
  - 路由优先级: 超预算 → 用户指定 → 预算警告 → 类型规则
- **并行调度器**: 综合 DAG/冲突/预算/路由的批次决策中心
  - `scripts/runtime/scheduler.py`
  - 决策管道: DAG ready → 冲突过滤 → 预算检查 → 并行度限制 → 模型路由
  - `scheduler.enabled=false` 退化为 v2.1.0 串行行为
- **Kernel 调度器集成**: 可选接入 Scheduler 的 EXECUTE 循环
  - `init_scheduler()`, `get_next_batch()`, `complete_task()`, `fail_task()`
  - 调度器状态同步到 `sessions.json` `_runtime.scheduler`
- **compat_v2 批次感知**: pretool 输出批次/并行信息，posttool 感知批次完成
- **并行模拟验收**: 50 个随机 DAG 工作流的端到端模拟
  - `scripts/runtime/bench_parallel_sim.py`

### Changed

- `StateMachineContext`: 新增 `scheduler_enabled`, `current_batch_id`, `parallel_tasks`, `total_batches` 字段
- `kernel.py`: 新增可选 Scheduler 集成方法，不影响原有 `dispatch()` 路径
- `compat_v2.py`: pretool/posttool 扩展显示调度器批次信息
- `__init__.py`: 导出所有 Phase 2 模块
- `templates/config.yaml`: 新增 `scheduler` 和 `budget` 配置段
- `regression_runner.py`: 扩展至 60 场景 (Phase 1: 35 + Phase 2: 25)
- `bench_hook_latency.py`: 新增 Scheduler 决策延迟基准

### Performance

- Scheduler.pick_next_batch() p95: **0.09ms** (20 tasks, 阈值 200ms)
- PreToolUse (pretool) p95: **0.36ms** (阈值 80ms)
- PostToolUse (posttool) p95: **0.34ms** (阈值 80ms)
- StopHook (stop-guard) p95: **0.47ms** (阈值 150ms)

### Acceptance Metrics

| 指标 | 实际 | 目标 |
|------|------|------|
| DAG 依赖违规数 | 0 | = 0 |
| 中位加速比 | 2.00x | ≥ 1.4x |
| 冲突回滚率 | 4.8% | ≤ 5% |
| Token 超支率 | 6.0% | ≤ 10% |
| 硬上限突破 | 0 | = 0 |
| 调度决策 p95 | 0.09ms | < 200ms |
| v2.1.0 回归 | 139/139 | 全部通过 |

### Test Coverage

| 组件 | 测试数 | 通过率 |
|------|--------|--------|
| FSM + Kernel (v2.1.0) | 139 | 100% |
| DAG 任务图 (Week 5) | 39 | 100% |
| 冲突检测 (Week 5) | 15 | 100% |
| 预算管理 (Week 6) | 24 | 100% |
| 模型路由 (Week 6) | 12 | 100% |
| 调度器 (Week 6) | 16 | 100% |
| 调度器集成 (Week 7) | 23 | 100% |
| **Total** | **268** | **100%** |

回归: 60/60 场景 (100%) | 恢复可靠性: 20/20 (100%)

---

## [v2.0.0] - 2026-01-xx

> 初始版本: 8 阶段自主工作流 (Shell 脚本实现)

### Added

- 8 阶段执行协议 (INITIALIZE → DELIVER)
- Stop Guard 安全机制
- PreTool/PostTool 进度监控
- Loop Guardian 循环检测
- 会话恢复 (fusion-resume.sh)
- 跨平台兼容 (Linux/macOS/Windows)
