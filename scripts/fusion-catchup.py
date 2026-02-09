#!/usr/bin/env python3
"""
fusion-catchup.py - Session Recovery Engine

Analyzes previous Claude Code session to recover context after /clear or session restart.
Stronger than planning-with-files' session-catchup.py:
  - Cross-validates task_plan.md states vs actual code changes (git diff)
  - Outputs precise recovery instructions, not just message listings
  - Detects state inconsistencies

Usage: python3 fusion-catchup.py [project-path]
"""

import json
import sys
import os
import subprocess
from pathlib import Path
from typing import List, Dict, Optional, Tuple

FUSION_DIR = ".fusion"
PLANNING_FILES = ["task_plan.md", "progress.md", "findings.md", "sessions.json"]


def get_project_dir(project_path: str) -> Path:
    """Convert project path to Claude's storage path format."""
    # Normalize path separators for cross-platform compatibility
    normalized = project_path.replace("\\", "/")
    # Remove Windows drive letter if present (e.g., C:/Users -> /Users)
    if len(normalized) >= 2 and normalized[1] == ":":
        normalized = normalized[2:]
    sanitized = normalized.replace("/", "-")
    if not sanitized.startswith("-"):
        sanitized = "-" + sanitized
    sanitized = sanitized.replace("_", "-")
    return Path.home() / ".claude" / "projects" / sanitized


def get_sessions_sorted(project_dir: Path) -> List[Path]:
    """Get session files sorted by modification time (newest first)."""
    sessions = list(project_dir.glob("*.jsonl"))
    main_sessions = [s for s in sessions if not s.name.startswith("agent-")]
    return sorted(main_sessions, key=lambda p: p.stat().st_mtime, reverse=True)


def parse_session_messages(session_file: Path) -> List[Dict]:
    """Parse all messages from a session JSONL file."""
    messages = []
    try:
        with open(session_file, "r", encoding="utf-8", errors="replace") as f:
            for line_num, line in enumerate(f):
                try:
                    data = json.loads(line)
                    data["_line_num"] = line_num
                    messages.append(data)
                except json.JSONDecodeError:
                    pass
    except (IOError, OSError):
        pass
    return messages


def find_last_fusion_update(messages: List[Dict]) -> Tuple[int, Optional[str]]:
    """Find the last time a .fusion/ file was written/edited."""
    last_line = -1
    last_file = None

    for msg in messages:
        if msg.get("type") != "assistant":
            continue
        content = msg.get("message", {}).get("content", [])
        if not isinstance(content, list):
            continue
        for item in content:
            if not isinstance(item, dict):
                continue
            if item.get("type") != "tool_use":
                continue
            tool_name = item.get("name", "")
            tool_input = item.get("input", {})
            if tool_name in ("Write", "Edit"):
                file_path = tool_input.get("file_path", "")
                # Match .fusion/ files specifically, not just any file ending with the name
                for pf in PLANNING_FILES:
                    if f"{FUSION_DIR}/{pf}" in file_path or file_path.endswith(f"/{FUSION_DIR}/{pf}"):
                        last_line = msg["_line_num"]
                        last_file = pf

    return last_line, last_file


def extract_unsynced(messages: List[Dict], after_line: int) -> List[Dict]:
    """Extract meaningful messages after a given line number."""
    result = []
    for msg in messages:
        if msg["_line_num"] <= after_line:
            continue

        msg_type = msg.get("type")
        is_meta = msg.get("isMeta", False)

        if msg_type == "user" and not is_meta:
            content = msg.get("message", {}).get("content", "")
            if isinstance(content, list):
                for item in content:
                    if isinstance(item, dict) and item.get("type") == "text":
                        content = item.get("text", "")
                        break
                else:
                    content = ""
            if isinstance(content, str) and len(content) > 20:
                if content.startswith(("<local-command", "<command-", "<task-notification")):
                    continue
                result.append({"role": "user", "content": content[:300], "line": msg["_line_num"]})

        elif msg_type == "assistant":
            msg_content = msg.get("message", {}).get("content", "")
            text_content = ""
            tool_uses = []

            if isinstance(msg_content, str):
                text_content = msg_content
            elif isinstance(msg_content, list):
                for item in msg_content:
                    if not isinstance(item, dict):
                        continue
                    if item.get("type") == "text":
                        text_content = item.get("text", "")
                    elif item.get("type") == "tool_use":
                        tool_name = item.get("name", "")
                        tool_input = item.get("input", {})
                        if tool_name == "Edit":
                            tool_uses.append(f"Edit: {tool_input.get('file_path', '?')}")
                        elif tool_name == "Write":
                            tool_uses.append(f"Write: {tool_input.get('file_path', '?')}")
                        elif tool_name == "Bash":
                            cmd = tool_input.get("command", "")[:80]
                            tool_uses.append(f"Bash: {cmd}")
                        else:
                            tool_uses.append(tool_name)

            if text_content or tool_uses:
                result.append({
                    "role": "assistant",
                    "content": text_content[:400] if text_content else "",
                    "tools": tool_uses,
                    "line": msg["_line_num"],
                })

    return result


def read_task_plan(project_path: str) -> Dict:
    """Parse task_plan.md to extract task states."""
    plan_path = os.path.join(project_path, FUSION_DIR, "task_plan.md")
    if not os.path.exists(plan_path):
        return {"total": 0, "completed": 0, "pending": 0, "in_progress": 0, "failed": 0, "tasks": []}

    tasks = []
    completed = pending = in_progress = failed = 0

    try:
        with open(plan_path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                line = line.strip()
                if line.startswith("### Task"):
                    task_name = line.split(": ", 1)[1] if ": " in line else line
                    status = "unknown"
                    if "[COMPLETED]" in line:
                        status = "COMPLETED"
                        completed += 1
                    elif "[PENDING]" in line:
                        status = "PENDING"
                        pending += 1
                    elif "[IN_PROGRESS]" in line:
                        status = "IN_PROGRESS"
                        in_progress += 1
                    elif "[FAILED]" in line:
                        status = "FAILED"
                        failed += 1

                    clean_name = task_name.split(" [")[0] if " [" in task_name else task_name
                    tasks.append({"name": clean_name, "status": status})
    except (IOError, OSError):
        pass

    return {
        "total": completed + pending + in_progress + failed,
        "completed": completed,
        "pending": pending,
        "in_progress": in_progress,
        "failed": failed,
        "tasks": tasks,
    }


def read_sessions_json(project_path: str) -> Dict:
    """Read sessions.json for workflow state."""
    sessions_path = os.path.join(project_path, FUSION_DIR, "sessions.json")
    if not os.path.exists(sessions_path):
        return {}
    try:
        with open(sessions_path, "r", encoding="utf-8", errors="replace") as f:
            return json.load(f)
    except (json.JSONDecodeError, IOError, OSError):
        return {}


def get_git_diff_stat(project_path: str) -> str:
    """Get git diff summary for cross-validation."""
    try:
        result = subprocess.run(
            ["git", "diff", "--stat", "HEAD"],
            capture_output=True,
            text=True,
            cwd=project_path,
            timeout=5,
        )
        return result.stdout.strip() if result.returncode == 0 else ""
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return ""


def cross_validate(task_info: Dict, session_info: Dict, git_diff: str) -> List[str]:
    """Cross-validate task_plan.md states vs sessions.json and git diff."""
    warnings = []

    # Check phase consistency
    phase = session_info.get("current_phase", "")
    if phase == "EXECUTE" and task_info["pending"] == 0 and task_info["in_progress"] == 0:
        if task_info["completed"] > 0:
            warnings.append("Phase mismatch: sessions.json says EXECUTE but all tasks completed. Should be VERIFY.")

    # Check if there are uncommitted changes but no in_progress task
    if git_diff and task_info["in_progress"] == 0 and task_info["pending"] > 0:
        warnings.append("Git has uncommitted changes but no task is IN_PROGRESS. A task may have been worked on without status update.")

    # Check for stuck in_progress without recent progress
    if task_info["in_progress"] > 0 and task_info["completed"] == 0 and task_info["total"] > 3:
        warnings.append("Task marked IN_PROGRESS but no tasks completed yet. May be stuck on first task.")

    return warnings


def main():
    project_path = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
    project_dir = get_project_dir(project_path)

    # Check for .fusion directory
    fusion_path = os.path.join(project_path, FUSION_DIR)
    if not os.path.isdir(fusion_path):
        return

    if not project_dir.exists():
        return

    sessions = get_sessions_sorted(project_dir)
    if not sessions:
        return

    # Find a substantial session
    target_session = None
    for session in sessions:
        if session.stat().st_size > 5000:
            target_session = session
            break

    if not target_session:
        return

    # Parse session
    messages = parse_session_messages(target_session)
    last_update_line, last_update_file = find_last_fusion_update(messages)

    # Extract unsynced messages
    if last_update_line < 0:
        unsynced = extract_unsynced(messages, max(0, len(messages) - 30))
    else:
        unsynced = extract_unsynced(messages, last_update_line)

    # Read current state
    task_info = read_task_plan(project_path)
    session_info = read_sessions_json(project_path)
    git_diff = get_git_diff_stat(project_path)

    # Cross-validate
    warnings = cross_validate(task_info, session_info, git_diff)

    # --- Output recovery report ---
    print("")
    print("[fusion-catchup] SESSION RECOVERY REPORT")
    print("=" * 60)

    # Current state
    goal = session_info.get("goal", "?")
    phase = session_info.get("current_phase", "?")
    status = session_info.get("status", "?")
    codex_session = session_info.get("codex_session", "")

    print(f"\nGoal: {goal}")
    print(f"Status: {status} | Phase: {phase}")
    print(f"Tasks: {task_info['completed']}/{task_info['total']} completed", end="")
    if task_info["in_progress"] > 0:
        print(f" | {task_info['in_progress']} in progress", end="")
    if task_info["failed"] > 0:
        print(f" | {task_info['failed']} failed", end="")
    print("")

    if codex_session:
        print(f"Codex Session: {codex_session}")

    # Warnings
    if warnings:
        print(f"\n--- WARNINGS ({len(warnings)}) ---")
        for w in warnings:
            print(f"  ⚠ {w}")

    # Git status
    if git_diff:
        print("\n--- UNCOMMITTED CHANGES ---")
        for line in git_diff.split("\n")[:10]:
            print(f"  {line}")

    # Unsynced context (last 10 messages)
    if unsynced:
        print(f"\n--- UNSYNCED CONTEXT ({len(unsynced)} messages) ---")
        if last_update_line >= 0:
            print(f"Last .fusion update: {last_update_file} at line #{last_update_line}")

        for msg in unsynced[-10:]:
            if msg["role"] == "user":
                print(f"  USER: {msg['content'][:200]}")
            else:
                if msg.get("content"):
                    print(f"  CLAUDE: {msg['content'][:200]}")
                if msg.get("tools"):
                    print(f"    Tools: {', '.join(msg['tools'][:4])}")

    # Recovery instructions
    print("\n--- RECOVERY INSTRUCTIONS ---")

    # Find next task
    next_task = None
    for t in task_info["tasks"]:
        if t["status"] == "IN_PROGRESS":
            next_task = t
            break
    if not next_task:
        for t in task_info["tasks"]:
            if t["status"] == "PENDING":
                next_task = t
                break

    if task_info["total"] == 0:
        print("  1. Create task plan: Read goal and run DECOMPOSE phase")
    elif next_task:
        print(f"  1. Continue task: {next_task['name']} [{next_task['status']}]")
        print(f"  2. Read .fusion/task_plan.md for full context")
        print(f"  3. Read .fusion/progress.md for recent history")
        if codex_session:
            print(f"  4. Resume Codex session: {codex_session}")
    elif task_info["pending"] == 0 and task_info["in_progress"] == 0:
        print("  1. All tasks completed! Proceed to VERIFY phase.")
    else:
        print("  1. Read .fusion/task_plan.md to find next action")
        print("  2. Read .fusion/progress.md for recent history")

    print("")
    print("=" * 60)


if __name__ == "__main__":
    main()
