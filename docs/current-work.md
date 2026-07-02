# Current Work

## Current phase

v4.6.0 implementation — TripStats.days semantics fix (**release pending**)

## Latest completed

- v4.5.1 doctor / advisor Receipt utilization — **released**
- v4.5.0 Receipt Inbox responsibilities review — **released**
- v4.4.8 Travel Book presentation helper cleanup — **released**

## Repository state

- Cargo version: `4.6.0`
- Latest release: **v4.5.1** — [v4.5.1-notes.md](releases/v4.5.1-notes.md)
- **v4.6.0 spec:** [v4.6.0-trip-stats-days-semantics-fix.md](specifications/v4.6.0-trip-stats-days-semantics-fix.md)
- **v4.6.0 notes (draft):** [v4.6.0-notes.md](releases/v4.6.0-notes.md)

## Next action

**v4.6.0 release** — `TripStats.days` semantics fix

- `make check` PASS 後 `tools/release/full-release.sh v4.6.0 "TripStats.days semantics fix"`
- P0 docs sync（README latest → v4.6.0）

**Then (defer):**

```text
v4.6.1 — SQLite FK / orphan data hardening review
v4.6.2 — SQLite migration strategy review
v4.6.3 — command handler split Phase 1
```

## Do not start yet

- Receipt 専用 `image_path` 先行実装
- trip stats / Planned vs Actual への Receipt・Pending 反映
- Balance / Settlement
- `TravelBookDocument` full abstraction（UI/Venue requirements）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
