# Current Work

## Current phase

v4.6.3 planning — command handler split Phase 1

## Latest completed

- v4.6.2 SQLite migration strategy review — **released**
- v4.6.1 SQLite FK / orphan data hardening review — **released**
- v4.6.0 TripStats.days semantics fix — **released**

## Repository state

- Cargo version: `4.6.2`
- Latest release: **v4.6.2** — [v4.6.2-notes.md](releases/v4.6.2-notes.md)
- **v4.6.2 review:** [v4.6.2-sqlite-migration-strategy-review.md](specifications/v4.6.2-sqlite-migration-strategy-review.md)

## Next action

**v4.6.3 — command handler split Phase 1**

- 将来の Tauri GUI 化を見据え、CLI command handler と core / service logic の分離方針を整理
- migration runner / FK 実装とは独立した refactor track

**Defer (from v4.6.2 review):**

- migration runner 実装
- `PRAGMA user_version` コード追加
- FK 制約追加
- orphan detection / auto-repair

## Do not start yet

- FK migration 実装
- Tauri / GUI 実装
- `TravelBookDocument` full abstraction

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
