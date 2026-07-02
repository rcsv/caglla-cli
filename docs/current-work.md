# Current Work

## Current phase

v4.6.1 planning — SQLite FK / orphan data hardening review

## Latest completed

- v4.6.0 TripStats.days semantics fix — **released**
- v4.5.1 doctor / advisor Receipt utilization — **released**
- v4.5.0 Receipt Inbox responsibilities review — **released**

## Repository state

- Cargo version: `4.6.0`
- Latest release: **v4.6.0** — [v4.6.0-notes.md](releases/v4.6.0-notes.md)
- **v4.6.0 spec:** [v4.6.0-trip-stats-days-semantics-fix.md](specifications/v4.6.0-trip-stats-days-semantics-fix.md)

## Next action

**v4.6.1 — SQLite FK / orphan data hardening review**（documentation-only）

- FK 制約、orphan data、削除 cascade の棚卸し
- v4.6.2+ は review で確定した優先順位に従って小さく実装

**Defer:**

- `duration_label` / `accommodation_night_count`（「何泊何日」表示）
- `TravelBookDocument` prototype（UI / Venue 要件まで）
- trip stats への Receipt 反映、Potential Actual 表示

## Do not start yet

- `accommodation_nights` DB 追加、Hotel 宿泊数の自動算出
- Receipt 専用 `image_path` 先行実装
- trip stats / Planned vs Actual への Receipt・Pending 反映
- Balance / Settlement
- `TravelBookDocument` full abstraction（UI/Venue requirements）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
