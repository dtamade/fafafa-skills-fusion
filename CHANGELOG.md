# Changelog

All notable changes to Fusion Skill will be documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/).

## [v2.1.0] - 2026-02-09

> Phase 1: Kernel Beta — 把 8 阶段从 Prompt 约束升级为可执行 FSM

### Added

- **有限状态机 (FSM)**: 13 状态、18 事件、36 条转移规则，含守卫条件
  - `scripts/runtime/state_machine.py`
- **运行时内核**: `FusionKernel` — 统一的状态转移执行器
  - `scripts/runtime/kernel.py`
  - 支持 `dispatch(event, payload, idempotency_key)` 幂等派发
- **事件溯源**: append-only `events.jsonl`，支持状态从日志重建
  - `scripts/runtime/session_store.py`
- **事件总线**: 发布/订阅模式，支持 `state_changed` 等内部事件
  - `scripts/runtime/event_bus.py`
- **v2 兼容适配层**: Shell→Python 桥接 (Strangler Fig Pattern)
  - `scripts/runtime/compat_v2.py`
  - 三个适配函数: `adapt_stop_guard`, `adapt_pretool`, `adapt_posttool`
- **阶段自动纠正**: 检测状态不一致并自动修复
  - EXECUTE + 全部完成 → 派发 `ALL_TASKS_DONE`
  - VERIFY + 有 PENDING → 派发 `VERIFY_FAIL`
- **Runtime 开关**: `config.yaml` 中 `runtime.enabled` 控制，默认关闭
- **回归测试套件**: 35 场景全量回归 + 20 次恢复可靠性测试
  - `scripts/runtime/regression_runner.py`
- **性能基准**: Hook 延迟基准测试工具
  - `scripts/runtime/bench_hook_latency.py`
- **端到端 CLI 测试**: 13 个 Shell→Python CLI 集成测试
  - `scripts/runtime/tests/test_hook_adapter.py`

### Changed

- `fusion-stop-guard.sh`: 新增 runtime adapter 分支，Python 失败时 fallback 到原 Shell 逻辑
- `fusion-pretool.sh`: 同上
- `fusion-posttool.sh`: 同上
- `sessions.json`: 新增 `_runtime` 字段 (`version`, `state`, `last_event_counter`)

### Fixed

- `seq` 命令兼容性 (Windows Git Bash)
- `md5sum`/`md5` 跨平台差异 (macOS)
- `python3`/`python` 命令检测 (Windows)
- Unicode 进度条字符 fallback (非现代终端)

### Performance

- PreToolUse (pretool) p95: **0.27ms** (阈值 80ms)
- PostToolUse (posttool) p95: **0.21ms** (阈值 80ms)
- StopHook (stop-guard) p95: **0.40ms** (阈值 150ms)

### Test Coverage

| 组件 | 测试数 | 通过率 |
|------|--------|--------|
| FSM + Kernel (Week 1) | 35 | 100% |
| EventBus + SessionStore + Resume (Week 2) | 70 | 100% |
| compat_v2 + Hook adapter (Week 3) | 34 | 100% |
| **Total** | **139** | **100%** |

回归: 35/35 场景 (100%) | 恢复可靠性: 20/20 (100%)

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
