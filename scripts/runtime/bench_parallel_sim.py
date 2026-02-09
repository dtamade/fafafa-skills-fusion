"""
Fusion v2.5.0 并行模拟验收测试

验证 Phase 2 验收指标:
- 加速比 ≥ 1.4x (串行 vs 并行批次数)
- 冲突回滚率 ≤ 5%
- Token 超支率 ≤ 10%
- 硬上限突破 = 0

用法:
    python3 scripts/runtime/bench_parallel_sim.py
    python3 scripts/runtime/bench_parallel_sim.py --workflows 30
"""

import argparse
import random
import sys
from pathlib import Path
from dataclasses import dataclass, field
from typing import List, Dict

sys.path.insert(0, str(Path(__file__).parent.parent))

from runtime.task_graph import TaskGraph, TaskNode
from runtime.conflict_detector import ConflictDetector
from runtime.budget_manager import BudgetManager, BudgetConfig
from runtime.router import Router
from runtime.scheduler import Scheduler, SchedulerConfig


@dataclass
class SimResult:
    """单个工作流模拟结果"""
    workflow_id: int
    task_count: int
    serial_batches: int
    parallel_batches: int
    speedup: float
    conflict_deferred: int
    total_scheduled: int
    tokens_used: int
    token_limit: int
    over_budget: bool
    hard_limit_breach: bool


def _generate_dag(task_count: int, seed: int) -> List[TaskNode]:
    """
    生成随机 DAG:
    - 第 1 层: 1-2 个根任务
    - 中间层: 随机依赖前面的任务
    - 最后: 可能有一个汇聚任务
    """
    rng = random.Random(seed)
    nodes = []
    files = [f"src/mod_{i}.py" for i in range(task_count * 5)]  # 足够大的文件池降低冲突
    task_types = ["implementation", "verification", "documentation", "configuration"]

    for i in range(task_count):
        deps = []
        if i > 0:
            # 依赖前面 1-2 个任务
            max_dep_count = min(i, 2)
            dep_count = rng.randint(0, max_dep_count)
            deps = [str(rng.randint(0, i - 1)) for _ in range(dep_count)]
            deps = list(set(deps))  # 去重

        # 每个任务写 1-2 个文件
        write_count = rng.randint(1, 2)
        writeset = rng.sample(files, min(write_count, len(files)))

        nodes.append(TaskNode(
            task_id=str(i),
            name=f"task_{i}",
            dependencies=deps,
            writeset=writeset,
            task_type=rng.choice(task_types),
            cost_budget=rng.randint(100, 5000),
        ))

    return nodes


def _simulate_serial(graph: TaskGraph) -> int:
    """模拟串行执行，返回批次数"""
    sched = Scheduler(
        graph=graph,
        config=SchedulerConfig(enabled=False),
    )
    batches = 0
    while True:
        decision = sched.pick_next_batch()
        if decision is None:
            break
        if len(decision.batch.tasks) == 0:
            break
        for task in decision.batch.tasks:
            sched.on_task_done(task.task_id)
        sched.on_batch_done()
        batches += 1
    return batches


def _simulate_parallel(
    nodes: List[TaskNode],
    max_parallel: int,
    token_limit: int,
) -> SimResult:
    """
    模拟并行执行，返回详细结果
    """
    graph = TaskGraph(nodes)
    budget = BudgetManager(BudgetConfig(global_token_limit=token_limit))
    conflict = ConflictDetector()
    router = Router(budget_manager=budget)

    sched = Scheduler(
        graph=graph,
        config=SchedulerConfig(enabled=True, max_parallel=max_parallel),
        conflict_detector=conflict,
        budget_manager=budget,
        router=router,
    )

    batches = 0
    total_deferred = 0
    total_scheduled = 0
    hard_limit_breach = False

    while True:
        decision = sched.pick_next_batch()
        if decision is None:
            break
        if len(decision.batch.tasks) == 0:
            # 全局超预算，无法继续
            break

        total_deferred += len(decision.deferred)
        total_scheduled += len(decision.batch.tasks)

        for task in decision.batch.tasks:
            tokens = task.cost_budget
            # 检查硬上限：执行前已超限还继续执行 = 硬上限突破
            pre_status = budget.get_status()
            if pre_status.tokens_used + tokens > token_limit * 1.0:
                # 调度器应该阻止这种情况，如果没阻止就是硬上限突破
                if pre_status.over_budget:
                    hard_limit_breach = True

            sched.on_task_done(task.task_id, tokens_used=tokens, latency_ms=100)

        sched.on_batch_done()
        batches += 1

    status = budget.get_status()
    return SimResult(
        workflow_id=0,
        task_count=len(nodes),
        serial_batches=0,
        parallel_batches=batches,
        speedup=0.0,
        conflict_deferred=total_deferred,
        total_scheduled=total_scheduled,
        tokens_used=status.tokens_used,
        token_limit=token_limit,
        over_budget=status.over_budget,
        hard_limit_breach=hard_limit_breach,
    )


def run_simulation(num_workflows: int = 50, seed_base: int = 42) -> Dict:
    """运行完整模拟"""
    results: List[SimResult] = []

    for i in range(num_workflows):
        seed = seed_base + i
        rng = random.Random(seed)
        task_count = rng.randint(5, 15)
        max_parallel = rng.randint(2, 4)
        # 设置一个合理的预算 (覆盖约 100-150% 的总任务成本)
        estimated_total_cost = task_count * 2500  # 平均每任务 2500
        budget_ratio = rng.uniform(1.0, 1.5)
        token_limit = int(estimated_total_cost * budget_ratio)

        nodes = _generate_dag(task_count, seed)

        # 串行基线
        serial_graph = TaskGraph([
            TaskNode(
                task_id=n.task_id, name=n.name,
                dependencies=list(n.dependencies),
                writeset=list(n.writeset),
                task_type=n.task_type,
                cost_budget=n.cost_budget,
            ) for n in nodes
        ])
        serial_batches = _simulate_serial(serial_graph)

        # 并行模拟
        sim = _simulate_parallel(nodes, max_parallel, token_limit)
        sim.workflow_id = i + 1
        sim.serial_batches = serial_batches
        if sim.parallel_batches > 0:
            sim.speedup = serial_batches / sim.parallel_batches
        else:
            sim.speedup = 1.0

        results.append(sim)

    # 汇总
    speedups = [r.speedup for r in results if r.parallel_batches > 0]
    conflict_rates = [
        r.conflict_deferred / max(r.total_scheduled + r.conflict_deferred, 1)
        for r in results
    ]
    over_budget_count = sum(1 for r in results if r.over_budget)
    hard_breach_count = sum(1 for r in results if r.hard_limit_breach)

    median_speedup = sorted(speedups)[len(speedups) // 2] if speedups else 1.0
    mean_conflict_rate = sum(conflict_rates) / len(conflict_rates) if conflict_rates else 0.0
    over_budget_rate = over_budget_count / len(results) if results else 0.0

    return {
        "total_workflows": num_workflows,
        "results": results,
        "median_speedup": median_speedup,
        "mean_conflict_rate": mean_conflict_rate,
        "over_budget_rate": over_budget_rate,
        "hard_breach_count": hard_breach_count,
    }


def main():
    parser = argparse.ArgumentParser(description="Fusion v2.5.0 并行模拟验收")
    parser.add_argument("--workflows", type=int, default=50, help="工作流数量")
    args = parser.parse_args()

    print(f"🔬 Parallel Simulation Acceptance Test ({args.workflows} workflows)")
    print("=" * 60)

    summary = run_simulation(args.workflows)

    # 详细结果
    print(f"\n{'ID':>3} {'Tasks':>5} {'Serial':>6} {'Parallel':>8} {'Speedup':>7} {'Conflicts':>9} {'Budget':>8}")
    print("-" * 60)
    for r in summary["results"]:
        budget_flag = "⚠️OVER" if r.over_budget else "OK"
        print(f"{r.workflow_id:>3} {r.task_count:>5} {r.serial_batches:>6} "
              f"{r.parallel_batches:>8} {r.speedup:>7.2f}x {r.conflict_deferred:>9} "
              f"{budget_flag:>8}")

    # 汇总表格
    print(f"\n{'='*60}")
    print("验收指标汇总:")
    print(f"{'='*60}")

    metrics = [
        ("中位加速比", f"{summary['median_speedup']:.2f}x", "≥ 1.4x",
         summary["median_speedup"] >= 1.4),
        ("冲突回滚率", f"{summary['mean_conflict_rate']*100:.1f}%", "≤ 5%",
         summary["mean_conflict_rate"] <= 0.05),
        ("Token 超支率", f"{summary['over_budget_rate']*100:.1f}%", "≤ 10%",
         summary["over_budget_rate"] <= 0.10),
        ("硬上限突破", str(summary["hard_breach_count"]), "= 0",
         summary["hard_breach_count"] == 0),
    ]

    all_pass = True
    for name, actual, target, passed in metrics:
        status = "✅" if passed else "❌"
        print(f"  {status} {name}: {actual} (目标: {target})")
        if not passed:
            all_pass = False

    print(f"\n{'='*60}")
    if all_pass:
        print("✅ ALL ACCEPTANCE METRICS PASSED")
    else:
        print("❌ SOME METRICS FAILED")

    sys.exit(0 if all_pass else 1)


if __name__ == "__main__":
    main()
