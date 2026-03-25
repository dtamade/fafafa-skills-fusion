# Fusion Bridge Rust 二进制路线图（v0.1）

> 日期：2026-02-10  
> 状态：Draft（执行前设计，含历史阶段性假设）  
> 目标：在不破坏当时现有双轨实现的前提下，增补一个可渐进切换的 Rust 二进制桥接层。
> 当前执行真源：当前 live 执行顺序与发布收口应以 `docs/V3_GA_EXECUTION_ROADMAP.md` 为准。
> 说明：本文保留了路线图撰写当时的阶段目标；若与当前仓库实现冲突，应以 `docs/CLI_CONTRACT_MATRIX.md`、`README.md`、`README.zh-CN.md` 和实际脚本行为为准。
> 当前实现注记：Rust 已成为主控制面；Shell 仅保留 live fallback 与 thin wrapper 职责；旧 runtime/reference 文件已从仓库移除。下文若出现更宽泛的双轨描述，应按历史阶段性口径理解。

---

## 1. 已确认的方向（来自本轮决策）

1. **双轨并存**：默认走内置 provider，异常时回退 `codeagent-wrapper`。
2. **增补不替换（历史阶段）**：迁移初期曾并行保留 shell 与旧 reference 实现；当前仓库已收敛为 Rust 主线 + Shell 薄包装。
3. **可回退优先（历史目标）**：优先保留 hook/兼容路径的回退能力；当前控制面薄包装脚本已收敛为 Rust 主线，不再以旧双轨实现作为对等主路径。
4. **先对齐行为再追求性能**：先做语义一致（parity），再做性能优化。

---

## 2. 为什么做 Rust Bridge

- 降低额外运行时依赖，减少跨平台脚本失败概率。
- 将关键 Hook 路径收敛到单二进制，提升稳定性和可观测性。
- 统一 provider 编排层，后续更容易接入新后端。
- 保留现有验证成果，避免重写带来的功能回退。

**非目标（当前阶段）**：

- 不一次性删除当时现有双轨实现。
- 不在首版引入复杂 UI 或远程控制面板。
- 不追求“所有模块一次 Rust 化”。

---

## 3. 目标架构（Rust Workspace）

> 注：本节中的 crate 名与命令集合保留路线图撰写时的计划命名；当前 live 对外命令面仍以 `docs/CLI_CONTRACT_MATRIX.md`、Shell thin wrapper 映射和实际 CLI 为准，不能把这里的 crate 名直接当成用户入口。

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
2. `.fusion/config.local.yaml`（宿主本地 override，如存在则优先于工作区默认，不入库）
3. `.fusion/config.yaml`（工作区生成配置；由受版本控制的 `templates/config.yaml` 初始化，不是模板真源）
4. `templates/config.yaml`（受版本控制基线；供 `fusion-bridge init` / `scripts/fusion-init.sh` 生成 `.fusion/config.yaml`）

建议字段（首版最小集）：

- `provider.mode`: `native_first | wrapper_first | native_only | wrapper_only`
- `provider.native.base_url`
- `provider.native.api_key`
- `provider.native.model`
- `provider.wrapper.bin`

---

## 4. 历史实现到 Rust 的映射基线

| 现有模块                               | Rust 对应                                            | 说明                                                                                     |
| -------------------------------------- | ---------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `scripts/fusion-start.sh`              | `fusion-bridge start`                                | 启动流程与阶段推进                                                                       |
| `scripts/fusion-resume.sh`             | `fusion-bridge resume`                               | 恢复中断工作流                                                                           |
| `scripts/fusion-status.sh`             | `fusion-bridge status`                               | 统一状态输出                                                                             |
| `scripts/fusion-codeagent.sh`          | `fusion-bridge codeagent`                            | provider 选择、fallback、依赖诊断                                                        |
| `scripts/fusion-pretool.sh`            | `fusion-bridge hook pretool`                         | 工具前上下文注入                                                                         |
| `scripts/fusion-posttool.sh`           | `fusion-bridge hook posttool`                        | 工具后进度探测                                                                           |
| `scripts/fusion-stop-guard.sh`         | `fusion-bridge hook stop-guard`                      | 停止拦截与继续策略                                                                       |
| historical hook parity/reference layer | removed                                              | 旧 runtime/reference 层已移除；对应验证由 shell wrapper smoke + Rust contract tests 承担 |
| previous safe backlog live path        | retired in favor of Rust `fusion-core::safe_backlog` | 无进展托底 live 路径已迁到 Rust                                                          |
| previous supervisor live path          | retired in favor of Rust `fusion-core::supervisor`   | advisory 监督建议 live 路径已迁到 Rust                                                   |

> 收缩进展：旧 runtime/reference 文件与对应单元测试已从仓库删除；当前回归契约由 Rust `fusion-bridge regression`、Rust `cli_smoke`、Rust `repo_contract`、Rust `shell_contract` 与 shell contract checks 承担。
> 若这条映射基线或当前回归契约发生变化，请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`。

---

## 5. 分阶段路线图

## Phase 0（1 周）：基线冻结与行为快照

**目标**：冻结当前历史双轨语义，建立 parity 参考。  
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

> 当前实现注记：这里的 `run/status/doctor` 是路线图阶段性桥接层目标；当前用户可见入口仍以 `fusion-start.sh`、`fusion-status.sh`、`fusion-resume.sh`、`fusion-codeagent.sh` 及 `docs/CLI_CONTRACT_MATRIX.md` 中列出的命令面为准。

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

- 与现行 shell hook fallback / Rust regression 契约关键决策一致率 ≥ 98%。
- Hook p95 延迟优于旧实现版本。

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

**目标（历史规划）**：让用户可配置选择引擎，并可快速回退。当前 live 配置文档已不再公开多 engine 选择；这里只保留迁移阶段设想。  
**交付**：

- 保留一个旧 engine 枚举值与 `rust | auto` 并存（历史规划；当前主线文档已不再把其他 engine 作为 live 选项）
- 文档与发布脚本更新
- 迁移指南与故障处理手册

**验收**：

- hook / parity / tooling 路径可按配置回退。
- 历史规划里的 `runtime.engine=rust` 验收，当前应理解为 Rust 主线配置能稳定完成端到端小目标流程。

---

## 6. 测试与验收策略

1. **Parity 测试（必须）**  
   用同一输入驱动历史参考实现与 Rust，比较：
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

| 风险         | 描述                       | 缓解                                            |
| ------------ | -------------------------- | ----------------------------------------------- |
| 行为漂移     | Rust 与历史参考语义不一致  | parity-first + 金样本对比                       |
| 凭证复杂度   | HTTP provider 需要用户凭证 | 环境变量优先 + 自动 fallback wrapper            |
| 迁移过快     | 一次替换引发回归           | 双轨并存 + 配置灰度                             |
| 调度乱套     | 并发任务相互干扰           | 仅允许无冲突并发 + 批次栅栏（全部完成后再推进） |
| 可维护性下降 | 复杂度转移到新仓           | 分 crate 边界 + 强测试门禁                      |

---

## 8. 首批执行任务（建议）

1. 建立 `rust/` workspace 与基础 CI。
2. 实现 `fusion-bridge status` 读取 `.fusion/sessions.json`。
3. 实现 `fusion-provider` 的 `native_first` + `wrapper fallback`。
4. 对齐 `.fusion/dependency_report.json` schema。
5. 实现 `hook stop-guard` 的最小等价逻辑。
6. 增加 parity 测试：先覆盖 10 个关键场景。

---

## 9. 发布策略

- **v2.x（路线图撰写时）**：双轨实现默认，Rust 作为实验开关。
- **当前仓库状态**：Rust 已成为控制面主路径；Shell 主要保留在 hook fallback 与 thin wrapper。
- **后续方向**：继续收缩旧直连面，最终只保留必要兼容周期。

---

## 10. 结论

这条路线是可行的，且风险可控：

- 不破坏现有能力（增补式迁移）
- 能逐步降低依赖与平台不确定性
- 可在每个阶段都交付“可运行、可验证”的结果；回退能力按路径区分，而非默认要求所有入口都回到旧双轨主实现

如果后续确认进入实现阶段，建议先按 **Phase 0 → Phase 1** 落地，优先拿到 provider 与 dependency doctor 的 Rust MVP 闭环。
