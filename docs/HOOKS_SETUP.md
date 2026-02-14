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


## Quick doctor + auto-fix

如果你怀疑 Hook 没有真正启用，可直接执行：

```bash
bash scripts/fusion-hook-doctor.sh --json --fix .
```

- `--json`: 输出机器可读诊断结果
- `--fix`: 自动写入项目 `.claude/settings.local.json`（PreToolUse/PostToolUse/Stop 三个 hooks）

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
  engine: "python"   # python | rust
```

- `engine: "python"`: 使用现有 Python runtime（默认）
- `engine: "rust"`: 优先调用 `fusion-bridge hook ...`，失败自动回退 Python/Shell

## What is wired in this repo

- Hook scripts:
  - `scripts/fusion-pretool.sh`
  - `scripts/fusion-posttool.sh`
  - `scripts/fusion-stop-guard.sh`
- Python runtime bridge:
  - `scripts/runtime/compat_v2.py`

## Verify wiring

Run:

```bash
pytest -q \
  scripts/runtime/tests/test_hook_shell_runtime_path.py \
  scripts/runtime/tests/test_compat_v2.py
```

If these pass, hook → shell → runtime path is healthy.

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
- 常用快速模式：`bash scripts/release-contract-audit.sh --fast`

## Machine-readable release checks

```bash
bash scripts/release-contract-audit.sh --dry-run --json --json-pretty --fast --skip-rust
python3 scripts/runtime/regression_runner.py --list-suites --json
```

Machine JSON key highlights:
- release audit payload: `schema_version`, `step_rate_basis`, `command_rate_basis`
- runner contract payload: `schema_version`, `rate_basis` (equals `total_scenarios`)
- denominator semantics: `step_rate_basis=total_steps`, `command_rate_basis=total_commands`, `rate_basis=total_scenarios`
- current schema contract: `schema_version=v1`

CI machine artifact examples:
- `/tmp/release-audit-dry-run.json`
- `/tmp/runner-suites.json`
- `/tmp/runner-contract.json`

## One-command hook selfcheck

如果你想一次性验证 "Hook wiring + Stop 行为 + 回归测试"，可直接运行：

```bash
bash scripts/fusion-hook-selfcheck.sh --fix .
```

- 默认模式会执行：doctor + stop 仿真 + hook 回归 pytest
- 若只想快速检查（不跑 pytest），可用：

```bash
bash scripts/fusion-hook-selfcheck.sh --json --quick --fix .
```

当返回 `result=ok`（JSON 模式）或看到 `all checks passed`（文本模式）时，说明 hook 机制可正常工作。
