# Fusion Skill 跨平台兼容性报告

> 分析日期: 2026-02-09
> 最后更新: 2026-03-25
> 当前口径: 按已验证证据分级；2026-03-25 的远端 promotion evidence（run `23539348456`）已将 macOS 与 Windows (Git Bash) 升级到“已验证”

## 平台支持状态

| 平台               | 当前状态    | 备注                                                                                    |
| ------------------ | ----------- | --------------------------------------------------------------------------------------- |
| Linux (GNU)        | ✅ 已验证   | 当前开发环境，CI `ubuntu-latest` 持续验证                                                                        |
| macOS (BSD)        | ✅ 已验证   | 远端 `macos-latest` smoke 与 Rust release smoke 已在 2026-03-25 主分支绿灯通过                                  |
| Windows (Git Bash) | ✅ 已验证   | 远端 `windows-latest` Git Bash smoke 与 Rust release smoke 已在 2026-03-25 主分支绿灯通过                       |
| Windows (WSL)      | ⚠️ 部分验证 | 路径语义接近 Linux；wrapper 与 catchup 契约已纳入 smoke，但仍待 WSL 实机或专门 CI 证据                            |

## 当前验证基线

当前自动化证据主要来自：

- GitHub Actions `ci-contract-gates.yml` 在 `ubuntu-latest` 运行完整 release 门禁：`cargo build --release -p fusion-cli --bin fusion-bridge`、`bash scripts/ci-machine-mode-smoke.sh`、`bash scripts/ci-cross-platform-smoke.sh`、`cargo clippy --release`、`cargo test --release`、`cargo fmt --all -- --check`
- GitHub Actions `macos-latest` smoke job 会运行 `bash -n scripts/*.sh`、`cargo build --release -p fusion-cli --bin fusion-bridge`、`bash scripts/ci-cross-platform-smoke.sh --artifacts-dir /tmp/cross-platform-smoke-macos --platform-label macos`、`bash scripts/ci-cross-platform-json-smoke.sh /tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json`、`cargo test --release -p fusion-cli --test cli_smoke`，并上传 `/tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json`
- GitHub Actions `windows-latest` Git Bash smoke job 会运行 `bash -n scripts/*.sh`、`cargo build --release -p fusion-cli --bin fusion-bridge`、`bash scripts/ci-cross-platform-smoke.sh --artifacts-dir /tmp/cross-platform-smoke-windows --platform-label windows-git-bash`、`bash scripts/ci-cross-platform-json-smoke.sh /tmp/cross-platform-smoke-windows/cross-platform-smoke-summary.json`、`cargo test --release -p fusion-cli --test cli_smoke`，并上传 `/tmp/cross-platform-smoke-windows/cross-platform-smoke-summary.json`
- 本仓库当前发布契约还由 `cargo test --release -p fusion-cli --test repo_contract`、`cargo test --release -p fusion-cli --test shell_contract`、`cargo test --release -p fusion-cli --test cli_smoke` 持续覆盖
- 当前远端 promotion evidence: `bash scripts/ci-remote-evidence.sh --repo dtamade/fafafa-skills-fusion --branch main --json` 在 2026-03-25 返回 `promotion_ready=true`，对应 run `23539348456`
- 如果这里描述的兼容性基线或命令面契约发生变化，请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`

因此，当前已验证平台为 Linux、macOS 和 Windows (Git Bash)；WSL 仍保持“已做兼容性处理，待补充证据”口径。

## 当前命令面契约

| 范围           | 当前主路径                                                                                                                                                                                                 | 兼容说明                                                                                                                             |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| 控制面薄包装   | `fusion-start.sh`、`fusion-status.sh`、`fusion-logs.sh`、`fusion-git.sh`、`fusion-codeagent.sh`、`fusion-achievements.sh`、`fusion-pause.sh`、`fusion-resume.sh`、`fusion-cancel.sh`、`fusion-continue.sh` | 这些脚本都是 shell thin wrapper，直接委托 `fusion-bridge <command>`；bridge 缺失或被禁用时返回依赖错误，不再回退到旧 Shell 主实现    |
| live hook 路径 | `fusion-stop-guard.sh`、`fusion-pretool.sh`、`fusion-posttool.sh`                                                                                                                                          | hook live 路径优先走 Rust bridge；只有在 bridge 不可用、被禁用或返回失败时，才允许最小 Shell fallback 保底                           |
| Hook 自检      | `fusion-hook-selfcheck.sh`                                                                                                                                                                                 | 当前是 `fusion-bridge selfcheck` 的 thin wrapper，shell 侧只保留参数与路径校验                                                       |
| 恢复路径       | `fusion-catchup.sh`                                                                                                                                                                                        | 默认走 `fusion-bridge catchup`；若 bridge 缺失但本地 Rust 工具链存在，则显式走 `cargo run --release -q -p fusion-cli -- catchup ...` |
| 已退场实现     | 旧 runtime/reference 层                                                                                                                                                                                    | 旧 runtime/reference 层已从仓库移除，不再保留任何公开或内部 live 路径                                                                |

当前 live 配置文档不再公开多 runtime engine 选择；生成的 `.fusion/config.yaml` 中 `engine: "rust"` 只是单引擎主线路径标记。

## Bash 版本要求

**最低版本: Bash 4.0+**

当前仓库稳定依赖的 Bash 特性包括：

- `[[ ... =~ ... ]]`
- `(( ... ))`
- `for ((i=0; ...))`
- `+=`

Windows Git Bash 通常自带 Bash 4.4+，版本要求上满足运行前提；截至 2026-03-25，远端 Git Bash smoke 与 Rust release smoke 已提供主分支端到端验证证据。

## 关键兼容点

以下工具/特性已在实现中做了兼容处理，但除 Linux 外，不代表已经完成全平台回归：

| 工具/特性          | Linux | macOS | Git Bash | 备注                                             |
| ------------------ | ----- | ----- | -------- | ------------------------------------------------ |
| `stat` 时间戳      | ✅    | ✅    | ✅       | 已做 GNU/BSD 分支                                |
| `sed -i` 原地编辑  | ✅    | ✅    | ✅       | 已做 GNU/BSD fallback                            |
| `date +%s%3N` 毫秒 | ✅    | ⚠️    | ✅       | 已有 `$(date +%s)000` fallback                   |
| `mktemp` 临时文件  | ✅    | ✅    | ✅       | 模板语法通用                                     |
| machine JSON 校验  | ✅    | ✅    | ✅       | `jq` 仅用于 smoke 或人工检查，不是运行时必需依赖 |
| `head/tail/cut`    | ✅    | ✅    | ✅       | POSIX 标准                                       |
| C-style for loop   | ✅    | ✅    | ✅       | Bash 内置                                        |
| `[[ =~ ]]` 正则    | ✅    | ✅    | ✅       | Bash 3.0+                                        |

另外，当前仓库还明确收敛了以下跨平台点：

- `rust/crates/fusion-cli/src/catchup.rs` 负责统一处理 Windows 路径分隔符与盘符语义；`scripts/fusion-catchup.sh` 只是恢复入口薄包装
- `scripts/fusion-pretool.sh` 在终端不支持块字符时会退回 ASCII 进度条，避免旧式 Windows 终端乱码
- 旧解释器探测与旧 runtime/reference 路径都已退出仓库，不再属于控制面、hook 路径或辅助路径
- `jq` 不是运行时必需依赖；当前活文档与 CI 只把它当成 machine smoke 或人工 JSON 检查工具

---

## 当前剩余工作

| 主题                                | 当前状态          | 下一步                                                                                                                                  |
| ----------------------------------- | ----------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Linux 主路径                        | ✅ 已验证         | 继续作为发布基线                                                                                                                        |
| macOS Shell / Hook 路径             | ✅ 已验证         | 继续跟踪 BSD 差异回归；如契约变化，重新跑远端 evidence                                                                                   |
| Windows Rust CLI 主路径             | ✅ 已验证         | 继续维持 `windows-latest` Rust release smoke                                                                                             |
| Windows Git Bash 薄包装 / Hook 路径 | ✅ 已验证         | 远端 smoke 已覆盖 bridge helper、status/achievements 薄包装、控制面薄包装校验、真实 hook 路径、catchup wrapper 与 fallback hook smoke   |
| WSL 恢复与 catchup 路径             | ⚠️ 待补证据       | wrapper 契约已纳入 smoke，但仍缺 WSL 实机或专门 CI 证据                                                                                  |

---

## 测试矩阵

建议按以下顺序补齐验证：

- [x] Linux (Ubuntu/Debian) - CI 与本地 release 验证已覆盖
- [x] macOS - 远端 CI release/smoke 已验证；实机验证仍可继续补强
- [x] Windows Git Bash - 远端 CI Git Bash smoke + Rust release smoke 已验证
- [ ] Windows WSL2 - 待实机验证
