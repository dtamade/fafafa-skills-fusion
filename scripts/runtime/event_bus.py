"""
Fusion Runtime Event Bus

进程内发布/订阅事件总线，提供错误隔离和通配符订阅。
"""

import logging
from typing import Callable, Dict, List, Any, Optional
from dataclasses import dataclass, field

logger = logging.getLogger(__name__)

# 订阅者回调类型
Subscriber = Callable[[str, Dict[str, Any]], None]


@dataclass
class Subscription:
    """订阅记录"""
    event_type: str
    callback: Subscriber
    priority: int = 0       # 数字越大越先执行
    error_handler: Optional[Callable[[Exception, str, Dict], None]] = None


class EventBus:
    """
    进程内事件总线

    特性:
    - 同步发布/订阅
    - 错误隔离：一个订阅者异常不影响其他订阅者
    - 通配符订阅 (*): 接收所有事件
    - 优先级：高优先级订阅者先执行
    """

    WILDCARD = "*"

    def __init__(self):
        self._subscriptions: Dict[str, List[Subscription]] = {}
        self._global_error_handler: Optional[Callable[[Exception, str, Dict], None]] = None

    def on(
        self,
        event_type: str,
        callback: Subscriber,
        priority: int = 0,
        error_handler: Optional[Callable[[Exception, str, Dict], None]] = None,
    ) -> Subscription:
        """
        订阅事件

        Args:
            event_type: 事件类型，"*" 表示订阅所有事件
            callback: 回调函数 (event_type, data) -> None
            priority: 优先级，数字越大越先执行
            error_handler: 该订阅者专属的错误处理器

        Returns:
            Subscription 对象，可用于取消订阅
        """
        sub = Subscription(
            event_type=event_type,
            callback=callback,
            priority=priority,
            error_handler=error_handler,
        )

        if event_type not in self._subscriptions:
            self._subscriptions[event_type] = []

        self._subscriptions[event_type].append(sub)
        # 按优先级降序排列
        self._subscriptions[event_type].sort(key=lambda s: s.priority, reverse=True)

        return sub

    def off(self, event_type: str, callback: Subscriber) -> bool:
        """
        取消订阅

        Returns:
            是否成功移除
        """
        subs = self._subscriptions.get(event_type, [])
        for i, sub in enumerate(subs):
            if sub.callback is callback:
                subs.pop(i)
                return True
        return False

    def off_subscription(self, subscription: Subscription) -> bool:
        """通过 Subscription 对象取消订阅"""
        return self.off(subscription.event_type, subscription.callback)

    def emit(self, event_type: str, data: Optional[Dict[str, Any]] = None) -> List[Exception]:
        """
        发布事件

        按优先级顺序通知订阅者。每个订阅者的异常被隔离捕获，
        不影响后续订阅者的执行。

        Args:
            event_type: 事件类型
            data: 事件数据

        Returns:
            执行过程中收集到的异常列表（空列表表示全部成功）
        """
        data = data or {}
        errors: List[Exception] = []

        # 收集目标订阅者：精确匹配 + 通配符
        targets: List[Subscription] = []
        targets.extend(self._subscriptions.get(event_type, []))
        if event_type != self.WILDCARD:
            targets.extend(self._subscriptions.get(self.WILDCARD, []))

        # 合并后重新按优先级排序
        targets.sort(key=lambda s: s.priority, reverse=True)

        for sub in targets:
            try:
                sub.callback(event_type, data)
            except Exception as e:
                errors.append(e)
                # 尝试订阅者专属错误处理器
                if sub.error_handler:
                    try:
                        sub.error_handler(e, event_type, data)
                    except Exception:
                        pass
                # 尝试全局错误处理器
                elif self._global_error_handler:
                    try:
                        self._global_error_handler(e, event_type, data)
                    except Exception:
                        pass
                else:
                    logger.warning(
                        "EventBus subscriber error on '%s': %s", event_type, e
                    )

        return errors

    def set_error_handler(
        self, handler: Callable[[Exception, str, Dict], None]
    ) -> None:
        """设置全局错误处理器"""
        self._global_error_handler = handler

    def subscriber_count(self, event_type: Optional[str] = None) -> int:
        """获取订阅者数量"""
        if event_type is not None:
            return len(self._subscriptions.get(event_type, []))
        return sum(len(subs) for subs in self._subscriptions.values())

    def clear(self, event_type: Optional[str] = None) -> None:
        """清除订阅"""
        if event_type is not None:
            self._subscriptions.pop(event_type, None)
        else:
            self._subscriptions.clear()

    def has_subscribers(self, event_type: str) -> bool:
        """检查是否有订阅者（包括通配符）"""
        if self._subscriptions.get(event_type):
            return True
        if event_type != self.WILDCARD and self._subscriptions.get(self.WILDCARD):
            return True
        return False
