# Current Work

## Current phase

v4.6.1 implementation — SQLite FK / orphan data hardening review (**release pending**)

## Latest completed

- v4.6.0 TripStats.days semantics fix — **released**
- v4.5.1 doctor / advisor Receipt utilization — **released**
- v4.5.0 Receipt Inbox responsibilities review — **released**

## Repository state

- Cargo version: `4.6.0`
- Latest release: **v4.6.0** — [v4.6.0-notes.md](releases/v4.6.0-notes.md)
- **v4.6.1 review (draft):** [v4.6.1-sqlite-fk-orphan-data-hardening-review.md](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md)
- **v4.6.1 notes (draft):** [v4.6.1-notes.md](releases/v4.6.1-notes.md)

## Next action

**v4.6.1 release** — documentation-only FK / orphan data hardening review

- `make check` PASS 後 `tools/release/full-release.sh v4.6.1 "SQLite FK / orphan data hardening review"`
- P0 docs sync（README latest → v4.6.1）

**Then:**

**v4.6.2 — SQLite migration strategy review**（documentation-only）

- DB ファイル内 `schema_version`、migration runner、backup、dry-run、legacy fixture 方針

**Defer (from v4.6.1 review):**

- FK 制約の即時追加
- orphan 自動修復
- `duration_label` / `accommodation_night_count`（「何泊何日」表示）

## Do not start yet

- FK migration 実装
- `accommodation_nights` DB 追加
- command handler split Phase 1（v4.6.3 候補まで defer）
- `TravelBookDocument` full abstraction

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
