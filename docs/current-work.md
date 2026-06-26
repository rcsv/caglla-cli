# Current Work

## Current phase

v3.9.2 release verification

## Latest completed

- v3.9.2 legacy migration smoke tests (`open_db_at` / `collect_db_status` for description→summary, no days table, receipts without `trashed_at`).
- v3.9.1 legacy `days.summary` migration order fix (`db status` on existing `caglla.db`).
- v3.9.0 Config and DB path foundation Phase 1 (`--db`, `CAGLLA_DB`, `./caglla.toml`, `db status` JSON v2).

## Repository state

- Latest tag: `v3.9.1` (before v3.9.2 release)
- Cargo version: `3.9.2` (test-only patch ready; release pending)

## Next action

After v3.9.2 release, pick **one** design topic (do not parallelize by default):

1. **Travel Book v4 concept design** — `trip export-md` as Generator v0; PDF / shared pre-trip output
2. **doctor / advisor utilization** — Estimate / Receipt / Pending hints only (no stats Actual change)
3. **DB path Phase 2** — `db use`, config auto-generation, parent-dir search (not started)

See [v3.8.0-roadmap-realignment-after-receipt-inbox.md](specifications/v3.8.0-roadmap-realignment-after-receipt-inbox.md) §5.

## Do not start yet

Canonical defer list (synced with [long-term-version-strategy.md](long-term-version-strategy.md)):

- Evidence / Attachment（共通レイヤー設計が先）
- image_path（Receipt / Expense 専用の先行実装）
- OCR
- automatic receipt parsing
- Balance / Settlement（精算・振込計算）
- Participant sharing 拡張（Settlement 連動）
- Expense reassign / unassign / trash
- receipt purge
- Travel Journal 実装（v5 — Evidence / Attachment 未設計）
- trip stats / Planned vs Actual への Receipt・Pending 反映
- Potential Actual 表示
- Cloud / Identity / Platform 実装
