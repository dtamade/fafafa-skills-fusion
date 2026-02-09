# Two-Phase Review Protocol

Fusion 使用两阶段审查确保代码质量：
1. **规格符合性审查** - 代码是否符合需求规格
2. **代码质量审查** - 代码是否达到生产标准

---

## Phase 1: 规格符合性审查

### 目的
确认实现与原始需求/规格完全匹配。

### 检查清单
- [ ] 所有需求功能已实现
- [ ] 没有遗漏的边界情况
- [ ] 没有额外的未请求功能
- [ ] API 接口符合设计规格
- [ ] 数据模型符合设计

### Prompt

```markdown
## Spec Compliance Review

### Original Requirements
{{REQUIREMENTS}}

### Implementation
{{CODE_OR_DIFF}}

### Review Questions
1. 是否所有需求都已实现？
2. 是否有遗漏的功能？
3. 是否有未请求的额外功能？
4. 实现是否与 API 设计一致？

### Output Format
```yaml
spec_compliance:
  status: PASS | FAIL

  requirements_coverage:
    - requirement: "<requirement 1>"
      implemented: true | false
      notes: "<any notes>"

  missing_features: []

  extra_features: []

  deviations:
    - spec: "<what spec says>"
      actual: "<what was implemented>"
      severity: critical | minor
```
```

---

## Phase 2: 代码质量审查

### 目的
确认代码达到生产级别标准。

### 检查清单
- [ ] 代码风格一致
- [ ] 命名清晰
- [ ] 错误处理完整
- [ ] 无安全漏洞
- [ ] 性能合理
- [ ] 测试覆盖充分

### Prompt

```markdown
## Code Quality Review

### Code
{{CODE_OR_DIFF}}

### Review Dimensions

1. **可读性**
   - 命名是否清晰？
   - 逻辑是否易懂？
   - 注释是否必要且有用？

2. **可维护性**
   - 代码是否模块化？
   - 是否遵循 DRY 原则？
   - 依赖是否合理？

3. **健壮性**
   - 错误处理是否完整？
   - 边界情况是否处理？
   - 输入验证是否充分？

4. **安全性**
   - 是否有注入漏洞？
   - 敏感数据是否保护？
   - 认证/授权是否正确？

5. **性能**
   - 是否有明显的性能问题？
   - 是否有 N+1 查询？
   - 是否需要缓存？

### Output Format
```yaml
code_quality:
  status: APPROVED | NEEDS_WORK

  score:
    readability: 1-10
    maintainability: 1-10
    robustness: 1-10
    security: 1-10
    performance: 1-10
    overall: 1-10

  issues:
    critical: []
    high: []
    medium: []
    low: []

  suggestions: []
```
```

---

## 审查流程

```
实现完成
    │
    ▼
Phase 1: 规格符合性审查
    │
    ├── PASS ──────────────┐
    │                      │
    ├── FAIL               │
    │     │                │
    │     ▼                │
    │   修复规格问题        │
    │     │                │
    │     ▼                │
    │   重新审查 Phase 1   │
    │                      │
    │                      ▼
    │              Phase 2: 代码质量审查
    │                      │
    │              ├── APPROVED ──► 完成
    │              │
    │              ├── NEEDS_WORK
    │              │     │
    │              │     ▼
    │              │   修复质量问题
    │              │     │
    │              │     ▼
    │              │   重新审查 Phase 2
    │              │
    └──────────────┴──────────────────────
```

---

## 关键原则

### 1. 顺序不能颠倒
**必须先通过规格审查，再进行质量审查。**

原因：
- 如果规格不对，质量再好也没用
- 避免浪费时间优化错误的代码

### 2. 不接受"差不多"
如果规格审查发现问题，必须修复后重新审查。
"Close enough" 是不可接受的。

### 3. 修复循环
```
发现问题 → 修复 → 重新审查 → 直到通过
```
不能跳过重新审查步骤。

---

## 集成到 Fusion 工作流

### 在 EXECUTE 阶段

每个实现任务完成后：
1. 运行 TDD 循环
2. Phase 1: 规格符合性审查
3. Phase 2: 代码质量审查
4. 全部通过后标记任务完成

### 在 REVIEW 阶段

对整个工作流的变更进行最终审查：
1. 汇总所有变更
2. 运行完整的两阶段审查
3. 生成审查报告
