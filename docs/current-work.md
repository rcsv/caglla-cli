# Current Work

## Current phase

v4.1.2 release verification

## Latest completed

- v4.1.2 Okinawa Travel Book sample enrichment (seed + golden).
- v4.1.1 Okinawa enrichment plan (documentation-only).
- v4.1.0 Travel Book chapter structure design.

## Repository state

- Latest tag: `v4.1.1` (expected; update after release)
- Cargo version: `4.1.2` (implementation ready; release pending)

## Next action

**v4.2.0 — export-md layout improvement** (after v4.1.2 release):

- Implement [v4.1.0 chapter structure](specifications/v4.1.0-travel-book-chapter-structure-design.md) in `src/io/markdown.rs`
- Verify with enriched Okinawa sample (`trip export-md 1`)

Deferred:

- DB path Phase 3
- PDF export
- Highlights auto-extraction

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
