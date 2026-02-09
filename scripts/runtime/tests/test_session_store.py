"""
SessionStore 单元测试
"""

import unittest
import tempfile
import shutil
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.session_store import SessionStore, StoredEvent
from runtime.state_machine import State


class TestSessionStoreAppend(unittest.TestCase):
    """事件追加写入"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.store = SessionStore(fusion_dir=str(self.fusion_dir))

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_append_event_creates_file(self):
        """追加事件创建 events.jsonl"""
        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.assertTrue(self.store.events_file.exists())

    def test_append_event_returns_stored_event(self):
        """追加事件返回 StoredEvent"""
        stored = self.store.append_event("START", "IDLE", "INITIALIZE")
        self.assertIsInstance(stored, StoredEvent)
        self.assertEqual(stored.id, "evt_000001")
        self.assertEqual(stored.event_type, "START")
        self.assertEqual(stored.from_state, "IDLE")
        self.assertEqual(stored.to_state, "INITIALIZE")

    def test_append_multiple_events(self):
        """多次追加，ID 递增"""
        e1 = self.store.append_event("START", "IDLE", "INITIALIZE")
        e2 = self.store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")

        self.assertEqual(e1.id, "evt_000001")
        self.assertEqual(e2.id, "evt_000002")

    def test_append_event_with_payload(self):
        """带 payload 的事件"""
        stored = self.store.append_event(
            "TASK_DONE", "EXECUTE", "EXECUTE",
            payload={"task_id": "task_003", "result": "success"}
        )
        self.assertEqual(stored.payload["task_id"], "task_003")

    def test_append_event_file_content(self):
        """写入文件的内容格式正确"""
        self.store.append_event("START", "IDLE", "INITIALIZE")

        with open(self.store.events_file, "r") as f:
            line = f.readline()
            data = json.loads(line)

        self.assertEqual(data["type"], "START")
        self.assertEqual(data["from_state"], "IDLE")
        self.assertEqual(data["to_state"], "INITIALIZE")
        self.assertIn("id", data)
        self.assertIn("idempotency_key", data)
        self.assertIn("timestamp", data)


class TestSessionStoreIdempotency(unittest.TestCase):
    """幂等性校验"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.store = SessionStore(fusion_dir=str(self.fusion_dir))

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_same_idempotency_key_skips(self):
        """相同 key 的重复写入被跳过"""
        e1 = self.store.append_event(
            "START", "IDLE", "INITIALIZE",
            idempotency_key="key_001"
        )
        e2 = self.store.append_event(
            "START", "IDLE", "INITIALIZE",
            idempotency_key="key_001"
        )

        self.assertIsNotNone(e1)
        self.assertIsNone(e2)  # 跳过

        # 文件中只有一条记录
        events = self.store.load_events()
        self.assertEqual(len(events), 1)

    def test_different_keys_both_written(self):
        """不同 key 正常写入"""
        e1 = self.store.append_event(
            "START", "IDLE", "INITIALIZE",
            idempotency_key="key_001"
        )
        e2 = self.store.append_event(
            "INIT_DONE", "INITIALIZE", "ANALYZE",
            idempotency_key="key_002"
        )

        self.assertIsNotNone(e1)
        self.assertIsNotNone(e2)

    def test_auto_generated_keys_are_unique(self):
        """自动生成的 key 不会重复"""
        e1 = self.store.append_event("START", "IDLE", "INITIALIZE")
        e2 = self.store.append_event("START", "IDLE", "INITIALIZE")

        # 两次都应该写入（因为自动 key 包含时间戳，不会重复）
        self.assertIsNotNone(e1)
        self.assertIsNotNone(e2)
        self.assertNotEqual(e1.idempotency_key, e2.idempotency_key)


class TestSessionStoreLoadEvents(unittest.TestCase):
    """加载事件"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.store = SessionStore(fusion_dir=str(self.fusion_dir))

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_load_empty(self):
        """没有事件文件时返回空列表"""
        events = self.store.load_events()
        self.assertEqual(events, [])

    def test_load_events_order(self):
        """事件按写入顺序加载"""
        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")
        self.store.append_event("ANALYZE_DONE", "ANALYZE", "DECOMPOSE")

        events = self.store.load_events()
        self.assertEqual(len(events), 3)
        self.assertEqual(events[0].event_type, "START")
        self.assertEqual(events[1].event_type, "INIT_DONE")
        self.assertEqual(events[2].event_type, "ANALYZE_DONE")

    def test_load_events_skips_corrupt_lines(self):
        """损坏的行被跳过"""
        self.store.append_event("START", "IDLE", "INITIALIZE")

        # 写入一行损坏数据
        with open(self.store.events_file, "a") as f:
            f.write("not valid json\n")

        self.store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")

        events = self.store.load_events()
        self.assertEqual(len(events), 2)  # 损坏行被跳过

    def test_get_last_event(self):
        """获取最后一个事件"""
        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")

        last = self.store.get_last_event()
        self.assertEqual(last.event_type, "INIT_DONE")

    def test_get_last_event_empty(self):
        """没有事件时返回 None"""
        self.assertIsNone(self.store.get_last_event())

    def test_get_event_count(self):
        """获取事件总数"""
        self.assertEqual(self.store.get_event_count(), 0)

        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")

        self.assertEqual(self.store.get_event_count(), 2)


class TestSessionStoreReplay(unittest.TestCase):
    """事件重放"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.store = SessionStore(fusion_dir=str(self.fusion_dir))

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_replay_all(self):
        """重放所有事件"""
        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")
        self.store.append_event("ANALYZE_DONE", "ANALYZE", "DECOMPOSE")

        replayed = []
        count = self.store.replay(lambda evt: replayed.append(evt.event_type))

        self.assertEqual(count, 3)
        self.assertEqual(replayed, ["START", "INIT_DONE", "ANALYZE_DONE"])

    def test_replay_from_event_id(self):
        """从指定事件后开始重放"""
        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")
        self.store.append_event("ANALYZE_DONE", "ANALYZE", "DECOMPOSE")

        replayed = []
        count = self.store.replay(
            lambda evt: replayed.append(evt.event_type),
            from_event_id="evt_000001"
        )

        self.assertEqual(count, 2)
        self.assertEqual(replayed, ["INIT_DONE", "ANALYZE_DONE"])

    def test_replay_empty(self):
        """没有事件时重放返回 0"""
        count = self.store.replay(lambda evt: None)
        self.assertEqual(count, 0)

    def test_replay_rebuilds_processed_keys(self):
        """重放后重建幂等键集合"""
        e1 = self.store.append_event(
            "START", "IDLE", "INITIALIZE",
            idempotency_key="key_001"
        )

        # 模拟崩溃：清空内存中的 key 记录
        self.store._processed_keys.clear()
        self.store._event_counter = 0

        # 重放
        self.store.replay(lambda evt: None)

        # key 应被重建
        self.assertIn("key_001", self.store._processed_keys)
        self.assertEqual(self.store._event_counter, 1)

        # 现在重复 key 应被跳过
        result = self.store.append_event(
            "START", "IDLE", "INITIALIZE",
            idempotency_key="key_001"
        )
        self.assertIsNone(result)

    def test_replay_rebuilds_event_counter(self):
        """重放后事件计数器正确恢复"""
        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")
        self.store.append_event("ANALYZE_DONE", "ANALYZE", "DECOMPOSE")

        # 模拟崩溃
        self.store._event_counter = 0

        self.store.replay(lambda evt: None)

        # 计数器应恢复到 3
        self.assertEqual(self.store._event_counter, 3)

        # 下一个事件 ID 应为 evt_000004
        e4 = self.store.append_event("DECOMPOSE_DONE", "DECOMPOSE", "EXECUTE")
        self.assertEqual(e4.id, "evt_000004")

    def test_replay_from_nonexistent_id_replays_all(self):
        """从不存在的 ID 开始会重放全部事件"""
        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.store.append_event("INIT_DONE", "INITIALIZE", "ANALYZE")

        replayed = []
        count = self.store.replay(
            lambda evt: replayed.append(evt.event_type),
            from_event_id="evt_999999"
        )

        # 找不到 ID 时 start_idx 保持 0，重放全部
        self.assertEqual(count, 2)


class TestSessionStoreSnapshot(unittest.TestCase):
    """状态快照同步"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.store = SessionStore(fusion_dir=str(self.fusion_dir))

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_sync_snapshot_creates_file(self):
        """同步快照创建 sessions.json"""
        self.store.sync_snapshot(State.EXECUTE)
        self.assertTrue(self.store.sessions_file.exists())

    def test_sync_snapshot_content(self):
        """快照内容正确"""
        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.store.sync_snapshot(State.INITIALIZE)

        with open(self.store.sessions_file, "r") as f:
            data = json.load(f)

        self.assertEqual(data["current_phase"], "INITIALIZE")
        self.assertEqual(data["_runtime"]["version"], "2.1.0")
        self.assertEqual(data["_runtime"]["state"], "INITIALIZE")
        self.assertEqual(data["_runtime"]["last_event_id"], "evt_000001")

    def test_sync_snapshot_merges_extra(self):
        """额外数据被合并"""
        self.store.sync_snapshot(
            State.EXECUTE,
            extra={"goal": "实现用户认证"}
        )

        data = self.store.load_snapshot()
        self.assertEqual(data["goal"], "实现用户认证")
        self.assertEqual(data["current_phase"], "EXECUTE")

    def test_sync_snapshot_preserves_existing(self):
        """快照保留已有数据"""
        # 先写入一些数据
        with open(self.store.sessions_file, "w") as f:
            json.dump({"workflow_id": "test_123", "goal": "测试"}, f)

        self.store.sync_snapshot(State.ANALYZE)

        data = self.store.load_snapshot()
        self.assertEqual(data["workflow_id"], "test_123")
        self.assertEqual(data["goal"], "测试")
        self.assertEqual(data["current_phase"], "ANALYZE")

    def test_load_snapshot_empty(self):
        """没有 sessions.json 时返回空 dict"""
        data = self.store.load_snapshot()
        self.assertEqual(data, {})


class TestSessionStoreTruncate(unittest.TestCase):
    """清空操作"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.store = SessionStore(fusion_dir=str(self.fusion_dir))

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_truncate_clears_events(self):
        """清空事件日志"""
        self.store.append_event("START", "IDLE", "INITIALIZE")
        self.store.truncate()

        self.assertFalse(self.store.events_file.exists())
        self.assertEqual(self.store._event_counter, 0)
        self.assertEqual(self.store._processed_keys, set())


class TestSessionStoreRebuildKeys(unittest.TestCase):
    """幂等键重建"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.fusion_dir = Path(self.temp_dir) / ".fusion"
        self.fusion_dir.mkdir()
        self.store = SessionStore(fusion_dir=str(self.fusion_dir))

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_rebuild_processed_keys(self):
        """从文件重建幂等键"""
        self.store.append_event(
            "START", "IDLE", "INITIALIZE",
            idempotency_key="key_a"
        )
        self.store.append_event(
            "INIT_DONE", "INITIALIZE", "ANALYZE",
            idempotency_key="key_b"
        )

        # 清空内存
        self.store._processed_keys.clear()
        self.store._event_counter = 0

        # 重建
        self.store.rebuild_processed_keys()

        self.assertIn("key_a", self.store._processed_keys)
        self.assertIn("key_b", self.store._processed_keys)
        self.assertEqual(self.store._event_counter, 2)


class TestStoredEvent(unittest.TestCase):
    """事件数据对象"""

    def test_to_dict(self):
        evt = StoredEvent(
            id="evt_000001",
            idempotency_key="key_001",
            event_type="START",
            from_state="IDLE",
            to_state="INITIALIZE",
            payload={"x": 1},
            timestamp=1234567890.0,
        )
        d = evt.to_dict()
        self.assertEqual(d["id"], "evt_000001")
        self.assertEqual(d["type"], "START")

    def test_from_dict(self):
        data = {
            "id": "evt_000001",
            "idempotency_key": "key_001",
            "type": "START",
            "from_state": "IDLE",
            "to_state": "INITIALIZE",
            "payload": {},
            "timestamp": 1234567890.0,
        }
        evt = StoredEvent.from_dict(data)
        self.assertEqual(evt.id, "evt_000001")
        self.assertEqual(evt.event_type, "START")

    def test_from_dict_missing_optional(self):
        """缺少可选字段时使用默认值"""
        data = {
            "id": "evt_000001",
            "type": "START",
            "from_state": "IDLE",
            "to_state": "INITIALIZE",
            "timestamp": 1234567890.0,
        }
        evt = StoredEvent.from_dict(data)
        self.assertEqual(evt.idempotency_key, "")
        self.assertEqual(evt.payload, {})


if __name__ == "__main__":
    unittest.main(verbosity=2)
