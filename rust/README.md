# Fusion Rust Bridge

`fusion-bridge` 是 Fusion 当前主控制面的 Rust 二进制桥接器。仓库中仍保留 Shell thin wrapper / hook 接线；旧 runtime/reference 层已从仓库移除，不再与 Rust 并列为主路径。

当前执行真源路线图请参考 `docs/V3_GA_EXECUTION_ROADMAP.md`；`docs/RUST_FUSION_BRIDGE_ROADMAP.md` 仅保留历史迁移背景。

## 当前实现

- `fusion-bridge init`：初始化 `.fusion/`（从受版本控制的 `templates/` 生成工作区文件）
- `fusion-bridge start`：写入 goal/workflow/status，并进入 `INITIALIZE`；文本输出会给出 `Current state` / `Next action`
- `fusion-bridge status`：读取 `.fusion/` 并输出运行状态（含 Runtime / Dependency Report / Safe Backlog 摘要）
- `fusion-bridge achievements`：输出当前工作区成就与跨项目排行榜
- `fusion-bridge run`：执行自治循环（stop-guard → pretool → codeagent → posttool）
- `fusion-bridge resume`：从 `paused` 或 `in_progress` 状态继续执行（内部复用 `run`）；文本输出会先概括 `Current state` / `Next action`
- `fusion-bridge catchup`：读取 Claude 会话 JSONL 与 `.fusion/` 规划文件，输出会话恢复报告；文本输出会先概括 `Current state` / `Next action`
- `fusion-bridge codeagent`：对齐 `fusion-codeagent.sh` 行为
  - 自动定位 `codeagent-wrapper`
  - 主后端失败后回退后端
  - 写回 `sessions.json` 的会话 ID
  - 缺依赖时写入 `.fusion/dependency_report.json`
- `fusion-bridge doctor`：输出 hook wiring / runtime workspace 诊断摘要，并支持 `--fix` 写入项目 `.claude/settings.local.json`（宿主本地 override 文件，不属于仓库输入）
- `fusion-bridge audit`：运行 release contract gates，并支持 `--dry-run` / `--json` / `--json-pretty` / `--fast` / `--skip-rust`
- `fusion-bridge selfcheck`：执行 hook doctor + stop simulation + contract regression，并支持 `--fix` / `--quick` / `--json`
- `fusion-bridge inspect`：为 Shell 兼容层提供 JSON / `task_plan.md` 读取 helper，逐步收敛文本解析
- `fusion-bridge hook pretool/posttool/stop-guard/set-goal`：Rust 原生 Hook 适配（不依赖旧 runtime helper）
  - pretool: 输出当前目标/阶段/进度上下文
  - posttool: 检测进度变化，支持 safe backlog 注入与 supervisor advisory
  - stop-guard: 输出 block/allow 决策 JSON（含 phase 自动纠正）
  - set-goal: 写入 `sessions.json.goal`

`fusion-bridge init` 会从 `templates/config.yaml` 生成 `.fusion/config.yaml`；当前模板默认值已经切到 `runtime.enabled=true`、`scheduler.enabled=true`、`safe_backlog.enabled=true`，并保持 `supervisor.enabled=false`。其中 `.fusion/config.yaml` 是工作区生成配置，`templates/config.yaml` 才是受版本控制基线。

当前仓库约定：

- Rust CLI 是唯一主实现
- Shell 仅保留 thin wrapper / Hook 接线职责
- 旧 runtime/reference 层已从仓库移除
- 当前 live 配置文档不再公开多 runtime engine 选择；`engine: "rust"` 只是单引擎主线路径标记
- `rust/target/`、`rust/.cargo-codex/` 属于本地 Rust cache，不作为仓库结构或活文档语义的一部分
- `.claude/settings.example.json` 是受版本控制的 Hook 模板；`.claude/settings.json` 与 `.claude/settings.local.json` 属于宿主本地配置，不作为仓库输入

## 构建

```bash
cd rust
cargo build --release
```

## 运行

```bash
cd /path/to/repo
cargo run --release -q -p fusion-cli -- init --fusion-dir .fusion --templates-dir templates
cargo run --release -q -p fusion-cli -- start "实现用户认证" --fusion-dir .fusion --templates-dir templates --force
cargo run --release -q -p fusion-cli -- status --fusion-dir .fusion
cargo run --release -q -p fusion-cli -- achievements --fusion-dir .fusion --local-only
cargo run --release -q -p fusion-cli -- run --fusion-dir .fusion
cargo run --release -q -p fusion-cli -- resume --fusion-dir .fusion
cargo run --release -q -p fusion-cli -- catchup --fusion-dir .fusion --project-path /path/to/repo
cargo run --release -q -p fusion-cli -- codeagent EXECUTE --fusion-dir .fusion
cargo run --release -q -p fusion-cli -- doctor --json --fix /path/to/repo

# Hooks
cargo run --release -q -p fusion-cli -- hook pretool --fusion-dir .fusion
cargo run --release -q -p fusion-cli -- hook posttool --fusion-dir .fusion
cargo run --release -q -p fusion-cli -- hook stop-guard --fusion-dir .fusion
```

## 验证

```bash
cd rust
cargo test --release
```

CI 也采用同一 release 验证策略：

```bash
cd rust
cargo test --release
```

如果这里描述的 Rust 主线路径、配置基线或验证契约发生变化，请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`。
