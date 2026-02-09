# 贡献指南（Fusion Skill）

感谢你为 Fusion Skill 做贡献。

## 开始之前

- 先阅读 [`README.zh-CN.md`](README.zh-CN.md) 和 [`SKILL.md`](SKILL.md)。
- 对于较大功能变更，建议先提 Issue 讨论。
- 尽量保持 PR 小而聚焦。

## 开发与验证

1. 克隆仓库并进入目录。
2. 准备 Python 3.10+ 环境。
3. 运行测试：

```bash
pytest -q
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
- [ ] 本地 `pytest -q` 通过
- [ ] 文档已同步（README / CHANGELOG / 相关文档）
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

- 运行环境（OS、shell、Python 版本）
- 复现步骤
- 期望行为与实际行为
- 相关日志（可附 `.fusion/events.jsonl` 片段）

## 安全问题

请不要在公开 Issue 中披露安全漏洞。
请按 [`SECURITY.md`](SECURITY.md) 的流程私下报告。
