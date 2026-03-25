# Hook Wiring Notes

Fusion 使用标准 Hook 映射方式，不依赖 `SKILL.md` frontmatter 自动注入。

## Recommended (Claude Code)

在仓库根目录创建 `.claude/settings.json`，内容可参考：

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit|Bash|Read|Glob|Grep",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh\""
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash \"${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh\""
          }
        ]
      }
    ]
  }
}
```

你也可以直接复制 `.claude/settings.example.json` 为 `.claude/settings.json`。

其中 `.claude/settings.example.json` 是受版本控制的模板；实际 `.claude/settings.json` 与 `.claude/settings.local.json` 属于宿主本地 Hook 配置，不是仓库结构证据，应继续保持忽略。

Hook 路径契约请只保留这一种写法：

```bash
bash "${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh"
bash "${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh"
bash "${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh"
```

以下都视为旧写法，不应继续保留：

- `bash "${CLAUDE_PROJECT_DIR}/scripts/..."`
- `bash scripts/fusion-*.sh`

## Quick doctor + auto-fix

如果你怀疑 Hook 没有真正启用，可直接执行：

```bash
bash scripts/fusion-hook-doctor.sh --json --fix .
```

- `--json`: 输出机器可读诊断结果
- `--fix`: 自动写入项目 `.claude/settings.local.json`（PreToolUse/PostToolUse/Stop 三个 hooks）

`--fix` 写入的 `.claude/settings.local.json` 也属于宿主本地 override 文件，不应作为受版本控制的仓库输入。

执行后可再次运行：

```bash
bash scripts/fusion-hook-doctor.sh --json .
```

确认 `result=ok` 且 `warn_count=0`。

> 注意：Claude Code 对 hook 变更采用会话安全审查机制。首次 `--fix` 后，请在 Claude Code 中打开 `/hooks` 确认变更，然后重开会话。

## Runtime engine selection

Set `.fusion/config.yaml`:

```yaml
runtime:
  enabled: true
  compat_mode: true
  engine: "rust" # rust primary control plane; single-engine runtime
```

当前模板默认值已经启用 `runtime.enabled: true`。完整推荐基线请参考 `templates/config.yaml`。

- `engine: "rust"`: 推荐默认值；优先调用 `fusion-bridge hook ...`；三支 hook 在 bridge 失败或被禁用时都直接回退最小 Shell 路径
- thin wrapper 只会自动发现已安装的 `fusion-bridge` 或本地 `rust/target/release/` 构建，不会静默回落到 `target/debug`
- `FUSION_BRIDGE_DISABLE=1`: 强制支持该模式的 hook/runtime 入口跳过 Rust bridge；hook 三件套会回退到最小 Shell 路径，便于排障。
- live 配置已不再提供其他 runtime engine 这类运行时选择；旧 runtime/reference 层已从仓库移除
- `compat_mode: true`: 表示 Rust bridge 失效或被禁用时，仍允许 hook 和辅助路径回退到最小 Shell；仓库已不再保留任何旧 hook parity 层，控制面薄包装脚本不受该开关兜底

控制面补充说明：

- `scripts/fusion-status.sh`
- `scripts/fusion-logs.sh`
- `scripts/fusion-git.sh`
- `scripts/fusion-achievements.sh`
- `scripts/fusion-pause.sh`
- `scripts/fusion-resume.sh`
- `scripts/fusion-catchup.sh`
- `scripts/fusion-cancel.sh`
- `scripts/fusion-continue.sh`

这些脚本现在都是 thin wrapper，直接委托 `fusion-bridge <command>`；其中 `fusion-catchup.sh` 在 bridge 缺失但本地 Rust 工具链存在时，会显式走 `cargo run --release -q -p fusion-cli -- catchup ...`。其余脚本在 bridge 缺失或被禁用时返回依赖错误 `127`，不再回退到旧 Shell 主实现；`fusion-git.sh` 也已收敛为 `fusion-bridge git` 的参数校验包装层。

总结当前实现归属：

- Rust：唯一主控制面
- Shell：最小薄包装 / Hook 接线层
- Former runtime/reference layer: removed from the repository

## What is wired in this repo

- Hook scripts:
  - `scripts/fusion-pretool.sh`
  - `scripts/fusion-posttool.sh`
  - `scripts/fusion-stop-guard.sh`
- The removed compat/reference layer for hooks is no longer in the repository. Live hook fallback is now Shell-only.

## Verify wiring

Run:

```bash
bash scripts/ci-cross-platform-smoke.sh

cd rust
cargo test --release -p fusion-cli --test shell_contract
```

If these pass, the live shell hook path and wrapper smoke contracts are healthy. No separate hook parity layer remains in the repository.

## Hook debug (visible in Claude Code)

To print hook trigger diagnostics directly during Claude Code execution:

```bash
# enable
touch .fusion/.hook_debug

# disable
rm -f .fusion/.hook_debug
```

With debug enabled, each hook emits stderr traces:

- `[fusion][hook-debug][pretool] ...`
- `[fusion][hook-debug][posttool] ...`
- `[fusion][hook-debug][stop] ...`

You can review recent traces in:

```bash
/fusion status
# or
tail -n 50 .fusion/hook-debug.log
```

## Release contract audit (pre-merge)

在准备发布或合并前，建议执行：

```bash
bash scripts/release-contract-audit.sh --dry-run
bash scripts/release-contract-audit.sh
```

- 对应 CI 门禁文件：`.github/workflows/ci-contract-gates.yml`
- `release-contract-audit.sh` 现在是 `fusion-bridge audit` 的 thin wrapper
- 常用快速模式：`bash scripts/release-contract-audit.sh --fast`
- `--skip-rust` 会跳过 rust clippy/test/fmt 门禁；默认仍建议保留完整 release 验证。
- 如果 Hook 路径、宿主本地 Hook 配置边界，或相关 wrapper 契约发生变化，请同步更新 `rust/crates/fusion-cli/tests/repo_contract.rs`、`rust/crates/fusion-cli/tests/shell_contract.rs` 与本页。

## Machine-readable release checks

```bash
bash scripts/release-contract-audit.sh --dry-run --json --json-pretty --fast --skip-rust
bash scripts/ci-machine-mode-smoke.sh
fusion-bridge regression --list-suites --json
bash scripts/ci-cross-platform-json-smoke.sh /tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json
bash scripts/ci-remote-evidence.sh --json
```

Machine JSON key highlights:

- release audit payload: `schema_version`, `step_rate_basis`, `command_rate_basis`
- runner contract payload: `schema_version`, `rate_basis` (equals `total_scenarios`)
- cross-platform smoke summary payload: `schema_version`, `platform_label`, `commands_count`, `completed_commands_count`
- denominator semantics: `step_rate_basis=total_steps`, `command_rate_basis=total_commands`, `rate_basis=total_scenarios`
- current schema contract: `schema_version=v1`

CI machine artifact examples:

- `/tmp/release-audit-dry-run.json`
- `/tmp/runner-suites.json`
- `/tmp/runner-contract.json`
- `/tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json`
- `/tmp/cross-platform-smoke-windows/cross-platform-smoke-summary.json`
- `/tmp/remote-ci-evidence/remote-ci-evidence-summary.json`

## One-command hook selfcheck

如果你想一次性验证 "Hook wiring + Stop 行为 + 回归测试"，可直接运行：

```bash
bash scripts/fusion-hook-selfcheck.sh --fix .
```

- `fusion-hook-selfcheck.sh` 现在是 `fusion-bridge selfcheck` 的 thin wrapper
- 默认模式会由 Rust selfcheck 执行：doctor + stop 仿真 + contract regression
- 若只想快速检查（不跑 contract regression），可用：

```bash
bash scripts/fusion-hook-selfcheck.sh --json --quick --fix .
```

当返回 `result=ok`（JSON 模式）或看到 `all checks passed`（文本模式）时，说明 hook 机制可正常工作。
