# Current Work

## Current phase

v4.4.7 Travel Book presentation helpers final review（planning / review）

## Latest completed

- v4.4.6 Travel Book presentation helpers extraction Phase 3 — **released**
- v4.4.5 Travel Book presentation extraction review — **released**
- v4.4.4 Travel Book presentation helpers extraction Phase 2 — **released**

## Repository state

- Cargo version: `4.4.6`
- **v4.4.7 review:** [v4.4.7-travel-book-presentation-helpers-final-review.md](specifications/v4.4.7-travel-book-presentation-helpers-final-review.md)
- Presentation module: `src/io/travel_book_presentation.rs`（Phase 1 + 2 + 3、~17 helper）

## Next action（v4.4.7 レビュー結論）

**B → Defer A** を推奨:

- **B. v4.4.8** — helper cleanup（category `-` / reservation `**` 構文分離、golden 不変・任意）
- **A. v4.5.0** — TravelBookDocument prototype（**Defer** — UI / Venue 要件が見えてから）
- C. v4.4.8 をスキップし Travel Book presentation を凍結、別ロードマップへ

Helper extraction arc（Phase 1〜3）は **十分完了**。

## Do not start yet

- `TravelBookDocument` 全体抽象（UI/Venue 要件まで Defer）
- GUI / native app コード
- Venue model / map provider

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
