"""
Fusion v2.5.0 Hook + Scheduler 性能基准测试

验证 Hook 延迟达标:
- PreToolUse (pretool) p95 < 80ms
- StopHook (stop-guard) p95 < 150ms
- Scheduler.pick_next_batch() p95 < 200ms

用法:
    python3 scripts/runtime/bench_hook_latency.py --runs 300
    python3 scripts/runtime/bench_hook_latency.py --runs 300 --pretool-p95-ms 80 --stop-p95-ms 150 --sched-p95-ms 200
"""

import argparse
import sys
import time
import tempfile
import shutil
import json
import statistics
from pathlib import Path
from dataclasses import dataclass
from typing import List

sys.path.insert(0, str(Path(__file__).parent.parent))

from runtime.compat_v2 import adapt_pretool, adapt_posttool, adapt_stop_guard
from runtime.task_graph import TaskGraph, TaskNode
from runtime.conflict_detector import ConflictDetector
from runtime.budget_manager import BudgetManager, BudgetConfig
from runtime.router import Router
from runtime.scheduler import Scheduler, SchedulerConfig
from runtime.task_graph import TaskGraph, TaskNode
from runtime.conflict_detector import ConflictDetector
from runtime.budget_manager import BudgetManager, BudgetConfig
from runtime.router import Router
from runtime.scheduler import Scheduler, SchedulerConfig


@dataclass
class BenchResult:
    name: str
    runs: int
    p50_ms: float
    p95_ms: float
    p99_ms: float
    min_ms: float
    max_ms: float
    mean_ms: float


def _setup_active_workflow() -> Path:
    """创建带活跃工作流的临时 .fusion 目录"""
    tmp = Path(tempfile.mkdtemp())
    fusion_dir = tmp / ".fusion"
    fusion_dir.mkdir()

    with open(fusion_dir / "sessions.json", "w") as f:
        json.dump({
            "status": "in_progress",
            "current_phase": "EXECUTE",
            "goal": "性能基准测试",
            "_runtime": {"version": "2.1.0", "state": "EXECUTE", "last_event_counter": 5}
        }, f)

    with open(fusion_dir / "task_plan.md", "w") as f:
        f.write(
            "### Task 1: 初始化模块 [COMPLETED]\n"
            "### Task 2: 实现核心逻辑 [COMPLETED]\n"
            "### Task 3: 添加错误处理 [IN_PROGRESS]\n"
            "### Task 4: 编写单元测试 [PENDING]\n"
            "### Task 5: 集成测试 [PENDING]\n"
        )

    (fusion_dir / ".progress_snapshot").write_text("2:2:1:0")

    return fusion_dir


def bench_pretool(fusion_dir: Path, runs: int) -> BenchResult:
    """基准测试 pretool 延迟"""
    latencies = []
    for _ in range(runs):
        t0 = time.monotonic()
        adapt_pretool(str(fusion_dir))
        latencies.append((time.monotonic() - t0) * 1000)

    latencies.sort()
    return BenchResult(
        name="pretool",
        runs=runs,
        p50_ms=_percentile(latencies, 50),
        p95_ms=_percentile(latencies, 95),
        p99_ms=_percentile(latencies, 99),
        min_ms=min(latencies),
        max_ms=max(latencies),
        mean_ms=statistics.mean(latencies),
    )


def bench_posttool(fusion_dir: Path, runs: int) -> BenchResult:
    """基准测试 posttool 延迟"""
    latencies = []
    for _ in range(runs):
        t0 = time.monotonic()
        adapt_posttool(str(fusion_dir))
        latencies.append((time.monotonic() - t0) * 1000)

    latencies.sort()
    return BenchResult(
        name="posttool",
        runs=runs,
        p50_ms=_percentile(latencies, 50),
        p95_ms=_percentile(latencies, 95),
        p99_ms=_percentile(latencies, 99),
        min_ms=min(latencies),
        max_ms=max(latencies),
        mean_ms=statistics.mean(latencies),
    )


def bench_stop_guard(fusion_dir: Path, runs: int) -> BenchResult:
    """基准测试 stop-guard 延迟"""
    latencies = []
    for _ in range(runs):
        t0 = time.monotonic()
        adapt_stop_guard(str(fusion_dir))
        latencies.append((time.monotonic() - t0) * 1000)

    latencies.sort()
    return BenchResult(
        name="stop-guard",
        runs=runs,
        p50_ms=_percentile(latencies, 50),
        p95_ms=_percentile(latencies, 95),
        p99_ms=_percentile(latencies, 99),
        min_ms=min(latencies),
        max_ms=max(latencies),
        mean_ms=statistics.mean(latencies),
    )


def _build_scheduler_graph(task_count: int) -> Scheduler:
    """构建指定规模的 DAG 用于调度基准"""
    # 构建菱形 DAG: 第一个任务是根，中间任务依赖根，最后一个依赖所有中间任务
    nodes = []
    mid_count = max(1, task_count - 2)

    nodes.append(TaskNode(task_id="0", name="root", writeset=["root.py"]))
    for i in range(1, mid_count + 1):
        nodes.append(TaskNode(
            task_id=str(i), name=f"task_{i}",
            dependencies=["0"],
            writeset=[f"src/mod_{i}.py"],
            cost_budget=1000,
        ))
    if task_count > 2:
        nodes.append(TaskNode(
            task_id=str(mid_count + 1), name="final",
            dependencies=[str(i) for i in range(1, mid_count + 1)],
            writeset=["tests/test_all.py"],
        ))

    graph = TaskGraph(nodes)
    # 完成根任务以使中间层就绪
    graph.mark_completed("0")

    conflict = ConflictDetector()
    budget = BudgetManager(BudgetConfig(global_token_limit=1000000))
    router = Router(budget_manager=budget)

    return Scheduler(
        graph=graph,
        config=SchedulerConfig(enabled=True, max_parallel=task_count),
        conflict_detector=conflict,
        budget_manager=budget,
        router=router,
    )


def bench_scheduler(task_count: int, runs: int) -> BenchResult:
    """基准测试 Scheduler.pick_next_batch() 延迟"""
    sched = _build_scheduler_graph(task_count)
    latencies = []

    for _ in range(runs):
        # 每次重建以避免状态累积
        sched = _build_scheduler_graph(task_count)
        t0 = time.monotonic()
        sched.pick_next_batch()
        latencies.append((time.monotonic() - t0) * 1000)

    latencies.sort()
    return BenchResult(
        name=f"scheduler({task_count}tasks)",
        runs=runs,
        p50_ms=_percentile(latencies, 50),
        p95_ms=_percentile(latencies, 95),
        p99_ms=_percentile(latencies, 99),
        min_ms=min(latencies),
        max_ms=max(latencies),
        mean_ms=statistics.mean(latencies),
    )


def _percentile(data: List[float], pct: int) -> float:
    """计算百分位数"""
    n = len(data)
    if n == 0:
        return 0.0
    idx = int(n * pct / 100)
    idx = min(idx, n - 1)
    return data[idx]


def print_result(result: BenchResult, threshold_ms: float = 0):
    """输出基准测试结果"""
    status = "✅" if result.p95_ms <= threshold_ms or threshold_ms == 0 else "❌"
    print(f"\n{status} {result.name} ({result.runs} runs)")
    print(f"  p50:  {result.p50_ms:6.2f}ms")
    print(f"  p95:  {result.p95_ms:6.2f}ms" + (f"  (threshold: {threshold_ms}ms)" if threshold_ms else ""))
    print(f"  p99:  {result.p99_ms:6.2f}ms")
    print(f"  min:  {result.min_ms:6.2f}ms")
    print(f"  max:  {result.max_ms:6.2f}ms")
    print(f"  mean: {result.mean_ms:6.2f}ms")


def main():
    parser = argparse.ArgumentParser(description="Fusion v2.5.0 Hook + Scheduler 性能基准")
    parser.add_argument("--runs", type=int, default=300, help="测试轮数")
    parser.add_argument("--pretool-p95-ms", type=float, default=80, help="pretool p95 阈值 (ms)")
    parser.add_argument("--stop-p95-ms", type=float, default=150, help="stop-guard p95 阈值 (ms)")
    parser.add_argument("--sched-p95-ms", type=float, default=200, help="scheduler p95 阈值 (ms)")
    args = parser.parse_args()

    print(f"⚡ Fusion Hook + Scheduler Latency Benchmark ({args.runs} runs)")
    print("=" * 50)

    fusion_dir = _setup_active_workflow()
    all_pass = True

    try:
        # Warmup (10 runs)
        for _ in range(10):
            adapt_pretool(str(fusion_dir))
            adapt_stop_guard(str(fusion_dir))

        # Bench pretool
        pretool_result = bench_pretool(fusion_dir, args.runs)
        print_result(pretool_result, args.pretool_p95_ms)
        if pretool_result.p95_ms > args.pretool_p95_ms:
            all_pass = False

        # Bench posttool
        posttool_result = bench_posttool(fusion_dir, args.runs)
        print_result(posttool_result, args.pretool_p95_ms)  # 同样阈值
        if posttool_result.p95_ms > args.pretool_p95_ms:
            all_pass = False

        # Bench stop-guard
        stop_result = bench_stop_guard(fusion_dir, args.runs)
        print_result(stop_result, args.stop_p95_ms)
        if stop_result.p95_ms > args.stop_p95_ms:
            all_pass = False

        # Bench scheduler (不同规模)
        for task_count in [5, 10, 20]:
            sched_result = bench_scheduler(task_count, args.runs)
            print_result(sched_result, args.sched_p95_ms)
            if sched_result.p95_ms > args.sched_p95_ms:
                all_pass = False

        print(f"\n{'='*50}")
        if all_pass:
            print("✅ ALL BENCHMARKS PASSED")
        else:
            print("❌ SOME BENCHMARKS FAILED")

    finally:
        shutil.rmtree(fusion_dir.parent, ignore_errors=True)

    sys.exit(0 if all_pass else 1)


if __name__ == "__main__":
    main()
