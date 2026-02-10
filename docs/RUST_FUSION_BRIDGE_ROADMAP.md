# Fusion Bridge Rust 二进制路线图（v0.1）

> 日期：2026-02-10  
> 状态：Draft（执行前设计）  
> 目标：在不破坏现有 Python/Shell 版本的前提下，增补一个可渐进切换的 Rust 二进制桥接层。

---

## 1. 已确认的方向（来自本轮决策）

1. **双轨并存**：默认走内置 provider，异常时回退 `codeagent-wrapper`。  
2. **增补不替换**：现有 `scripts/*.sh` + `scripts/runtime/*.py` 保留，作为验证过的参考实现。  
3. **可回退优先**：任何阶段都必须可一键退回 Python/Shell 主路径。  
4. **先对齐行为再追求性能**：先做语义一致（parity），再做性能优化。

---

## 2. 为什么做 Rust Bridge

- 降低 Python 运行时依赖，减少跨平台脚本失败概率。  
- 将关键 Hook 路径收敛到单二进制，提升稳定性和可观测性。  
- 统一 provider 编排层，后续更容易接入新后端。  
- 保留现有验证成果，避免重写带来的功能回退。

**非目标（当前阶段）**：

- 不一次性删除现有 Python/Shell。  
- 不在首版引入复杂 UI 或远程控制面板。  
- 不追求“所有模块一次 Rust 化”。

---

## 3. 目标架构（Rust Workspace）

```text
rust/
├── Cargo.toml
└── crates/
    ├── fusion-cli/          # 命令入口（run/resume/status/hook/doctor）
    ├── fusion-core/         # 状态/任务编排核心（与现有 runtime 行为对齐）
    ├── fusion-provider/     # provider trait + native + wrapper fallback
    ├── fusion-runtime-io/   # .fusion 文件读写、events.jsonl、dependency_report
    └── fusion-hook/         # pretool/posttool/stop-guard 适配
```

### 3.1 Provider 策略（关键）

默认策略：`native_first`（内置 provider 优先）

```text
try native provider
  ├─ success -> continue
  └─ fail (credential/network/rate-limit/transient)
      -> try wrapper provider
          ├─ success -> continue + record fallback event
          └─ fail -> write .fusion/dependency_report.json + return actionable error
```

### 3.2 凭证与配置优先级

推荐优先级：

1. 环境变量（CI/本地安全首选）
2. `.fusion/config.local.yaml`（本地私有，不入库）
3. `.fusion/config.yaml`（可分享默认）

建议字段（首版最小集）：

- `provider.mode`: `native_first | wrapper_first | native_only | wrapper_only`
- `provider.native.base_url`
- `provider.native.api_key`
- `provider.native.model`
- `provider.wrapper.bin`

---

## 4. Python/Shell 到 Rust 的映射基线

| 现有模块 | Rust 对应 | 说明 |
|---|---|---|
| `scripts/fusion-start.sh` | `fusion-cli run` | 启动流程与阶段推进 |
| `scripts/fusion-resume.sh` | `fusion-cli resume` | 恢复中断工作流 |
| `scripts/fusion-status.sh` | `fusion-cli status` | 统一状态输出 |
| `scripts/fusion-codeagent.sh` | `fusion-provider` | provider 选择、fallback、依赖诊断 |
| `scripts/fusion-pretool.sh` | `fusion-hook pretool` | 工具前上下文注入 |
| `scripts/fusion-posttool.sh` | `fusion-hook posttool` | 工具后进度探测 |
| `scripts/fusion-stop-guard.sh` | `fusion-hook stop-guard` | 停止拦截与继续策略 |
| `scripts/runtime/compat_v2.py` | `fusion-hook` + `fusion-core` | Shell↔Runtime 兼容桥 |
| `scripts/runtime/safe_backlog.py` | `fusion-core::safe_backlog` | 无进展托底注入 |
| `scripts/runtime/supervisor.py` | `fusion-core::supervisor` | advisory 监督建议 |

---

## 5. 分阶段路线图

## Phase 0（1 周）：基线冻结与行为快照

**目标**：冻结当前 Python/Shell 语义，建立 parity 参考。  
**交付**：

- 提取关键流程金样本（start/status/stop-guard/fallback）。
- 形成“行为等价清单”（输入条件 → 预期输出/状态）。
- 明确不可回归项（兼容、恢复、托底）。

**验收**：基线回归集全部可重复执行。

## Phase 1（1-2 周）：Rust CLI + Provider MVP

**目标**：先打通最短闭环，不动复杂调度。  
**交付**：

- `fusion-bridge run/status/doctor` 命令。
- `native_first + wrapper fallback` 策略。
- `.fusion/dependency_report.json` 对齐输出。

**验收**：

- wrapper 缺失时可生成可执行修复建议。
- native 凭证缺失时能自动回退 wrapper（若可用）。

## Phase 2（1-2 周）：Hook 链路 Rust 化

**目标**：覆盖高频 Hook，降低脚本依赖。  
**交付**：

- `fusion-bridge hook pretool`
- `fusion-bridge hook posttool`
- `fusion-bridge hook stop-guard`

**验收**：

- 与 `compat_v2` 关键决策一致率 ≥ 98%。
- Hook p95 延迟优于 Python 版本。

## Phase 3（2 周）：核心编排能力迁移

**目标**：迁移长期自治关键特性。  
**交付**：

- `safe_backlog`（含 backoff/jitter/probe）
- `supervisor` advisory
- 基础 scheduler/budget 接口兼容

**验收**：

- 无进展场景可稳定注入低风险任务。
- 不引入“机械重复”退化行为。

## Phase 4（1 周）：灰度切换与回退闭环

**目标**：让用户可配置选择引擎，并可快速回退。  
**交付**：

- `runtime.engine: python | rust | auto`
- 文档与发布脚本更新
- 迁移指南与故障处理手册

**验收**：

- `runtime.engine=python` 100% 可回退。
- `runtime.engine=rust` 能稳定完成端到端小目标流程。

---

## 6. 测试与验收策略

1. **Parity 测试（必须）**  
   用同一输入驱动 Python 与 Rust，比较：
   - phase 迁移结果
   - stop-guard 决策
   - dependency report 字段

2. **故障注入测试**  
   - 无凭证
   - wrapper 缺失
   - 网络超时
   - 同错误重复（3-Strike）

3. **性能与稳定性**  
   - Hook p95/p99
   - 长循环 100+ 轮稳定性
   - 事件日志完整性（`events.jsonl`）

4. **跨平台验证（最小）**  
   - Linux（优先）
   - macOS
   - Windows（后续补齐）

---

## 7. 关键风险与缓解

| 风险 | 描述 | 缓解 |
|---|---|---|
| 行为漂移 | Rust 与 Python 语义不一致 | parity-first + 金样本对比 |
| 凭证复杂度 | HTTP provider 需要用户凭证 | 环境变量优先 + 自动 fallback wrapper |
| 迁移过快 | 一次替换引发回归 | 双轨并存 + 配置灰度 |
| 调度乱套 | 并发任务相互干扰 | 仅允许无冲突并发 + 批次栅栏（全部完成后再推进） |
| 可维护性下降 | 复杂度转移到新仓 | 分 crate 边界 + 强测试门禁 |

---

## 8. 首批执行任务（建议）

1. 建立 `rust/` workspace 与基础 CI。  
2. 实现 `fusion-cli status` 读取 `.fusion/sessions.json`。  
3. 实现 `fusion-provider` 的 `native_first` + `wrapper fallback`。  
4. 对齐 `.fusion/dependency_report.json` schema。  
5. 实现 `hook stop-guard` 的最小等价逻辑。  
6. 增加 parity 测试：先覆盖 10 个关键场景。

---

## 9. 发布策略

- **v2.x（当前）**：Python/Shell 默认，Rust 作为实验开关。  
- **v2.x+1**：Rust 可选默认（`auto`），Python 作为强回退。  
- **v3.0**：Rust 成为主路径，Python 保留兼容周期后再评估退役。

---

## 10. 结论

这条路线是可行的，且风险可控：

- 不破坏现有能力（增补式迁移）
- 能逐步降低依赖与平台不确定性
- 可在每个阶段都交付“可运行、可回退、可验证”的结果

如果后续确认进入实现阶段，建议先按 **Phase 0 → Phase 1** 落地，优先拿到 provider 与 dependency doctor 的 Rust MVP 闭环。
