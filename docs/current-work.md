# Current Work

## Current phase

v4.6.3 implementation — command handler split Phase 1 (**release pending**)

## Latest completed

- v4.6.2 SQLite migration strategy review — **released**
- v4.6.1 SQLite FK / orphan data hardening review — **released**
- v4.6.0 TripStats.days semantics fix — **released**

## Repository state

- Cargo version: `4.6.2`
- Latest release: **v4.6.2** — [v4.6.2-notes.md](releases/v4.6.2-notes.md)
- **v4.6.3 review (draft):** [v4.6.3-command-handler-split-phase-1.md](specifications/v4.6.3-command-handler-split-phase-1.md)
- **v4.6.3 notes (draft):** [v4.6.3-notes.md](releases/v4.6.3-notes.md)

## Next action

**v4.6.3 release** — documentation-first command handler split Phase 1

- `make check` PASS 後 `tools/release/full-release.sh v4.6.3 "Command handler split Phase 1"`
- P0 docs sync（README latest → v4.6.3）

**Then (candidates):**

| Milestone | 内容 |
|---|---|
| **v4.6.4** | read-only service boundary pilot（`trip list/show`, `day list`, `itinerary timeline`, `stats`） |
| **v4.6.x** | migration track — orphan detection / migration runner（v4.6.2 順序、独立） |
| **v4.7.0** | Tauri shell spike（read-only GUI、defer） |

## Defer

- Tauri プロジェクト作成 / GUI 実装
- `main.rs` 一括 `commands/` 移動
- DB schema / FK / migration runner
- `domain/models.rs` 分割

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
