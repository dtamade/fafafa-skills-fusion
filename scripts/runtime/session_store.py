"""
Fusion Runtime Session Store

Append-only 事件溯源存储，支持幂等写入和事件重放。
"""

import json
import time
import hashlib
import os
import tempfile
from pathlib import Path
from typing import Optional, Dict, Any, List, Callable
from dataclasses import dataclass, field

from .state_machine import State, Event, phase_to_state, state_to_phase


RUNTIME_VERSION = "2.6.3"


@dataclass
class StoredEvent:
    """持久化的事件记录"""
    id: str
    idempotency_key: str
    event_type: str
    from_state: str
    to_state: str
    payload: Dict[str, Any]
    timestamp: float

    def to_dict(self) -> Dict[str, Any]:
        return {
            "id": self.id,
            "idempotency_key": self.idempotency_key,
            "type": self.event_type,
            "from_state": self.from_state,
            "to_state": self.to_state,
            "payload": self.payload,
            "timestamp": self.timestamp,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "StoredEvent":
        return cls(
            id=data["id"],
            idempotency_key=data.get("idempotency_key", ""),
            event_type=data["type"],
            from_state=data["from_state"],
            to_state=data["to_state"],
            payload=data.get("payload") or {},
            timestamp=data["timestamp"],
        )


class SessionStore:
    """
    事件溯源存储

    职责:
    - append-only 事件写入到 events.jsonl
    - 幂等性校验 (idempotency key)
    - 事件重放 (replay) 重建状态
    - 状态快照同步到 sessions.json
    """

    def __init__(self, fusion_dir: str = ".fusion"):
        self.fusion_dir = Path(fusion_dir)
        self._event_counter: int = 0
        self._processed_keys: set = set()

    @property
    def events_file(self) -> Path:
        return self.fusion_dir / "events.jsonl"

    @property
    def sessions_file(self) -> Path:
        return self.fusion_dir / "sessions.json"

    def ensure_dir(self) -> None:
        """确保存储目录存在，并设置安全权限"""
        self.fusion_dir.mkdir(parents=True, exist_ok=True)
        # Set restrictive permissions (owner only) to prevent symlink attacks
        try:
            os.chmod(self.fusion_dir, 0o700)
        except OSError:
            pass  # Best effort - may fail on some filesystems

    def append_event(
        self,
        event_type: str,
        from_state: str,
        to_state: str,
        payload: Optional[Dict[str, Any]] = None,
        idempotency_key: Optional[str] = None,
    ) -> Optional[StoredEvent]:
        """
        追加事件到 events.jsonl

        如果提供了 idempotency_key 且该 key 已存在，则跳过写入
        并返回 None（幂等保证）。

        Args:
            event_type: 事件类型名称
            from_state: 源状态名称
            to_state: 目标状态名称
            payload: 事件附带数据
            idempotency_key: 幂等键（可选，不提供则自动生成）

        Returns:
            写入的 StoredEvent，或 None（幂等跳过时）
        """
        self.ensure_dir()

        # 生成或使用提供的幂等键
        key = idempotency_key or self._generate_idempotency_key(
            event_type, from_state, to_state, payload
        )

        # 幂等校验：已处理过的 key 直接跳过
        if key in self._processed_keys:
            return None

        # 生成事件 ID
        self._event_counter += 1
        event_id = f"evt_{self._event_counter:06d}"

        stored = StoredEvent(
            id=event_id,
            idempotency_key=key,
            event_type=event_type,
            from_state=from_state,
            to_state=to_state,
            payload=payload or {},
            timestamp=time.time(),
        )

        # 写入 events.jsonl
        try:
            with open(self.events_file, "a", encoding="utf-8") as f:
                f.write(json.dumps(stored.to_dict(), ensure_ascii=False) + "\n")
        except IOError as e:
            raise IOError(f"Failed to write event: {e}") from e

        # 记录已处理的 key
        self._processed_keys.add(key)

        return stored

    def load_events(self) -> List[StoredEvent]:
        """
        从 events.jsonl 加载所有事件

        Returns:
            事件列表，按写入顺序排列
        """
        events: List[StoredEvent] = []

        if not self.events_file.exists():
            return events

        with open(self.events_file, "r", encoding="utf-8") as f:
            for line_num, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    data = json.loads(line)
                    events.append(StoredEvent.from_dict(data))
                except (json.JSONDecodeError, KeyError) as e:
                    # 跳过损坏的行，记录警告
                    continue

        return events

    def replay(
        self,
        apply_fn: Callable[[StoredEvent], None],
        from_event_id: Optional[str] = None,
    ) -> int:
        """
        重放事件流

        按顺序对每个事件调用 apply_fn，用于重建状态。
        如果指定了 from_event_id，则从该事件之后开始重放。

        策略:
        - from_event_id 代表"已处理到的最后一个事件"，从其下一条开始
        - apply_fn 抛异常时立即停止（保证状态一致性）
        - 重放过程同时重建 _processed_keys 和 _event_counter

        Args:
            apply_fn: 应用函数，对每个事件执行
            from_event_id: 从哪个事件 ID 之后开始重放（可选）

        Returns:
            成功重放的事件数量
        """
        events = self.load_events()

        if not events:
            return 0

        # 确定起始位置
        start_idx = 0
        if from_event_id is not None:
            for i, evt in enumerate(events):
                if evt.id == from_event_id:
                    start_idx = i + 1
                    break

        # 重建幂等键集合和事件计数器（覆盖全部事件，不仅是重放部分）
        self._processed_keys.clear()
        self._event_counter = 0
        for evt in events:
            if evt.idempotency_key:
                self._processed_keys.add(evt.idempotency_key)
            try:
                num = int(evt.id.split("_")[1])
                self._event_counter = max(self._event_counter, num)
            except (IndexError, ValueError):
                pass

        # 对目标事件逐一应用（异常时停止，保证一致性）
        applied = 0
        for evt in events[start_idx:]:
            apply_fn(evt)
            applied += 1

        return applied

    def get_last_event(self) -> Optional[StoredEvent]:
        """获取最后一个事件"""
        events = self.load_events()
        return events[-1] if events else None

    def get_event_count(self) -> int:
        """获取事件总数"""
        if not self.events_file.exists():
            return 0
        count = 0
        with open(self.events_file, "r", encoding="utf-8") as f:
            for line in f:
                if line.strip():
                    count += 1
        return count

    def sync_snapshot(self, state: State, extra: Optional[Dict[str, Any]] = None) -> None:
        """
        将当前状态同步到 sessions.json 快照

        这是 events.jsonl 的辅助缓存，加速启动时的状态恢复。

        Args:
            state: 当前状态
            extra: 额外数据（合并到 sessions.json）
        """
        self.ensure_dir()

        data = {}
        if self.sessions_file.exists():
            try:
                with open(self.sessions_file, "r", encoding="utf-8") as f:
                    data = json.load(f)
            except (json.JSONDecodeError, IOError):
                pass

        # 更新核心字段
        data["current_phase"] = state_to_phase(state)

        # 更新 runtime 扩展
        last_event = self.get_last_event()
        data["_runtime"] = {
            "version": RUNTIME_VERSION,
            "state": state.name,
            "last_event_id": last_event.id if last_event else None,
            "last_event_counter": self._event_counter,
            "updated_at": time.time(),
        }

        # 合并额外数据
        if extra:
            data.update(extra)

        try:
            fd, tmp_path = tempfile.mkstemp(
                dir=str(self.sessions_file.parent), suffix=".tmp"
            )
            try:
                with os.fdopen(fd, "w", encoding="utf-8") as f:
                    json.dump(data, f, indent=2, ensure_ascii=False)
                os.replace(tmp_path, str(self.sessions_file))
            except BaseException:
                os.unlink(tmp_path)
                raise
        except IOError as e:
            raise IOError(f"Failed to sync snapshot: {e}") from e

    def load_snapshot(self) -> Dict[str, Any]:
        """从 sessions.json 加载快照"""
        if not self.sessions_file.exists():
            return {}
        try:
            with open(self.sessions_file, "r", encoding="utf-8") as f:
                return json.load(f)
        except (json.JSONDecodeError, IOError):
            return {}

    def rebuild_processed_keys(self) -> None:
        """从 events.jsonl 重建已处理的幂等键集合"""
        self._processed_keys.clear()
        events = self.load_events()
        for evt in events:
            if evt.idempotency_key:
                self._processed_keys.add(evt.idempotency_key)
        if events:
            # 恢复事件计数器
            last_num = 0
            for evt in events:
                try:
                    num = int(evt.id.split("_")[1])
                    last_num = max(last_num, num)
                except (IndexError, ValueError):
                    pass
            self._event_counter = last_num

    def truncate(self) -> None:
        """清空事件日志（仅用于测试）"""
        if self.events_file.exists():
            self.events_file.unlink()
        self._event_counter = 0
        self._processed_keys.clear()

    def _generate_idempotency_key(
        self,
        event_type: str,
        from_state: str,
        to_state: str,
        payload: Optional[Dict[str, Any]],
    ) -> str:
        """生成幂等键：基于事件内容的哈希 + 时间戳"""
        content = f"{event_type}:{from_state}:{to_state}:{json.dumps(payload or {}, sort_keys=True)}"
        content_hash = hashlib.sha256(content.encode()).hexdigest()[:12]
        return f"idem_{content_hash}_{time.time_ns()}"
