"""
EventBus 单元测试
"""

import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from runtime.event_bus import EventBus, Subscription


class TestEventBusBasic(unittest.TestCase):
    """EventBus 基础功能"""

    def setUp(self):
        self.bus = EventBus()

    def test_on_and_emit(self):
        """订阅并接收事件"""
        received = []

        def handler(event_type, data):
            received.append((event_type, data))

        self.bus.on("test_event", handler)
        self.bus.emit("test_event", {"key": "value"})

        self.assertEqual(len(received), 1)
        self.assertEqual(received[0][0], "test_event")
        self.assertEqual(received[0][1]["key"], "value")

    def test_emit_no_subscribers(self):
        """没有订阅者时 emit 不报错"""
        errors = self.bus.emit("nonexistent_event", {"data": 1})
        self.assertEqual(errors, [])

    def test_off_removes_subscriber(self):
        """off 移除订阅者"""
        received = []

        def handler(event_type, data):
            received.append(data)

        self.bus.on("test", handler)
        result = self.bus.off("test", handler)
        self.assertTrue(result)

        self.bus.emit("test", {"x": 1})
        self.assertEqual(len(received), 0)

    def test_off_nonexistent_returns_false(self):
        """移除不存在的订阅返回 False"""
        result = self.bus.off("test", lambda et, d: None)
        self.assertFalse(result)

    def test_off_subscription_object(self):
        """通过 Subscription 对象取消订阅"""
        received = []
        handler = lambda et, d: received.append(d)
        sub = self.bus.on("test", handler)

        result = self.bus.off_subscription(sub)
        self.assertTrue(result)

        self.bus.emit("test", {})
        self.assertEqual(len(received), 0)

    def test_multiple_subscribers(self):
        """多个订阅者都能收到事件"""
        results = {"a": [], "b": []}

        self.bus.on("evt", lambda et, d: results["a"].append(d))
        self.bus.on("evt", lambda et, d: results["b"].append(d))

        self.bus.emit("evt", {"val": 42})

        self.assertEqual(len(results["a"]), 1)
        self.assertEqual(len(results["b"]), 1)

    def test_emit_returns_empty_on_success(self):
        """所有订阅者成功时返回空列表"""
        self.bus.on("ok", lambda et, d: None)
        errors = self.bus.emit("ok", {})
        self.assertEqual(errors, [])

    def test_emit_with_none_data(self):
        """data 为 None 时转为空 dict"""
        received = []
        self.bus.on("test", lambda et, d: received.append(d))
        self.bus.emit("test")
        self.assertEqual(received[0], {})


class TestEventBusWildcard(unittest.TestCase):
    """通配符订阅"""

    def setUp(self):
        self.bus = EventBus()

    def test_wildcard_receives_all_events(self):
        """通配符订阅者接收所有事件"""
        received = []
        self.bus.on("*", lambda et, d: received.append(et))

        self.bus.emit("event_a", {})
        self.bus.emit("event_b", {})
        self.bus.emit("event_c", {})

        self.assertEqual(received, ["event_a", "event_b", "event_c"])

    def test_wildcard_and_specific_both_fire(self):
        """精确订阅和通配符订阅同时触发"""
        specific = []
        wildcard = []

        self.bus.on("my_event", lambda et, d: specific.append(d))
        self.bus.on("*", lambda et, d: wildcard.append(d))

        self.bus.emit("my_event", {"x": 1})

        self.assertEqual(len(specific), 1)
        self.assertEqual(len(wildcard), 1)

    def test_wildcard_not_triggered_by_wildcard_emit(self):
        """emit('*') 不会导致通配符订阅者被调用两次"""
        received = []
        self.bus.on("*", lambda et, d: received.append(et))

        self.bus.emit("*", {})
        # 当 event_type == "*" 时不额外收集通配符订阅
        self.assertEqual(len(received), 1)


class TestEventBusPriority(unittest.TestCase):
    """优先级排序"""

    def setUp(self):
        self.bus = EventBus()

    def test_higher_priority_fires_first(self):
        """高优先级订阅者先执行"""
        order = []

        self.bus.on("evt", lambda et, d: order.append("low"), priority=0)
        self.bus.on("evt", lambda et, d: order.append("high"), priority=10)
        self.bus.on("evt", lambda et, d: order.append("mid"), priority=5)

        self.bus.emit("evt", {})

        self.assertEqual(order, ["high", "mid", "low"])

    def test_priority_across_wildcard_and_specific(self):
        """跨通配符和精确订阅的优先级排序"""
        order = []

        self.bus.on("evt", lambda et, d: order.append("specific"), priority=5)
        self.bus.on("*", lambda et, d: order.append("wildcard"), priority=10)

        self.bus.emit("evt", {})

        self.assertEqual(order, ["wildcard", "specific"])


class TestEventBusErrorIsolation(unittest.TestCase):
    """错误隔离"""

    def setUp(self):
        self.bus = EventBus()

    def test_error_does_not_block_other_subscribers(self):
        """一个订阅者异常不影响后续订阅者"""
        results = []

        def good_handler(et, d):
            results.append("ok")

        def bad_handler(et, d):
            raise ValueError("boom")

        self.bus.on("evt", good_handler, priority=0)
        self.bus.on("evt", bad_handler, priority=10)  # 先执行，会抛异常

        errors = self.bus.emit("evt", {})

        self.assertEqual(len(errors), 1)
        self.assertIsInstance(errors[0], ValueError)
        self.assertEqual(results, ["ok"])  # 后续订阅者正常执行

    def test_subscriber_error_handler(self):
        """订阅者专属错误处理器"""
        caught = []

        def error_handler(exc, event_type, data):
            caught.append((str(exc), event_type))

        def bad_handler(et, d):
            raise RuntimeError("fail")

        self.bus.on("evt", bad_handler, error_handler=error_handler)
        self.bus.emit("evt", {"x": 1})

        self.assertEqual(len(caught), 1)
        self.assertEqual(caught[0], ("fail", "evt"))

    def test_global_error_handler(self):
        """全局错误处理器"""
        caught = []

        self.bus.set_error_handler(
            lambda exc, et, d: caught.append(str(exc))
        )

        def bad_handler(et, d):
            raise RuntimeError("global_error")

        self.bus.on("evt", bad_handler)
        self.bus.emit("evt", {})

        self.assertEqual(caught, ["global_error"])


class TestEventBusUtility(unittest.TestCase):
    """辅助方法"""

    def setUp(self):
        self.bus = EventBus()

    def test_subscriber_count_specific(self):
        """指定事件类型的订阅者计数"""
        self.bus.on("a", lambda et, d: None)
        self.bus.on("a", lambda et, d: None)
        self.bus.on("b", lambda et, d: None)

        self.assertEqual(self.bus.subscriber_count("a"), 2)
        self.assertEqual(self.bus.subscriber_count("b"), 1)
        self.assertEqual(self.bus.subscriber_count("c"), 0)

    def test_subscriber_count_total(self):
        """总订阅者计数"""
        self.bus.on("a", lambda et, d: None)
        self.bus.on("b", lambda et, d: None)

        self.assertEqual(self.bus.subscriber_count(), 2)

    def test_clear_specific(self):
        """清除特定事件类型的订阅"""
        self.bus.on("a", lambda et, d: None)
        self.bus.on("b", lambda et, d: None)

        self.bus.clear("a")

        self.assertEqual(self.bus.subscriber_count("a"), 0)
        self.assertEqual(self.bus.subscriber_count("b"), 1)

    def test_clear_all(self):
        """清除所有订阅"""
        self.bus.on("a", lambda et, d: None)
        self.bus.on("b", lambda et, d: None)

        self.bus.clear()

        self.assertEqual(self.bus.subscriber_count(), 0)

    def test_has_subscribers(self):
        """检查是否有订阅者"""
        self.assertFalse(self.bus.has_subscribers("test"))

        self.bus.on("test", lambda et, d: None)
        self.assertTrue(self.bus.has_subscribers("test"))

    def test_has_subscribers_via_wildcard(self):
        """通配符订阅让所有事件类型都 has_subscribers"""
        self.bus.on("*", lambda et, d: None)
        self.assertTrue(self.bus.has_subscribers("any_event"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
