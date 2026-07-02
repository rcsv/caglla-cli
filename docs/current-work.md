# Current Work

## Current phase

v4.6.2 implementation — SQLite migration strategy review (**release pending**)

## Latest completed

- v4.6.1 SQLite FK / orphan data hardening review — **released**
- v4.6.0 TripStats.days semantics fix — **released**
- v4.5.1 doctor / advisor Receipt utilization — **released**

## Repository state

- Cargo version: `4.6.1`
- Latest release: **v4.6.1** — [v4.6.1-notes.md](releases/v4.6.1-notes.md)
- **v4.6.2 review (draft):** [v4.6.2-sqlite-migration-strategy-review.md](specifications/v4.6.2-sqlite-migration-strategy-review.md)
- **v4.6.2 notes (draft):** [v4.6.2-notes.md](releases/v4.6.2-notes.md)

## Next action

**v4.6.2 release** — documentation-only SQLite migration strategy review

- `make check` PASS 後 `tools/release/full-release.sh v4.6.2 "SQLite migration strategy review"`
- P0 docs sync（README latest → v4.6.2）

**Then:**

**v4.6.3 — command handler split Phase 1**

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
