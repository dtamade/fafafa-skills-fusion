"""
Fusion UNDERSTAND 阶段最小执行器

目标：在不依赖外部 LLM 的前提下，提供可执行、可测试的理解确认流程：
- 基础技术栈扫描
- 目标清晰度评分（规则启发式）
- 生成摘要并写入 findings.md
- 满足阈值后触发 UNDERSTAND_DONE
"""

from __future__ import annotations

import json
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Dict, List, Tuple

from .config import load_fusion_config
from .kernel import create_kernel
from .state_machine import Event, State


@dataclass
class UnderstandScores:
    clarity: int
    outcome: int
    scope: int
    constraints: int

    @property
    def total(self) -> int:
        return self.clarity + self.outcome + self.scope + self.constraints


@dataclass
class UnderstandResult:
    goal: str
    scores: UnderstandScores
    threshold: int
    pass_threshold: bool
    require_confirmation: bool
    needs_confirmation: bool
    context: Dict[str, str]
    assumptions: List[str]
    missing: List[str]
    summary_md: str


KEYWORDS_SCOPE = [
    "api", "endpoint", "前端", "后端", "backend", "frontend", "db", "database", "测试", "test", "docs", "文档",
]
KEYWORDS_OUTCOME = [
    "支持", "实现", "返回", "通过", "pass", "完成", "生成", "验证", "登录", "注册", "部署",
]
KEYWORDS_CONSTRAINT = [
    "使用", "must", "only", "不要", "不改", "兼容", "保持", "基于", "within", "under", "限制",
]


def detect_project_context(project_root: Path) -> Dict[str, str]:
    """轻量技术栈检测"""
    stack = "unknown"
    test_framework = "unknown"
    structure = []

    if (project_root / "package.json").exists():
        stack = "Node.js"
        try:
            pkg = json.loads((project_root / "package.json").read_text(encoding="utf-8"))
            deps = {
                **(pkg.get("dependencies") or {}),
                **(pkg.get("devDependencies") or {}),
            }
            if "vitest" in deps:
                test_framework = "vitest"
            elif "jest" in deps:
                test_framework = "jest"
            elif "mocha" in deps:
                test_framework = "mocha"
        except Exception:
            pass
    elif (project_root / "pyproject.toml").exists() or (project_root / "requirements.txt").exists():
        stack = "Python"
        test_framework = "pytest" if (project_root / "pytest.ini").exists() else "unknown"
    elif (project_root / "go.mod").exists():
        stack = "Go"
        test_framework = "go test"
    elif (project_root / "Cargo.toml").exists():
        stack = "Rust"
        test_framework = "cargo test"

    for p in ("src", "tests", "test", "docs", "scripts"):
        if (project_root / p).exists():
            structure.append(p)

    return {
        "tech_stack": stack,
        "test_framework": test_framework,
        "structure": ", ".join(structure) if structure else "unknown",
    }


def _contains_any(text: str, keywords: List[str]) -> bool:
    lowered = text.lower()
    return any(k.lower() in lowered for k in keywords)


def score_goal(goal: str) -> Tuple[UnderstandScores, List[str], List[str]]:
    """
    规则评分（0-10）：
    - clarity 0-3
    - outcome 0-3
    - scope 0-2
    - constraints 0-2
    """
    stripped = goal.strip()
    length = len(stripped)

    if length < 8:
        clarity = 0
    elif length < 20:
        clarity = 1
    elif length < 60:
        clarity = 2
    else:
        clarity = 3

    outcome = 2 if _contains_any(stripped, KEYWORDS_OUTCOME) else 1
    if "?" in stripped or "怎么" in stripped:
        outcome = max(0, outcome - 1)

    scope = 2 if _contains_any(stripped, KEYWORDS_SCOPE) else 1
    constraints = 2 if _contains_any(stripped, KEYWORDS_CONSTRAINT) else 0

    missing: List[str] = []
    assumptions: List[str] = []

    if outcome <= 1:
        missing.append("预期结果不够明确")
    if scope <= 1:
        missing.append("范围边界不够明确")
    if constraints == 0:
        assumptions.append("默认遵循现有代码风格与技术栈")

    scores = UnderstandScores(
        clarity=max(0, min(3, clarity)),
        outcome=max(0, min(3, outcome)),
        scope=max(0, min(2, scope)),
        constraints=max(0, min(2, constraints)),
    )
    return scores, missing, assumptions


def build_summary(
    goal: str,
    context: Dict[str, str],
    scores: UnderstandScores,
    assumptions: List[str],
    threshold: int,
    require_confirmation: bool,
) -> str:
    assumptions_lines = "\n".join([f"• {a}" for a in assumptions]) if assumptions else "• 无"
    mode = "strict" if require_confirmation else "auto-continue"
    return (
        "## 📋 Fusion 理解确认\n\n"
        f"**目标**：{goal}\n\n"
        "**上下文**：\n"
        f"• 技术栈：{context.get('tech_stack', 'unknown')}\n"
        f"• 测试框架：{context.get('test_framework', 'unknown')}\n"
        f"• 目录结构：{context.get('structure', 'unknown')}\n\n"
        "**评分**：\n"
        f"• clarity={scores.clarity}, outcome={scores.outcome}, scope={scores.scope}, constraints={scores.constraints}\n"
        f"• total={scores.total}/10 (threshold={threshold}, mode={mode})\n\n"
        "**假设** ⚠️：\n"
        f"{assumptions_lines}\n"
    )


def write_findings(fusion_dir: Path, result: UnderstandResult) -> None:
    findings = fusion_dir / "findings.md"
    with open(findings, "a", encoding="utf-8") as f:
        f.write("\n## UNDERSTAND Phase\n\n")
        f.write(f"**原始目标**: {result.goal}\n")
        f.write(f"**评分**: {result.scores.total}/10 (threshold={result.threshold})\n")
        f.write(f"**模式**: {'strict' if result.require_confirmation else 'auto-continue'}\n")
        if result.needs_confirmation:
            f.write("**状态**: 需要补充澄清后再推进\n")
        f.write("\n### 上下文\n")
        for k, v in result.context.items():
            f.write(f"- {k}: {v}\n")
        if result.missing:
            f.write("\n### 缺失信息\n")
            for m in result.missing:
                f.write(f"- {m}\n")
        if result.assumptions:
            f.write("\n### 假设\n")
            for a in result.assumptions:
                f.write(f"- {a}\n")


def run_understand(goal: str, fusion_dir: str = ".fusion", project_root: str = ".") -> UnderstandResult:
    fusion_path = Path(fusion_dir)
    context = detect_project_context(Path(project_root))
    scores, missing, assumptions = score_goal(goal)

    cfg = load_fusion_config(fusion_dir)
    threshold = int(cfg.get("understand_pass_threshold", 7))
    if threshold < 0:
        threshold = 0
    if threshold > 10:
        threshold = 10
    require_confirmation = bool(cfg.get("understand_require_confirmation", False))

    pass_threshold = scores.total >= threshold
    needs_confirmation = (not pass_threshold) and require_confirmation

    summary = build_summary(
        goal,
        context,
        scores,
        assumptions,
        threshold,
        require_confirmation,
    )

    result = UnderstandResult(
        goal=goal,
        scores=scores,
        threshold=threshold,
        pass_threshold=pass_threshold,
        require_confirmation=require_confirmation,
        needs_confirmation=needs_confirmation,
        context=context,
        assumptions=assumptions,
        missing=missing,
        summary_md=summary,
    )

    write_findings(fusion_path, result)

    kernel = create_kernel(str(fusion_path))
    if kernel.current_state == State.UNDERSTAND and not needs_confirmation:
        kernel.dispatch(
            Event.UNDERSTAND_DONE,
            payload={
                "goal": goal,
                "understand": {
                    "scores": asdict(scores),
                    "total": scores.total,
                    "threshold": threshold,
                    "pass": result.pass_threshold,
                    "needs_confirmation": needs_confirmation,
                    "missing": missing,
                    "assumptions": assumptions,
                },
            },
            idempotency_key=f"understand:{goal}",
        )

    return result


def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(description="Run Fusion UNDERSTAND phase")
    parser.add_argument("goal", help="Workflow goal")
    parser.add_argument("--fusion-dir", default=".fusion")
    parser.add_argument("--project-root", default=".")
    parser.add_argument("--json", action="store_true", help="Emit machine-readable JSON output")
    args = parser.parse_args()

    result = run_understand(
        goal=args.goal,
        fusion_dir=args.fusion_dir,
        project_root=args.project_root,
    )

    if args.json:
        print(
            json.dumps(
                {
                    "goal": result.goal,
                    "scores": asdict(result.scores),
                    "total": result.scores.total,
                    "threshold": result.threshold,
                    "pass_threshold": result.pass_threshold,
                    "require_confirmation": result.require_confirmation,
                    "needs_confirmation": result.needs_confirmation,
                    "missing": result.missing,
                    "assumptions": result.assumptions,
                    "summary_md": result.summary_md,
                },
                ensure_ascii=False,
            )
        )
    else:
        print(result.summary_md)

    if result.needs_confirmation:
        print("[fusion] ⚠️ UNDERSTAND needs clarification (< threshold, strict mode).")
        return 20

    if not result.pass_threshold:
        print("[fusion] ⚠️ Goal clarity below threshold. Proceeding with assumptions.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
