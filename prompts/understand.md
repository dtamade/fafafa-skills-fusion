# Fusion UNDERSTAND 阶段提示词

## 概述

UNDERSTAND 阶段在执行前验证目标清晰度，通过静默扫描 + 智能追问确保 AI 正确理解用户意图。

---

## 1. 目标评分 (Goal Scoring)

### 输入
- `{goal}`: 用户输入的目标描述
- `{context}`: 项目上下文（技术栈、结构、相关文件）

### Prompt

```
评估目标完整性，输出 JSON 格式评分。

## 用户目标
{goal}

## 项目上下文
{context}

## 评分维度

### 1. 目标明确性 (clarity: 0-3)
- 3: 具体可执行 — "实现 JWT 认证的 POST /login API"
- 2: 清晰需少量推断 — "实现用户认证"
- 1: 模糊需大量假设 — "改进安全性"
- 0: 无法理解意图

### 2. 预期结果 (outcome: 0-3)
- 3: 明确验收标准 — "返回 JWT token，包含 userId"
- 2: 隐含可推断 — "用户能登录"
- 1: 不清晰
- 0: 完全缺失

### 3. 边界范围 (scope: 0-2)
- 2: 明确边界 — "仅后端 API，不含前端"
- 1: 可从上下文推断
- 0: 完全不清楚

### 4. 约束条件 (constraints: 0-2)
- 2: 明确约束 — "使用现有 Express 框架"
- 1: 可从项目推断
- 0: 完全缺失

## 输出格式

```json
{
  "scores": {
    "clarity": <0-3>,
    "outcome": <0-3>,
    "scope": <0-2>,
    "constraints": <0-2>
  },
  "total": <0-10>,
  "pass": <true if total >= 7>,
  "missing": ["缺失信息1", "缺失信息2"],
  "assumptions": ["基于上下文的假设1", "假设2"]
}
```

只输出 JSON，不要其他内容。
```

---

## 2. 追问生成 (Clarification Questions)

### 输入
- `{missing}`: 缺失的信息列表
- `{context}`: 项目上下文

### Prompt

```
为缺失信息生成一个多选问题。

## 缺失信息
{missing}

## 项目上下文
{context}

## 规则
1. 一次只生成一个问题（取最重要的缺失项）
2. 优先多选题（2-4 选项）
3. 基于项目上下文标注推荐选项
4. 选项简洁，每个 < 10 字
5. 如果能从上下文 100% 推断，返回 null

## 输出格式

```json
{
  "question": "问题文本",
  "options": [
    {"value": "opt1", "label": "选项1", "recommended": true, "reason": "推荐原因"},
    {"value": "opt2", "label": "选项2", "recommended": false},
    {"value": "opt3", "label": "选项3", "recommended": false},
    {"value": "other", "label": "其他", "recommended": false}
  ]
}
```

如果无需追问，输出：
```json
null
```
```

---

## 3. 理解摘要 (Understanding Summary)

### 输入
- `{goal}`: 用户目标
- `{context}`: 项目上下文
- `{answers}`: 追问答案（如有）
- `{assumptions}`: 推断的假设

### Prompt

```
生成简洁的理解确认摘要。

## 用户目标
{goal}

## 项目上下文
{context}

## 用户补充
{answers}

## 推断假设
{assumptions}

## 输出要求

生成 Markdown 格式的确认卡片：

1. **推断上下文**（3-5 条）
   - 技术栈、框架、测试工具
   - 每条 < 20 字

2. **计划范围**（3-5 个任务）
   - 高层任务描述
   - 每条 < 15 字

3. **关键假设**（如有）
   - 需要用户确认的推断
   - 用 ⚠️ 标记

4. **预估**
   - 任务数量
   - 时间范围（分钟）

## 风格
- 用 bullet points
- 简洁专业，无解释性语言
- 不要 "我理解..."、"您的需求是..."
- 直接陈述事实

## 输出格式

```markdown
## 📋 Fusion 理解确认

**目标**：<一句话目标>

**上下文**：
• <技术栈>
• <测试框架>
• <相关模块>

**计划范围**：
• <任务1>
• <任务2>
• <任务3>
• 预计 N 个任务，约 X-Y 分钟

**假设** ⚠️：
• <假设1>（如有误请纠正）
```
```

---

## 4. 技术栈检测规则

### 检测优先级

| 文件 | 技术栈 | 测试框架 |
|------|--------|----------|
| `package.json` | Node.js | jest/mocha/vitest |
| `go.mod` | Go | go test |
| `deno.json` | Deno | deno test |
| `Cargo.toml` | Rust | cargo test |
| `pom.xml` / `build.gradle` | Java | JUnit |
| `composer.json` | PHP | PHPUnit |

### 框架检测

| 依赖 | 框架 |
|------|------|
| `express` | Express.js |
| `fastapi` | FastAPI |
| `gin-gonic/gin` | Gin |
| `react` | React |
| `vue` | Vue |
| `svelte` | Svelte |

### 结构检测

| 目录 | 含义 |
|------|------|
| `src/` | 源代码 |
| `tests/` / `test/` / `__tests__/` | 测试 |
| `docs/` | 文档 |
| `scripts/` | 脚本 |
| `migrations/` | 数据库迁移 |

---

## 5. 追问阈值

| 场景 | 行为 |
|------|------|
| 评分 ≥ 7 | 直接展示理解摘要 |
| 评分 5-6 | 最多追问 1 轮 |
| 评分 < 5 | 最多追问 2 轮 |
| 2 轮后仍 < 7 | 建议用户重述目标 |

---

## 6. 跳过机制

用户可使用 `--force` 或 `--yolo` 跳过 UNDERSTAND 阶段：

```bash
/fusion --force "实现用户认证"
/fusion --yolo "快速修复 bug"
```

跳过时记录警告：
```
⚠️ 跳过理解确认（--force），直接开始执行
```
