# 贡献指南（Fusion Skill）

感谢你为 Fusion Skill 做贡献。

## 开始之前

- 先阅读 [`README.zh-CN.md`](README.zh-CN.md) 和 [`SKILL.md`](SKILL.md)。
- 对于较大功能变更，建议先提 Issue 讨论。
- 尽量保持 PR 小而聚焦。

## 开发与验证

1. 克隆仓库并进入目录。
2. 准备本地工具（`bash`、Rust stable toolchain，以及可选的 `jq`，仅用于 machine JSON smoke 或人工 JSON 检查）。
3. 运行时状态统一放在 `.fusion/`；版本库中以 `templates/` 为模板真源，不要把实时会话产物重新提交到仓库根目录。
4. `rust/target/`、`rust/.cargo-codex/` 这类本地 Rust cache 属于机器生成状态，应继续保持忽略，也不要把它们当作仓库结构证据。
5. `.ace-tool/`、`.claude/settings.json`、`.claude/settings.local.json` 属于宿主本地工具状态，也应继续保持忽略；只有 `.claude/settings.example.json` 保持为受版本控制的模板。
6. 如果需要放一个受版本控制的示例布局，请放到 `examples/`（例如 `examples/root-session/README.md`），不要重新引入可变的根目录运行时文件。
7. 运行测试：

```bash
cd rust
cargo test --release
```

## 代码约定

- 与现有代码风格保持一致。
- 优先修复根因，避免表层补丁。
- Hook/Runtime 相关逻辑必须故障安全（异常不应阻塞主流程）。
- 变更行为时必须补充或更新测试。
- 不在同一个 PR 里做无关重构。

## PR 自检清单

- [ ] 标题和描述清晰
- [ ] 测试已补充/更新
- [ ] 本地 `cd rust && cargo test --release` 通过
- [ ] 若修改 Rust 代码，已执行 `cd rust && cargo test --release`
- [ ] 文档已同步（README / CHANGELOG / 相关文档）
- [ ] 若活文档或仓库/runtime 契约发生变化，已同步更新 `rust/crates/fusion-cli/tests/repo_contract.rs`
- [ ] 未提交敏感信息

## 提交信息建议

推荐使用 Conventional Commits：

- `feat: ...`
- `fix: ...`
- `docs: ...`
- `test: ...`
- `refactor: ...`

## Bug 反馈建议

请尽量提供：

- 运行环境（OS、shell、`fusion-bridge --version` 或构建来源）
- 复现步骤
- 期望行为与实际行为
- 相关日志（可附 `.fusion/events.jsonl` 片段）

## 安全问题

请不要在公开 Issue 中披露安全漏洞。
请按 [`SECURITY.md`](SECURITY.md) 的流程私下报告。
