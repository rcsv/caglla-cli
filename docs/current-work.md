# Current Work

## Current phase

v4.1.1 release verification

## Latest completed

- v4.1.1 Okinawa Travel Book sample enrichment plan (documentation-only).
- v4.1.0 Travel Book chapter structure design.
- v4.0.0 Travel Book concept design.

## Repository state

- Latest tag: `v4.1.0` (expected; update after release)
- Latest release: `v4.1.0 — Travel Book chapter structure design` (expected; update after release)
- Cargo version: `4.1.1` (enrichment plan ready; release pending)

## Next action

**v4.1.2 — Okinawa Travel Book sample enrichment** (single topic):

- `seed.sh` — Summary / Note / Reservation / Estimate per [v4.1.1 plan](specifications/v4.1.1-okinawa-travel-book-sample-enrichment-plan.md)
- `expected-export-v3.json` + `normalize_export_v3` (`trip.summary`)
- 58 / 49 / ¥561,780 / Receipt 不変

Then **v4.2.0** — `export-md` layout improvement (use enriched Okinawa for chapter verification).

Deferred until after v4.1.2 / v4.2:

- DB path Phase 3
- PDF export
- Highlights auto-extraction
- Travel Journal / Evidence / Settlement

See [v4.1.1-okinawa-travel-book-sample-enrichment-plan.md](specifications/v4.1.1-okinawa-travel-book-sample-enrichment-plan.md).

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
