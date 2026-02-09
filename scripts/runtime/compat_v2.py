"""
Fusion v2 兼容适配层

将 v2 Shell 脚本语义映射为 FSM 事件，作为 Shell → Python 内核的桥梁。
Shell 脚本通过 runtime.enabled 开关决定是否调用此模块。

设计约束:
- 极轻量：pretool 路径 < 50ms
- 故障安全：任何异常不阻塞 Shell 脚本原有逻辑
- 双向兼容：runtime.enabled=false 时不影响 v2 行为
"""

import json
import sys
import os
from pathlib import Path
from typing import Dict, Any, Optional, Tuple
from dataclasses import dataclass

from .state_machine import State, Event, phase_to_state, state_to_phase
from .kernel import FusionKernel, KernelConfig
from .session_store import SessionStore


@dataclass
class StopGuardResult:
    """stop-guard 适配结果"""
    should_block: bool
    decision: str           # "allow" | "block" | "stuck"
    reason: str             # 给 Claude 的继续提示
    system_message: str     # 状态栏消息
    phase_corrected: bool   # 是否发生阶段纠正
    events_dispatched: list  # 已派发的 FSM 事件列表


@dataclass
class PretoolResult:
    """pretool 适配结果"""
    active: bool            # 是否有活跃工作流
    lines: list             # 输出行列表


@dataclass
class PosttoolResult:
    """posttool 适配结果"""
    changed: bool           # 进度是否变化
    lines: list             # 输出行列表


def is_runtime_enabled(fusion_dir: str = ".fusion") -> bool:
    """检查 runtime 是否启用"""
    config_file = Path(fusion_dir) / "config.yaml"
    if not config_file.exists():
        return False

    try:
        content = config_file.read_text(encoding="utf-8")
        # 简单解析：查找 runtime.enabled 或 enabled: true
        # 不引入 yaml 依赖，用简单的行级解析
        in_runtime_section = False
        for line in content.splitlines():
            stripped = line.strip()
            if stripped == "runtime:":
                in_runtime_section = True
                continue
            if in_runtime_section:
                if not line.startswith(" ") and not line.startswith("\t"):
                    in_runtime_section = False
                    continue
                if "enabled:" in stripped:
                    return stripped.split(":", 1)[1].strip().lower() == "true"
        return False
    except Exception:
        return False


def _read_task_counts(fusion_dir: str) -> Dict[str, int]:
    """从 task_plan.md 读取任务计数"""
    task_plan = Path(fusion_dir) / "task_plan.md"
    counts = {"completed": 0, "pending": 0, "in_progress": 0, "failed": 0}

    if not task_plan.exists():
        return counts

    try:
        content = task_plan.read_text(encoding="utf-8")
        counts["completed"] = content.count("[COMPLETED]")
        counts["pending"] = content.count("[PENDING]")
        counts["in_progress"] = content.count("[IN_PROGRESS]")
        counts["failed"] = content.count("[FAILED]")
    except IOError:
        pass

    return counts


def _find_next_task(fusion_dir: str) -> str:
    """查找下一个待执行任务的名称"""
    task_plan = Path(fusion_dir) / "task_plan.md"
    if not task_plan.exists():
        return "unknown"

    try:
        content = task_plan.read_text(encoding="utf-8")
        for line in content.splitlines():
            if "[IN_PROGRESS]" in line or "[PENDING]" in line:
                # 提取任务名：### Task N: 任务名 [STATUS]
                if "### Task" in line:
                    name = line.split(":", 1)[-1].strip() if ":" in line else line
                    # 去掉 [STATUS] 部分
                    for tag in ["[IN_PROGRESS]", "[PENDING]", "[COMPLETED]", "[FAILED]"]:
                        name = name.replace(tag, "").strip()
                    return name or "unknown"
        return "unknown"
    except IOError:
        return "unknown"


def adapt_stop_guard(fusion_dir: str = ".fusion") -> StopGuardResult:
    """
    适配 fusion-stop-guard.sh 的逻辑

    将 stop-guard 的判断逻辑映射为 FSM 事件：
    - 所有任务完成 → dispatch ALL_TASKS_DONE
    - EXECUTE 阶段但所有任务完成 → dispatch ALL_TASKS_DONE（阶段纠正）
    - 后期阶段但有 PENDING → dispatch VERIFY_FAIL/REVIEW_FAIL（退回 EXECUTE）
    - 检测到卡住 → dispatch LOOP_DETECTED

    Returns:
        StopGuardResult
    """
    kernel = FusionKernel(fusion_dir=fusion_dir)
    kernel.load_state()

    events_dispatched = []
    phase_corrected = False

    # 如果不在活跃状态，允许停止
    if kernel.current_state in (State.IDLE, State.COMPLETED, State.CANCELLED):
        return StopGuardResult(
            should_block=False,
            decision="allow",
            reason="",
            system_message="",
            phase_corrected=False,
            events_dispatched=[],
        )

    counts = _read_task_counts(fusion_dir)
    total_remaining = counts["pending"] + counts["in_progress"] + counts["failed"]
    total = sum(counts.values())
    next_task = _find_next_task(fusion_dir)

    # 阶段一致性纠正
    current_phase = state_to_phase(kernel.current_state)

    # EXECUTE + 所有任务完成 → ALL_TASKS_DONE
    if current_phase == "EXECUTE" and total_remaining == 0 and counts["completed"] > 0:
        result = kernel.dispatch(Event.ALL_TASKS_DONE)
        if result.success:
            events_dispatched.append("ALL_TASKS_DONE")
            phase_corrected = True

    # VERIFY/REVIEW/COMMIT 但有 PENDING → 退回 EXECUTE
    elif current_phase in ("VERIFY", "REVIEW", "COMMIT", "DELIVER") and counts["pending"] > 0:
        if current_phase == "VERIFY":
            result = kernel.dispatch(Event.VERIFY_FAIL)
        elif current_phase == "REVIEW":
            result = kernel.dispatch(Event.REVIEW_FAIL)
        else:
            # COMMIT/DELIVER 没有直接退回的事件，用 ERROR_OCCURRED + RECOVER
            kernel.dispatch(Event.ERROR_OCCURRED, {"error": "pending tasks found"})
            result = kernel.dispatch(Event.RECOVER)
            events_dispatched.append("ERROR_OCCURRED")

        if result.success:
            events_dispatched.append(result.event.name)
            phase_corrected = True

    # 所有任务完成，允许停止
    if total_remaining == 0 and counts["completed"] > 0:
        return StopGuardResult(
            should_block=False,
            decision="allow",
            reason="",
            system_message="",
            phase_corrected=phase_corrected,
            events_dispatched=events_dispatched,
        )

    # 没有 task_plan.md 且在早期阶段
    if total == 0:
        if kernel.current_state in (State.INITIALIZE, State.ANALYZE, State.DECOMPOSE):
            goal = _read_goal(fusion_dir)
            return StopGuardResult(
                should_block=True,
                decision="block",
                reason=f"Continue with task decomposition for goal: {goal or '(not set)'}. Create .fusion/task_plan.md with tasks.",
                system_message=f"🔄 Fusion | Phase: {current_phase} | Create task_plan.md",
                phase_corrected=phase_corrected,
                events_dispatched=events_dispatched,
            )

    # 有剩余任务，阻止停止
    goal = _read_goal(fusion_dir)
    updated_phase = state_to_phase(kernel.current_state)

    reason = f"""Continue executing the Fusion workflow.

Goal: {goal or '(not set)'}
Phase: {updated_phase}
Remaining: {total_remaining} tasks
Next task: {next_task}

Instructions:
1. Read .fusion/task_plan.md
2. Find next PENDING or IN_PROGRESS task
3. Execute based on task type:
   - implementation/verification → TDD flow (RED→GREEN→REFACTOR)
   - design/documentation/configuration/research → direct execution
4. Update task status to [COMPLETED]
5. Continue until all tasks done

Only ask user if 3-Strike exhausted."""

    if phase_corrected:
        reason += f"\n\nNote: Phase auto-corrected to {updated_phase} based on task states."

    system_message = f"🔄 Fusion | Phase: {updated_phase} | Remaining: {total_remaining} | Next: {next_task}"

    return StopGuardResult(
        should_block=True,
        decision="block",
        reason=reason,
        system_message=system_message,
        phase_corrected=phase_corrected,
        events_dispatched=events_dispatched,
    )


def _read_scheduler_status(fusion_dir: str) -> Optional[Dict[str, Any]]:
    """从 sessions.json 读取调度器状态（如果存在）"""
    sessions_file = Path(fusion_dir) / "sessions.json"
    if not sessions_file.exists():
        return None
    try:
        with open(sessions_file, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data.get("_runtime", {}).get("scheduler")
    except (json.JSONDecodeError, IOError):
        return None


def adapt_pretool(fusion_dir: str = ".fusion") -> PretoolResult:
    """
    适配 fusion-pretool.sh 的逻辑

    从 FSM 状态生成上下文提示行（只读操作，不修改状态）。
    """
    store = SessionStore(fusion_dir=fusion_dir)
    snapshot = store.load_snapshot()

    if not snapshot:
        return PretoolResult(active=False, lines=[])

    status = snapshot.get("status", "")
    if status != "in_progress":
        return PretoolResult(active=False, lines=[])

    goal = snapshot.get("goal", "?")[:60]
    phase = snapshot.get("current_phase", "EXECUTE")

    phase_map = {
        "INITIALIZE": "1/8", "ANALYZE": "2/8", "DECOMPOSE": "3/8",
        "EXECUTE": "4/8", "VERIFY": "5/8", "REVIEW": "6/8",
        "COMMIT": "7/8", "DELIVER": "8/8",
    }
    phase_num = phase_map.get(phase, "?/8")

    lines = [f"[fusion] Goal: {goal} | Phase: {phase} ({phase_num})"]

    counts = _read_task_counts(fusion_dir)
    total = sum(counts.values())

    if total > 0:
        next_task = _find_next_task(fusion_dir)
        completed = counts["completed"]
        task_index = completed + 1
        percent = completed * 100 // total if total > 0 else 0

        # 进度条
        filled = completed * 10 // total if total > 0 else 0
        bar = "█" * filled + "░" * (10 - filled)

        task_status = "IN_PROGRESS" if counts["in_progress"] > 0 else "PENDING"
        lines.append(f"[fusion] Task {task_index}/{total}: {next_task} [{task_status}]")
        lines.append(f"[fusion] Progress: {bar} {percent}% | Guardian: OK")

    # v2.5.0 调度器批次信息
    sched_status = _read_scheduler_status(fusion_dir)
    if sched_status and sched_status.get("enabled"):
        batch_id = sched_status.get("current_batch_id", 0)
        parallel = sched_status.get("parallel_tasks", 0)
        if batch_id > 0 or parallel > 0:
            lines.append(f"[fusion] Batch: {batch_id} | Parallel: {parallel} tasks")

    return PretoolResult(active=True, lines=lines)


def adapt_posttool(fusion_dir: str = ".fusion") -> PosttoolResult:
    """
    适配 fusion-posttool.sh 的逻辑

    检测 task_plan.md 进度变化（只读，不修改状态）。
    """
    store = SessionStore(fusion_dir=fusion_dir)
    snapshot = store.load_snapshot()

    if not snapshot or snapshot.get("status") != "in_progress":
        return PosttoolResult(changed=False, lines=[])

    counts = _read_task_counts(fusion_dir)
    total = sum(counts.values())
    current_snap = f"{counts['completed']}:{counts['pending']}:{counts['in_progress']}:{counts['failed']}"

    # 读取前一个快照
    snap_file = Path(fusion_dir) / ".progress_snapshot"
    prev_snap = ""
    if snap_file.exists():
        try:
            prev_snap = snap_file.read_text(encoding="utf-8").strip()
        except IOError:
            pass

    # 保存当前快照
    try:
        snap_file.write_text(current_snap, encoding="utf-8")
    except IOError:
        pass

    if current_snap == prev_snap:
        return PosttoolResult(changed=False, lines=[])

    # 解析变化
    lines = []
    prev_parts = prev_snap.split(":") if prev_snap else ["0", "0", "0", "0"]
    try:
        prev_completed = int(prev_parts[0])
    except (ValueError, IndexError):
        prev_completed = 0
    try:
        prev_failed = int(prev_parts[3])
    except (ValueError, IndexError):
        prev_failed = 0

    completed_delta = counts["completed"] - prev_completed
    failed_delta = counts["failed"] - prev_failed

    if completed_delta > 0:
        lines.append(f"[fusion] Task completed ({counts['completed']}/{total} done)")
        next_task = _find_next_task(fusion_dir)
        if counts["pending"] + counts["in_progress"] > 0:
            lines.append(f"[fusion] Next: {next_task}")
        else:
            lines.append("[fusion] All tasks completed! Proceed to VERIFY phase.")

    if failed_delta > 0:
        lines.append("[fusion] Task FAILED. Apply 3-Strike protocol.")

    # v2.5.0 调度器批次完成
    sched_status = _read_scheduler_status(fusion_dir)
    if sched_status and sched_status.get("enabled"):
        batch_id = sched_status.get("current_batch_id", 0)
        if batch_id > 0 and completed_delta > 0:
            lines.append(f"[fusion] Batch {batch_id} progress: +{completed_delta} tasks completed")

    return PosttoolResult(changed=True, lines=lines)


def _read_goal(fusion_dir: str) -> str:
    """从 sessions.json 读取 goal"""
    sessions_file = Path(fusion_dir) / "sessions.json"
    if not sessions_file.exists():
        return ""
    try:
        with open(sessions_file, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data.get("goal", "")
    except (json.JSONDecodeError, IOError):
        return ""


# ── CLI 入口 ──────────────────────────────────────
# Shell 脚本通过: python3 -m runtime.compat_v2 <command> [fusion_dir]

def main():
    """CLI 入口点，供 Shell 脚本调用"""
    if len(sys.argv) < 2:
        print("Usage: python3 -m runtime.compat_v2 <stop-guard|pretool|posttool> [fusion_dir]", file=sys.stderr)
        sys.exit(1)

    command = sys.argv[1]
    fusion_dir = sys.argv[2] if len(sys.argv) > 2 else ".fusion"

    try:
        if command == "stop-guard":
            result = adapt_stop_guard(fusion_dir)
            output = {
                "decision": result.decision,
                "should_block": result.should_block,
                "reason": result.reason,
                "systemMessage": result.system_message,
                "phase_corrected": result.phase_corrected,
                "events_dispatched": result.events_dispatched,
            }
            print(json.dumps(output, ensure_ascii=False))

        elif command == "pretool":
            result = adapt_pretool(fusion_dir)
            for line in result.lines:
                print(line)

        elif command == "posttool":
            result = adapt_posttool(fusion_dir)
            for line in result.lines:
                print(line)

        else:
            print(f"Unknown command: {command}", file=sys.stderr)
            sys.exit(1)

    except Exception as e:
        # 故障安全：任何异常都不阻塞 Shell 脚本
        print(f"compat_v2 error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
