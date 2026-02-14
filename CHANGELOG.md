# Changelog

All notable changes to Fusion Skill will be documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/).



## [v2.6.3] - 2026-02-10

> Rust Bridge MVP landed: parity-first binary path for bootstrap/runtime/hooks.

### Added

- **Rust workspace (MVP)** under `rust/`
  - `fusion-cli` (`fusion-bridge` binary)
  - `fusion-provider` (wrapper resolution, backend execution, fallback helpers)
  - `fusion-runtime-io` (`.fusion` JSON/YAML I/O + dependency report schema)
- **`fusion-bridge init/start`** commands (MVP bootstrap path)
  - `init` initializes `.fusion/` from templates
  - `start` writes goal/workflow/session phase and enters `INITIALIZE`
- **`fusion-bridge status`** command (parity-oriented)
  - Runtime summary
  - Safe backlog latest injection summary
  - Dependency report summary
- **`fusion-bridge hook pretool/posttool/stop-guard`** commands (Rust native hook path)
  - pretool: context injection
  - posttool: progress delta + safe backlog + supervisor advisory
  - stop-guard: block/allow JSON + phase correction
- **`fusion-bridge codeagent`** command (parity-oriented)
  - `codeagent-wrapper` auto-discovery
  - primary backend failover to fallback backend
  - session id extraction and write-back to `sessions.json`
  - missing dependency report generation (`.fusion/dependency_report.json`)
- **Rust integration tests**
  - missing wrapper -> dependency report
  - backend fallback -> `claude_session` update
  - status output -> dependency report section
  - hook no-progress path -> safe backlog injection
  - hook stop-guard -> block decision JSON

### Changed

- Docs index now links Rust bridge usage:
  - `rust/README.md`
  - `docs/RUST_FUSION_BRIDGE_ROADMAP.md`
- `.gitignore` now excludes `rust/target/`
- Hook shell scripts now support `runtime.engine: rust` with automatic Python/Shell fallback

### Verification

- Rust tests: `cd rust && cargo test` -> passed
- Existing runtime tests: `pytest -q` -> `317 passed`


## [v2.6.2] - 2026-02-10

> Strict batch barrier + fail-fast scheduling for safe parallel orchestration.

### Added

- **严格批次屏障 (Strict Batch Barrier)**
  - 活动批次未全部结算前，不派发下一批任务
  - 批次派发后任务标记为 `IN_PROGRESS`，避免重复派发
- **调度可观测性增强**
  - `get_progress()` 新增 `active_batch` 与 `fail_fast_halted`

### Changed

- **`pick_next_batch()` 调度语义更新**
  - 先执行批次屏障结算，再判断是否允许继续调度
  - `fail_fast=true` 且检测到失败时停止后续派发
- **`on_batch_done()` 兼容行为收敛**
  - 保留接口，只有在活动批次已结算时才推进批次计数

### Verification

- 调度专项测试：`46 passed`
- 全量测试：`312 passed`
- 对应提交：`1070413`

## [v2.6.1] - 2026-02-10

> Additive virtual supervisor advisory mode for long-running autonomous loops.

### Added

- **虚拟监督官 advisory 模式（增补式）**
  - 新增模块：`scripts/runtime/supervisor.py`
  - 默认关闭，开启后仅输出建议，不接管主流程
- **compat_v2 监督建议事件**
  - 新增 `SUPERVISOR_ADVISORY` 事件写入 `.fusion/events.jsonl`
- **配置与测试扩展**
  - `scripts/runtime/config.py` 增加 supervisor 配置项
  - 新增/更新测试覆盖 supervisor + compat + config

### Verification

- 全量测试：`305 passed`
- 对应提交：`d966436`
- 发布标签：`v2.6.1`

## [v2.6.0] - 2026-02-09

> 收尾发布版本：启动闭环、UNDERSTAND 执行器、运行时配置统一、脚本可观测性增强

### Added

- **统一配置加载器**: `scripts/runtime/config.py`
  - 统一读取 `.fusion/config.yaml`
  - 支持 PyYAML 与轻量 fallback 解析
- **Safe Backlog 托底系统（防停摆）**
  - 新模块：`scripts/runtime/safe_backlog.py`
  - 支持 `quality/documentation/optimization` 三类低风险任务自动发现
  - 任务注入写入 `task_plan.md`，并打上 `[SAFE_BACKLOG]`
  - 状态持久化到 `.fusion/safe_backlog.json`
  - 注入事件写入 `events.jsonl`：`SAFE_BACKLOG_INJECTED`
- **托底反机械与统计化调度**
  - 类别轮转（diversity rotation）
  - 新颖窗口去重（novelty window）
  - 候选优先级评分（`priority_score`）
  - 停滞评分（`stall_score`）
- **指数退避（Exponential Backoff）托底控制**
  - 冷却参数：`backoff_base_rounds/backoff_max_rounds/backoff_jitter`
  - 强制探测：`backoff_force_probe_rounds`
  - 真实进展时自动复位 backoff
- **UNDERSTAND 最小执行器**: `scripts/runtime/understand.py`
  - 启动后自动评分、上下文扫描、写入 `findings.md`
  - 自动派发 `UNDERSTAND_DONE` 推进到 `INITIALIZE`
- **codeagent-wrapper 桥接脚本**: `scripts/fusion-codeagent.sh`
  - 支持主后端调用、失败回退、会话 ID 落盘
  - 区分 `codex_session` / `claude_session`
- **虚拟监督官（增补式 advisory）**
  - 新模块：`scripts/runtime/supervisor.py`
  - 默认关闭，开启后仅输出建议，不直接改任务状态
  - 事件写入 `events.jsonl`：`SUPERVISOR_ADVISORY`

- **脚本级回归测试**
  - `scripts/runtime/tests/test_fusion_codeagent_script.py`
  - `scripts/runtime/tests/test_fusion_status_script.py`
  - `scripts/runtime/tests/test_understand.py`
  - `scripts/runtime/tests/test_config_loader.py`

### Changed

- **Hook Runtime 路径修复**
  - `fusion-pretool.sh` / `fusion-posttool.sh` / `fusion-stop-guard.sh`
  - 统一注入 `PYTHONPATH=$SCRIPT_DIR`，避免从外部 cwd 调用时模块不可达
- **Kernel 自动调度器接线**
  - `create_kernel()` 自动读取 scheduler/budget/backend 配置并初始化
  - `init_scheduler()` 支持 `default_backend`
- **初始化配置升级**
  - `fusion-init.sh` 生成 `runtime/scheduler/budget` 配置段
  - `templates/config.yaml` 默认开启 `runtime` 与 `scheduler`
- **状态查看增强**
  - `fusion-status.sh` 增加 runtime/scheduler 摘要输出
  - `fusion-status.sh` 增加 safe backlog 最近注入摘要
    - `safe_backlog.last_added`
    - `safe_backlog.last_injected_at`
    - `safe_backlog.last_injected_at_iso`

### Verification

- 全量测试：`283 passed`
- 关键新增回归覆盖：Hook 路径、UNDERSTAND 执行、配置解析、codeagent 桥接、status 输出

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
