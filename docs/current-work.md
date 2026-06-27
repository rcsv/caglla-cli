# Current Work

## Current phase

v4.1.0 release verification

## Latest completed

- v4.1.0 Travel Book chapter structure design (documentation-only).
- v4.0.0 Travel Book concept design.
- v3.11.0 DB Use implementation.

## Repository state

- Latest tag: `v4.0.0` (expected; update after release)
- Latest release: `v4.0.0 — Travel Book concept design` (expected; update after release)
- Cargo version: `4.1.0` (chapter structure design ready; release pending)

## Next action

Pick **one** design or implementation topic (do not parallelize by default):

1. **Travel Book v4.2** — `export-md` layout improvement（v4.1.0 章立てに沿った実装）
2. **DB path Phase 3** — parent-directory `caglla.toml` search (design first)
3. **doctor / advisor utilization** — Estimate / Receipt / Pending hints only

Deferred:

- PDF export (until Markdown structure is implemented in v4.2+)
- Highlights auto-extraction
- Travel Journal (v5 — Evidence / Attachment 未設計)
- Evidence / Attachment / OCR / Settlement

See [v4.1.0-travel-book-chapter-structure-design.md](specifications/v4.1.0-travel-book-chapter-structure-design.md) §8–9.

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
