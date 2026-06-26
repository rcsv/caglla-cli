# Current Work

## Current phase

v3.8.0 release verification

## Latest completed

- v3.8.0 roadmap realignment documentation was created.
- v3.7.1 Receipt Inbox workflow, Okinawa sample, and post-implementation review are complete.
- Package binary is `travel-ledger-cli`.

## Repository state

- Latest tag: `v3.8.0` (after release)
- Latest release: `v3.8.0 — Roadmap realignment after Receipt Inbox`
- Cargo version: `3.8.0`
- Working tree: clean (after release)
- `origin/master`: up to date (after push)

## Next action

Pick **one** design topic to start (do not parallelize by default):

1. **DB path switching** — `--db` / `CAGLLA_DB` / `db use` (extends v3.2.0 `db path` / `db status`)
2. **Travel Book v4 concept design** — `trip export-md` as Generator v0; PDF / shared pre-trip output
3. **doctor / advisor utilization** — Estimate / Receipt / Pending hints only (no stats Actual change)

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
