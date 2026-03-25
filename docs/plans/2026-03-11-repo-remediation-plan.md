# Repo Remediation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce migration risk after the Rust control-plane convergence by tightening review boundaries, shrinking documentation-contract coupling, clarifying the compatibility boundary around the now-removed runtime, and consolidating canonical maintainer documentation.

**Architecture:** Treat the repository as a converged-but-not-yet-fully-closed system. Keep Rust / `fusion-bridge` as the primary control plane, keep shell as a thin wrapper and hook layer, and explicitly decide which pieces from the now-removed runtime still matter as reference-only surfaces. Separate machine contracts from explanatory docs so CI continues to protect behavior without freezing wording.

**Tech Stack:** Rust workspace (`clap`, `serde`), Bash, Markdown, GitHub Actions.

---

### Task 1: Split the current migration work into reviewable change groups

**Files:**
- Modify: `reviews/repo-convergence-change-groups.md`
- Modify: `reviews/repo-convergence-pr-summary.md`
- Review only: `rust/crates/fusion-cli/src/`
- Review only: `scripts/`
- Review only: `scripts/runtime/`
- Review only: `docs/`

**Step 1: Write the grouping rubric**

Define four change groups with exact inclusion rules:
- Rust control plane
- Shell wrappers and hooks
- removed-runtime compat/reference
- Docs and CI contracts

**Step 2: Snapshot the current mixed state**

Run: `git status --short`
Expected: Mixed modified/deleted/untracked files across Rust, shell, removed-runtime, and docs paths.

**Step 3: Update reviewer guidance**

Document the exact review order and file buckets in:
- `reviews/repo-convergence-change-groups.md`
- `reviews/repo-convergence-pr-summary.md`

**Step 4: Verify the grouping doc is actionable**

Run: `sed -n '1,220p' reviews/repo-convergence-change-groups.md`
Expected: The file lists the four groups, the review order, and which paths belong to each group.

### Task 2: Reduce documentation-contract coupling to stable contract assertions

**Files:**
- Modify: `scripts/runtime/tests/test_docs_freshness`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`
- Modify: `docs/HOOKS_SETUP.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: Write failing/adjusted tests**

Refactor `test_docs_freshness` so it only locks:
- command names
- machine JSON fields
- schema/version markers
- required release/build commands
- required artifact paths

Do not require broad explanatory wording where a smaller structural assertion would protect the same behavior.

**Step 2: Run the focused docs test**

测试记录： `scripts/runtime/tests/test_docs_freshness`
Expected: Fail before the reduction if wording-level assertions are still too broad.

**Step 3: Apply the minimal docs/test changes**

Update:
- tests to target stable contract fragments
- docs to centralize contract details without duplicating the same wording in multiple places

**Step 4: Re-run the focused docs suite**

测试记录： `scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_ci_contract_gates`
Expected: PASS

### Task 3: Close or formally retain the remaining live-path boundary around the now-removed runtime

**Files:**
- Modify: `docs/RUST_FUSION_BRIDGE_ROADMAP.md`
- Modify: `docs/RUNTIME_KERNEL_DESIGN.md`
- Modify: `scripts/runtime/compat_v2`
- Modify: `scripts/runtime/kernel`
- Modify: `scripts/runtime/_session_store`
- Modify: `scripts/runtime/tests/test_compat_v2`
- Modify: `scripts/runtime/tests/test_session_store`
- Modify: `scripts/runtime/tests/test_resume_replay`

**Step 1: Decide the boundary explicitly**

Choose one repository truth and encode it in docs/tests:
- `kernel` is reference-only and not part of the live control plane
- or `kernel` remains a supported live compat path and must keep explicit parity guarantees

**Step 2: Lock the decision in tests**

测试记录： `scripts/runtime/tests/test_compat_v2 scripts/runtime/tests/test_session_store scripts/runtime/tests/test_resume_replay`
Expected: Existing expectations reveal what is still treated as supported behavior.

**Step 3: Apply the minimal implementation/doc cleanup**

If reference-only:
- remove live-path wording
- move remaining direct state-writing helpers behind clearly internal/private wrappers

If retained:
- document it as a supported compat path
- add explicit ownership/exit criteria in roadmap docs

**Step 4: Verify boundary clarity**

Run: `rg -n "live path|legacy fallback|reference only|compat" docs scripts/runtime`
Expected: No contradictory wording about whether the older runtime is live, fallback-only, or reference-only.

### Task 4: Consolidate canonical maintainer documentation

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/HOOKS_SETUP.md`
- Modify: `docs/CLI_CONTRACT_MATRIX.md`
- Modify: `docs/REPO_HYGIENE.md`
- Modify: `docs/REPO_CONVERGENCE_SUMMARY_2026-03.md`

**Step 1: Assign document roles**

Set one canonical purpose per document:
- `README*`: operator overview and quick start
- `HOOKS_SETUP`: hook wiring and health recovery
- `CLI_CONTRACT_MATRIX`: exact CLI contract table
- `REPO_HYGIENE`: repository artifact policy
- `REPO_CONVERGENCE_SUMMARY`: migration outcome summary

**Step 2: Remove duplicated detailed contract prose**

Move repeated command/field explanations to the most appropriate canonical file and replace duplicates with short references.

**Step 3: Validate doc consistency**

测试记录： `scripts/runtime/tests/test_docs_freshness scripts/runtime/tests/test_repo_hygiene`
Expected: PASS

**Step 4: Spot-check the maintainer flow**

Run:
- `sed -n '1,220p' README.md`
- `sed -n '1,240p' docs/HOOKS_SETUP.md`
- `sed -n '1,240p' docs/CLI_CONTRACT_MATRIX.md`
Expected: Each file has a clear, non-overlapping responsibility.

### Task 5: Re-run the full release-oriented verification bundle before merge

**Files:**
- Verify: `.github/workflows/ci-contract-gates.yml`
- Verify: `rust/README.md`
- Verify: `scripts/`
- Verify: `rust/`

**Step 1: Run the compatibility and shell verification bundle**

全量验证记录
Expected: PASS

**Step 2: Run the Rust release bundle**

Run: `cargo test --release`
Workdir: `rust`
Expected: PASS

**Step 3: Run the Rust lint/format bundle**

Run:
- `cargo clippy --release --workspace --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
Workdir: `rust`
Expected: PASS

**Step 4: Run shell syntax validation**

Run: `bash -n scripts/*.sh scripts/lib/*.sh`
Expected: PASS

**Step 5: Record final merge criteria**

Only merge when all of the following are true:
- docs tests assert stable contracts rather than broad wording
- the compatibility boundary around the now-removed runtime is explicit and non-contradictory
- review grouping docs match the actual change buckets
- release-oriented compatibility, shell, and Rust test gates all pass

> 归档说明：本文保留其历史上下文。当前行为请以 Rust 与 Shell 契约为准。
