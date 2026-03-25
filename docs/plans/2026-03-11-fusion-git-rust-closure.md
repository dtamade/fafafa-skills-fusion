# Fusion Git Rust Closure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `fusion-bridge git` 子命令的 Rust 实现与 shell 包装收敛，恢复 `fusion-cli` 的 release 构建，并让 Git 控制面符合“Rust 主实现、Shell thin wrapper”约定。

**Architecture:** 以现有 shell `fusion-git.sh` 行为和 `cli_smoke.rs` 中已经写好的 Rust smoke tests 为契约来源。先用 release 编译失败和现有/新增测试锁定缺口，再新增 `rust/crates/fusion-cli/src/git.rs` 实现 `GitCommands` 的六个动作，通过 `main.rs` 接线；随后把 `scripts/fusion-git.sh` 改为 thin wrapper，统一委托 `fusion-bridge git`，并最小化更新 CLI contract 文档。

**Tech Stack:** Rust (`clap`, `anyhow`), Bash, Git CLI, Markdown.

---

### Task 1: 锁定 `fusion-bridge git` 的失败面

**Files:**
- Modify: `rust/crates/fusion-cli/tests/cli_smoke.rs`
- Review only: `rust/crates/fusion-cli/src/cli.rs`
- Review only: `rust/crates/fusion-cli/src/main.rs`
- Review only: `scripts/fusion-git.sh`

**Step 1: 补齐/确认 Rust git 命令测试覆盖**

在 `rust/crates/fusion-cli/tests/cli_smoke.rs` 保留并补齐这些断言：
- `git branch` 在临时仓库内成功返回当前分支
- `git status` 输出 `=== Fusion Git Status ===`、`Current branch:`、`demo.txt`
- `git create-branch demo-goal` 输出 `fusion/demo-goal`
- `git changes` 在有未提交文件时输出 `=== Git Status ===`
- `git diff` 在修改后输出 diff 片段
- `git cleanup <branch>` 能切回原始分支

**Step 2: 用 release 构建验证当前失败**

Run: `cd rust && cargo check --release -p fusion-cli`
Expected: FAIL，包含 `Commands::Git { .. } not covered`

**Step 3: 用现有 smoke 测试锁定预期行为**

Run: `cd rust && cargo test --release -p fusion-cli git_`
Expected: FAIL（当前会先卡在编译阶段）

### Task 2: 实现 Rust `git` 子命令并接线到主入口

**Files:**
- Create: `rust/crates/fusion-cli/src/git.rs`
- Modify: `rust/crates/fusion-cli/src/main.rs`
- Modify: `rust/crates/fusion-cli/src/cli.rs`

**Step 1: 新增 Rust git 分发模块**

创建 `rust/crates/fusion-cli/src/git.rs`，实现：

```rust
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

use crate::cli::GitCommands;

pub(crate) fn dispatch_git(command: GitCommands) -> Result<()> {
    match command {
        GitCommands::Status => cmd_status(),
        GitCommands::CreateBranch { goal_slug } => cmd_create_branch(&goal_slug),
        GitCommands::Commit { message, task_id: _ } => cmd_commit(&message),
        GitCommands::Branch => cmd_branch(),
        GitCommands::Changes => cmd_changes(),
        GitCommands::Diff => cmd_diff(),
        GitCommands::Cleanup { original_branch } => cmd_cleanup(original_branch.as_deref()),
    }
}
```

模块内保持 shell 语义对齐：
- 分支前缀固定为 `fusion/`
- 先校验当前目录是 git 仓库
- `commit` 走 `git add -A` 后提交
- `status` 输出与 shell 版本相同的标题结构
- 出错时返回明确 `anyhow` 上下文，避免静默失败

**Step 2: 在主入口接线**

修改 `rust/crates/fusion-cli/src/main.rs`：
- 增加 `mod git;`
- 增加 `use git::dispatch_git;`
- 在 `match cli.command` 中加入：

```rust
Commands::Git { command } => dispatch_git(command),
```

**Step 3: 保持 CLI 面与实现一致**

如果 `rust/crates/fusion-cli/src/cli.rs` 中 `GitCommands` 的参数形式与 shell 契约不一致，直接在这里修正，而不是在实现层做隐式兼容。

**Step 4: 先跑 release 编译，再跑 Rust git smoke**

Run: `cd rust && cargo check --release -p fusion-cli`
Expected: PASS

Run: `cd rust && cargo test --release -p fusion-cli git_`
Expected: PASS

### Task 3: 把 `scripts/fusion-git.sh` 收敛成 thin wrapper

**Files:**
- Modify: `scripts/fusion-git.sh`
- Modify: `scripts/runtime/tests/test_fusion_control_script_validation`
- Review only: `scripts/lib/fusion-bridge.sh`

**Step 1: 先写 shell contract 回归测试**

在 `scripts/runtime/tests/test_fusion_control_script_validation` 为 `fusion-git.sh` 增加/确认这些断言：
- `--help` 返回 `0`
- 未知 action 返回非零并输出 usage
- bridge 缺失时返回 `127`，并提示 `Missing Rust fusion-bridge`
- 正常路径会把参数原样委托给 `fusion-bridge git ...`

**Step 2: 用测试先验证当前 shell 行为不满足 thin wrapper 目标**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation -k git`
Expected: FAIL（至少 bridge 委托相关断言失败）

**Step 3: 最小化改写 shell wrapper**

把 `scripts/fusion-git.sh` 改造成：
- 保留 `usage()` 与现有 action 名
- 参数校验仍在 shell 层完成，避免错误输入进入 Rust
- 成功路径通过 bridge helper 调用：

```bash
"$bridge_bin" git "$ACTION" "${EXTRA_ARGS[@]}"
```

- 与其它 control wrapper 一致：bridge 缺失/禁用时返回 `127`

**Step 4: 回归 shell contract**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation -k git`
Expected: PASS

Run: `bash -n scripts/fusion-git.sh`
Expected: PASS

### Task 4: 更新 Git control-plane 文档契约

**Files:**
- Modify: `docs/CLI_CONTRACT_MATRIX.md`
- Modify: `rust/README.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: 更新 CLI contract matrix**

把 `fusion-git.sh` 一行更新为 thin wrapper 语义：
- `valid args` 仍保留 `status|create-branch|commit|branch|changes|diff|cleanup`
- `exit code` 增加 `missing/disabled bridge 127`
- `stdout/stderr/json expectations` 改为 “Thin wrapper around `fusion-bridge git`”

**Step 2: 修正文档里 Git 控制面的定位**

在 README / rust README 中补一句，明确：
- `fusion-git.sh` 也属于 shell compatibility entry layer
- Git 控制面主实现位于 Rust `fusion-bridge git`

**Step 3: 验证 docs freshness 没被破坏**

测试记录： `scripts/runtime/tests/test_docs_freshness`
Expected: PASS

### Task 5: 做完整的 release-oriented 验证闭环

**Files:**
- Verify: `rust/crates/fusion-cli/src/git.rs`
- Verify: `rust/crates/fusion-cli/src/main.rs`
- Verify: `scripts/fusion-git.sh`
- Verify: `docs/CLI_CONTRACT_MATRIX.md`

**Step 1: Rust release 编译**

Run: `cd rust && cargo check --release -p fusion-cli`
Expected: PASS

**Step 2: Rust Git smoke + 全 CLI smoke**

Run: `cd rust && cargo test --release -p fusion-cli git_`
Expected: PASS

Run: `cd rust && cargo test --release -p fusion-cli cli_smoke`
Expected: PASS

**Step 3: Shell 验证**

测试记录： `scripts/runtime/tests/test_fusion_control_script_validation -k git`
Expected: PASS

Run: `bash -n scripts/fusion-git.sh`
Expected: PASS

**Step 4: 记录完成标准**

只有当以下条件全部满足时，才算 Git 收敛线关闭：
- `Commands::Git` 不再打断 release 构建
- `fusion-bridge git` 六个动作均有 Rust 实现
- `scripts/fusion-git.sh` 变成 thin wrapper，而不是独立主实现
- Rust smoke 和 shell contract 测试全部通过
- 文档不再把 `fusion-git.sh` 描述成孤立的 shell-only 主路径

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。

