# Root Session Example Notes

This repository does **not** keep live runtime session artifacts at the repository root.

## Canonical locations

- Live mutable session state: `.fusion/`
- Checked-in seed files: `templates/`
- Checked-in planning/review documents: `docs/plans/`, `reviews/`

## Why this example exists

Historically, files like `task_plan.md`, `progress.md`, and `findings.md` could accumulate at the repository root during local runs. They are now intentionally treated as runtime artifacts, not canonical repository inputs.

Use this note as the checked-in reference point instead of reintroducing mutable root files.

## Typical local layout

```text
.fusion/
├── task_plan.md
├── progress.md
├── findings.md
├── sessions.json
├── config.yaml
└── events.jsonl
```

In this local tree, `.fusion/config.yaml` is generated workspace state initialized from the checked-in `templates/config.yaml` baseline.
