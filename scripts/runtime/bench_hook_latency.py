"""
Fusion v2.1.0 Hook 性能基准测试

验证 Hook 延迟达标:
- PreToolUse (pretool) p95 < 80ms
- StopHook (stop-guard) p95 < 150ms

用法:
    python3 scripts/runtime/bench_hook_latency.py --runs 300
    python3 scripts/runtime/bench_hook_latency.py --runs 300 --pretool-p95-ms 80 --stop-p95-ms 150
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
    parser = argparse.ArgumentParser(description="Fusion v2.1.0 Hook 性能基准")
    parser.add_argument("--runs", type=int, default=300, help="测试轮数")
    parser.add_argument("--pretool-p95-ms", type=float, default=80, help="pretool p95 阈值 (ms)")
    parser.add_argument("--stop-p95-ms", type=float, default=150, help="stop-guard p95 阈值 (ms)")
    args = parser.parse_args()

    print(f"⚡ Fusion Hook Latency Benchmark ({args.runs} runs)")
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
