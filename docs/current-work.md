# Current Work

## Current phase

v4.6.5 planning — read-only service boundary expansion

## Latest completed

- v4.6.4 Read-only service boundary pilot — **released**
- v4.6.3 Command handler split Phase 1 — **released**
- v4.6.2 SQLite migration strategy review — **released**
- v4.6.1 SQLite FK / orphan data hardening review — **released**

## Repository state

- Cargo version: `4.6.4` (planned: `4.6.5`)
- Latest release: **v4.6.4** — [v4.6.4-notes.md](releases/v4.6.4-notes.md)
- **v4.6.5 draft spec:** [v4.6.5-read-only-service-boundary-expansion.md](specifications/v4.6.5-read-only-service-boundary-expansion.md)

## Next action

**v4.6.5 — read-only service boundary expansion**（optional implementation）

優先 pilot 候補: **`trip list`**（単独推奨）

```text
trip list
trip show
day list
itinerary list / timeline
```

- `trip stats` pilot と同様に service + CLI display 分離
- CLI behavior / golden 不変を gate とする

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection（[v4.6.2 review](specifications/v4.6.2-sqlite-migration-strategy-review.md) 順序）

## Defer

- Tauri / GUI 実装
- `main.rs` 一括 `commands/` 移動
- write command の service 化（Tier 3+）
- `trip delete` / `import` / `duplicate` / `receipt assign`

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
