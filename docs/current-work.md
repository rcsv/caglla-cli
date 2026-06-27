# Current Work

## Current phase

v3.10.0 release verification

## Latest completed

- v3.10.0 DB Use concept design (`db use` / `db use --clear` / config write rules — documentation-only).
- v3.9.2 legacy migration smoke tests (test-only patch).
- v3.9.1 legacy `days.summary` migration order fix.
- v3.9.0 Config and DB path foundation Phase 1 (`--db`, `CAGLLA_DB`, `./caglla.toml`, `db status` JSON v2).

## Repository state

- Latest tag: `v3.9.2`
- Latest release: `v3.9.2 — Legacy migration test hardening`
- Cargo version: `3.10.0` (documentation-only; release pending)

## Next action

Pick **one** implementation or design topic (do not parallelize by default):

1. **`db use` Implementation Plan + Phase 2 implementation** — per [v3.10.0-db-use-concept-design.md](specifications/v3.10.0-db-use-concept-design.md)
2. **Travel Book v4 concept design** — `trip export-md` as Generator v0; PDF / shared pre-trip output
3. **doctor / advisor utilization** — Estimate / Receipt / Pending hints only (no stats Actual change)

Deferred from DB path track (not next by default):

- Parent-directory `caglla.toml` search (Phase 3+)
- User-global config / profile switching

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
