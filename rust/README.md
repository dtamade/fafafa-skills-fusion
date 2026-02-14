# Fusion Rust Bridge (MVP)

`fusion-bridge` 是 Fusion 的 Rust 二进制桥接器，目标是逐步替代当前 Python/Shell 路径。

## 当前实现（MVP）

- `fusion-bridge init`：初始化 `.fusion/`（从 `templates/` 拷贝模板）
- `fusion-bridge start`：写入 goal/workflow/status，并进入 `INITIALIZE`
- `fusion-bridge status`：读取 `.fusion/` 并输出运行状态（含 Runtime / Dependency Report / Safe Backlog 摘要）
- `fusion-bridge run`：执行自治循环（stop-guard → pretool → codeagent → posttool）
- `fusion-bridge resume`：从 `paused` 或 `in_progress` 状态继续执行（内部复用 `run`）
- `fusion-bridge codeagent`：对齐 `fusion-codeagent.sh` 行为
  - 自动定位 `codeagent-wrapper`
  - 主后端失败后回退后端
  - 写回 `sessions.json` 的会话 ID
  - 缺依赖时写入 `.fusion/dependency_report.json`
- `fusion-bridge hook pretool/posttool/stop-guard/set-goal`：Rust 原生 Hook 适配（不依赖 Python）
  - pretool: 输出当前目标/阶段/进度上下文
  - posttool: 检测进度变化，支持 safe backlog 注入与 supervisor advisory
  - stop-guard: 输出 block/allow 决策 JSON（含 phase 自动纠正）
  - set-goal: 写入 `sessions.json.goal`

## 构建

```bash
cd rust
cargo build
```

## 运行

```bash
cd /path/to/repo
cargo run -q -p fusion-cli -- init --fusion-dir .fusion --templates-dir templates
cargo run -q -p fusion-cli -- start "实现用户认证" --fusion-dir .fusion --templates-dir templates --force
cargo run -q -p fusion-cli -- status --fusion-dir .fusion
cargo run -q -p fusion-cli -- run --fusion-dir .fusion
cargo run -q -p fusion-cli -- resume --fusion-dir .fusion
cargo run -q -p fusion-cli -- codeagent EXECUTE --fusion-dir .fusion

# Hooks
cargo run -q -p fusion-cli -- hook pretool --fusion-dir .fusion
cargo run -q -p fusion-cli -- hook posttool --fusion-dir .fusion
cargo run -q -p fusion-cli -- hook stop-guard --fusion-dir .fusion
```

## 验证

```bash
cd rust
cargo test
```
