# Current Work

## Current phase

v4.6.2 planning — SQLite migration strategy review

## Latest completed

- v4.6.1 SQLite FK / orphan data hardening review — **released**
- v4.6.0 TripStats.days semantics fix — **released**
- v4.5.1 doctor / advisor Receipt utilization — **released**

## Repository state

- Cargo version: `4.6.1`
- Latest release: **v4.6.1** — [v4.6.1-notes.md](releases/v4.6.1-notes.md)
- **v4.6.1 review:** [v4.6.1-sqlite-fk-orphan-data-hardening-review.md](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md)

## Next action

**v4.6.2 — SQLite migration strategy review**（documentation-only）

- DB ファイル内 `schema_version`、migration runner、backup before migrate、dry-run
- legacy DB fixture 方針、destructive migration 回避原則
- v4.6.1 review の FK 導入前提を満たす設計

**Defer (from v4.6.1 review):**

- FK 制約の即時追加・migration 実装
- orphan detection / doctor 連携（v4.6.x 候補）
- `duration_label` / `accommodation_night_count`

## Do not start yet

- FK migration 実装
- orphan 自動修復
- command handler split Phase 1（v4.6.3 候補）
- `TravelBookDocument` full abstraction

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
