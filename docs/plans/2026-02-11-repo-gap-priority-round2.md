# Repo Gap Priority Round 2 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `fusion-start.sh` 与 `fusion-init.sh` 的关键可用性缺口：参数校验、Rust 引擎快速切换、机器可读输出，并通过严格 TDD 固化。

**Architecture:** 本轮继续“测试先行 + 最小实现”策略。先为每个行为写失败测试，再做脚本最小改动，最后执行目标回归与全量回归。所有新增能力保持向后兼容（默认行为不变）。

**Tech Stack:** Bash, Markdown。

---

### Task 1: `fusion-start.sh` 参数校验加强

**Priority:** P0  
**Files:**
- Create: `scripts/runtime/tests/test_fusion_start_script`
- Modify: `scripts/fusion-start.sh`

**Step 1: Write the failing test**

新增测试：
- 传入未知选项（如 `--bad`）应退出非 0，并输出明确错误。
- 传入多个目标参数（`goal1 goal2`）应退出非 0，并提示仅支持一个目标。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_rejects_unknown_option scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_rejects_multiple_goals`  
Expected: FAIL。

**Step 3: Write minimal implementation**

在参数解析阶段增加：
- `-h|--help` 显示帮助并退出 0
- `-*` 非白名单选项报错并退出 1
- 多个目标参数时报错并退出 1

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_rejects_unknown_option scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_rejects_multiple_goals`  
Expected: PASS。

---

### Task 2: `fusion-init.sh` 增加 `--engine rust|旧 runtime`

**Priority:** P1  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_start_script`
- Modify: `scripts/fusion-init.sh`

**Step 1: Write the failing test**

新增测试：执行 `bash scripts/fusion-init.sh --engine rust` 后，生成 `.fusion/config.yaml` 中应含 `engine: "rust"`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_fusion_init_supports_rust_engine`  
Expected: FAIL（当前固定为 `旧 runtime`）。

**Step 3: Write minimal implementation**

在 `fusion-init.sh` 增加参数解析：
- `--engine rust|旧 runtime`（默认旧 runtime）
- 非法 engine 值报错退出
- 将 `config.yaml` 中 engine 从硬编码替换为变量值

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_fusion_init_supports_rust_engine`  
Expected: PASS。

---

### Task 3: `fusion-init.sh` 增加 `--json` 输出

**Priority:** P1  
**Files:**
- Modify: `scripts/runtime/tests/test_fusion_start_script`
- Modify: `scripts/fusion-init.sh`

**Step 1: Write the failing test**

新增测试：
- `bash scripts/fusion-init.sh --json` 输出合法 JSON，含 `result`, `fusion_dir`, `engine`。
- 触发错误（例如非法 engine）时返回非 0 且 JSON `result=error`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_fusion_init_json_success scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_fusion_init_json_error_on_invalid_engine`  
Expected: FAIL。

**Step 3: Write minimal implementation**

在 `fusion-init.sh` 中：
- 增加 `--json` 标志
- 成功时输出 JSON 摘要
- 错误路径统一在 JSON 模式输出结构化错误
- 文本模式保持现状

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_fusion_init_json_success scripts/runtime/tests/test_fusion_start_script::TestFusionStartScript::test_fusion_init_json_error_on_invalid_engine`  
Expected: PASS。

---

## Final Regression (Round 2)

Run:
- 测试记录： `scripts/runtime/tests/test_fusion_start_script scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_fusion_hook_doctor_script scripts/runtime/tests/test_fusion_status_script`
- 全量验证记录

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
