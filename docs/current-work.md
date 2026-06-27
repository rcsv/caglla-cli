# Current Work

## Current phase

v4.0.0 release verification

## Latest completed

- v4.0.0 Travel Book concept design (documentation-only).
- v3.11.0 DB Use implementation (`db use` / `db use --clear`).
- v3.10.0 DB Use concept design.
- v3.9.x Config and DB path foundation series.

## Repository state

- Latest tag: `v3.11.0` (expected; update after release)
- Latest release: `v3.11.0 — DB Use implementation` (expected; update after release)
- Cargo version: `4.0.0` (concept design ready; release pending)

## Next action

Pick **one** design or implementation topic (do not parallelize by default):

1. **Travel Book v4.1** — Markdown structure design (`export-md` layout)
2. **DB path Phase 3** — parent-directory `caglla.toml` search (design first)
3. **doctor / advisor utilization** — Estimate / Receipt / Pending hints only

Deferred:

- PDF export (until Markdown structure stabilizes)
- Travel Journal (v5 — Evidence / Attachment 未設計)
- User-global config / profile switching
- Evidence / Attachment / OCR / Settlement

See [v4.0.0-travel-book-concept-design.md](specifications/v4.0.0-travel-book-concept-design.md) §11–12 and [v3.8.0-roadmap-realignment-after-receipt-inbox.md](specifications/v3.8.0-roadmap-realignment-after-receipt-inbox.md) §5.

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
