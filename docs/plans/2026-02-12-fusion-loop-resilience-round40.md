# Fusion Loop Resilience (Session + Timeout) Implementation Plan (Round40)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 `fusion-codeagent.sh` 的会话 ID 提取与 resume/fallback 行为，避免错误 session id 导致无谓 fallback；并增加可配置 timeout 以在 `codex` 后端 hang 时触发 fallback，保证自主循环可持续推进。

**Architecture:** 用归档中的端到端测试调用记录来驱动 `scripts/fusion-codeagent.sh` 行为（session ID 持久化、resume 失败降级策略、timeout->fallback）。实现保持最小改动：优先从 `SESSION_ID:` 标记提取 session；resume 失败时同后端无 resume 重试一次；可选 timeout 通过 `FUSION_CODEAGENT_TIMEOUT_SEC` 启用。

**Tech Stack:** Bash.

---

### Task 1: 支持 UUID session id 提取与持久化

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_codeagent_script`
- Modify: `scripts/fusion-codeagent.sh`

**Step 1: Write the failing test**

新增/启用测试：当 `codeagent-wrapper` 输出 `SESSION_ID: <uuid>` 时，必须把该 UUID 写入 `.fusion/sessions.json` 的 `claude_session`。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_execute_phase_stores_uuid_session_id`
Expected: FAIL（UUID 未被持久化）

**Step 3: Write minimal implementation**

在 `scripts/fusion-codeagent.sh`：
- `extract_session_id` 改为优先解析 `SESSION_ID:` 行，避免从日志中误抓数字（PID/时间戳）。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_execute_phase_stores_uuid_session_id`
Expected: PASS

**Step 5: Verify task scope regression**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script`
Expected: PASS

---

### Task 2: resume 失败时同后端无 resume 重试一次

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_codeagent_script`
- Modify: `scripts/fusion-codeagent.sh`

**Step 1: Write the failing test**

新增/启用测试：当已有 `claude_session`（可能是历史错误 ID）且 `resume <id>` 失败时，不应直接 fallback 到 `codex`；应在同后端用“新会话”（无 resume）重试一次。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_execute_phase_resume_failure_retries_without_resume`
Expected: FAIL（直接 fallback，触发 codex）

**Step 3: Write minimal implementation**

在 `scripts/fusion-codeagent.sh` 的 primary 执行分支：
- 若 primary 带 session resume 失败：先 `run_backend primary` 不带 session 再试一次；
- 仅当该重试仍失败，才 fallback 到备用后端。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_execute_phase_resume_failure_retries_without_resume`
Expected: PASS

**Step 5: Verify task scope regression**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script`
Expected: PASS

---

### Task 3: 增加超时保护以在 backend hang 时触发 fallback

**Files:**
- Modify: `scripts/runtime/tests/test_fusion_codeagent_script`
- Modify: `scripts/fusion-codeagent.sh`

**Step 1: Write the failing test**

新增测试：当 primary backend（例如 `codex`）执行超过 `FUSION_CODEAGENT_TIMEOUT_SEC` 时，应被 timeout 中止并触发 fallback（例如到 `claude`）完成调用。

**Step 2: Run test to verify it fails**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_timeout_falls_back_to_claude`
Expected: FAIL（无 timeout，primary 最终成功，未 fallback）

**Step 3: Write minimal implementation**

在 `scripts/fusion-codeagent.sh`：
- `run_backend` 支持环境变量 `FUSION_CODEAGENT_TIMEOUT_SEC=<seconds>`；
- 若设置且 `timeout/gtimeout` 可用，则用其包裹 `codeagent-wrapper` 调用；
- timeout 视为失败，走既有 fallback 逻辑。

**Step 4: Run test to verify it passes**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script::TestFusionCodeagentScript::test_timeout_falls_back_to_claude`
Expected: PASS

**Step 5: Verify task scope regression**

测试记录： `scripts/runtime/tests/test_fusion_codeagent_script`
Expected: PASS

---

### Final Verification Bundle

Run:
- `bash -n scripts/*.sh`
- 测试记录： `scripts/runtime/tests/test_fusion_codeagent_script`
- 全量验证记录
- `(cd rust && cargo clippy --workspace --all-targets -- -D warnings)`
- `(cd rust && cargo fmt --all -- --check)`

Expected:
- all commands pass with no regressions


> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
