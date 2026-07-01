# Current Work

## Current phase

v4.4.6 Travel Book presentation helpers extraction Phase 3（planning / next）

## Latest completed

- v4.4.5 Travel Book presentation extraction review — **released**
- v4.4.4 Travel Book presentation helpers extraction Phase 2 — **released**
- v4.4.3 Travel Book presentation helpers extraction — **released**
- Dependabot #65 `toml` 1.1 — **merged**

## Repository state

- Cargo version: `4.4.5`
- **v4.4.5 review:** [v4.4.5-travel-book-presentation-extraction-review.md](specifications/v4.4.5-travel-book-presentation-extraction-review.md)
- Presentation module: `src/io/travel_book_presentation.rs`（13 helper + 1 struct）
- Release notes: [v4.4.5-notes.md](releases/v4.4.5-notes.md)

## Next action

v4.4.5 レビュー結論（**A → その後 B**）を踏まえ、次は v4.4.6 を推奨:

- **A. v4.4.6** — Phase 3 small helper extraction（note heading / day label / date range、golden 不変）
- **B. v4.5.0** — TravelBookDocument prototype（UI / Venue 要件が見えてから）
- C. 別ロードマップ優先なら Defer

## Do not start yet

- `TravelBookDocument` 全体抽象（UI/Venue 要件まで Defer）
- GUI / native app コード
- Venue model / map provider

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
