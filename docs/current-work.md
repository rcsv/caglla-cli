# Current Work

## Current phase

v4.4.x Travel Book presentation model — Phase 2 判断待ち

## Latest completed

- v4.4.3 Travel Book presentation helpers extraction — **released**
- v4.4.2 Travel Book presentation helper review — **released**

## Repository state

- Cargo version: `4.4.3`
- Presentation module: `src/io/travel_book_presentation.rs`
- **v4.4.3 plan:** [v4.4.3-travel-book-presentation-helpers-extraction-plan.md](specifications/v4.4.3-travel-book-presentation-helpers-extraction-plan.md)
- Release notes: [v4.4.3-notes.md](releases/v4.4.3-notes.md)

## Next action

v4.4.2 review §13 Phase 2（trip overview / days overview / planned cost 判定）を別 milestone で判断。大規模 `TravelBookDocument` は Defer。

Dependabot #65（toml 0.8 → 1.1）は別作業 — rebase 後 CI 確認してから判断。

## Do not start yet

- `TravelBookDocument` 全体抽象
- GUI / native app コード
- Venue model / map provider
- Dependabot #65 を v4.4.3 と混在させない

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
