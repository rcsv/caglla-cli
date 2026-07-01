# Current Work

## Current phase

v4.4.3 Travel Book presentation helpers extraction（planning）

## Latest completed

- v4.4.2 Travel Book presentation helper review — **released**
- v4.4.1 Category display name in Travel Book — **released**

## Repository state

- Cargo version: `4.4.2`
- **v4.4.2 review:** [v4.4.2-travel-book-presentation-helper-review.md](specifications/v4.4.2-travel-book-presentation-helper-review.md)
- Release notes: [v4.4.2-notes.md](releases/v4.4.2-notes.md)

## Next action

**v4.4.3 — Travel Book presentation helpers extraction**（Phase 1）

既存 `pub(crate)` helper を小さなモジュールへ移動（golden 不変）:

- `format_travel_book_category_detail_line`
- `reservation_provider_line_redundant`
- `format_travel_book_reservation_period`
- `format_travel_book_reservation_heading`（ロジック部分）
- `travel_book_note_sort_key`

根拠: [v4.4.2-travel-book-presentation-helper-review.md](specifications/v4.4.2-travel-book-presentation-helper-review.md) §13

## Do not start yet

- `TravelBookDocument` 全体抽象
- 大規模 view model 一括導入
- GUI / native app コード
- Venue model / map provider
- Dependabot #65（toml 0.8 → 1.1）— v4.4.2 とは分離、rebase 後 CI 確認

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
