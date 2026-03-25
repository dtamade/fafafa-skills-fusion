# Session Recovery Protocol

Fusion 当前的会话恢复主路径已经收敛到 Rust 控制面。

- 用户入口：`/fusion resume`
- thin wrapper：`scripts/fusion-resume.sh`
- 主控制面：`fusion-bridge resume --fusion-dir .fusion`
- 深度补账/恢复报告：`scripts/fusion-catchup.sh`

如果你看到任何把 `codeagent-wrapper ... resume` 当成用户恢复入口的说法，那已经不是当前协议；当前用户入口仍是 `/fusion resume`。

---

## 什么时候用 resume，什么时候用 catchup

### 用 `/fusion resume`

适用于这些场景：

- `/fusion pause` 之后继续
- 会话中断后继续当前 workflow
- `.fusion/sessions.json` 里状态仍是 `paused`、`stuck` 或 `in_progress`

当前真实链路：

```text
/fusion resume
  -> scripts/fusion-resume.sh
  -> fusion-bridge resume --fusion-dir .fusion
```

`fusion-bridge resume` 会读取 `.fusion/sessions.json`，按当前状态处理：

- 先输出统一的人类可读摘要：`Current state: <status> @ <phase>` 与 `Next action: <...>`

- `paused`：改回 `in_progress`，然后继续运行
- `stuck`：提示先查看 `.fusion/progress.md`，随后恢复到 `in_progress`
- `in_progress`：直接继续
- `completed`：打印 `Next action: No resume needed`，并直接返回“Nothing to resume”
- `cancelled` 或未知状态：报错，不继续

### 用 `scripts/fusion-catchup.sh`

适用于这些场景：

- `/clear` 之后需要快速补回上下文
- 想知道 Claude 会话里有哪些消息还没同步到 `.fusion/`
- 想交叉检查任务计划、会话状态和工作区改动是否一致

当前真实链路：

```text
scripts/fusion-catchup.sh
  -> fusion-bridge catchup --fusion-dir .fusion --project-path <repo>
  -> 若 bridge 缺失，则尝试 cargo run --release -q -p fusion-cli -- catchup ...
```

这个命令不会直接“替你恢复执行”，而是先生成恢复报告，帮助你判断下一步该继续 `resume`、先修状态，还是先处理工作区漂移。

恢复报告的顶部也会先给出统一摘要：

- `Current state: <status> @ <phase>`
- `Next action: <...>`

---

## 当前恢复流程

### 1. 先看 `.fusion/`

恢复相关的主数据都在 `.fusion/`：

```text
.fusion/
├── sessions.json
├── task_plan.md
├── progress.md
├── findings.md
└── events.jsonl
```

最关键的是：

- `sessions.json`：workflow 状态、阶段、当前任务
- `task_plan.md`：剩余任务和执行顺序
- `progress.md`：最近一次执行轨迹和错误
- `findings.md`：之前已经确认的结论

### 2. 跑恢复入口

继续当前 workflow：

```bash
./scripts/fusion-resume.sh
```

生成补账/恢复报告：

```bash
./scripts/fusion-catchup.sh
```

指定项目路径或自定义 fusion 目录：

```bash
./scripts/fusion-catchup.sh --project-path "$PWD"
./scripts/fusion-catchup.sh --fusion-dir .fusion --project-path "$PWD"
```

### 3. 根据输出决定动作

- 先看 `Current state` 和 `Next action` 两行，确认 bridge 认为的 live 状态与推荐动作
- `resume` 成功：直接回到当前 workflow
- `catchup` 显示有未同步消息：先理解报告，再决定是否补写 `.fusion/progress.md` 或继续执行
- `catchup` 显示任务计划、会话状态、git diff 不一致：先修正漂移，再恢复

---

## `scripts/fusion-resume.sh` 的当前约定

`scripts/fusion-resume.sh` 是薄包装，不再自己实现恢复逻辑。

它只做三件事：

1. 校验参数，只接受空参数或 `--help`
2. 解析 `fusion-bridge`
3. 执行 `fusion-bridge resume --fusion-dir .fusion`

如果 bridge 被禁用或不存在，脚本会直接返回依赖错误 `127`。它不会再回退到仓库已移除的旧恢复实现。

---

## `scripts/fusion-catchup.sh` 的当前约定

`scripts/fusion-catchup.sh` 也是薄包装，但它保留了一层显式的 Rust CLI 回退：

1. 优先执行 `fusion-bridge catchup --fusion-dir .fusion --project-path <repo>`
2. 如果 bridge 缺失，但本地有 Rust 工具链和 `rust/Cargo.toml`
3. 则执行 `cargo run --release -q -p fusion-cli -- catchup --fusion-dir .fusion --project-path <repo>`

它会综合这些来源生成恢复报告：

- Claude 会话 JSONL
- `.fusion/task_plan.md`
- `.fusion/sessions.json`
- 当前仓库的 git diff 统计

---

## 推荐的恢复检查清单

恢复前后，至少确认这 5 个问题：

| 问题                | 当前来源                                        |
| ------------------- | ----------------------------------------------- |
| 我在哪个 workflow？ | `.fusion/sessions.json`                         |
| 当前阶段是什么？    | `.fusion/sessions.json -> current_phase`        |
| 还剩哪些任务？      | `.fusion/task_plan.md`                          |
| 最近做到了哪里？    | `.fusion/progress.md`                           |
| 工作区有没有漂移？  | `git status` / `git diff` / `fusion-catchup.sh` |

如果这 5 个问题里有一个答不上来，不要盲目继续跑 agent，先做 catchup。

---

## Hook 接线不要写在这里

Session recovery 文档不再内嵌旧式 Hook frontmatter 片段。

当前 Hook 配置请看：

- `docs/HOOKS_SETUP.md`
- `.claude/settings.example.json`（受版本控制模板）
- `scripts/fusion-pretool.sh`
- `scripts/fusion-posttool.sh`
- `scripts/fusion-stop-guard.sh`

也就是说，当前仓库的 Hook 接线是 `.claude/settings.example.json` 模板 + 宿主本地 `.claude/settings.json` / `.claude/settings.local.json` + Shell hook 脚本，不是 `SKILL.md` frontmatter。

其中 `.claude/settings.example.json` 是受版本控制的模板；`.claude/settings.json` 与 `.claude/settings.local.json` 才属于宿主本地 Hook 配置，不是受版本控制的仓库输入。

如果这里描述的恢复入口、Hook 接线边界，或相关仓库/runtime 契约发生变化，请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`。

---

## 常用命令

```bash
# 继续当前 workflow
./scripts/fusion-resume.sh

# 生成恢复报告
./scripts/fusion-catchup.sh

# 查看当前状态
./scripts/fusion-status.sh

# 查看最近日志
./scripts/fusion-logs.sh
```

如果你在本地没有预构建 bridge，但有 Rust 工具链，`scripts/fusion-catchup.sh` 会显式走：

```bash
cargo run --release -q -p fusion-cli -- catchup --fusion-dir .fusion --project-path "$PWD"
```

`resume` 不提供这种自动 cargo 回退；它要求可用的 Rust `fusion-bridge`。
